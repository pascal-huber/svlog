use crate::printer::LogPriority;
use crate::util::service::*;
use clap::ColorChoice;
use clap::Parser;
use std::error::Error;
use std::fmt;

static HELP_TEMPLATE: &str = "USAGE: {usage}\n{about}\n\n{all-args}";

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
    // pub priority: Option<(Option<u8>, Option<u8>)>,
    /// Services to log (all by default)
    #[clap(parse(try_from_str = parse_service))]
    pub services: Vec<String>,
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
    return Err(Box::new(InvalidArgError(format!(
        "Service \"{}\" not found",
        s
    ))));
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
