#[macro_use]
extern crate lazy_static;

mod cli;
mod error;
mod printer;
mod util;

use std::path::PathBuf;

use clap::Parser;
use cli::Args;
use error::*;
use printer::LogFilterSettings;
use util::service::*;

use crate::printer::LogPrinter;

fn main() {
    let args = Args::parse();
    if let Err(e) = try_main(&args) {
        eprintln!("An error occurred: {}", e);
    }
}

fn try_main(args: &Args) -> SvLogResult<()> {
    if args.list {
        list_services();
        std::process::exit(0);
    }
    let service_file_paths: Vec<PathBuf> = file_paths(&args.services);
    let log_files = service_log_files(&service_file_paths);
    let log_filter_settings = LogFilterSettings::from_args(args)?;
    let mut printer = LogPrinter::new(log_files, &log_filter_settings);
    let use_pager = !args.no_pager && !args.follow;
    printer.print_logs(args.jobs, use_pager, args.lines.unwrap_or(None))?;
    if args.follow {
        printer.watch_logs()?;
    }
    Ok(())
}
