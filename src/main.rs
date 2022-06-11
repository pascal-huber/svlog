#[macro_use]
extern crate lazy_static;

mod cli;
mod error;
mod macros;
mod printer;
mod util;

use crate::printer::{LogFile, LogPrinter};
use chrono::NaiveDateTime;
use clap::Parser;
use cli::Args;
use error::*;
use regex::Regex;
use std::path::PathBuf;
use util::{boottime::*, regex::*, service::*};

fn main() {
    let args = Args::parse();
    if let Err(e) = try_main(&args) {
        eprintln!("An error occurred: {}", e);
    }
}

fn try_main(args: &Args) -> Result<(), SvLogError> {
    if args.list {
        list_services();
        std::process::exit(0);
    }
    let (from, until): (Option<NaiveDateTime>, Option<NaiveDateTime>) =
        match (args.boot, args.boot_offset) {
            (true, _) => boot_times(0)?,
            (_, Some(offset)) => boot_times(offset)?,
            _ => (None, None),
        };
    let re: Option<Regex> = build_regex(&args.filter);
    let paths: Vec<PathBuf> = file_paths(&args.services);
    let log_files: Vec<LogFile> = paths
        .iter()
        .map(|path| LogFile::new(path.to_str().unwrap()))
        .collect();
    let use_pager = !args.no_pager && !args.follow;
    let mut printer = LogPrinter::new(
        log_files,
        re,
        args.jobs,
        from,
        until,
        args.priority.0,
        args.priority.1,
        use_pager,
    );
    match args.lines {
        Some(n) if n.unwrap() == 0 => printer.jump_to_end(),
        Some(n) => printer.print_logs(n)?,
        _ => printer.print_logs(None)?,
    }
    if args.follow {
        printer.watch_logs()?;
    }
    Ok(())
}
