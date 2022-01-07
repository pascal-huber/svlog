use clap::ColorChoice;
use clap::Parser;

static HELP_TEMPLATE: &str = "USAGE: {usage}\n{about}\n\n{all-args}";

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
    #[clap()]
    pub services: Vec<String>,
}
