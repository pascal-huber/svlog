#[cfg_attr(feature = "cargo-clippy", allow(clippy::useless_attribute))]
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

mod cache;
mod cli;
mod logfile;
mod logline;
mod logprinter;
mod util;

use chrono::NaiveDateTime;
use clap::Parser;
use cli::Args;
use logfile::*;
use logprinter::*;
use pager::Pager;
use regex::Regex;
use std::path::PathBuf;
use util::*;

fn list_services() {
    for service in ALL_SERVICES.iter() {
        println!(" - {}", service);
    }
}

fn main() {
    let args = Args::parse();

    if args.list {
        list_services();
        std::process::exit(0);
    }

    let (from, until): (Option<NaiveDateTime>, Option<NaiveDateTime>) =
        match (args.boot, args.boot_offset) {
            (true, _) => boot_times(0),
            (_, Some(offset)) => boot_times(offset),
            _ => (None, None),
        };

    let re: Option<Regex> = build_regex(&args.filter);
    let paths: Vec<PathBuf> = file_paths(&args.services);
    let log_files: Vec<LogFile> = paths
        .iter()
        .map(|path| LogFile::new(path.to_str().unwrap()))
        .collect();

    let (min_priority, max_priority): (Option<u8>, Option<u8>) = match args.priority {
        Some((x, y)) => (x, y),
        _ => (None, None),
    };

    let mut printer = LogPrinter::new(
        log_files,
        re,
        args.jobs,
        from,
        until,
        min_priority,
        max_priority,
    );

    if !(args.no_pager || args.follow) {
        Pager::new().setup();
    }

    match args.lines {
        Some(n) if n.unwrap() == 0 => printer.jump_to_end(),
        Some(n) => printer.print_logs(n),
        _ => printer.print_logs(None),
    }

    if args.follow {
        printer.watch_logs();
    }
}
