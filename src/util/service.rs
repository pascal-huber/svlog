use std::path::{Path, PathBuf};

use glob::glob;

use crate::util::settings::*;

lazy_static! {
    pub static ref ALL_SERVICES: Vec<String> = all_services();
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

pub fn list_services() {
    for service in ALL_SERVICES.iter() {
        println!(" - {}", service);
    }
}

fn all_services() -> Vec<String> {
    let mut services = Vec::new();
    let log_dir = Path::new(LOG_DIR);
    for entry in log_dir.read_dir().expect("read_dir call failed").flatten() {
        let service_dir = entry.path();
        if service_dir.is_dir() {
            let filename = service_dir.file_name().unwrap().to_str().unwrap();
            services.push(filename.to_string());
        }
    }
    services
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
