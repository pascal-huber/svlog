use clap::{App, load_yaml};
use glob::glob;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use chrono::NaiveDateTime;
use std::collections::BTreeSet;

static LOG_DIR: &str = "/var/log/socklog/";
static DATE_FORMAT : &str = "%Y-%m-%dT%H:%M:%S%.6f";

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct LogLine {
    date: NaiveDateTime,
    date_str: String,
    content: String
}

fn create_logline(line : String) -> LogLine {
    let date_str : &str = &line[..25];
    let date = NaiveDateTime::parse_from_str(&date_str[..], DATE_FORMAT).unwrap();
    let content_str : &str = &line[26..];
    LogLine{
        date,
        date_str: date_str.to_string(),
        content: content_str.to_string(),
    }
}

fn list_services(){
    println!("Services:");
    let path = Path::new(LOG_DIR);
    for entry in path.read_dir().expect("read_dir call failed")  {
        if let Ok(entry) = entry {
            let p = entry.path();
            let filename = p.file_name().unwrap().to_str().unwrap();
            println!(" - {}", filename);
        }
    }
}

fn file_paths<'a>(services: Vec<&str>, globs: Vec<&str>, files: &mut Vec<PathBuf>){
    for service in services {
        for glob_str_ext in &globs {
            let glob_str = String::from(LOG_DIR) + service + glob_str_ext;
            for entry in glob(&glob_str[..]).expect("Failed to read glob pattern") {
                if let Ok(path) = entry {
                    files.push(path);
                }
            }
        }
    }
}

fn extract_loglines(file: PathBuf, loglines: &mut BTreeSet<LogLine>){
    let file = File::open(file);
    if let Ok(file) = file {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(content) = line {
                let logline = create_logline(content);
                loglines.insert(logline);
            }
        }
    }
}

fn show_logs(services: Vec<&str>){

    let mut files : Vec<PathBuf> = Vec::new();
    let glob_suffixes = ["/current", "/*[su]"].to_vec();
    file_paths(services, glob_suffixes, &mut files);


    // let mut loglines: Vec<LogLine> = Vec::new();
    let mut loglines : BTreeSet<LogLine> = BTreeSet::new();
    for file in files {
        extract_loglines(file, &mut loglines);
    }
    for logline in loglines {
        println!("{} {}", logline.date_str, logline.content);
    }
}

fn watch_changes(services: Vec<&str>){

    let mut files : Vec<PathBuf> = Vec::new();
    let glob_suffixes = ["/current"].to_vec();
    file_paths(services, glob_suffixes, &mut files);

    let mut cmd : String = String::from("tail -Fq -n0 ");
    for file in files {
        let x = file.to_str();
        cmd = cmd + x.unwrap() + " ";
    }

    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
}


fn main() {

    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    if matches.is_present("list"){
        list_services();
        std::process::exit(0);
    }

    let mut services: Vec<&str> = ["**"].to_vec();
    if matches.is_present("services"){
        services = matches.values_of("services").unwrap().collect();
    }

    if matches.is_present("follow"){
        watch_changes(services);
    } else {
        show_logs(services);
    }

}
