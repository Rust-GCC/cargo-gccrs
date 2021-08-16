use anyhow::{anyhow, Result};
use cargo_gccrs::{Error, Gccrs};

fn main() -> Result<()> {
    Gccrs::maybe_install()?;

    let args: Vec<String> = std::env::args().collect();

    let res = match args.get(1).map(String::as_str) {
        Some("rustc") => Gccrs::handle_rust_args(&args),
        _ => Err(Error::InvocationError),
    };

    res.map_err(|e| anyhow!(e))
}
