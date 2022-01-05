use chrono::NaiveDateTime;
use std::process::{Command, Stdio};

static DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

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
