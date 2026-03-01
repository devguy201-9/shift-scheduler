use chrono::NaiveDate;
use scheduling_service::domain::rules::balance_rule::BalanceRule;
use scheduling_service::domain::rules::day_off_rule::DayOffRule;
use scheduling_service::domain::rules::no_morning_after_evening::NoMorningAfterEvening;
use scheduling_service::domain::rules::rule_engine::{RuleContext, SchedulingRule};
use scheduling_service::domain::schedule::ShiftAssignment;
use shared::types::ShiftType;
use uuid::Uuid;

#[test]
fn balance_rule_detects_imbalance() {
    let staff_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date");

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

    assert!(rule.validate(&ctx).is_err());
}

#[test]
fn day_off_rule_detects_invalid_range() {
    let staff_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date");

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

#[test]
fn no_morning_after_evening_detects_violation() {
    let staff = Uuid::new_v4();

    let assignments = vec![
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date"),
            shift: ShiftType::Evening,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 2).expect("invalid static test date"),
            shift: ShiftType::Morning,
        },
    ];

    let rule = NoMorningAfterEvening { is_enabled: true };
    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_err());
}
