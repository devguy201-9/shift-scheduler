use super::rule_engine::{RuleContext, RuleViolation, SchedulingRule};
use shared::types::ShiftType;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct NoMorningAfterEvening {
    pub is_enabled: bool,
}

impl SchedulingRule for NoMorningAfterEvening {
    fn name(&self) -> &'static str {
        "NoMorningAfterEvening"
    }

    fn validate(&self, ctx: &RuleContext) -> Result<(), RuleViolation> {
        // group assignments by staff
        let mut staff_map: HashMap<Uuid, Vec<_>> = HashMap::new();

        for a in ctx.assignments {
            staff_map.entry(a.staff_id).or_default().push(a);
        }

        for (_staff, mut list) in staff_map {
            list.sort_by_key(|a| a.date);

            for i in 1..list.len() {
                if list[i - 1].shift == ShiftType::Evening && list[i].shift == ShiftType::Morning {
                    return Err(RuleViolation {
                        rule: self.name(),
                        message: format!("Morning follows Evening on {}", list[i].date),
                    });
                }
            }
        }

        Ok(())
    }

    fn enabled(&self) -> bool {
        self.is_enabled
    }
}
