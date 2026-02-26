use chrono::NaiveDate;
use scheduling_service::domain::rules::balance_rule::BalanceRule;
use scheduling_service::domain::rules::day_off_rule::DayOffRule;
use scheduling_service::domain::rules::rule_engine::{RuleContext, SchedulingRule};
use scheduling_service::domain::schedule::ShiftAssignment;
use shared::types::ShiftType;
use uuid::Uuid;

#[test]
fn balance_rule_detects_imbalance() {
    let staff_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

    let assignments = vec![
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id,
            date,
            shift: ShiftType::Morning,
        },
    ];

    let rule = BalanceRule {
        max_diff: 0,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    let result = rule.validate(&ctx);

    assert!(result.is_err());
}

#[test]
fn day_off_rule_validates_weekly_limits() {
    let staff_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

    let assignments = vec![ShiftAssignment {
        id: Uuid::new_v4(),
        staff_id,
        date,
        shift: ShiftType::DayOff,
    }];

    let rule = DayOffRule {
        min: 2,
        max: 5,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    let result = rule.validate(&ctx);

    assert!(result.is_err());
}
