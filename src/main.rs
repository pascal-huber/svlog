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
use snafu::ensure;
use util::service::*;

use crate::printer::LogPrinter;

fn main() {
    let args = Args::parse();
    if let Err(e) = try_main(&args) {
        eprintln!("{}", e);
    }
}

fn try_main(args: &Args) -> SvLogResult<()> {
    check_args(args)?;
    if args.list {
        list_services();
        std::process::exit(0);
    }

    let service_file_paths: Vec<PathBuf> = file_paths(&args.services);
    let log_files = service_log_files(&service_file_paths);
    let log_filter_settings = LogFilterSettings::from_args(args)?;
    let mut printer = LogPrinter::new(log_files, &log_filter_settings);
    let use_pager = !args.no_pager && !args.follow;
    printer.print_logs(args.jobs, use_pager, args.lines)?;
    if args.follow {
        printer.watch_logs()?;
    }
    Ok(())
}

// TODO: try checking argument combinations with clap
fn check_args(args: &Args) -> SvLogResult<()> {
    if let Some(Some(boot)) = args.boot {
        ensure!(
            boot == 0 || !args.follow,
            InvalidArgCombinationSnafu {
                message: "--follow can only be used with --boot for OFFSET \"0\"".to_string()
            }
        )
    }
    Ok(())
}
