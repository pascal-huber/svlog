use std::{ops::Sub, process::Command};

use chrono::{Duration, NaiveDateTime};
use chrono_tz::Tz;
use snafu::{ensure, ResultExt};
use sysinfo::{System, SystemExt};

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

pub fn boot_times(
    offset: Option<usize>,
) -> SvLogResult<(Option<NaiveDateTime>, Option<NaiveDateTime>)> {
    match offset {
        Some(offset) if offset > 0 => {
            // FIXME: This only works for glibc
            let boot_times = get_boot_time_with_offset(offset)?;
            Ok(boot_times)
        }
        _ => {
            let since = get_last_boot_time()?;
            Ok((Some(since), None))
        }
    }
}

fn get_last_boot_time() -> SvLogResult<NaiveDateTime> {
    let sys = System::new();
    let boot_time_seconds = sys.boot_time();
    let boot_time = NaiveDateTime::from_timestamp(boot_time_seconds as i64, 0);
    Ok(boot_time)
}

fn get_boot_time_with_offset(
    offset: usize,
) -> SvLogResult<(Option<NaiveDateTime>, Option<NaiveDateTime>)> {
    // FIXME: on musl, "last" will throw an error as wtmp does not exist
    let output_result = Command::new("last")
        .arg("-a")
        .arg("--time-format")
        .arg("iso")
        .env("TZ", "UTC")
        .output();
    match output_result {
        Err(e) => Err(SvLogError::CommandOutputError {
            message: "failed to retrieve output of `TZ=UTC last -a --time-format iso`".to_string(),
            source: e,
        }),
        Ok(output) if !output.status.success() => Err(SvLogError::BootTimeNotFound {
            message: format!(
                "exit status {:?} for `TZ=UTC last -a --time-format iso`",
                output.status.code().unwrap()
            ),
        }),
        Ok(output) => {
            let output_str = String::from_utf8(output.stdout).unwrap();
            let boot_time_lines: Vec<String> = output_str
                .split('\n')
                .filter(|x| x.contains("system boot"))
                .map(|x| x.to_string())
                .collect();
            extract_boot_times(boot_time_lines, offset)
        }
    }
}

fn extract_boot_times(
    boot_time_lines: Vec<String>,
    offset: usize,
) -> SvLogResult<(Option<NaiveDateTime>, Option<NaiveDateTime>)> {
    assert!(offset >= 1);
    ensure!(
        boot_time_lines.len() > offset,
        BootTimeNotFoundSnafu {
            message: format!("couldn't find boot with offset {}", offset),
        }
    );
    let offset_line = &boot_time_lines[offset];
    ensure!(
        offset_line.len() >= 41,
        BootTimeNotFoundSnafu {
            message: format!("can't find valid timestamp on line: {}", offset_line),
        }
    );
    let from = Some(
        NaiveDateTime::parse_from_str(&offset_line[22..41], DATE_FORMAT).context(
            ParsingChronoSnafu {
                line: &offset_line[22..41],
            },
        )?,
    );
    let until = if offset_line.len() >= 69 {
        Some(
            NaiveDateTime::parse_from_str(&offset_line[50..69], DATE_FORMAT).context(
                ParsingChronoSnafu {
                    line: &offset_line[50..69],
                },
            )?,
        )
    } else {
        // NOTE: if no shutdown time is present, try to use the next boot time instead
        let prev_line = &boot_time_lines[offset - 1];
        Some(
            NaiveDateTime::parse_from_str(&prev_line[22..41], DATE_FORMAT)
                .context(ParsingChronoSnafu {
                    line: &offset_line[22..41],
                })?
                .sub(Duration::seconds(1)),
        )
    };
    Ok((from, until))
}
