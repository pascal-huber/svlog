use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Args {
    /// Only show logs since last boot (same as --boot-offset 0 but allowed in
    /// combination with --follow)
    #[clap(short, long)]
    pub boot: bool,

    /// Follow the services for new logs
    #[clap(short, long)]
    pub follow: bool,

    /// List available services
    #[clap(short, long)]
    pub list: bool,

    /// Only show entries which match the pattern/regex
    #[clap(short = 'm', long = "match", required = false, value_name = "PATTERN")]
    pub filter: Option<String>,

    /// Don't show any past logs (only makes sense with --follow)
    #[clap(short, long, requires = "follow")]
    pub none: bool,

    /// Show logs of some old boot with offset <OFFSET> where an offset of 0 is
    /// the current boot, an offset of 1 the previous and so on.
    #[clap(
        short = 'o',
        long = "boot-offset",
        value_name = "OFFSET",
        conflicts_with = "follow"
    )]
    pub boot_offset: Option<usize>,

    /// Print to stdout and don't run a pager on the output.
    #[clap(short, long)]
    pub plain: bool,

    /// Services to log. If none are specified, all services will be logged
    #[clap()]
    pub services: Vec<String>,
}
