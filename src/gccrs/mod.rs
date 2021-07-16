//! This module aims at abstracting the usage of `gccrs` via Rust code. This is a simple
//! wrapper around spawning a `gccrs` command with various arguments

mod args;
mod config;
mod error;
mod rustc_options;

use args::GccrsArgs;
use config::GccrsConfig;
use error::Error;
use rustc_options::RustcOptions;

use std::process::{Command, ExitStatus, Stdio};

pub struct Gccrs;

pub type Result<T = ()> = std::result::Result<T, Error>;

/// Internal type to use when executing commands. The errors should be converted into
/// [`Error`]s using the `?` operator.
type CmdResult<T = ()> = std::io::Result<T>;

impl Gccrs {
    fn install() -> Result {
        // TODO: Remove this once `gccrs` gets stable releases or packages
        unreachable!("cargo-gccrs cannot install gccrs yet")
    }

    /// Output fake information because gccrs does not implement the required feature
    /// yet. This function is only available in debug mode, not release, in order for
    /// users to be aware of the limitations.
    fn fake_output(s: &str) {
        println!("{}", s);
    }

    fn dump_config() -> CmdResult<ExitStatus> {
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

    fn cfg_print() -> Result {
        // FIXME: The output needs to be adapted based on the target triple. For example,
        // produce a .dll on windows, etc etc
        Gccrs::fake_output(r#"___"#);
        Gccrs::fake_output(r#"lib___.rlib"#);
        Gccrs::fake_output(r#"lib___.so"#);
        Gccrs::fake_output(r#"lib___.so"#);
        Gccrs::fake_output(r#"lib___.a"#);
        Gccrs::fake_output(r#"lib___.so"#);

        Gccrs::dump_config()?;
        let config = GccrsConfig::new()?;

        println!("{}", config);

        Ok(())
    }

    fn spawn_with_args(args: &[String]) -> CmdResult<ExitStatus> {
        Command::new("gccrs").args(args).status()
    }

    fn compile(args: &[String]) -> Result {
        let gccrs_args = GccrsArgs::from_rustc_args(args)?;

        for arg_set in gccrs_args.into_iter() {
            let exit_status = Gccrs::spawn_with_args(&arg_set.as_args())?;
            if !exit_status.success() {
                return Err(Error::CompileError);
            }

            if let Some(callback) = arg_set.callback() {
                callback(&arg_set)?
            }
        }

        Ok(())
    }

    /// Convert arguments given to `rustc` into valid arguments for `gccrs`
    pub fn handle_rust_args(args: &[String]) -> Result {
        let first_rustc_arg = args.get(2);

        match first_rustc_arg.map(|s| s.as_str()) {
            // FIXME: Is that true? Should we use getopts and just parse it and convert
            // it as well?
            // If rustc is invoked with stdin as input, then it's simply to print the
            // configuration option in our case, since we are compiling a rust project
            // with files and crates
            Some("-") => Gccrs::cfg_print(),
            _ => Gccrs::compile(args),
        }
    }
}
