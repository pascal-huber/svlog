extern crate pager;
extern crate sys_info;

use chrono::NaiveDateTime;
use clap::{load_yaml, App};
use calm_io::stdoutln;
use glob::glob;
use pager::Pager;
use regex::{Regex, RegexBuilder};
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};
use sys_info::boottime;

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

fn file_paths(services: &Option<Vec<&str>>, only_current: bool) -> Vec<PathBuf> {
    let file_globs = match only_current {
        true => GLOB_CURRENT_FILES,
        false => GLOB_ALL_FILES,
    };
    let service_globs = match services {
        Some(s) => s.as_slice(),
        _ => &["**"],
    };
    let mut files = Vec::new();
    for service in service_globs {
        for glob_str_ext in file_globs {
            let glob_str = String::from(LOG_DIR) + service + glob_str_ext;
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
    boottime: Option<NaiveDateTime>,
    re: &Option<Regex>,
) {
    let file = File::open(file);
    if let Ok(file) = file {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let logline = create_logline(line);
            // TODO: can we make this nicer (e.g. https://github.com/rust-lang/rfcs/pull/2497)?
            if let Some(boottime) = boottime {
                if boottime > logline.date {
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

// TODO: check if the boot time is exact enough
fn boot_time() -> NaiveDateTime {
    let now = SystemTime::now();
    let uptime = boottime().unwrap();

    let duration = Duration::new(uptime.tv_sec.try_into().unwrap(), 0);
    let boottime = now.checked_sub(duration).unwrap();

    let secs = boottime.duration_since(SystemTime::UNIX_EPOCH);
    NaiveDateTime::from_timestamp(secs.unwrap().as_secs().try_into().unwrap(), 0)
}

fn build_regex(pattern: Option<&str>) -> Option<Regex> {
    pattern.map(|x| RegexBuilder::new(x).case_insensitive(true).build().unwrap())
}

fn show_logs(services: &Option<Vec<&str>>, since_boot: bool, pattern: Option<&str>) {
    let files = file_paths(services, false);
    let re = build_regex(pattern);

    let mut loglines: BTreeSet<LogLine> = BTreeSet::new();
    let boottime: Option<NaiveDateTime> = match since_boot {
        true => Some(boot_time()),
        _ => None,
    };
    for file in files {
        extract_loglines(file, &mut loglines, boottime, &re);
    }
    for logline in loglines {
        // TODO: find out if this is okay or risky.
        stdoutln!("{} {}", logline.date_str, logline.content);
    }
}

// TODO: make file watching better
fn watch_changes(services: &Option<Vec<&str>>, pattern: Option<&str>) {
    let files = file_paths(services, true);

    let mut cmd: String = String::from("tail -Fq -n0 ");
    for file in files {
        let x = file.to_str();
        cmd = cmd + x.unwrap() + " ";
    }
    if let Some(pattern) = pattern {
        cmd += "| grep -i -h --line-buffered \"";
        cmd += pattern;
        cmd += "\"";
    }

    // TODO: this doesn't work reliably as the messages are not sorted
    cmd += " | uniq ";

    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
}

fn read_services(services: Option<clap::Values>) -> Option<Vec<&str>> {
    if let Some(services) = services {
        let wanted_services: Vec<&str> = services.collect();
        let all_services = all_services();
        wanted_services.iter().all(|value| {
            all_services.contains(&value.to_string()) || panic!("service \"{}\" not found", value)
        });
        return Some(wanted_services);
    }
    None
}

fn main() {
    let cli = load_yaml!("cli.yaml");
    let args = App::from(cli).get_matches();

    if args.is_present("list") {
        list_services();
        std::process::exit(0);
    }

    if !args.is_present("plain") && !args.is_present("follow") && !args.is_present("none") {
        Pager::new().setup();
    }

    let services = read_services(args.values_of("services"));
    let pattern = args.value_of("match");
    if !args.is_present("none") {
        show_logs(&services, args.is_present("boot"), pattern);
    }
    if args.is_present("follow") || args.is_present("none") {
        watch_changes(&services, pattern);
    }
}
