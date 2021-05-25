//! This module aims at abstracting the usage of `gccrs` via Rust code. This is a simple
//! wrapper around spawning a `gccrs` command with various arguments

use std::process::Command;

pub struct Gccrs;

type Result<T = ()> = std::io::Result<T>;

impl Gccrs {
    fn install() -> Result {
        unreachable!("cargo-gccrs cannot install gccrs yet")
    }

    fn is_installed() -> bool {
        // On UNIX, we can check using the `command` built-in command, but it's not cross-platform.
        // The slow but sure way to do this is to just try and spawn a `gccrs` process. Since
        // we only have to do this once
        match Command::new("gccrs").arg("-v").status() {
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
            false => Gccrs::install()
        }
    }

}
