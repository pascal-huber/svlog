use std::fmt::{self, Display, Formatter};

use chrono::{LocalResult, NaiveDateTime, Offset, TimeZone, Timelike};
use chrono_tz::Tz;
use regex::Regex;
use snafu::{ensure, ResultExt};

use crate::{error::*, printer::LogPriority};

// NOTE: Socklog timestamps only have 5 digits at the end. Therefore the last is always 0.
static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f";

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct LogLine {
    date: NaiveDateTime,
    date_str: String,
    content: String,
    priority: LogPriority,
}

impl LogLine {
    pub fn new(line: String) -> SvLogResult<Self> {
        ensure!(line.len() >= 25, ParsingLogLineSnafu { line });
        let date_str: &str = &line[..25];
        let date = NaiveDateTime::parse_from_str(date_str, DATE_FORMAT)
            .context(ParsingChronoSnafu { line: &line[..] })?;
        let content_str = line[25..].trim();
        let priority = Self::read_priority(content_str);
        Ok(LogLine {
            date,
            date_str: date_str.to_string(),
            content: content_str.to_string(),
            priority,
        })
    }

    pub fn is_between(&self, from: &Option<NaiveDateTime>, until: &Option<NaiveDateTime>) -> bool {
        match (from, until) {
            (Some(from), _) if self.date < *from => false,
            (_, Some(until)) if self.date > *until => false,
            _ => true,
        }
    }

    pub fn has_priority(&self, min_priority: &LogPriority, max_priority: &LogPriority) -> bool {
        self.priority >= *min_priority && self.priority <= *max_priority
    }

    pub fn is_match(&self, re: &Option<Regex>) -> bool {
        !matches!(re, Some(re) if !re.is_match(&self.content[..]))
    }

    pub fn format_with_tz(&self, tz: &Option<Tz>) -> SvLogResult<String> {
        if let Some(tz) = *tz {
            let local_time = tz.from_utc_datetime(&self.date);
            let offset = tz.offset_from_local_datetime(&self.date);
            if let LocalResult::Single(offset) = offset {
                Ok(format!(
                    "{}.{:0>5}{} {}",
                    local_time.format("%Y-%m-%dT%H:%M:%S"),
                    local_time.nanosecond(),
                    offset.fix(),
                    self.content,
                ))
            } else {
                Err(SvLogError::TimeZoneError {
                    message: format!(
                        "Failed to compute tz offset for date \"{}\" and tz \"{}\"",
                        self.date, tz
                    ),
                })
            }
        } else {
            Ok(format!("{}Z {}", self.date_str, self.content,))
        }
    }

    fn read_priority(content_str: &str) -> LogPriority {
        let priority_str = content_str
            .split_whitespace()
            .next()
            .unwrap_or("")
            .split('.')
            .last()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        LogPriority::from_str_or_max(priority_str)
    }
}

impl Display for LogLine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.date_str, self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok() {
        let ll_str = "2021-12-11T09:12:45.35141 kern.info message";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
        let log_line = ll.unwrap();
        assert_eq!(log_line.content, String::from("kern.info message"));
        assert_eq!(log_line.priority, LogPriority::from_str("info").unwrap());
        assert_eq!(log_line.date_str, "2021-12-11T09:12:45.35141");
    }

    #[test]
    fn parse_ok_smallest_possible_content() {
        let ll_str = "2021-12-11T09:12:45.35141 x";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
        let log_line = ll.unwrap();
        assert_eq!(log_line.content, String::from("x"));
        assert_eq!(log_line.priority, LogPriority::max());
        assert_eq!(log_line.date_str, "2021-12-11T09:12:45.35141");
    }

    #[test]
    fn parse_ok_only_date() {
        let ll_str = "2021-12-11T09:12:45.35141";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
        let log_line = ll.unwrap();
        assert_eq!(log_line.content, String::from(""));
        assert_eq!(log_line.priority, LogPriority::max());
        assert_eq!(log_line.date_str, "2021-12-11T09:12:45.35141");
    }

    #[test]
    fn parse_err_too_short_timestamp() {
        let ll_str = "2021-12-11T09:12:45.3514";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_err());
    }

    #[test]
    fn parse_ok_spaces_1() {
        let ll_str = "2021-12-11T09:12:45.35141  x";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
        let log_line = ll.unwrap();
        assert_eq!(log_line.content, String::from("x"));
        assert_eq!(log_line.priority, LogPriority::max());
        assert_eq!(log_line.date_str, "2021-12-11T09:12:45.35141");
    }

    #[test]
    fn parse_ok_spaces_2() {
        let ll_str = "2021-12-11T09:12:45.35141  ";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
        let log_line = ll.unwrap();
        assert_eq!(log_line.content, String::from(""));
        assert_eq!(log_line.priority, LogPriority::max());
        assert_eq!(log_line.date_str, "2021-12-11T09:12:45.35141");
    }

    #[test]
    fn parse_ok_spaces_3() {
        let ll_str = "2021-12-11T09:12:45.35141\u{2009}x\u{2009}y";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
        let log_line = ll.unwrap();
        assert_eq!(log_line.content, String::from("x\u{2009}y"));
        assert_eq!(log_line.priority, LogPriority::max());
        assert_eq!(log_line.date_str, "2021-12-11T09:12:45.35141");
    }

    #[test]
    fn parse_ok_tab() {
        let ll_str = "2021-12-11T09:12:45.35141\u{0009}kernel.err\u{0009}y";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
        let log_line = ll.unwrap();
        assert_eq!(log_line.content, String::from("kernel.err\u{0009}y"));
        assert_eq!(log_line.priority, LogPriority::from_str("err").unwrap());
        assert_eq!(log_line.date_str, "2021-12-11T09:12:45.35141");
    }

    #[test]
    fn priority_empty_string() {
        let s = "";
        let prio = LogLine::read_priority(s);
        assert_eq!(prio, LogPriority::max());
    }

    #[test]
    fn priority_err() {
        let s = "kern.err: x y z";
        let prio = LogLine::read_priority(s);
        assert_eq!(prio, LogPriority::from_str("err").unwrap());
    }

    #[test]
    fn priority_err_without_colon() {
        let s = "kern.err x y z";
        let prio = LogLine::read_priority(s);
        assert_eq!(prio, LogPriority::from_str("err").unwrap());
    }

    #[test]
    fn priority_err_2() {
        let s = ".err: x y z";
        let prio = LogLine::read_priority(s);
        assert_eq!(prio, LogPriority::from_str("err").unwrap());
    }

    #[test]
    fn priority_no_dot() {
        let s = "kernel: x y z";
        let prio = LogLine::read_priority(s);
        assert_eq!(prio, LogPriority::max());
    }

    #[test]
    fn priority_ambiguous() {
        let s = "kernel.ambiguous: x y z";
        let prio = LogLine::read_priority(s);
        assert_eq!(prio, LogPriority::max());
    }
}
