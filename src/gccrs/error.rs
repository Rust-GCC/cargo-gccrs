//! Error type of the `gccrs` abstraction

use std::io::Error as IoError;

use getopts::Fail;
use thiserror::Error;

/// Public enum of possible errors
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid argument given to `gccrs`
    #[error("Invalid argument given to `gccrs`: {0}")]
    InvalidArg(String),
    /// Invalid config line dumped when executing `gccrs -frust-dump-*`
    #[error("Invalid configuration returned when executing `gccrs -frust-dump-*`")]
    InvalidCfgDump,
    /// Error when compiling a program using `gccrs`
    #[error("Error when compiling project using `gccrs`")]
    CompileError,
    /// IO Error when executing a `gccrs` command
    #[error("IO Error when executing `gccrs`: {0}")]
    CommandError(#[from] IoError),
}

// If parsing the options using `getopts` fail, then it was because an unhandled argument
// was given to the translation unit
impl From<Fail> for Error {
    fn from(arg_fail: Fail) -> Self {
        Error::InvalidArg(arg_fail.to_string())
    }
}
