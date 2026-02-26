use super::rule_engine::{RuleContext, RuleViolation, SchedulingRule};
use chrono::Datelike;
use shared::types::ShiftType;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct DayOffRule {
    pub min: i32,
    pub max: i32,
    pub is_enabled: bool,
}

impl SchedulingRule for DayOffRule {
    fn name(&self) -> &'static str {
        "DayOffRule"
    }

    fn validate(&self, ctx: &RuleContext) -> Result<(), RuleViolation> {
        let mut weekly_count: HashMap<(Uuid, i32, u32), i32> = HashMap::new();

        for a in ctx.assignments {
            if a.shift == ShiftType::DayOff {
                let iso = a.date.iso_week();

                *weekly_count
                    .entry((a.staff_id, iso.year(), iso.week()))
                    .or_insert(0) += 1;
            }
        }

        for ((staff, year, week), count) in weekly_count {
            if count < self.min || count > self.max {
                return Err(RuleViolation {
                    rule: self.name(),
                    message: format!(
                        "Staff {} year {} week {} day_off_count={}",
                        staff, year, week, count
                    ),
                });
            }
        }

        Ok(())
    }

    fn enabled(&self) -> bool {
        self.is_enabled
    }
}
