use std::{
    error::Error,
    fmt,
    ops::{Add, Sub},
};

use chrono::{Duration, NaiveDateTime, Utc};
use clap::{ColorChoice, Parser};

use crate::{
    printer::LogPriority,
    util::{regex, service::*},
};

static HELP_TEMPLATE: &str = "USAGE: {usage}\n{about}\n\n{all-args}";
static CLI_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Parser, Debug)]
#[clap(
    about,
    author,
    color = ColorChoice::Never,
    help_template = HELP_TEMPLATE,
    term_width = 80,
    version
)]
pub struct Args {
    /// Only show logs since last boot (short for --boot-offset 0 but allowed in
    /// combination with --follow)
    #[clap(short, long, conflicts_with = "boot-offset")]
    pub boot: bool,

    /// Follow the services for new logs after printing the present logs
    #[clap(short, long)]
    pub follow: bool,

    /// List available services and exit
    #[clap(short, long)]
    pub list: bool,

    /// Only show entries which match the regular expression <REGEX>
    #[clap(short = 'm', long = "match", required = false, value_name = "REGEX")]
    pub filter: Option<String>,

    /// Limit the number of lines shown. <N> may be a positive integer or "all".
    /// If --follow is used, a default value of 10 is used.
    #[clap(
        short = 'n',
        long = "lines",
        value_name = "N",
        default_value_if("follow", None, Some("10")),
        default_value("all"),
        parse(try_from_str = parse_lines),
        hide_default_value = true,
    )]
    pub lines: Option<Option<usize>>,

    /// Number parallel jobs to process log files, by default number of logical
    /// processors (rayon's default)
    #[clap(short, long, default_value = "0")]
    pub jobs: usize,

    /// Show logs of some old boot with offset <OFFSET> where an offset of 0 is
    /// the current boot, an offset of 1 the previous and so on
    #[clap(
        short = 'o',
        long = "boot-offset",
        value_name = "OFFSET",
        conflicts_with_all = &["follow", "boot"],
    )]
    pub boot_offset: Option<usize>,

    /// Just print to stdout and don't pipe the output into a pager
    #[clap(long = "no-pager")]
    pub no_pager: bool,

    /// Specify the priority (e.g. "warn") or a range of priorities
    /// (e.g. "warn..5") to display. A priority can be specified either as text
    /// or number. Available priorities: emerg(0), alert(1), crit(2), err(3),
    /// warn(4), notice(5), info(6), debug(7).
    #[clap(short, long, parse(try_from_str = parse_priorities), default_value = "0..7")]
    pub priority: (LogPriority, LogPriority),

    /// Services to log (all by default)
    #[clap(parse(try_from_str = parse_service))]
    pub services: Vec<String>,

    /// Only consider logs from this time on forward. Possible values: "today",
    /// "yesterday", "YYYY-MM-DD HH:MM:SS", "YYYY-MM-DD HH:MM", "YYYY-MM-DD",
    /// "HH:MM:SS", "HH:MM". If no date is specified, "today" is assumed. If no
    /// hour/minute/second is specified, 0 is assumed. The timestamps are in
    /// locatime unless the --utc option is set.
    #[clap(
        short,
        long,
        parse(try_from_str = parse_ndt_since),
        conflicts_with_all = &["boot", "boot-offset"]
    )]
    pub since: Option<NaiveDateTime>,

    /// Only consider logs until (and including) this time. Possible values:
    /// "today", "yesterday", "YYYY-MM-DD HH:MM:SS", "YYYY-MM-DD HH:MM",
    /// "YYYY-MM-DD", "HH:MM:SS", "HH:MM". If no date is specified, "today" is
    /// assumed. If no hour/minute/second is specified, 0 is assumed. The
    /// timestamps are in locatime unless the --utc option is set.
    #[clap(
        short,
        long,
        parse(try_from_str = parse_ndt_until),
        conflicts_with_all = &["boot", "boot-offset"]
    )]
    pub until: Option<NaiveDateTime>,

    // Use UTC for timestamps instead of localtime
    #[clap(long = "utc")]
    pub utc: bool,
}

#[derive(Debug)]
struct InvalidArgError(String);
impl fmt::Display for InvalidArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for InvalidArgError {}

fn parse_lines(s: &str) -> Result<Option<Option<usize>>, Box<dyn Error + Send + Sync + 'static>> {
    if s == "all" {
        return Ok(None);
    }
    let n = s.parse::<usize>();
    match n {
        Ok(n) => Ok(Some(Some(n))),
        _ => Err(Box::new(InvalidArgError(format!(
            "Invalid line number \"{}\"",
            s
        )))),
    }
}

fn parse_service(s: &str) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
    if ALL_SERVICES.contains(&s.to_string()) {
        return Ok(s.to_string());
    }
    Err(Box::new(InvalidArgError(format!(
        "Service \"{}\" not found",
        s
    ))))
}

fn parse_priorities(
    s: &str,
) -> Result<(LogPriority, LogPriority), Box<dyn Error + Send + Sync + 'static>> {
    let priorities: Vec<&str> = s.split("..").collect();
    let return_value = match priorities.len() {
        1 => {
            let p = LogPriority::from_str(priorities.first().unwrap());
            p.map(|p| (p, p))
        }
        2 => {
            let p1 = *priorities.first().unwrap();
            let priority_1 = match p1 {
                "" => Some(LogPriority::min()),
                _ => LogPriority::from_str(p1),
            };
            let p2 = *priorities.last().unwrap();
            let priority_2 = match p2 {
                "" => Some(LogPriority::max()),
                _ => LogPriority::from_str(p2),
            };
            match (priority_1, priority_2) {
                (Some(priority_1), Some(priority_2)) => Some((priority_1, priority_2)),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(return_value) = return_value {
        Ok(return_value)
    } else {
        Err(Box::new(InvalidArgError(format!(
            "Invalid priority \"{}\"",
            s
        ))))
    }
}

// until-time represents the last point in time included in the logs
// since-time represents the first point in time included in the logs
//
// value                        since   until
// today               today 00:00:00   + 1.day - 1.ns
// yesterday       yesterday 00:00:00   + 1.day - 1.ns
// xxxx-yy-zz     xxxx-yy-zz 00:00:00   + 1.day - 1.ns
// xx:yy:zz            today xx:yy:zz   + 1.sec - 1.ns
// xx:yy               today xx:yy:00   + 1.min - 1.ns

#[derive(Debug)]
struct TimeError(String);
impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for TimeError {}

fn parse_ndt_until(s: &str) -> Result<NaiveDateTime, Box<dyn Error + Send + Sync + 'static>> {
    let time_point = parse_ndt(s)?;
    Ok(time_point.0.add(time_point.1).sub(Duration::nanoseconds(1)))
}

fn parse_ndt_since(s: &str) -> Result<NaiveDateTime, Box<dyn Error + Send + Sync + 'static>> {
    let time_point = parse_ndt(s)?;
    Ok(time_point.0)
}

fn parse_ndt(s: &str) -> Result<(NaiveDateTime, Duration), Box<dyn Error + Send + Sync + 'static>> {
    let today = Utc::now().naive_utc();
    let yesterday = today.sub(Duration::days(1));
    let today_date_str: String = today.format("%Y-%m-%d").to_string();
    let yesterday_date_str: String = yesterday.format("%Y-%m-%d").to_string();
    let (time_str, granularity) = match s {
        "today" => (today_date_str + " 00:00:00", Duration::days(1)),
        "yesterday" => (yesterday_date_str + " 00:00:00", Duration::days(1)),
        s if regex::RE_DATE.is_match(s) => (String::from(s) + " 00:00:00", Duration::days(1)),
        s if regex::RE_DATETIME_MIN.is_match(s) => (String::from(s) + ":00", Duration::minutes(1)),
        s if regex::RE_DATETIME_SEC.is_match(s) => (s.to_string(), Duration::seconds(1)),
        s if regex::RE_TIME_SEC.is_match(s) => (today_date_str + " " + s, Duration::seconds(1)),
        s if regex::RE_TIME_MIN.is_match(s) => {
            (today_date_str + " " + s + ":00", Duration::minutes(1))
        }
        _ => {
            return Err(Box::new(TimeError(format!(
                "Could not parse time \"{}\"",
                s
            ))))
        }
    };
    Ok((
        NaiveDateTime::parse_from_str(&time_str[..], CLI_DATE_FORMAT)?,
        granularity,
    ))
}
