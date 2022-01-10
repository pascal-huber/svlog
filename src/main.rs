#[cfg_attr(feature = "cargo-clippy", allow(clippy::useless_attribute))]
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

mod cli;
mod logline;
mod util;

use calm_io::stdoutln;
use chrono::NaiveDateTime;
use clap::Parser;
use cli::Args;
use logline::*;
use pager::Pager;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::process::{Command, Stdio};
use util::*;

fn list_services() {
    for service in ALL_SERVICES.iter() {
        println!(" - {}", service);
    }
}

fn show_logs(
    jobs: usize,
    services: &[String],
    from: Option<NaiveDateTime>,
    until: Option<NaiveDateTime>,
    pattern: &Option<String>,
) {
    let files = file_paths(services, false);
    let re = build_regex(pattern);

    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build_global()
        .unwrap();

    let loglines: BTreeSet<LogLine> = files
        .into_par_iter()
        .flat_map(|f| extract_loglines(f, from, until, &re).into_par_iter())
        .collect();

    #[allow(unused_must_use)]
    for logline in loglines {
        // TODO: find out if stdoutln! is okay or risky.
        stdoutln!("{} {}", logline.date_str, logline.content);
    }
}

// TODO: make file watching better
fn watch_changes(services: &[String], pattern: &Option<String>) {
    let files = file_paths(services, true);

    let mut cmd: String = String::from("tail -Fq -n0 ");
    for file in files {
        let x = file.to_str();
        cmd = cmd + x.unwrap() + " ";
    }
    if let Some(pattern) = pattern {
        cmd += "| grep -i -h --line-buffered \"";
        cmd += &pattern[..];
        cmd += "\"";
    }

    // TODO: this doesn't work reliably as the messages are not necessarily sorted
    cmd += " | uniq ";

    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
}

fn main() {
    let args = Args::parse();

    if args.list {
        list_services();
        std::process::exit(0);
    }

    let mut from: Option<NaiveDateTime> = None;
    let mut until: Option<NaiveDateTime> = None;
    if args.boot {
        let bt = boot_times(0);
        from = bt.0;
        until = bt.1;
    }
    if let Some(offset) = args.boot_offset {
        let bt = boot_times(offset);
        from = bt.0;
        until = bt.1;
    }

    if !(args.plain || args.follow || args.none) {
        Pager::new().setup();
    }
    if !args.none {
        show_logs(args.jobs, &args.services, from, until, &args.filter);
    }
    if args.follow || args.none {
        watch_changes(&args.services, &args.filter);
    }
}
