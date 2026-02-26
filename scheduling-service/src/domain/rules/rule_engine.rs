use super::super::schedule::ShiftAssignment;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct RuleViolation {
    pub rule: &'static str,
    pub message: String,
}

pub struct RuleContext<'a> {
    pub assignments: &'a [ShiftAssignment],
}

pub trait SchedulingRule: Send + Sync + Debug {
    fn name(&self) -> &'static str;

    // Check rule
    fn validate(&self, ctx: &RuleContext) -> Result<(), RuleViolation>;

    // Allow turn on/off rule
    fn enabled(&self) -> bool {
        true
    }
}

pub struct RuleEngine {
    rules: Vec<Box<dyn SchedulingRule>>,
}

impl RuleEngine {
    pub fn new(rules: Vec<Box<dyn SchedulingRule>>) -> Self {
        Self { rules }
    }

    // Validate all rule
    pub fn validate(&self, ctx: &RuleContext) -> Result<(), Vec<RuleViolation>> {
        let mut errors = vec![];

        for rule in &self.rules {
            if !rule.enabled() {
                continue;
            }

            if let Err(e) = rule.validate(ctx) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
