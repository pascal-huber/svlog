use std::{
    ops::Sub,
    process::{Command, Stdio},
};

use chrono::{Duration, NaiveDateTime};
use chrono_tz::Tz;
use snafu::{ensure, ResultExt};

use crate::{error::*, util::settings::*};

pub fn local_tz() -> SvLogResult<Tz> {
    let path = std::fs::read_link("/etc/localtime")
        .context(CommandOutputSnafu {
            message: "Could not read \"/etc/localtime\"".to_string(),
        })?
        .into_os_string()
        .into_string()
        .unwrap_or_else(|_| "".to_string());
    let zone_str = path.split("zoneinfo/").last().unwrap_or("");
    let zone = zone_str.parse::<Tz>();
    ensure!(
        zone.is_ok(),
        TimeZoneSnafu {
            message: format!("Could not find timezone \"{zone_str}\""),
        }
    );
    Ok(zone.unwrap())
}

pub fn boot_times(offset: usize) -> SvLogResult<(Option<NaiveDateTime>, Option<NaiveDateTime>)> {
    let boot_time_lines = boot_time_lines()?;
    ensure!(boot_time_lines.len() > offset, BootTimeNotFoundSnafu {});
    let mut result = boot_time_tuple(boot_time_lines[offset].clone())?;
    if result.1.is_none() && offset >= 1 {
        let next_boot = boot_time_tuple(boot_time_lines[offset - 1].clone())?;
        if let Some(startup_time) = next_boot.0 {
            result.1 = Some(startup_time.sub(Duration::nanoseconds(1)))
        }
    }
    Ok(result)
}

fn boot_time_lines() -> SvLogResult<Vec<String>> {
    let output = Command::new("last")
        .arg("-a")
        .arg("--time-format")
        .arg("iso")
        .env("TZ", "UTC")
        .stdout(Stdio::piped())
        .output()
        .context(CommandOutputSnafu {
            message: "failed to retrieve output of `TZ=UTC last -a --time-format iso`",
        })?;
    let output_str = String::from_utf8(output.stdout).unwrap();
    let boot_lines: Vec<String> = output_str
        .split('\n')
        .filter(|x| x.contains("system boot"))
        .map(|x| x.to_string())
        .collect();
    Ok(boot_lines)
}

fn boot_time_tuple(line: String) -> SvLogResult<(Option<NaiveDateTime>, Option<NaiveDateTime>)> {
    ensure!(line.len() > 41, BootTimeNotFoundSnafu {});
    let from =
        NaiveDateTime::parse_from_str(&line[22..41], DATE_FORMAT).context(ParsingChronoSnafu {
            line: &line[22..41],
        })?;
    let until = if line.len() > 69 {
        NaiveDateTime::parse_from_str(&line[50..69], DATE_FORMAT).ok()
    } else {
        None
    };
    Ok((Some(from), until))
}
