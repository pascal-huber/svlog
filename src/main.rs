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

    let mut from: Option<NaiveDateTime> = None;
    let mut until: Option<NaiveDateTime> = None;
    set_boot_times(&mut from, &mut until, args.boot, args.boot_offset);

    let paths: Vec<PathBuf> = file_paths(&args.services, false);
    let re: Option<Regex> = build_regex(&args.filter);
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

    if !(args.plain || args.follow || args.none) {
        Pager::new().setup();
    }

    if !args.none {
        printer.print_logs();
    } else {
        printer.jump_to_end();
    }

    if args.follow || args.none {
        printer.watch_logs();
    }
}
