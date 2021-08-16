use anyhow::{anyhow, Result};
use cargo_gccrs::Error;

/// Create a new `cargo` process with `cargo-gccrs` set as the RUSTC_WRAPPER environment
/// variable. This causes `cargo` to invoke this binary as a compiler, which we can
/// then use to give various options to `gccrs` instead of `rustc`.
pub fn spawn() -> Result<(), Error> {
    // Skip `cargo` and `gccrs` in the invocation. Since we spawn a new cargo command,
    // `cargo gccrs arg0 arg1` will become `cargo run arg0 arg1`
    let mut cargo_gccrs = std::process::Command::new("cargo")
        .env("RUSTC_WRAPPER", "gccrs-driver")
        .args(std::env::args().skip(2))
        .spawn()
        .map_err(|_| Error::WrapperLaunch)?;

    match cargo_gccrs
        .wait()
        .map_err(|_| Error::WrapperExitError)?
        .success()
    {
        true => Ok(()),
        false => Err(Error::WrapperExitError),
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let res = match args.get(1).map(String::as_str) {
        Some("gccrs") => spawn(),
        _ => Err(Error::InvocationError),
    };

    res.map_err(|e| anyhow!(e))
}
