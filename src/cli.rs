use crate::util::*;
use clap::ColorChoice;
use clap::Parser;
use std::error::Error;
use std::fmt;

static HELP_TEMPLATE: &str = "USAGE: {usage}\n{about}\n\n{all-args}";

#[derive(Debug)]
struct ServiceNotFoundError(String);
impl fmt::Display for ServiceNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for ServiceNotFoundError {}

fn parse_service(s: &str) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
    if ALL_SERVICES.contains(&s.to_string()) {
        return Ok(s.to_string());
    }
    return Err(Box::new(ServiceNotFoundError(format!(
        "Service \"{}\" not found",
        s
    ))));
}

#[derive(Parser, Debug)]
#[clap(about, version, author, color = ColorChoice::Never, help_template = HELP_TEMPLATE)]
pub struct Args {
    /// Only show logs since last boot (same as --boot-offset 0 but allowed in
    /// combination with --follow)
    #[clap(short, long, conflicts_with = "boot-offset")]
    pub boot: bool,

    /// Follow the services for new logs
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
    #[clap(short, long)]
    pub plain: bool,

    /// Services to log (all by default)
    #[clap(parse(try_from_str = parse_service))]
    pub services: Vec<String>,
}
