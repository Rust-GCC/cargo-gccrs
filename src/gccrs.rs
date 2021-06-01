//! This module aims at abstracting the usage of `gccrs` via Rust code. This is a simple
//! wrapper around spawning a `gccrs` command with various arguments

use std::process::{Command, ExitStatus, Stdio};

pub struct Gccrs;

type Result<T = ()> = std::io::Result<T>;

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
        // The slow but sure way to do this is to just try and spawn a `gccrs` process. Since
        // we only have to do this once
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
    pub fn handle_rust_args() {
        Gccrs::fake_output(r#"___"#);
        Gccrs::fake_output(r#"lib___.rlib"#);
        Gccrs::fake_output(r#"lib___.so"#);
        Gccrs::fake_output(r#"lib___.so"#);
        Gccrs::fake_output(r#"lib___.a"#);
        Gccrs::fake_output(r#"lib___.so"#);
        Gccrs::fake_output(r#"debug_assertions"#);
        Gccrs::fake_output(r#"panic="unwind""#);
        Gccrs::fake_output(r#"proc_macro"#);
        Gccrs::fake_output(r#"target_arch="x86_64""#);
        Gccrs::fake_output(r#"target_endian="little""#);
        Gccrs::fake_output(r#"target_env="gnu""#);
        Gccrs::fake_output(r#"target_family="unix""#);
        Gccrs::fake_output(r#"target_feature="fxsr""#);
        Gccrs::fake_output(r#"target_feature="sse""#);
        Gccrs::fake_output(r#"target_feature="sse2""#);
        Gccrs::fake_output(r#"target_has_atomic="16""#);
        Gccrs::fake_output(r#"target_has_atomic="32""#);
        Gccrs::fake_output(r#"target_has_atomic="64""#);
        Gccrs::fake_output(r#"target_has_atomic="8""#);
        Gccrs::fake_output(r#"target_has_atomic="ptr""#);
        Gccrs::fake_output(r#"target_has_atomic_equal_alignment="16""#);
        Gccrs::fake_output(r#"target_has_atomic_equal_alignment="32""#);
        Gccrs::fake_output(r#"target_has_atomic_equal_alignment="64""#);
        Gccrs::fake_output(r#"target_has_atomic_equal_alignment="8""#);
        Gccrs::fake_output(r#"target_has_atomic_equal_alignment="ptr""#);
        Gccrs::fake_output(r#"target_has_atomic_load_store="16""#);
        Gccrs::fake_output(r#"target_has_atomic_load_store="32""#);
        Gccrs::fake_output(r#"target_has_atomic_load_store="64""#);
        Gccrs::fake_output(r#"target_has_atomic_load_store="8""#);
        Gccrs::fake_output(r#"target_has_atomic_load_store="ptr""#);
        Gccrs::fake_output(r#"target_os="linux""#);
        Gccrs::fake_output(r#"target_pointer_width="64""#);
        Gccrs::fake_output(r#"target_thread_local"#);
        Gccrs::fake_output(r#"target_vendor="unknown""#);
        Gccrs::fake_output(r#"unix"#);
    }
}
