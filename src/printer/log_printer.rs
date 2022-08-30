use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
    sync::mpsc::channel,
};

use calm_io::{pipefail, stdoutln};
use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};
use pager::Pager;
use rayon::prelude::*;
use snafu::prelude::*;

use super::LogFilterSettings;
use crate::{
    error::*,
    printer::{log_file::*, log_line::*},
    util::{cache::*, settings::*},
    SvLogResult,
};

pub struct LogPrinter<'a> {
    log_files: Vec<LogFile<'a>>,
    cache: Cache<String>,
    log_settings: &'a LogFilterSettings,
}

impl<'a> LogPrinter<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(log_files: Vec<LogFile<'a>>, log_settings: &'a LogFilterSettings) -> LogPrinter<'a> {
        let cache: Cache<String> = Cache::new(20);
        LogPrinter {
            log_files,
            cache,
            log_settings,
        }
    }

    pub fn jump_to_end(&mut self) {
        for log_file in &mut self.log_files {
            log_file.jump_to_end();
        }
    }

    pub fn print_logs(
        &mut self,
        jobs: usize,
        use_pager: bool,
        lines: Option<usize>,
    ) -> SvLogResult<()> {
        if let Some(lines) = lines {
            if lines == 0 {
                self.jump_to_end();
                return Ok(());
            }
        }
        let log_lines = self.retrieve_log_lines(jobs, lines)?;
        let formatted_log_lines = log_lines
            .par_iter()
            .map(|log_line| log_line.format_with_tz(&self.log_settings.tz))
            .collect::<Result<BTreeSet<String>, _>>();
        match formatted_log_lines {
            Ok(lines) => self
                .print_lines(use_pager, lines)
                .context(PrintLinesSnafu {}),
            Err(e) => Err(e),
        }
    }

    pub fn watch_logs(&mut self) -> SvLogResult<()> {
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

    #[pipefail]
    fn print_lines(&mut self, use_pager: bool, lines: BTreeSet<String>) -> std::io::Result<()> {
        if use_pager {
            Pager::new().setup();
        }
        for line in lines {
            stdoutln!("{line}")?;
        }
        Ok(())
    }

    fn retrieve_log_lines(
        &mut self,
        jobs: usize,
        lines: Option<usize>,
    ) -> SvLogResult<BTreeSet<LogLine>> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .unwrap();
        let mut log_lines: BTreeSet<LogLine> = self
            .log_files
            .par_iter_mut()
            .map(|f| f.extract_loglines(self.log_settings))
            .collect::<SvLogResult<BTreeSet<_>>>()?
            .into_par_iter()
            .flatten()
            .collect();
        if let Some(n) = lines {
            log_lines = log_lines
                .iter()
                .skip(log_lines.len() - n)
                .cloned()
                .collect();
        }
        Ok(log_lines)
    }

    fn handle_write_event(&mut self, path: &Path) -> SvLogResult<()> {
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
                self.process_new_lines(i, &file, file_length)?;
            }
        }
        Ok(())
    }

    fn process_new_lines(
        &mut self,
        log_file_index: usize,
        file: &File,
        file_position: u64,
    ) -> SvLogResult<()> {
        let mut reader = BufReader::new(file);
        let log_file: &mut LogFile = &mut self.log_files[log_file_index];
        reader
            .seek(SeekFrom::Start(log_file.position))
            .context(WatchFilesSnafu {
                message: "Failed to seek file position.",
            })?;
        for line in reader.lines().flatten() {
            if self.cache.push(String::from(&line)) {
                let log_line = LogLine::new(line)?;
                if log_line.is_match(&self.log_settings.re)
                    && log_line.has_priority(
                        &self.log_settings.min_priority,
                        &self.log_settings.max_priority,
                    )
                {
                    println!("{}", log_line.format_with_tz(&self.log_settings.tz)?);
                }
            }
        }
        log_file.position = file_position;
        Ok(())
    }
}
