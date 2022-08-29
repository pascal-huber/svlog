use chrono::{FixedOffset, NaiveDateTime};
use derive_more::Constructor;
use regex::Regex;

use crate::printer::LogPriority;

#[derive(Constructor)]
pub struct LogSettings {
    pub re: Option<Regex>,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
    pub time_offset: FixedOffset,
    pub min_priority: LogPriority,
    pub max_priority: LogPriority,
    pub use_pager: bool,
}
