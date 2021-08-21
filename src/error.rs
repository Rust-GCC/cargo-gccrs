//! Error type of the `gccrs` abstraction

use std::ffi::OsString;
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
    Compile,
    /// IO Error when executing a `gccrs` command
    #[error("IO Error when executing `gccrs`: {0}")]
    Command(#[from] IoError),
    /// Error when dealing with UTF-8 strings
    #[error("Error when dealing with UTF-8 strings")]
    Utf8(Option<OsString>),
    /// Error when invoking `cargo-gccrs`
    #[error("Error when invoking `cargo-gccrs`")]
    Invocation,
    /// The `gccrs` compiler is not present in your path
    #[error("`gccrs` must be installed")]
    Installation,
    /// Error when initially launching `cargo-gccrs` as a wrapper to `rustc`
    #[error("Error when launching `cargo-gccrs` as a `rustc` wrapper")]
    WrapperLaunch,
    /// The `cargo-gccrs` process did not complete successfully
    #[error("`cargo-gccrs` did not complete succesfully")]
    WrapperExit,
}

// If parsing the options using `getopts` fail, then it was because an unhandled argument
// was given to the translation unit
impl From<Fail> for Error {
    fn from(arg_fail: Fail) -> Self {
        Error::InvalidArg(arg_fail.to_string())
    }
}

/// UTF-8 errors happen when dealing with paths that are not UTF-8 encoded. For now,
/// `cargo-gccrs` cannot handle them
impl From<OsString> for Error {
    fn from(s: OsString) -> Self {
        Error::Utf8(Some(s))
    }
}
