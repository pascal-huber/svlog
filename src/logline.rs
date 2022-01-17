use crate::util::*;
use chrono::NaiveDateTime;
use regex::Regex;
use std::fmt::{self, Display, Formatter};

// NOTE: Socklog timestamps only have 5 digits at the end. Therefore the last is always 0.
static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6f";

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LogLine {
    date: NaiveDateTime,
    date_str: String, // TODO: check how much time we actually save by not formatting dates.
    content: String,
    priority: Option<u8>,
}

impl LogLine {
    pub fn new(line: String) -> Self {
        let date_str: &str = &line[..25];
        let date = NaiveDateTime::parse_from_str(date_str, DATE_FORMAT).unwrap();
        let content_str: &str = &line[26..];
        let priority_str = content_str
            .split_whitespace()
            .next()
            .unwrap_or("")
            .split('.')
            .last()
            .unwrap_or("");
        let mut priority = priority_value(&priority_str[..priority_str.len() - 1]);
        if priority == None {
            // NOTE: assume "debug" if no valid priority found
            priority = priority_value("debug");
        }
        LogLine {
            date,
            date_str: date_str.to_string(),
            content: content_str.to_string(),
            priority,
        }
    }

    pub fn is_between(&self, from: Option<NaiveDateTime>, until: Option<NaiveDateTime>) -> bool {
        match (from, until) {
            (Some(from), _) if self.date < from => false,
            (_, Some(until)) if self.date > until => false,
            _ => true,
        }
    }

    pub fn has_priority(&self, min_priority: Option<u8>, max_priority: Option<u8>) -> bool {
        match (self.priority, min_priority, max_priority) {
            (Some(prio), Some(min_priority), _) if prio < min_priority => false,
            (Some(prio), _, Some(max_priority)) if prio > max_priority => false,
            _ => true,
        }
    }

    pub fn is_match(&self, re: &Option<Regex>) -> bool {
        !matches!(re, Some(re) if !re.is_match(&self.content[..]))
    }
}

impl Display for LogLine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.date_str, self.content)
    }
}
