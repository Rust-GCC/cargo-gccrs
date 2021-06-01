mod gccrs;

use gccrs::Gccrs;

fn spawn_as_wrapper() {
    // Skip `cargo` and `gccrs` in the invocation. Since we spawn a new cargo command,
    // `cargo gccrs arg0 arg1` will become `cargo run arg0 arg1`
    let mut cargo_gccrs = std::process::Command::new("cargo")
        .env("RUSTC_WRAPPER", "cargo-gccrs")
        .args(std::env::args().skip(2))
        .spawn()
        .expect("Unable to launch cargo-gccrs as RUSTC_WRAPPER");

    cargo_gccrs
        .wait()
        .expect("Subprocess cargo-gccrs didn't complete properly");
}

fn main() {
    #[cfg(not(release))]
    dbg!(std::env::args());

    Gccrs::maybe_install().expect("gccrs should be installed");

    let first_arg = std::env::args().nth(1);

    match first_arg.as_deref() {
        Some("gccrs") => spawn_as_wrapper(),
        Some("rustc") => Gccrs::handle_rust_args(),
        _ => eprintln!(
            "cargo-gccrs should not be invoked directly. Use the `cargo gccrs <...>` subcommand"
        ),
    }
}
