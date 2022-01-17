use crate::util::*;
use clap::ColorChoice;
use clap::Parser;
use std::error::Error;
use std::fmt;

static HELP_TEMPLATE: &str = "USAGE: {usage}\n{about}\n\n{all-args}";

#[derive(Parser, Debug)]
#[clap(about, version, author, color = ColorChoice::Never, help_template = HELP_TEMPLATE)]
pub struct Args {
    /// Only show logs since last boot (same as --boot-offset 0 but allowed in
    /// combination with --follow)
    #[clap(short, long, conflicts_with = "boot-offset")]
    pub boot: bool,

    /// Follow the services for new logs after printing the present logs
    #[clap(short, long)]
    pub follow: bool,

    /// List available services and exit
    #[clap(short, long)]
    pub list: bool,

    /// Only show entries which match the pattern/regex
    #[clap(short = 'm', long = "match", required = false, value_name = "PATTERN")]
    pub filter: Option<String>,

    /// Like --follow but don't show any past logs
    #[clap(short, long)]
    pub none: bool,

    // TODO: do we really need the option to limit the number of jobs?
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

    /// Print to stdout and don't run a pager on the output
    #[clap(long = "no-pager")]
    pub no_pager: bool,

    /// Max priority level emerg(0), alert(1), crit(2), err(3), warn(4),
    /// notice(5), info(6), debug(7) or a priority range (e.g. "crit..3").
    #[clap(short, long, parse(try_from_str = parse_priorities))]
    pub priority: Option<(Option<u8>, Option<u8>)>,

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
) -> Result<(Option<u8>, Option<u8>), Box<dyn Error + Send + Sync + 'static>> {
    let priorities: Vec<&str> = s.split("..").collect();
    println!("{:?}", priorities);
    let y: Option<u8> = priority_value(priorities.last().unwrap());
    let mut x: Option<u8> = None;
    if priorities.len() == 2 {
        x = priority_value(priorities.first().unwrap());
    }
    match (x, y) {
        (Some(x_val), Some(y_val)) if x_val <= y_val => Ok((x, y)),
        (None, Some(_)) if priorities.len() == 1 => Ok((x, y)),
        _ => Err(Box::new(InvalidArgError(format!(
            "Invalid priority \"{}\"",
            s
        )))),
    }
}
