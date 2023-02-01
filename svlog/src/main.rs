mod printer;

use std::path::PathBuf;

use clap::Parser;
use svlog_cli::Args;
use svlog_util::{
    services::{check_services, file_paths, list_services},
    SvLogResult,
};

use crate::printer::{LogFile, LogFilterSettings, LogPrinter};

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

pub fn service_log_files(service_file_paths: &[PathBuf]) -> Vec<LogFile> {
    service_file_paths
        .iter()
        .map(|path| LogFile::new(path.to_str().unwrap()))
        .collect()
}
