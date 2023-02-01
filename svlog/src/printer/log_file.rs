use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{printer::log_line::*, LogFilterSettings, SvLogResult};

pub struct LogFile<'a> {
    pub name: &'a str,
    pub position: u64,
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
        log_settings: &LogFilterSettings,
    ) -> SvLogResult<BTreeSet<LogLine>> {
        let file = File::open(self.name).unwrap();
        let reader = BufReader::new(&file);
        let log_lines: BTreeSet<LogLine> = reader
            .lines()
            .flatten()
            .map(LogLine::new)
            .collect::<SvLogResult<BTreeSet<LogLine>>>()?
            // TODO: check out the unstable "try_collect"
            .iter()
            .filter(|&l| l.is_between(&log_settings.since, &log_settings.until))
            .filter(|&l| l.is_match(&log_settings.re))
            .filter(|&l| l.has_priority(&log_settings.min_priority, &log_settings.max_priority))
            .cloned()
            .collect();
        let meta = file.metadata();
        let length = meta.unwrap().len();
        self.position = length;
        Ok(log_lines)
    }
}
