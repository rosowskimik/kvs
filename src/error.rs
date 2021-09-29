use std::io;

use thiserror::Error;

/// A specialized `Result` type for kvs operations.
///
/// This type is broadly used across `kvs` for any operation that may produce an error.
pub type Result<T> = std::result::Result<T, KvsError>;

/// The error type for `kvs` operations.
#[derive(Error, Debug)]
pub enum KvsError {
    /// IO Error
    #[error("io error")]
    Io(#[from] io::Error),

    /// Serde Error
    #[error("serde error")]
    Serde(#[from] serde_json::Error),

    /// Missing Logfile
    #[error("tried to access missing logfile: {0}.log")]
    MissingLogfile(usize),

    /// Unexpected Command
    #[error("unexpected command (expected: {expected}, got: {got})")]
    UnexpectedCommand {
        /// Expexted command kind
        expected: &'static str,
        /// Actual command kind
        got: &'static str,
    },
}
