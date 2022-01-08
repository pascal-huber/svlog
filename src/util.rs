use chrono::NaiveDateTime;
use glob::glob;
use regex::{Regex, RegexBuilder};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
static GLOB_ALL_FILES: &[&str] = &["/current", "/*.[su]"];
static GLOB_CURRENT_FILES: &[&str] = &["/current"];
// TODO: make LOG_DIR an env variable or argument
static LOG_DIR: &str = "/var/log/socklog/";

lazy_static! {
    pub static ref ALL_SERVICES: Vec<String> = all_services();
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

pub fn boot_times(offset: usize) -> (Option<NaiveDateTime>, Option<NaiveDateTime>) {
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
    let until_opt = NaiveDateTime::parse_from_str(&boot_lines[offset][50..69], DATE_FORMAT);
    match until_opt {
        Ok(until) => (Some(from), Some(until)),
        _ => (Some(from), None),
    }
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

fn service_file_paths(service_glob: &str, file_globs: &[&str]) -> Vec<PathBuf> {
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
