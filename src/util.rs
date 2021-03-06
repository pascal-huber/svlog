use chrono::{Duration, NaiveDateTime};
use glob::glob;
use regex::{Regex, RegexBuilder};
use std::ops::Sub;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
static GLOB_ALL_FILES: &[&str] = &["/current", "/*.[su]"];
static GLOB_CURRENT_FILES: &[&str] = &["/current"];
pub static LOG_DIR: &str = "/var/log/socklog/";

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

pub fn file_paths(services: &[String], only_current: bool) -> Vec<PathBuf> {
    let file_globs = match only_current {
        true => GLOB_CURRENT_FILES,
        false => GLOB_ALL_FILES,
    };
    let mut service_globs = services.to_owned();
    if service_globs.is_empty() {
        service_globs.push(String::from("**"));
    }
    service_globs
        .iter()
        .flat_map(|g| service_file_paths(g, file_globs).into_iter())
        .collect()
}

pub fn set_boot_times(
    from: &mut Option<NaiveDateTime>,
    until: &mut Option<NaiveDateTime>,
    boot: bool,
    boot_offset: Option<usize>,
) {
    if boot {
        let bt = boot_times(0);
        *from = bt.0;
        *until = bt.1;
    }
    if let Some(offset) = boot_offset {
        let bt = boot_times(offset);
        *from = bt.0;
        *until = bt.1;
    }
}

fn boot_times(offset: usize) -> (Option<NaiveDateTime>, Option<NaiveDateTime>) {
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
    let boot_lines: Vec<&&str> = output_lines
        .iter()
        .filter(|x| x.contains("system boot"))
        .collect();

    if boot_lines.len() - 1 < offset {
        panic!("boot not found");
    }
    let from = NaiveDateTime::parse_from_str(&boot_lines[offset][22..41], DATE_FORMAT).unwrap();
    let mut until_result = NaiveDateTime::parse_from_str(&boot_lines[offset][50..69], DATE_FORMAT);

    // If no until time found, use subsequent boot time if possible
    if until_result.is_err() && offset >= 1 {
        let next_boot = NaiveDateTime::parse_from_str(&boot_lines[offset - 1][22..41], DATE_FORMAT);
        if let Ok(next_boot) = next_boot {
            until_result = Ok(next_boot.sub(Duration::nanoseconds(1)));
        }
    }

    match until_result {
        Ok(until) => (Some(from), Some(until)),
        _ => (Some(from), None),
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
