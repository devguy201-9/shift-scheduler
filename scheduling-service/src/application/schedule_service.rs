use crate::application::data_client_trait::DataClient;
use crate::application::traits::ScheduleRepository;
use crate::config::RuleConfig;
use crate::domain::rules::{
    balance_rule::BalanceRule,
    day_off_rule::DayOffRule,
    no_morning_after_evening::NoMorningAfterEvening,
    rule_engine::{RuleContext, RuleEngine},
};
use crate::domain::schedule::{ScheduleJob, ShiftAssignment};
use chrono::{Datelike, NaiveDate};
use shared::types::{JobStatus, ShiftType};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct ScheduleService {
    repo: Arc<dyn ScheduleRepository>,
    data_client: Arc<dyn DataClient>,
    config: RuleConfig,
}

impl ScheduleService {
    pub fn new(
        repo: Arc<dyn ScheduleRepository>,
        data_client: Arc<dyn DataClient>,
        config: RuleConfig,
    ) -> Self {
        Self {
            repo,
            data_client,
            config,
        }
    }

    // Create JOB
    pub async fn create_job(
        &self,
        staff_group_id: Uuid,
        start_date: NaiveDate,
    ) -> anyhow::Result<Uuid> {
        let job_id = Uuid::new_v4();

        self.repo
            .insert_job(job_id, staff_group_id, start_date)
            .await?;

        Ok(job_id)
    }

    // Worker processing
    pub async fn process_next_job(&self) -> anyhow::Result<()> {
        let job = match self.repo.fetch_pending().await? {
            Some(j) => j,
            None => return Ok(()),
        };

        self.repo.mark_processing(job.id).await?;

        match self.process_job(&job).await {
            Ok(_) => self.repo.mark_completed(job.id).await?,
            Err(e) => {
                self.repo.mark_failed(job.id, &e.to_string()).await?;
            }
        }

        Ok(())
    }

    async fn process_job(&self, job: &ScheduleJob) -> anyhow::Result<()> {
        let staff_ids = self
            .data_client
            .get_group_members(job.staff_group_id)
            .await?;

        let assignments = self.generate_schedule(staff_ids, job.period_begin_date)?;

        self.repo.save_assignments(job.id, assignments).await?;

        Ok(())
    }

    fn generate_schedule(
        &self,
        staff_ids: Vec<Uuid>,
        start_date: NaiveDate,
    ) -> anyhow::Result<Vec<ShiftAssignment>> {
        if staff_ids.is_empty() {
            return Ok(vec![]);
        }

        let total_days = 28;
        let mut assignments = Vec::with_capacity(total_days * staff_ids.len());

        let mut last_shift: HashMap<Uuid, ShiftType> = HashMap::new();
        let mut weekly_day_off: HashMap<(Uuid, i32), i32> = HashMap::new();

        let staff_len = staff_ids.len();

        for day in 0..total_days {
            let date = start_date + chrono::Duration::days(day as i64);
            let week = day / 7;
            let weekday = date.weekday().number_from_monday() as i32;

            let mut morning_count = 0;
            let mut evening_count = 0;

            for i in 0..staff_len {
                let staff_id = &staff_ids[(i + day) % staff_len];
                let key = (*staff_id, week as i32);
                let day_off_count = weekly_day_off.get(&key).copied().unwrap_or(0);

                let days_left = 7 - weekday;
                let need_min = self.config.min_day_off_per_week - day_off_count;

                // --- Force DayOff if necessary
                let shift = if need_min > 0 && need_min > days_left {
                    ShiftType::DayOff
                } else {
                    Self::assign_work_shift(
                        staff_id,
                        &mut last_shift,
                        &mut morning_count,
                        &mut evening_count,
                        self.config.max_daily_shift_diff,
                        self.config.no_morning_after_evening,
                    )
                };

                if shift == ShiftType::DayOff {
                    *weekly_day_off.entry(key).or_insert(0) += 1;

                    // reset state after day off
                    last_shift.remove(staff_id);
                } else {
                    last_shift.insert(*staff_id, shift.clone());
                }

                assignments.push(ShiftAssignment {
                    id: Uuid::new_v4(),
                    staff_id: *staff_id,
                    date,
                    shift,
                });
            }
        }

        let engine = RuleEngine::new(vec![
            Box::new(NoMorningAfterEvening {
                is_enabled: self.config.no_morning_after_evening,
            }),
            Box::new(DayOffRule {
                min: self.config.min_day_off_per_week,
                max: self.config.max_day_off_per_week,
                is_enabled: true,
            }),
            Box::new(BalanceRule {
                max_diff: self.config.max_daily_shift_diff,
                is_enabled: true,
            }),
        ]);

        let ctx = RuleContext {
            assignments: &assignments,
        };

        engine.validate(&ctx).map_err(|violations| {
            let message = violations
                .into_iter()
                .map(|v| format!("[{}] {}", v.rule, v.message))
                .collect::<Vec<_>>()
                .join(", ");

            anyhow::anyhow!(message)
        })?;

        Ok(assignments)
    }

    fn assign_work_shift(
        staff_id: &Uuid,
        last_shift: &mut HashMap<Uuid, ShiftType>,
        morning: &mut i32,
        evening: &mut i32,
        max_diff: i32,
        no_morning_after_evening: bool,
    ) -> ShiftType {
        use ShiftType::*;

        let prev = last_shift.get(staff_id);

        // Prefer shift that keeps daily balance closer to 0
        let prefer_morning = *morning <= *evening;

        let mut candidate = if prefer_morning { Morning } else { Evening };

        // Enforce rule 3
        if no_morning_after_evening {
            if matches!(prev, Some(Evening)) && candidate == Morning {
                candidate = Evening;
            }
        }

        // Final balance guard
        match candidate {
            Morning => {
                if (*morning + 1 - *evening).abs() <= max_diff {
                    *morning += 1;
                    Morning
                } else {
                    *evening += 1;
                    Evening
                }
            }
            Evening => {
                if (*evening + 1 - *morning).abs() <= max_diff {
                    *evening += 1;
                    Evening
                } else {
                    *morning += 1;
                    Morning
                }
            }
            _ => ShiftType::DayOff,
        }
    }

    // For API Query
    pub async fn get_status(&self, id: Uuid) -> anyhow::Result<Option<JobStatus>> {
        self.repo.get_status(id).await
    }

    pub async fn get_result(&self, id: Uuid) -> anyhow::Result<Vec<ShiftAssignment>> {
        self.repo.get_result(id).await
    }

    pub async fn get_job(&self, id: Uuid) -> anyhow::Result<Option<ScheduleJob>> {
        self.repo.find_by_id(id).await
    }
}
