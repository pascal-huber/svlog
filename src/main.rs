#[macro_use]
extern crate lazy_static;

mod cli;
mod error;
mod macros;
mod printer;
mod util;

use std::path::PathBuf;

use chrono::{FixedOffset, Local, NaiveDateTime, Offset, TimeZone, Utc};
use clap::Parser;
use cli::Args;
use error::*;
use regex::Regex;
use util::{boottime::*, regex::*, service::*};

use crate::printer::{LogFile, LogPrinter};

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
    let re: Option<Regex> = build_regex(&args.filter);
    let paths: Vec<PathBuf> = file_paths(&args.services);
    let log_files: Vec<LogFile> = paths
        .iter()
        .map(|path| LogFile::new(path.to_str().unwrap()))
        .collect();
    let use_pager = !args.no_pager && !args.follow;
    let (time_offset, since_time_utc, until_time_utc) = extract_time_info(args)?;
    let mut printer = LogPrinter::new(
        log_files,
        re,
        args.jobs,
        since_time_utc,
        until_time_utc,
        time_offset,
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

// TODO: check if datetime and timezone info can be extracted with clap
fn extract_time_info(
    args: &Args,
) -> Result<(FixedOffset, Option<NaiveDateTime>, Option<NaiveDateTime>), SvLogError> {
    let time_offset = if args.utc {
        Utc::now().offset().fix()
    } else {
        *Local::now().offset()
    };
    let mut since_time_utc: Option<NaiveDateTime> = args
        .since
        .map(|since| time_offset.from_local_datetime(&since).unwrap().naive_utc());
    let mut until_time_utc: Option<NaiveDateTime> = args
        .until
        .map(|until| time_offset.from_local_datetime(&until).unwrap().naive_utc());
    (since_time_utc, until_time_utc) = match (args.boot, args.boot_offset) {
        (true, _) => boot_times(0)?,
        (_, Some(offset)) => boot_times(offset)?,
        _ => (since_time_utc, until_time_utc),
    };
    Ok((time_offset, since_time_utc, until_time_utc))
}
