mod gccrs;

use gccrs::{Error, Gccrs};

/// Error type around spawning cargo-gccrs as a `rustc` wrapper
enum WrapperError {
    /// Error when invoking `cargo-gccrs`
    InvocationError,
    /// Error when initially launching `cargo-gccrs` as a wrapper to `rustc`
    Launch,
    /// The `cargo-gccrs` process did not complete successfully
    ExitError,
    /// Error when handling `rustc` arguments and compiling with `gccrs`
    GccrsError(Error),
}

impl WrapperError {
    /// Exit code to return on errors
    const EXIT_CODE: i32 = 1;

    /// Display information regarding the error on `stderr` and exit with a proper error
    /// code
    fn handle(&self) {
        match self {
            WrapperError::InvocationError => eprintln!("cargo-gccrs should not be invoked directly. Use the `cargo gccrs <...>` subcommand"),
            WrapperError::Launch => eprintln!("Unable to launch cargo-gccrs as RUSTC_WRAPPER"),
            WrapperError::ExitError => eprintln!("Subprocess cargo-gccrs didn't complete successfully"),
            WrapperError::GccrsError(gccrs_err) => eprintln!("Couldn't execute gccrs properly: {:?}", gccrs_err),
        }

        std::process::exit(WrapperError::EXIT_CODE);
    }
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    Gccrs::maybe_install().expect("gccrs should be installed");

    let first_arg = args.get(1).expect("Invalid arguments");

    let res = match first_arg.as_str() {
        "gccrs" => spawn_as_wrapper(),
        "rustc" => Gccrs::handle_rust_args(&args).map_err(WrapperError::GccrsError),
        _ => Err(WrapperError::InvocationError),
    };

    if let Err(wrapper_error) = res {
        wrapper_error.handle();
    }
}
