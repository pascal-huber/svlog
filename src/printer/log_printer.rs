use crate::error::*;
use crate::printer::log_file::*;
use crate::printer::log_line::*;
use crate::util::cache::*;

use crate::printer::LogPriority;
use crate::util::settings::*;
use chrono::NaiveDateTime;
use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};
use pager::Pager;
use rayon::prelude::*;
use regex::Regex;
use snafu::prelude::*;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::sync::mpsc::channel;

pub struct LogPrinter<'a> {
    log_files: Vec<LogFile<'a>>,
    cache: Cache<String>,
    re: Option<Regex>,
    jobs: usize,
    from: Option<NaiveDateTime>,
    until: Option<NaiveDateTime>,
    min_priority: LogPriority,
    max_priority: LogPriority,
    use_pager: bool,
}

impl<'a> LogPrinter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        log_files: Vec<LogFile<'a>>,
        re: Option<Regex>,
        jobs: usize,
        from: Option<NaiveDateTime>,
        until: Option<NaiveDateTime>,
        min_priority: LogPriority,
        max_priority: LogPriority,
        use_pager: bool,
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
            use_pager,
        }
    }

    pub fn jump_to_end(&mut self) {
        for log_file in &mut self.log_files {
            log_file.jump_to_end();
        }
    }

    pub fn print_logs(&mut self, lines: Option<usize>) -> Result<(), SvLogError> {
        let log_lines = self.retrieve_log_lines()?;
        if self.use_pager {
            Pager::new().setup();
        }
        let start = if let Some(n) = lines {
            log_lines.len() - n
        } else {
            0
        };
        // FIXME: allow only pipe errors, other erros should be delegated
        #[allow(unused_must_use)]
        for log_line in log_lines.iter().skip(start) {
            println!("{}", log_line);
        }
        Ok(())
    }

    pub fn watch_logs(&mut self) -> Result<(), SvLogError> {
        let (tx, rx) = channel();
        let mut watcher = raw_watcher(tx).context(WatchFilesNotifySnafu {
            message: "Failed to create watcher".to_string(),
        })?;
        watcher
            .watch(LOG_DIR, RecursiveMode::Recursive)
            .context(WatchFilesNotifySnafu {
                message: format!("Failed to start watching {LOG_DIR}"),
            })?;
        loop {
            let event = rx.recv().context(WatchFilesRecvSnafu {
                message: format!("Receiveing events failed for dir {LOG_DIR}"),
            })?;
            if let RawEvent {
                path: Some(path),
                op: Ok(Op::WRITE),
                cookie: _,
            } = event
            {
                self.handle_write_event(&path)?;
            }
        }
    }

    fn retrieve_log_lines(&mut self) -> Result<BTreeSet<LogLine>, SvLogError> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .build_global()
            .unwrap();
        let from_copy = self.from;
        let until_copy = self.until;
        let re_clone = self.re.clone();
        let min_priority_copy = self.min_priority;
        let max_priority_copy = self.max_priority;
        let log_lines: BTreeSet<LogLine> = self
            .log_files
            .par_iter_mut()
            .map(|f| {
                f.extract_loglines(
                    from_copy,
                    until_copy,
                    &re_clone,
                    min_priority_copy,
                    max_priority_copy,
                )
            })
            .collect::<Result<BTreeSet<_>, SvLogError>>()?
            .into_par_iter()
            .flatten()
            .collect();
        Ok(log_lines)
    }

    fn handle_write_event(&mut self, path: &Path) -> Result<(), SvLogError> {
        for i in 0..self.log_files.len() {
            let path_name = path.to_str().unwrap();
            if self.log_files[i].name == path_name {
                let file = File::open(&path).context(OpenFileSnafu {
                    path: format!("{:?}", path),
                })?;
                let file_length = file.metadata().unwrap().len();
                if file_length < self.log_files[i].position {
                    self.log_files[i].position = 0;
                }
                self.process_new_lines(i, file, file_length)?;
            }
        }
        Ok(())
    }

    fn process_new_lines(&mut self, i: usize, file: File, length: u64) -> Result<(), SvLogError> {
        let mut reader = BufReader::new(file);
        let log_file: &mut LogFile = &mut self.log_files[i];
        reader
            .seek(SeekFrom::Start(log_file.position))
            .context(WatchFilesSnafu {
                message: "Failed to seek file position.",
            })?;
        for line in reader.lines().flatten() {
            if self.cache.push(String::from(&line)) {
                let log_line = LogLine::new(line)?;
                if log_line.is_match(&self.re)
                    && log_line.has_priority(self.min_priority, self.max_priority)
                {
                    println!("{}", log_line);
                }
            }
        }
        log_file.position = length;
        Ok(())
    }
}
