use chrono::{Duration, NaiveDateTime};
use glob::glob;
use regex::{Regex, RegexBuilder};
use std::ops::Sub;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub static LOG_DIR: &str = "/var/log/socklog/";
static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
static GLOB_ALL_FILES: &[&str] = &["/current", "/*.[su]"];

lazy_static! {
    pub static ref ALL_SERVICES: Vec<String> = all_services();
}

pub fn priority_value(s: &str) -> Option<u8> {
    match s {
        "0" => Some(0),
        "emerg" => Some(0),
        "1" => Some(1),
        "alert" => Some(1),
        "2" => Some(2),
        "crit" => Some(2),
        "3" => Some(3),
        "err" => Some(3),
        "4" => Some(4),
        "warn" => Some(4),
        "5" => Some(5),
        "notice" => Some(5),
        "6" => Some(6),
        "info" => Some(6),
        "7" => Some(7),
        "debug" => Some(7),
        _ => None,
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

pub fn build_regex(pattern: &Option<String>) -> Option<Regex> {
    pattern.as_ref().map(|pattern| {
        RegexBuilder::new(&pattern[..])
            .case_insensitive(true)
            .build()
            .unwrap()
    })
}

pub fn file_paths(services: &[String]) -> Vec<PathBuf> {
    let mut service_globs = services.to_owned();
    if service_globs.is_empty() {
        service_globs.push(String::from("**"));
    }
    service_globs
        .iter()
        .flat_map(|g| service_file_paths(g, GLOB_ALL_FILES).into_iter())
        .collect()
}

fn boot_time_tuple(line: &str) -> (Option<NaiveDateTime>, Option<NaiveDateTime>) {
    let from = NaiveDateTime::parse_from_str(&line[22..41], DATE_FORMAT);
    let until = NaiveDateTime::parse_from_str(&line[50..69], DATE_FORMAT);
    match (from, until) {
        (Ok(x), Ok(y)) => (Some(x), Some(y)),
        (Ok(x), Err(_)) => (Some(x), None),
        _ => (None, None),
    }
}

fn boot_time_lines() -> Vec<(Option<NaiveDateTime>, Option<NaiveDateTime>)> {
    let output = Command::new("last")
        .arg("-a")
        .arg("--time-format")
        .arg("iso")
        .env("TZ", "UTC")
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let output_str = String::from_utf8(output.stdout).unwrap();
    let output_lines: Vec<&str> = output_str.split('\n').collect();
    output_lines
        .iter()
        .filter(|x| x.contains("system boot"))
        .map(|x| boot_time_tuple(x))
        .collect()
}

pub fn boot_times(offset: usize) -> (Option<NaiveDateTime>, Option<NaiveDateTime>) {
    let boot_times = boot_time_lines();
    if boot_times.len() - 1 < offset {
        panic!("boot not found");
    }
    match boot_times[offset] {
        // NOTE: If no shutdown time is found (e.g. system crash), the
        // subsequent boot time is used.
        (Some(from), None) if offset > 0 => {
            let until = boot_times[offset - 1].0;
            match until {
                Some(x) => (Some(from), Some(x.sub(Duration::nanoseconds(1)))),
                _ => (Some(from), until),
            }
        }
        _ => boot_times[offset],
    }
}

pub fn service_file_paths(service_glob: &str, file_globs: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for glob_str_ext in file_globs {
        let glob_str = String::from(LOG_DIR) + service_glob + glob_str_ext;
        for entry in glob(&glob_str[..])
            .expect("Failed to read glob pattern")
            .flatten()
        {
            files.push(entry);
        }
    }
    files
}
