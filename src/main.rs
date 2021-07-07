use anyhow::{anyhow, Result};
use cargo_gccrs::{Error, Gccrs, Wrapper};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    Gccrs::maybe_install()?;

    let first_arg = args.get(1).map(String::as_str);

    let res = match first_arg {
        Some("gccrs") => Wrapper::spawn(),
        Some("rustc") => Gccrs::handle_rust_args(&args),
        _ => Err(Error::InvocationError),
    };

    res.map_err(|e| anyhow!(e))
}
