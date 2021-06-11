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
    let args: Vec<String> = std::env::args().collect();

    Gccrs::maybe_install().expect("gccrs should be installed");

    let first_arg = args.get(1).expect("Invalid arguments");

    match first_arg.as_str() {
        "gccrs" => spawn_as_wrapper(),
        "rustc" => Gccrs::handle_rust_args(&args)
            .expect("cannot translate rustc arguments into gccrs ones"),
        _ => eprintln!(
            "cargo-gccrs should not be invoked directly. Use the `cargo gccrs <...>` subcommand"
        ),
    }
}
