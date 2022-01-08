use chrono::NaiveDateTime;
use regex::Regex;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::BTreeSet;

// TODO: find out why there are only 5 digits at the end of socklog timestamps
static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6f";

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LogLine {
    date: NaiveDateTime,
    pub date_str: String,
    pub content: String,
}

pub fn extract_loglines(
    path: PathBuf,
    from: Option<NaiveDateTime>,
    until: Option<NaiveDateTime>,
    re: &Option<Regex>,
) -> BTreeSet<LogLine> {
    let file = File::open(path);
    let mut loglines: BTreeSet<LogLine> = BTreeSet::new();
    if let Ok(file) = file {
        let reader = BufReader::new(file);
        loglines = reader
            .lines()
            .flatten()
            .map(create_logline)
            .filter(|l| required_logline(l, from, until, re))
            .collect();
    }
    loglines
}

fn create_logline(line: String) -> LogLine {
    let date_str: &str = &line[..25];
    let date = NaiveDateTime::parse_from_str(date_str, DATE_FORMAT).unwrap();
    let content_str: &str = &line[26..];
    LogLine {
        date,
        date_str: date_str.to_string(),
        content: content_str.to_string(),
    }
}

fn required_logline(
    logline: &LogLine,
    from: Option<NaiveDateTime>,
    until: Option<NaiveDateTime>,
    re: &Option<Regex>,
) -> bool {
    // TODO: can we make this nicer (e.g. https://github.com/rust-lang/rfcs/pull/2497)?
    if let Some(from) = from {
        if from > logline.date {
            return false;
        }
    };
    if let Some(until) = until {
        if until <= logline.date {
            return false;
        }
    };
    if let Some(re) = re {
        if !re.is_match(&logline.content[..]) {
            return false;
        }
    };
    true
}
