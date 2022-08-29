use std::{
    collections::BTreeSet,
    fs::File,
    io::{prelude::*, BufReader},
};

use chrono::NaiveDateTime;
use regex::Regex;

use crate::{
    printer::{log_line::*, LogPriority},
    SvLogError, SvLogResult,
};

#[derive(Copy, Clone)]
pub struct LogFile<'a> {
    pub name: &'a str,
    pub position: u64,
    // TODO: check if I should add BufReader or File to LogFile
    // pub reader: BufReader<File>,
}

impl<'a> LogFile<'a> {
    pub fn new(name: &'a str) -> Self {
        LogFile { name, position: 0 }
    }

    pub fn jump_to_end(&mut self) {
        let file = File::open(self.name).unwrap();
        let meta = file.metadata();
        let length = meta.unwrap().len();
        self.position = length;
    }

    pub fn extract_loglines(
        &mut self,
        from: Option<NaiveDateTime>,
        until: Option<NaiveDateTime>,
        re: &Option<Regex>,
        min_priority: LogPriority,
        max_priority: LogPriority,
    ) -> SvLogResult<BTreeSet<LogLine>> {
        let file = File::open(self.name).unwrap();
        let reader = BufReader::new(&file);
        let mut log_lines: BTreeSet<LogLine> = reader
            .lines()
            .flatten()
            .map(LogLine::new)
            .collect::<Result<BTreeSet<LogLine>, SvLogError>>()?;
        log_lines = log_lines
            .iter()
            .filter(|l| l.is_between(from, until))
            .filter(|l| l.is_match(re))
            .filter(|l| l.has_priority(min_priority, max_priority))
            .cloned()
            .collect();
        let meta = file.metadata();
        let length = meta.unwrap().len();
        self.position = length;
        Ok(log_lines)
    }
}
