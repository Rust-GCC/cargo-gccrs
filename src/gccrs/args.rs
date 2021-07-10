//! This module interprets arguments given to `rustc` and transforms them into valid
//! arguments for `gccrs`.

use std::{env, path::PathBuf, process::Command};

use getopts::{Matches, Options};

use super::{Error, Result};

/// Crate types supported by `gccrs`
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CrateType {
    /// Binary application
    Bin,
    /// Dynamic library/Shared object
    DyLib,
    /// Statically linked library
    StaticLib,
    /// Remaining options, handled by `rustc` but not `gccrs`
    Unknown,
}

/// Get the corresponding [`CrateType`] from a given option to the `--crate-type`
/// option
impl<'a> From<&'a str> for CrateType {
    fn from(s: &'a str) -> CrateType {
        match s {
            "bin" => CrateType::Bin,
            "dylib" => CrateType::DyLib,
            "staticlib" => CrateType::StaticLib,
            _ => CrateType::Unknown,
        }
    }
}

fn format_output_filename(
    matches: &Matches,
    crate_type: CrateType,
) -> Result<(PathBuf, CrateType)> {
    // Return an [`Error::InvalidArg`] error if `--crate-name` or `out-dir` weren't
    // given as arguments at this point of the translation
    let crate_name = matches
        .opt_str("crate-name")
        .ok_or_else(|| Error::InvalidArg(String::from("no `--crate-name` provided")))?;
    let out_dir = matches
        .opt_str("out-dir")
        .ok_or_else(|| Error::InvalidArg(String::from("no `--out-dir` provided")))?;
    let c_options = matches.opt_strs("C");

    let mut output_file = PathBuf::from(&out_dir);

    // FIXME: Horrendous. We need to create a separate "C options" parser since we'll
    // probably use more than just `extra-filename`. Issue #6 on Rust-GCC/cargo-gccrs
    let extra_filename = c_options
        .iter()
        .filter_map(|c_opt| {
            let mut split = c_opt.split('=');

            if let Some("extra-filename") = split.next().as_deref() {
                split.next()
            } else {
                None
            }
        })
        .collect::<Vec<&str>>()
        .pop()
        .unwrap_or("");

    match crate_type {
        CrateType::Bin => output_file.push(&format!("{}{}", crate_name, extra_filename)),
        CrateType::DyLib => output_file.push(&format!("lib{}{}.so", crate_name, extra_filename)),
        CrateType::StaticLib => output_file.push(&format!("lib{}{}.a", crate_name, extra_filename)),
        _ => unreachable!(
            "gccrs cannot handle other crate types than bin, dylib or staticlib at the moment"
        ),
    }

    Ok((output_file, crate_type))
}

/// Add `.tmp.o` to the expected output filename. Since we will already have produced the
/// expected filename at this point, and we are likely currently converting it to a String
/// to spawn as an argument, this function can avoid taking a Path as parameter and returning
/// a PathBuf.
fn object_file_name(output_file: &str) -> String {
    format!("{}.tmp.o", output_file)
}

/// Structure used to represent arguments passed to `gccrs`. Convert them from `rustc`
/// arguments using [`GccrsArg::from_rustc_arg`]
pub struct GccrsArgs {
    source_files: Vec<String>,
    crate_type: CrateType,
    output_file: PathBuf,
    callback: Option<&'static dyn Fn(&GccrsArgs) -> Result>,
}

impl GccrsArgs {
    fn generate_parser() -> Options {
        let mut options = Options::new();
        options.optopt("", "crate-name", "Name of the crate to compile", "NAME");
        options.optopt("", "edition", "Rust edition to use", "YEAR");
        options.optopt("", "error-format", "Requested error format", "EXTENSION");
        options.optopt(
            "",
            "out-dir",
            "Directory in which to output generated files",
            "DIR",
        );
        options.optopt("", "emit", "Requested output to emit", "KIND");
        options.optopt("", "json", "JSON Rendering type", "RENDER");
        options.optmulti("C", "", "Extra compiler options", "OPTION[=VALUE]");
        options.optmulti(
            "L",
            "",
            "Add a directory to the library's search path",
            "KIND[=PATH]",
        );
        options.optmulti("", "crate-type", "Type of binary to output", "TYPE");

        options
    }

    fn generate_static_lib(args: &GccrsArgs) -> Result {
        let output_file = args
            .output_file
            .to_str()
            .expect("Cannot handle non-unicode filenames yet");

        let mut ar_args = vec![
            String::from("rcs"), // Create the archive and add the files to it
            output_file.to_owned(),
            object_file_name(&output_file),
        ];

        if let Some(mut extra_ar_args) = GccrsArgs::env_extra_args(GccrsArgs::AR_ENV_ARGS) {
            ar_args.append(&mut extra_ar_args);
        }

        Command::new("ar").args(ar_args).status()?;

        Ok(())
    }

    fn with_callback(self, function: &'static dyn Fn(&GccrsArgs) -> Result) -> GccrsArgs {
        GccrsArgs {
            callback: Some(function),
            ..self
        }
    }

    // Set a callback to the arguments if necessary
    fn maybe_with_callback(self) -> GccrsArgs {
        if self.crate_type == CrateType::StaticLib {
            self.with_callback(&GccrsArgs::generate_static_lib)
        } else {
            self
        }
    }

    fn new(source_files: &[String], crate_type: CrateType, output_file: PathBuf) -> GccrsArgs {
        GccrsArgs {
            source_files: Vec::from(source_files),
            crate_type,
            output_file,
            callback: None,
        }
    }

    // Execute an argument set's callback if present. Returns Ok if no callback was
    // present
    pub fn callback(&self) -> Option<&'static dyn Fn(&GccrsArgs) -> Result> {
        self.callback
    }

    /// Get the corresponding `gccrs` argument from a given `rustc` argument
    pub fn from_rustc_args(rustc_args: &[String]) -> Result<Vec<GccrsArgs>> {
        let options = GccrsArgs::generate_parser();

        // Parse arguments, skipping `cargo-gccrs` and `rustc` in the invocation
        let matches = options.parse(&rustc_args[2..])?;

        matches
            .opt_strs("crate-type")
            .iter()
            .map(|type_str| CrateType::from(type_str.as_str()))
            .map(|crate_type| format_output_filename(&matches, crate_type))
            .map(|result_tuple| {
                result_tuple.map(|(output_file, crate_type)| {
                    GccrsArgs::new(&matches.free, crate_type, output_file).maybe_with_callback()
                })
            })
            .collect()
    }

    const COMPILER_ENV_ARGS: &'static str = "GCCRS_EXTRA_ARGS";
    const AR_ENV_ARGS: &'static str = "AR_EXTRA_ARGS";

    /// Fetch the extra arguments given by the user for a specific environment string
    fn env_extra_args(key: &str) -> Option<Vec<String>> {
        env::var(key)
            .map(|s| s.split(' ').map(|arg| arg.to_owned()).collect())
            .ok()
    }

    /// Create arguments usable when spawning a process from an instance of [`GccrsArgs`]
    pub fn as_args(&self) -> Vec<String> {
        let mut args = self.source_files.clone();

        if let Some(mut user_compiler_args) =
            GccrsArgs::env_extra_args(GccrsArgs::COMPILER_ENV_ARGS)
        {
            args.append(&mut user_compiler_args);
        }

        // FIXME: How does gccrs behave with non-unicode filenames? Is gcc[rs] available
        // on the OSes that support non-unicode filenames?
        let output_file = self
            .output_file
            .to_str()
            .expect("Cannot handle non-unicode filenames yet")
            .to_owned();

        match self.crate_type {
            CrateType::Bin => args.append(&mut vec![
                String::from("-o"),
                output_file,
                String::from("-fPIE"),
                String::from("-pie"),
            ]),
            CrateType::DyLib => args.append(&mut vec![
                String::from("-fPIC"),
                String::from("-shared"),
                String::from("-o"),
                output_file,
            ]),
            // FIXME: Maybe don't format temporary object file like this?
            CrateType::StaticLib => args.append(&mut vec![
                String::from("-c"),
                String::from("-o"),
                // We can unwrap here since converting the PathBuf to string at the beginning
                // of the function would have thrown an error on a non UTF-8 output_file
                object_file_name(&output_file),
            ]),
            _ => {}
        }

        args
    }
}
