mod gccrs;

use anyhow::{anyhow, Result};
use gccrs::{Error, Gccrs};
use thiserror::Error;

/// Error type around spawning cargo-gccrs as a `rustc` wrapper
#[derive(Error, Debug)]
enum WrapperError {
    /// Error when invoking `cargo-gccrs`
    #[error("Error when invoking `cargo-gccrs`")]
    InvocationError,
    /// Error when initially launching `cargo-gccrs` as a wrapper to `rustc`
    #[error("Error when launching `cargo-gccrs` as a `rustc` wrapper")]
    Launch,
    /// The `cargo-gccrs` process did not complete successfully
    #[error("`cargo-gccrs` did not complete succesfully")]
    ExitError,
    /// Error when handling `rustc` arguments and compiling with `gccrs`
    #[error("{0}")] // Display is already handled by gccrs::Error
    GccrsError(Error),
}

fn spawn_as_wrapper() -> Result<(), WrapperError> {
    // Skip `cargo` and `gccrs` in the invocation. Since we spawn a new cargo command,
    // `cargo gccrs arg0 arg1` will become `cargo run arg0 arg1`
    let mut cargo_gccrs = std::process::Command::new("cargo")
        .env("RUSTC_WRAPPER", "cargo-gccrs")
        .args(std::env::args().skip(2))
        .spawn()
        .map_err(|_| WrapperError::Launch)?;

    match cargo_gccrs
        .wait()
        .map_err(|_| WrapperError::ExitError)?
        .success()
    {
        true => Ok(()),
        false => Err(WrapperError::ExitError),
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    Gccrs::maybe_install().expect("gccrs should be installed");

    let first_arg = args.get(1).expect("Invalid arguments");

    let res = match first_arg.as_str() {
        "gccrs" => spawn_as_wrapper(),
        "rustc" => Gccrs::handle_rust_args(&args).map_err(WrapperError::GccrsError),
        _ => Err(WrapperError::InvocationError),
    };

    res.map_err(|e| anyhow!(e))
}
