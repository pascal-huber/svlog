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
        if let Some(from) = from {
            if from > self.date {
                return false;
            }
        };
        if let Some(until) = until {
            if until <= self.date {
                return false;
            }
        };
        true
    }

    pub fn has_priority(&self, min_priority: Option<u8>, max_priority: Option<u8>) -> bool {
        if let Some(self_priority) = self.priority {
            if let Some(min_priority) = min_priority {
                if self_priority < min_priority {
                    return false;
                }
            }
            if let Some(max_priority) = max_priority {
                if self_priority > max_priority {
                    return false;
                }
            }
        } else {
            // if the log  line does not have a priority, don't print it
            return false;
        }
        true
    }

    pub fn is_match(&self, re: &Option<Regex>) -> bool {
        if let Some(re) = re {
            if !re.is_match(&self.content[..]) {
                return false;
            }
        };
        true
    }
}

impl Display for LogLine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.date_str, self.content)
    }
}
