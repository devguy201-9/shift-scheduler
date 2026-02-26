use crate::application::data_client_trait::DataClient;
use crate::application::traits::ScheduleRepository;
use crate::config::RuleConfig;
use crate::domain::rules::{
    balance_rule::BalanceRule,
    day_off_rule::DayOffRule,
    no_morning_after_evening::NoMorningAfterEvening,
    rule_engine::{RuleContext, RuleEngine},
};
use crate::domain::schedule::ShiftAssignment;
use chrono::{Duration, NaiveDate};
use shared::types::{JobStatus, ShiftType};
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

        match self.process_job(job.clone()).await {
            Ok(_) => self.repo.mark_completed(job.id).await?,
            Err(e) => {
                self.repo.mark_failed(job.id, &e.to_string()).await?;
            }
        }

        Ok(())
    }

    async fn process_job(&self, job: crate::domain::schedule::ScheduleJob) -> anyhow::Result<()> {
        let staff_ids = self
            .data_client
            .get_group_members(job.staff_group_id)
            .await?;

        let assignments = Self::generate_schedule(staff_ids, job.period_begin_date, &self.config)?;

        self.repo.save_assignments(job.id, assignments).await?;

        Ok(())
    }

    fn generate_schedule(
        staff_ids: Vec<Uuid>,
        start_date: NaiveDate,
        config: &RuleConfig,
    ) -> anyhow::Result<Vec<ShiftAssignment>> {
        if staff_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut assignments = vec![];

        for (staff_index, staff_id) in staff_ids.iter().enumerate() {
            for day in 0..28 {
                let date = start_date + Duration::days(day);

                let shift = match (day + staff_index as i64) % 3 {
                    0 => ShiftType::Morning,
                    1 => ShiftType::Evening,
                    _ => ShiftType::DayOff,
                };

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
                is_enabled: config.no_morning_after_evening,
            }),
            Box::new(DayOffRule {
                min: config.min_day_off_per_week,
                max: config.max_day_off_per_week,
                is_enabled: true,
            }),
            Box::new(BalanceRule {
                max_diff: config.max_daily_shift_diff,
                is_enabled: true,
            }),
        ]);

        let ctx = RuleContext {
            assignments: &assignments,
        };

        engine.validate(&ctx).map_err(|violations| {
            anyhow::anyhow!(
                violations
                    .into_iter()
                    .map(|v| format!("[{}] {}", v.rule, v.message))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

        Ok(assignments)
    }

    // For API Query
    pub async fn get_status(&self, id: Uuid) -> anyhow::Result<Option<JobStatus>> {
        self.repo.get_status(id).await
    }

    pub async fn get_result(&self, id: Uuid) -> anyhow::Result<Vec<ShiftAssignment>> {
        self.repo.get_result(id).await
    }
}
