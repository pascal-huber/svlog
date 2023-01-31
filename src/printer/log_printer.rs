use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
};

use calm_io::{pipefail, stdoutln};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use pager::Pager;
use rayon::prelude::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use snafu::ResultExt;

use crate::{
    error::*,
    printer::{log_file::*, log_line::*},
    util::cache::*,
    LogFilterSettings, SvLogResult,
};

pub struct LogPrinter<'a> {
    log_dir: &'a str,
    log_files: Vec<LogFile<'a>>,
    cache: Cache<String>,
    log_settings: &'a LogFilterSettings,
}

impl<'a> LogPrinter<'a> {
    pub fn new(
        log_dir: &'a str,
        log_files: Vec<LogFile<'a>>,
        log_settings: &'a LogFilterSettings,
    ) -> LogPrinter<'a> {
        let cache: Cache<String> = Cache::new(20);
        LogPrinter {
            log_dir,
            log_files,
            cache,
            log_settings,
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
        let log_dir_path = Path::new(self.log_dir);
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher =
            RecommendedWatcher::new(tx, Config::default()).context(WatchFilesNotifySnafu {
                message: "Failed to create watcher".to_string(),
            })?;
        watcher
            .watch(log_dir_path.as_ref(), RecursiveMode::Recursive)
            .context(WatchFilesNotifySnafu {
                message: "Failed to create watcher".to_string(),
            })?;
        for res in rx {
            match res {
                Ok(event) => self.handle_event(&event)?,
                Err(e) => println!("watch error: {:?}", e),
            }
        }
        Ok(())
    }

    fn jump_to_end(&mut self) {
        for log_file in &mut self.log_files {
            log_file.jump_to_end();
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
            let skip_amount = if log_lines.len() < n {
                0
            } else {
                log_lines.len() - n
            };
            log_lines = log_lines.iter().skip(skip_amount).cloned().collect();
        }
        Ok(log_lines)
    }

    fn handle_event(&mut self, event: &notify::Event) -> SvLogResult<()> {
        if let notify::Event {
            kind: notify::event::EventKind::Modify(notify::event::ModifyKind::Data(_)),
            paths,
            attrs: _,
        } = event
        {
            for path in paths {
                self.handle_modified_path(path)?
            }
        }
        Ok(())
    }

    fn handle_modified_path(&mut self, path: &Path) -> SvLogResult<()> {
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
