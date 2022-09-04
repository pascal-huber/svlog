use std::path::{Path, PathBuf};

use glob::glob;

use super::settings::GLOB_ALL_FILES;
use crate::printer::LogFile;

pub fn service_log_files(service_file_paths: &[PathBuf]) -> Vec<LogFile> {
    service_file_paths
        .iter()
        .map(|path| LogFile::new(path.to_str().unwrap()))
        .collect()
}

pub fn file_paths(log_dir: &str, services: &[String]) -> Vec<PathBuf> {
    let mut service_globs = services.to_owned();
    if service_globs.is_empty() {
        service_globs.push(String::from("**"));
    }
    service_globs
        .iter()
        .flat_map(|g| service_file_paths(log_dir, g, GLOB_ALL_FILES).into_iter())
        .collect()
}

pub fn list_services(log_dir: &str) {
    for service in all_services(log_dir).iter() {
        println!(" - {}", service);
    }
}

pub fn all_services(log_dir: &str) -> Vec<String> {
    let mut services = Vec::new();
    let log_dir = Path::new(&log_dir);
    for entry in log_dir.read_dir().expect("read_dir call failed").flatten() {
        let service_dir = entry.path();
        if service_dir.is_dir() {
            let filename = service_dir.file_name().unwrap().to_str().unwrap();
            services.push(filename.to_string());
        }
    }
    services
}

fn service_file_paths(log_dir: &str, service_glob: &str, file_globs: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for glob_str_ext in file_globs {
        let glob_str = String::from(log_dir) + service_glob + glob_str_ext;
        for entry in glob(&glob_str[..])
            .expect("Failed to read glob pattern")
            .flatten()
        {
            files.push(entry);
        }
    }
    files
}
