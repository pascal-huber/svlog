use snafu::Snafu;
use std::sync::mpsc::RecvError;

// same error, different source
// https://github.com/shepmaster/snafu/issues/123

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SvLogError {
    #[snafu(display("BootTimeNotFound"))]
    BootTimeNotFound {},

    #[snafu(display("ComandOutputError {}", message))]
    CommandOutput {
        message: String,
        source: std::io::Error,
    },

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
