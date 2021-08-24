//! The role of the driver is to convert arguments that would originally be passed to
//! `rustc` into valid `gccrs` arguments, and then compile the cargo project.
//! The driver is invoked from the wrapper, and should not be called directly.

use anyhow::{anyhow, Result};
use cargo_gccrs::{Error, Gccrs};

fn main() -> Result<()> {
    Gccrs::maybe_install()?;

    let args: Vec<String> = std::env::args().collect();

    let res = match args.get(1).map(String::as_str) {
        Some("rustc") => Gccrs::compile_with_rust_args(&args),
        _ => Err(Error::Invocation),
    };

    res.map_err(|e| anyhow!(e))
}
