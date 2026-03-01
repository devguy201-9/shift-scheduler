use super::rule_engine::{RuleContext, RuleViolation, SchedulingRule};
use chrono::NaiveDate;
use shared::types::ShiftType;
use std::collections::HashMap;

#[derive(Debug)]
pub struct BalanceRule {
    pub max_diff: i32,
    pub is_enabled: bool,
}

impl SchedulingRule for BalanceRule {
    fn name(&self) -> &'static str {
        "BalanceRule"
    }

    fn validate(&self, ctx: &RuleContext) -> Result<(), RuleViolation> {
        let mut daily: HashMap<NaiveDate, (i32, i32)> = HashMap::new();

        for a in ctx.assignments {
            let entry = daily.entry(a.date).or_insert((0, 0));

            match a.shift {
                ShiftType::Morning => entry.0 += 1,
                ShiftType::Evening => entry.1 += 1,
                _ => {}
            }
        }

        for (date, (m, e)) in daily {
            if (m - e).abs() > self.max_diff {
                return Err(RuleViolation {
                    rule: self.name(),
                    message: format!("Shift imbalance on {}: M={}, E={}", date, m, e),
                });
            }
        }

        Ok(())
    }

    fn enabled(&self) -> bool {
        self.is_enabled
    }
}
