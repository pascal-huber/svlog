use crate::cache::*;
use crate::logfile::*;
use crate::logline::*;
use crate::util::*;

use calm_io::stdoutln;
use chrono::NaiveDateTime;
use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};
use rayon::prelude::*;
use regex::Regex;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::sync::mpsc::channel;

pub struct LogPrinter<'a> {
    log_files: Vec<LogFile<'a>>,
    cache: Cache<String>,
    re: Option<Regex>,
    jobs: usize,
    from: Option<NaiveDateTime>,
    until: Option<NaiveDateTime>,
    min_priority: Option<u8>,
    max_priority: Option<u8>,
}

impl<'a> LogPrinter<'a> {
    pub fn new(
        log_files: Vec<LogFile<'a>>,
        re: Option<Regex>,
        jobs: usize,
        from: Option<NaiveDateTime>,
        until: Option<NaiveDateTime>,
        min_priority: Option<u8>,
        max_priority: Option<u8>,
    ) -> LogPrinter<'a> {
        let cache: Cache<String> = Cache::new(20);
        LogPrinter {
            log_files,
            cache,
            re,
            jobs,
            from,
            until,
            min_priority,
            max_priority,
        }
    }

    pub fn jump_to_end(&mut self) {
        for log_file in &mut self.log_files {
            log_file.jump_to_end();
        }
    }

    pub fn print_logs(&mut self, lines: Option<usize>) {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .build_global()
            .unwrap();

        // TODO: can those copies/clones be avoided?
        let fc = self.from;
        let uc = self.until;
        let rc = self.re.clone();
        let minpc = self.min_priority;
        let maxpc = self.max_priority;
        let loglines: BTreeSet<LogLine> = self
            .log_files
            .par_iter_mut()
            .flat_map(|f| {
                f.extract_loglines(fc, uc, &rc, minpc, maxpc)
                    .into_par_iter()
            })
            .collect();

        match lines {
            Some(n) =>
            {
                #[allow(unused_must_use)]
                for logline in loglines.iter().rev().take(n).rev().into_iter() {
                    stdoutln!("{}", logline);
                }
            }
            _ =>
            {
                #[allow(unused_must_use)]
                for logline in loglines {
                    stdoutln!("{}", logline);
                }
            }
        }
    }

    pub fn watch_logs(&mut self) {
        let (tx, rx) = channel();
        let mut watcher = raw_watcher(tx).unwrap();
        watcher.watch(LOG_DIR, RecursiveMode::Recursive).unwrap();
        loop {
            let (path, event) = match rx.recv() {
                Ok(RawEvent {
                    path: Some(path),
                    op: Ok(op),
                    cookie: _,
                }) => (Some(path), Some(op)),
                Ok(_) => (None, None),
                Err(_) => (None, None),
            };
            if let Some(e) = event {
                // TODO: should I handle Op::CLOSE_WRITE or Op::WRITE?
                // println!("{:?} on {:?}", e, path);
                if e == Op::WRITE {
                    self.handle_event(&path.unwrap());
                }
            }
        }
    }

    fn handle_event(&mut self, path: &Path) {
        // TODO: refactor this hard
        for entry in &mut self.log_files {
            let path_name = path.to_str();
            if let Some(path_name) = path_name {
                if entry.name == path_name {
                    let file = File::open(&path);
                    if let Ok(file) = file {
                        let meta = file.metadata();
                        let length = meta.unwrap().len();
                        if length >= entry.position {
                            let mut reader = BufReader::new(file);
                            let res = reader.seek(SeekFrom::Start(entry.position));
                            if res.is_ok() {
                                for line in reader.lines().flatten() {
                                    if self.cache.push(String::from(&line)) {
                                        let logline = LogLine::new(line);
                                        if logline.is_match(&self.re)
                                            && logline
                                                .has_priority(self.min_priority, self.max_priority)
                                        {
                                            println!("{}", logline);
                                        }
                                    }
                                }
                                entry.position = length;
                            }
                        } else {
                            entry.position = 0;
                            self.handle_event(path);
                            break;
                        }
                    }
                }
            }
        }
    }
}
