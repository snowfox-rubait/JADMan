use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    Schedule(DateTime<Utc>),
    Interval(u64),
}

#[allow(dead_code)]
pub struct SchedulerRule {
    pub trigger: Trigger,
    pub action: String,
}
