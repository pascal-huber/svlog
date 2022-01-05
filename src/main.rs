extern crate pager;
extern crate sys_info;

mod cli;
mod util;
use calm_io::stdoutln;
use chrono::NaiveDateTime;
use clap::Parser;
use cli::Args;
use glob::glob;
use pager::Pager;
use regex::{Regex, RegexBuilder};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use util::*;

static LOG_DIR: &str = "/var/log/socklog/";
// TODO: find out why there are only 5 digits at the end of socklog timestamps
static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6f";
static GLOB_ALL_FILES: &[&str] = &["/current", "/*.[su]"];
static GLOB_CURRENT_FILES: &[&str] = &["/current"];

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct LogLine {
    date: NaiveDateTime,
    date_str: String,
    content: String,
}

fn create_logline(line: String) -> LogLine {
    let date_str: &str = &line[..25];
    let date = NaiveDateTime::parse_from_str(date_str, DATE_FORMAT).unwrap();
    let content_str: &str = &line[26..];
    LogLine {
        date,
        date_str: date_str.to_string(),
        content: content_str.to_string(),
    }
}

fn all_services() -> Vec<String> {
    let mut services = Vec::new();
    let path = Path::new(LOG_DIR);
    for entry in path.read_dir().expect("read_dir call failed").flatten() {
        let p = entry.path();
        let filename = p.file_name().unwrap().to_str().unwrap();
        services.push(filename.to_string());
    }
    services
}

fn list_services() {
    for service in all_services() {
        println!(" - {}", service);
    }
}

fn file_paths(services: &[String], only_current: bool) -> Vec<PathBuf> {
    let file_globs = match only_current {
        true => GLOB_CURRENT_FILES,
        false => GLOB_ALL_FILES,
    };
    let mut service_globs = services.to_owned();
    if service_globs.is_empty() {
        service_globs.push(String::from("**"));
    }
    let mut files = Vec::new();
    for service in service_globs {
        for glob_str_ext in file_globs {
            let glob_str = String::from(LOG_DIR) + &service[..] + glob_str_ext;
            for entry in glob(&glob_str[..])
                .expect("Failed to read glob pattern")
                .flatten()
            {
                files.push(entry);
            }
        }
    }
    files
}

fn extract_loglines(
    file: PathBuf,
    loglines: &mut BTreeSet<LogLine>,
    from: Option<NaiveDateTime>,
    until: Option<NaiveDateTime>,
    re: &Option<Regex>,
) {
    let file = File::open(file);
    if let Ok(file) = file {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let logline = create_logline(line);
            // TODO: can we make this nicer (e.g. https://github.com/rust-lang/rfcs/pull/2497)?
            if let Some(from) = from {
                if from > logline.date {
                    continue;
                }
            };
            if let Some(until) = until {
                if until <= logline.date {
                    continue;
                }
            };
            if let Some(re) = re {
                if !re.is_match(&logline.content[..]) {
                    continue;
                }
            };
            loglines.insert(logline);
        }
    }
}

fn build_regex(pattern: &Option<String>) -> Option<Regex> {
    pattern.as_ref().map(|pattern| {
        RegexBuilder::new(&pattern[..])
            .case_insensitive(true)
            .build()
            .unwrap()
    })
}

fn show_logs(
    services: &[String],
    from: Option<NaiveDateTime>,
    until: Option<NaiveDateTime>,
    pattern: &Option<String>,
) {
    let files = file_paths(services, false);
    let re = build_regex(pattern);

    let mut loglines: BTreeSet<LogLine> = BTreeSet::new();

    for file in files {
        extract_loglines(file, &mut loglines, from, until, &re);
    }
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

fn check_services(services: &[String]) {
    let all_services = all_services();
    services.iter().all(|value| {
        all_services.contains(&value.to_string()) || panic!("service \"{}\" not found", value)
    });
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

    check_services(&args.services);

    if !(args.plain || args.follow || args.none) {
        Pager::new().setup();
    }
    if !args.none {
        show_logs(&args.services, from, until, &args.filter);
    }
    if args.follow || args.none {
        watch_changes(&args.services, &args.filter);
    }
}
