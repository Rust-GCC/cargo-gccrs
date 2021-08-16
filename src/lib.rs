mod args;
mod config;
mod env_args;
mod error;
mod gccrs;
mod rustc_args;

pub use error::Error;
pub use gccrs::Gccrs;

pub type Result<T = ()> = std::result::Result<T, Error>;

/// Abstraction around spawning a new `cargo` process with `cargo-gccrs` set as a wrapper
/// to `rustc`
pub struct Wrapper;

impl Wrapper {
    /// Create a new `cargo` process with `cargo-gccrs` set as the RUSTC_WRAPPER environment
    /// variable. This causes `cargo` to invoke this binary as a compiler, which we can
    /// then use to give various options to `gccrs` instead of `rustc`.
    pub fn spawn() -> Result {
        // Skip `cargo` and `gccrs` in the invocation. Since we spawn a new cargo command,
        // `cargo gccrs arg0 arg1` will become `cargo run arg0 arg1`
        let mut cargo_gccrs = std::process::Command::new("cargo")
            .env("RUSTC_WRAPPER", "cargo-gccrs")
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
}
