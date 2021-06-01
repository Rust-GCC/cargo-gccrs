//! This module aims at abstracting the usage of `gccrs` via Rust code. This is a simple
//! wrapper around spawning a `gccrs` command with various arguments

mod config;
use config::GccrsConfig;

use std::process::{Command, ExitStatus, Stdio};

pub struct Gccrs;

pub type Result<T = ()> = std::io::Result<T>;

impl Gccrs {
    fn install() -> Result {
        unreachable!("cargo-gccrs cannot install gccrs yet")
    }

    /// Output fake information because gccrs does not implement the required feature
    /// yet. This function is only available in debug mode, not release, in order for
    /// users to be aware of the limitations.
    fn fake_output(s: &str) {
        if cfg!(release) {
            unreachable!(
                "gccrs cannot yet produce the following, expected output: {}",
                s
            );
        } else {
            println!("{}", s);
        }
    }

    fn dump_config() -> Result<ExitStatus> {
        Command::new("gccrs")
            .arg("-x")
            .arg("rs")
            .arg("-frust-dump-target_options")
            .arg("-")
            .status()
    }

    fn is_installed() -> bool {
        // On UNIX, we can check using the `command` built-in command, but it's not cross-platform.
        // The slow but sure way to do this is to just try and spawn a `gccrs` process.
        match Command::new("gccrs")
            .stderr(Stdio::null())
            .arg("-v")
            .status()
        {
            // We can check that gccrs exited successfully when passed with the version argument
            Ok(exit_status) => exit_status.success(),
            // If the command failed to spawn, then gccrs probably isn't in the path or installed
            Err(_) => false,
        }
    }

    /// Install `gccrs` if the binary is not found in the path
    pub fn maybe_install() -> Result {
        match Gccrs::is_installed() {
            true => Ok(()),
            false => Gccrs::install(),
        }
    }

    /// Convert arguments given to `rustc` into valid arguments for `gccrs`
    pub fn handle_rust_args() -> Result {
        Gccrs::fake_output(r#"___"#);
        Gccrs::fake_output(r#"lib___.rlib"#);
        Gccrs::fake_output(r#"lib___.so"#);
        Gccrs::fake_output(r#"lib___.so"#);
        Gccrs::fake_output(r#"lib___.a"#);
        Gccrs::fake_output(r#"lib___.so"#);

        Gccrs::dump_config()?;
        GccrsConfig::display()
    }
}
