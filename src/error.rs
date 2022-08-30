use std::sync::mpsc::RecvError;

use snafu::Snafu;

// same error, different source
// https://github.com/shepmaster/snafu/issues/123

pub type SvLogResult<T> = Result<T, SvLogError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SvLogError {
    #[snafu(display("BootTimeNotFound"))]
    BootTimeNotFound {},

    #[snafu(display("TimeZoneError {}", message))]
    TimeZoneError { message: String },

    #[snafu(display("ComandOutputError {}", message))]
    CommandOutput {
        message: String,
        source: std::io::Error,
    },

    #[snafu(display("PrintLinesError: failed to print lines"))]
    PrintLines { source: std::io::Error },

    #[snafu(display("Failed to parse timestamp in: {line}"))]
    ParsingChronoError {
        line: String,
        source: chrono::ParseError,
    },

    #[snafu(display("ParsingLogLineError: {line}"))]
    ParsingLogLineError { line: String },

    #[snafu(display("OpenFileError: {path}"))]
    OpenFile {
        path: String,
        source: std::io::Error,
    },

    #[snafu(display("WatchFilesError: {message}"))]
    WatchFiles {
        message: String,
        source: std::io::Error,
    },

    #[snafu(display("WatchFilesRecvError: {message}"))]
    WatchFilesRecv { message: String, source: RecvError },

    #[snafu(display("WatchFilesNotifyError: {message}"))]
    WatchFilesNotify {
        message: String,
        source: notify::Error,
    },
}
