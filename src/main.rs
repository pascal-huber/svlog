#[macro_use]
extern crate lazy_static;

mod cli;
mod error;
mod printer;
mod util;

use std::path::PathBuf;

use clap::Parser;
use cli::Args;

use crate::{
    error::{SvLogError, SvLogResult},
    printer::{LogFilterSettings, LogPrinter},
    util::service::{all_services, file_paths, list_services, service_log_files},
};

fn main() {
    let args = Args::parse();
    if let Err(e) = try_main(&args) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}

fn try_main(args: &Args) -> SvLogResult<()> {
    if args.list {
        list_services(&args.log_dir);
        std::process::exit(0);
    }
    check_services(&args.log_dir, &args.services)?;

    let service_file_paths: Vec<PathBuf> = file_paths(&args.log_dir, &args.services);
    let log_files = service_log_files(&service_file_paths);
    let log_filter_settings = LogFilterSettings::from_args(args)?;
    let mut printer = LogPrinter::new(&args.log_dir, log_files, &log_filter_settings);
    let use_pager = !args.no_pager && !args.follow;
    printer.print_logs(args.jobs, use_pager, args.lines)?;
    if args.follow {
        printer.watch_logs()?;
    }
    Ok(())
}

fn check_services(log_dir: &str, services: &Vec<String>) -> SvLogResult<()> {
    let all_services = all_services(log_dir);
    for service in services {
        if !all_services.contains(service) {
            return Err(SvLogError::ServiceNotFoundError {
                service: service.clone(),
            });
        }
    }
    Ok(())
}
