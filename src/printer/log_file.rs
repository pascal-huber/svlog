use std::{
    collections::BTreeSet,
    fs::File,
    io::{prelude::*, BufReader},
};

use super::LogSettings;
use crate::{printer::log_line::*, SvLogError, SvLogResult};

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
        log_settings: &LogSettings,
        // from: Option<NaiveDateTime>,
        // until: Option<NaiveDateTime>,
        // re: &Option<Regex>,
        // min_priority: LogPriority,
        // max_priority: LogPriority,
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
            .filter(|l| l.is_between(&log_settings.since, &log_settings.until))
            .filter(|l| l.is_match(&log_settings.re))
            .filter(|l| l.has_priority(&log_settings.min_priority, &log_settings.max_priority))
            .cloned()
            .collect();
        let meta = file.metadata();
        let length = meta.unwrap().len();
        self.position = length;
        Ok(log_lines)
    }
}
