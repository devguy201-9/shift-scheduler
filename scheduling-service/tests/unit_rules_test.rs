/*
// This file tests each scheduling rule Completely Independently.
// Don't need DB, Don't need HTTP, Don't need mock service.

// Each rule is 1 struct implement trait SchedulingRule:
//   fn validate(&self, ctx: &RuleContext) -> Result<(), RuleViolation>

// RuleContext only contain: &[ShiftAssignment]
// → Setup slice assignments, call validate(), check for Ok/Err.
//
// How to read this file:
//   1. generate_schedule_output_satisfies_all_rules  — test end-to-end importance
//   2. BalanceRule tests — Check the morning/afternoon shift balance by day
//   3. DayOffRule tests — Check the number of days off per week
//   4. NoMorningAfterEvening — Check consecutive case constraints
*/
use chrono::NaiveDate;
use scheduling_service::application::schedule_service::ScheduleService;
use scheduling_service::config::RuleConfig;
use scheduling_service::domain::rules::balance_rule::BalanceRule;
use scheduling_service::domain::rules::day_off_rule::DayOffRule;
use scheduling_service::domain::rules::no_morning_after_evening::NoMorningAfterEvening;
use scheduling_service::domain::rules::rule_engine::{RuleContext, RuleEngine, SchedulingRule};
use scheduling_service::domain::schedule::ShiftAssignment;
use shared::types::ShiftType;
use std::sync::Arc;
use uuid::Uuid;

/*
Verify that the output of generate_schedule() does not violate any rules
when validated by RuleEngine. In other words, the schedule generation
algorithm and the rule validation logic must remain consistent.

Why this test is necessary:
generate_schedule() contains its own logic to avoid rule violations
during schedule creation. However, if that internal logic has a bug,
the generated schedule could violate a rule and cause the job to fail.
This test catches that type of bug without needing to run the database
or background workers.

Setup:
DummyRepo and DummyClient implement the required traits, but all methods
use unreachable!() because generate_schedule() is a pure function and
does not call the repository or client. We only need ScheduleService
to access the generate_schedule() method.

Flow:
1. Create the service with the standard configuration
   (min = 1, max = 2, diff = 1, no_morning_after_evening = true)
2. Call generate_schedule with 4 staff members and 28 days
3. Run RuleEngine.validate() on the generated output
4. Assert Ok(()) — meaning no rule violations are detected
 */
#[tokio::test]
async fn generate_schedule_output_satisfies_all_rules() {
    // Dummy implementations are only to satisfy the bound trait of ScheduleService::new()
    // All methods are unreachable because generate_schedule() does not call the repo/client.
    struct DummyRepo;
    struct DummyClient;

    use async_trait::async_trait;
    use scheduling_service::application::data_client_trait::DataClient;
    use scheduling_service::application::traits::ScheduleRepository;

    #[async_trait]
    impl ScheduleRepository for DummyRepo {
        async fn insert_job(&self, _: Uuid, _: Uuid, _: NaiveDate) -> anyhow::Result<()> {
            unreachable!()
        }
        async fn fetch_pending(
            &self,
        ) -> anyhow::Result<Option<scheduling_service::domain::schedule::ScheduleJob>> {
            unreachable!()
        }
        async fn mark_processing(&self, _: Uuid) -> anyhow::Result<()> {
            unreachable!()
        }
        async fn mark_completed(&self, _: Uuid) -> anyhow::Result<()> {
            unreachable!()
        }
        async fn mark_failed(&self, _: Uuid, _: &str) -> anyhow::Result<()> {
            unreachable!()
        }
        async fn save_assignments(&self, _: Uuid, _: Vec<ShiftAssignment>) -> anyhow::Result<()> {
            unreachable!()
        }
        async fn get_status(&self, _: Uuid) -> anyhow::Result<Option<shared::types::JobStatus>> {
            unreachable!()
        }
        async fn get_result(&self, _: Uuid) -> anyhow::Result<Vec<ShiftAssignment>> {
            unreachable!()
        }
        async fn find_by_id(
            &self,
            _: Uuid,
        ) -> anyhow::Result<Option<scheduling_service::domain::schedule::ScheduleJob>> {
            unreachable!()
        }
    }

    #[async_trait]
    impl DataClient for DummyClient {
        async fn get_group_members(&self, _: Uuid) -> anyhow::Result<Vec<Uuid>> {
            unreachable!()
        }
    }

    let config = RuleConfig {
        min_day_off_per_week: 1,
        max_day_off_per_week: 2,
        max_daily_shift_diff: 1,
        no_morning_after_evening: true,
    };

    let service = ScheduleService::new(Arc::new(DummyRepo), Arc::new(DummyClient), config.clone());

    let staff_ids = vec![
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    ];

    let start_date = NaiveDate::from_ymd_opt(2025, 1, 6).expect("invalid static test date");

    let assignments = service
        .generate_schedule(staff_ids, start_date)
        .expect("schedule generation failed");

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
    assert!(engine.validate(&ctx).is_ok());
}

// Test for Balance rule
/*
Rule: For each day, |morning_count - evening_count| must be less than or equal to max_daily_shift_diff.

Why this rule is needed:
This ensures that morning and evening shifts are distributed evenly each day.
It prevents situations where a day has too many morning shifts and no evening shifts,
or vice versa, which could negatively impact operations.

How the rule works (balance_rule.rs):
1. Group assignments by date → HashMap<NaiveDate, (morning_count, evening_count)>
2. For each day: if (m - e).abs() > max_diff → return Err
3. DayOff assignments are NOT included in the count

Note:
The threshold is INCLUSIVE — diff == max_diff is considered valid.
*/

// Basic case: 2 Morning, 0 Evening, max_diff=0 → fail because |2-0|=2 > 0
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

// 2M + 2E on the same day, max_diff=1 → ok because |2-2|=0 <= 1
#[test]
fn balance_rule_valid_when_equal() {
    let staff = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date");

    let assignments = vec![
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Evening,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Evening,
        },
    ];

    let rule = BalanceRule {
        max_diff: 1,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_ok());
}

// 3M + 2E, max_diff=1 → OK because |3-2|=1 is exactly equal to the threshold.
// This test verifies that the threshold is inclusive (<=), not strict (<).
// If the implementation is incorrect and uses < instead of <=, this test will fail.
#[test]
fn balance_rule_valid_at_exact_threshold() {
    let staff = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date");

    let assignments = vec![
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Evening,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Evening,
        },
    ];

    let rule = BalanceRule {
        max_diff: 1,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_ok());
}

// 3M + 1E, max_diff=1 → fail because |3-1|=2 > 1
#[test]
fn balance_rule_fails_above_threshold() {
    let staff = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date");

    let assignments = vec![
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date,
            shift: ShiftType::Evening,
        },
    ];

    let rule = BalanceRule {
        max_diff: 1,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_err());
}

// Day 1: 2M/2E → OK | Day 2: 3M/0E → Fail
// The rule must scan ALL days, without stopping after the first valid day.
// If you implement it using `find()` instead of a `for` loop through all the days, you will miss day 2.
#[test]
fn balance_rule_multiple_days_one_bad_day() {
    let staff = Uuid::new_v4();

    let d1 = NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date");
    let d2 = NaiveDate::from_ymd_opt(2025, 1, 2).expect("invalid static test date");

    let assignments = vec![
        // day 1 -> ok
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: d1,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: d1,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: d1,
            shift: ShiftType::Evening,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: d1,
            shift: ShiftType::Evening,
        },
        // day 2 -> fail
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: d2,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: d2,
            shift: ShiftType::Morning,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: d2,
            shift: ShiftType::Morning,
        },
    ];

    let rule = BalanceRule {
        max_diff: 1,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_err());
}

// Test for Day_off rule
/*
Rule: Each staff member must have between min..=max DayOff days per ISO week.

Why this rule is needed:
It ensures that staff receive sufficient rest (not working all 7 days)
while also preventing excessive time off that could lead to staffing
shortages during the week.

How the rule works (day_off_rule.rs):
1. Iterate through all assignments where shift == DayOff
2. Group by (staff_id, iso_year, iso_week) → count DayOff per week
3. If count < min or count > max → return Err

IMPORTANT LIMITATION of the current implementation:
The rule only counts staff who have at least one DayOff in a week.
Staff with 0 DayOff will not appear in the HashMap, so the rule
will NOT detect them.

This is a known limitation — it should either be fixed in the rule
implementation or guaranteed that generate_schedule always assigns
at least min DayOff per staff per week.

ISO week note:
Monday is considered the first day of the week.
Example: 2025-01-06 (Mon) → 2025-01-12 (Sun) belong to ISO week 2.
*/

// 1 DayOff, min=2 → fail because 1 < 2
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

// 1 DayOff per week, min=1 max=2 → ok because 1 is in [1, 2]
#[test]
fn day_off_rule_valid_at_minimum() {
    let staff = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date");

    let assignments = vec![ShiftAssignment {
        id: Uuid::new_v4(),
        staff_id: staff,
        date,
        shift: ShiftType::DayOff,
    }];

    let rule = DayOffRule {
        min: 1,
        max: 2,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_ok());
}

// 2 DayOffs per week (2025-01-01 and 2025-01-02, same ISO week), min=1 max=2 → ok
// Note: January 1, 2025 is Wednesday, January 2, 2025 is Thursday — same week as ISO 1
#[test]
fn day_off_rule_valid_at_maximum() {
    let staff = Uuid::new_v4();

    let assignments = vec![
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date"),
            shift: ShiftType::DayOff,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 2).expect("invalid static test date"),
            shift: ShiftType::DayOff,
        },
    ];

    let rule = DayOffRule {
        min: 1,
        max: 2,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_ok());
}

// 3 DayOffs in the same ISO week, max=2 → fail because 3 > 2
// This test verifies that max_day_off_per_week is correctly enforced.
#[test]
fn day_off_rule_fails_exceeds_maximum() {
    let staff = Uuid::new_v4();

    let assignments = vec![
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid static test date"),
            shift: ShiftType::DayOff,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 2).expect("invalid static test date"),
            shift: ShiftType::DayOff,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 3).expect("invalid static test date"),
            shift: ShiftType::DayOff,
        },
    ];

    let rule = DayOffRule {
        min: 1,
        max: 2,
        is_enabled: true,
    };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_err());
}

// Test for No Morning After Evening
/*
Rule: If day N is an Evening shift, then day N+1 cannot be a Morning shift.

Why this rule is needed:
Staff need sufficient rest after an Evening shift (which typically ends late).
They should not be required to start a Morning shift (usually very early)
the following day.

How the rule works (no_morning_after_evening.rs):
1. Group assignments by staff_id
2. Sort assignments by date within each group
3. Iterate through each consecutive pair (list[i-1], list[i])
4. If list[i-1].shift == Evening && list[i].shift == Morning → return Err

IMPORTANT EDGE CASE:
Evening → DayOff → Morning is VALID.

This is because the rule checks only consecutive assignments in the list.
When a DayOff exists between them, the Evening and Morning shifts are
no longer adjacent.

The test `no_morning_after_evening_allows_dayoff_between_evening_and_morning`
verifies this behavior, which is one of the easiest cases to implement incorrectly.

Equivalent behavior in generate_schedule():
When assigning DayOff → last_shift.remove(staff_id) is called to reset state.
Therefore, the day after a DayOff can be Morning without violating the rule.
*/

// Evening of day 1 → Morning of day 2 → fail (direct violation)
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

// Evening of Day 1 → Day Off of Day 2 → Morning of Day 3 → OK
// DayOff "breaks" the Evening-Morning sequence → no violation
#[test]
fn no_morning_after_evening_allows_evening_then_dayoff_then_morning() {
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
            shift: ShiftType::DayOff,
        },
        ShiftAssignment {
            id: Uuid::new_v4(),
            staff_id: staff,
            date: NaiveDate::from_ymd_opt(2025, 1, 3).expect("invalid static test date"),
            shift: ShiftType::Morning,
        },
    ];

    let rule = NoMorningAfterEvening { is_enabled: true };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_ok());
}

// Evening → Evening → ok (only Evening → Morning is prohibited, Evening → Evening is not)
#[test]
fn no_morning_after_evening_allows_evening_then_evening() {
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
            shift: ShiftType::Evening,
        },
    ];

    let rule = NoMorningAfterEvening { is_enabled: true };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_ok());
}

// is_enabled=false → completely ignores validation, always works even if the data violates the rules.
// This test verifies that the rule engine's on/off switch is working correctly.
#[test]
fn no_morning_after_evening_disabled_allows_violation() {
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

    // is_enabled=false → rule.validate() return OK immediately
    let rule = NoMorningAfterEvening { is_enabled: false };

    let ctx = RuleContext {
        assignments: &assignments,
    };

    assert!(rule.validate(&ctx).is_ok());
}
