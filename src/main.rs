use anyhow::{anyhow, Result};
use cargo_gccrs::{spawn_as_wrapper, Gccrs, Error};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    Gccrs::maybe_install().expect("gccrs should be installed");

    let first_arg = args.get(1).expect("Invalid arguments");

    let res = match first_arg.as_str() {
        "gccrs" => spawn_as_wrapper(),
        "rustc" => Gccrs::handle_rust_args(&args),
        _ => Err(Error::InvocationError),
    };

    res.map_err(|e| anyhow!(e))
}
