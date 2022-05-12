//! This module aims at abstracting the usage of `gccrs` via Rust code. This is a simple
//! wrapper around spawning a `gccrs` command with various arguments

use super::args::{Args, ArgsCollection, CrateType};
use super::{config::GccrsConfig, env_args::EnvArgs, rustc_args::RustcArgs, Error, Result};

use std::convert::TryFrom;
use std::process::{Command, ExitStatus};

pub struct Gccrs;

/// Internal type to use when executing commands. The errors should be converted into
/// [`Error`]s using the `?` operator.
type CmdResult<T = ()> = std::io::Result<T>;

impl Gccrs {
    fn install() -> Result {
        // TODO: Remove this once `gccrs` gets stable releases or packages
        Err(Error::Installation)
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
        which::which("gccrs").is_ok()
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

    /// Spawn a `gccrs` command with arguments extracted from a `rustc` invokation
    fn compile(gccrs_args: &Args) -> Result {
        let exit_status = Gccrs::spawn_with_args(&gccrs_args.as_args()?)?;

        match exit_status.success() {
            false => Err(Error::Compile),
            true => Ok(()),
        }
    }

    /// Execute a callback if necessary, based on the different options used to build
    /// the `gccrs` arguments
    fn maybe_callback(gccrs_args: &Args) -> Result {
        // If we are ordered to generate a static library, call `ar` after compiling
        // the object files
        if gccrs_args.crate_type() == CrateType::StaticLib {
            Gccrs::generate_static_lib(gccrs_args)?;
        }

        Ok(())
    }

    fn translate_and_compile(args: &[String]) -> Result {
        let rustc_args = RustcArgs::try_from(args)?;
        let gccrs_args = ArgsCollection::try_from(&rustc_args)?;

        for arg_set in gccrs_args.data().iter() {
            Gccrs::compile(arg_set)?;
            Gccrs::maybe_callback(arg_set)?;
        }

        Ok(())
    }

    fn generate_static_lib(args: &Args) -> Result {
        let output_file = args
            .output_file()
            .to_str()
            .expect("Cannot handle non-unicode filenames yet");

        let mut ar_args = vec![
            String::from("rcs"), // Create the archive and add the files to it
            output_file.to_owned(),
            args.object_file_name().into_os_string().into_string()?,
        ];

        if let Some(mut extra_ar_args) = EnvArgs::Ar.as_args() {
            ar_args.append(&mut extra_ar_args);
        }

        Command::new("ar").args(ar_args).status()?;

        Ok(())
    }

    /// Convert arguments given to `rustc` into valid arguments for `gccrs`
    pub fn compile_with_rust_args(args: &[String]) -> Result {
        let first_rustc_arg = args.get(2);

        match first_rustc_arg.map(|s| s.as_str()) {
            // FIXME: Is that true? Should we use getopts and just parse it and convert
            // it as well?
            // If rustc is invoked with stdin as input, then it's simply to print the
            // configuration option in our case, since we are compiling a rust project
            // with files and crates
            Some("-") => Gccrs::cfg_print(),
            _ => Gccrs::translate_and_compile(args),
        }
    }
}
