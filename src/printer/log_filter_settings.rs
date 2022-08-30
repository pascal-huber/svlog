use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use regex::Regex;

use crate::{
    cli::Args,
    error::SvLogResult,
    printer::LogPriority,
    util::{
        os_times::{boot_times, local_tz},
        regex::build_regex,
    },
};

pub struct LogFilterSettings {
    pub re: Option<Regex>,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
    pub tz: Option<Tz>,
    pub min_priority: LogPriority,
    pub max_priority: LogPriority,
}

impl LogFilterSettings {
    pub fn from_args(args: &Args) -> SvLogResult<Self> {
        let re: Option<Regex> = build_regex(&args.filter);
        let tz = if args.utc { None } else { Some(local_tz()?) };
        let (since_time_utc, until_time_utc) = if args.boot {
            boot_times(0)?
        } else if let Some(offset) = args.boot_offset {
            boot_times(offset)?
        } else if let Some(tz) = tz {
            let since_time_utc: Option<NaiveDateTime> = args
                .since
                .map(|since| tz.from_local_datetime(&since).unwrap().naive_utc());
            let until_time_utc: Option<NaiveDateTime> = args
                .until
                .map(|until| tz.from_local_datetime(&until).unwrap().naive_utc());
            (since_time_utc, until_time_utc)
        } else {
            (args.since, args.until)
        };
        Ok(LogFilterSettings {
            re,
            since: since_time_utc,
            until: until_time_utc,
            tz,
            min_priority: args.priority.0,
            max_priority: args.priority.1,
        })
    }
}
