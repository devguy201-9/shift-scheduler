use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RuleConfig {
    pub min_day_off_per_week: i32,
    pub max_day_off_per_week: i32,
    pub no_morning_after_evening: bool,
    pub max_daily_shift_diff: i32,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub rules: RuleConfig,
}
