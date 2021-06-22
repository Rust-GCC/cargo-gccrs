//! Error type of the `gccrs` abstraction

use std::io::Error as IoError;

use getopts::Fail;

/// Public enum of possible errors
#[derive(Debug)]
pub enum Error {
    /// Invalid argument given to `gccrs`
    InvalidArg(String),
    /// Invalid config line dumped when executing `gccrs -frust-dump-*`
    InvalidCfgDump,
    /// Error when compiling a program using `gccrs`
    CompileError,
    /// IO Error when executing a `gccrs` command
    CommandError(IoError),
}

// IO Error should be kept for better debugging
impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::CommandError(e)
    }
}

// If parsing the options using `getopts` fail, then it was because an unhandled argument
// was given to the translation unit
impl From<Fail> for Error {
    fn from(arg_fail: Fail) -> Self {
        Error::InvalidArg(arg_fail.to_string())
    }
}
