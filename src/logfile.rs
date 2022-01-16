use crate::logline::*;

use chrono::NaiveDateTime;
use regex::Regex;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Copy, Clone)]
pub struct LogFile<'a> {
    pub name: &'a str,
    pub position: u64,
    // TODO: maybe add BufReader/File to LogFile
    //pub reader: BufReader<File>,
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
        min_priority: Option<u8>,
        max_priority: Option<u8>,
    ) -> BTreeSet<LogLine> {
        let file = File::open(self.name).unwrap();
        let reader = BufReader::new(&file);

        let loglines: BTreeSet<LogLine> = reader
            .lines()
            .flatten()
            .map(LogLine::new)
            .filter(|l| l.is_between(from, until))
            .filter(|l| l.is_match(re))
            .filter(|l| l.has_priority(min_priority, max_priority))
            .collect();

        let meta = file.metadata();
        let length = meta.unwrap().len();
        self.position = length;

        loglines
    }
}
