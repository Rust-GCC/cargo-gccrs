use anyhow::{anyhow, Result};
use cargo_gccrs::{Error, Wrapper};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let res = match args.get(1).map(String::as_str) {
        Some("gccrs") => Wrapper::spawn(),
        _ => Err(Error::InvocationError),
    };

    res.map_err(|e| anyhow!(e))
}
