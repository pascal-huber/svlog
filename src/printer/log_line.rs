use crate::error::*;
use crate::printer::LogPriority;
use crate::true_or_err;
use chrono::NaiveDateTime;
use regex::Regex;
use snafu::prelude::*;
use std::fmt::{self, Display, Formatter};

// NOTE: Socklog timestamps only have 5 digits at the end. Therefore the last is always 0.
static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6f";

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct LogLine {
    date: NaiveDateTime,
    date_str: String,
    content: String,
    priority: LogPriority,
}

impl LogLine {
    pub fn new(line: String) -> Result<Self, SvLogError> {
        true_or_err!(line.len() >= 27, SvLogError::ParsingLogLineError { line });
        let date_str: &str = &line[..25];
        let date = NaiveDateTime::parse_from_str(date_str, DATE_FORMAT)
            .context(ParsingChronoSnafu { line: &line[..] })?;
        let content_str: &str = &line[26..];
        let priority = Self::read_priority(content_str);
        Ok(LogLine {
            date,
            date_str: date_str.to_string(),
            content: content_str.to_string(),
            priority,
        })
    }

    pub fn is_between(&self, from: Option<NaiveDateTime>, until: Option<NaiveDateTime>) -> bool {
        match (from, until) {
            (Some(from), _) if self.date < from => false,
            (_, Some(until)) if self.date > until => false,
            _ => true,
        }
    }

    pub fn has_priority(&self, min_priority: LogPriority, max_priority: LogPriority) -> bool {
        self.priority >= min_priority && self.priority <= max_priority
    }

    pub fn is_match(&self, re: &Option<Regex>) -> bool {
        !matches!(re, Some(re) if !re.is_match(&self.content[..]))
    }

    fn read_priority(content_str: &str) -> LogPriority {
        let priority_str = content_str
            .split_whitespace()
            .next()
            .unwrap_or("")
            .split('.')
            .last()
            .unwrap_or("");
        LogPriority::from_str_or_max(&priority_str[..priority_str.len() - 1])
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
        let ll_str = "2021-12-11T09:12:45.35141 kern.info: message";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
    }

    #[test]
    fn parse_ok_smallest_possible_content() {
        let ll_str = "2021-12-11T09:12:45.35141 x";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_ok());
    }

    #[test]
    fn parse_err_no_content() {
        let ll_str = "2021-12-11T09:12:45.35141 ";
        let ll: Result<LogLine, _> = LogLine::new(ll_str.to_string());
        assert!(ll.is_err());
    }
}
