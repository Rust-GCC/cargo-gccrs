//! This module interprets arguments given to `rustc` and transforms them into valid
//! arguments for `gccrs`.

use std::path::PathBuf;

use getopts::{Matches, Options};

use super::{Error, Result};

/// Crate types supported by `gccrs`
#[derive(Clone, Copy)]
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

impl CrateType {
    /// Get the corresponding [`CrateType`] from a given option to the `--crate-type`
    /// option
    pub fn from_str(s: &str) -> CrateType {
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
    let crate_name = matches.opt_str("crate-name").ok_or(Error::InvalidArg)?;
    let out_dir = matches.opt_str("out-dir").ok_or(Error::InvalidArg)?;
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
        .collect::<Vec<&str>>()[0];

    match crate_type {
        CrateType::Bin => output_file.push(&format!("{}{}", crate_name, extra_filename)),
        CrateType::DyLib => output_file.push(&format!("lib{}{}.so", crate_name, extra_filename)),
        CrateType::StaticLib => output_file.push(&format!("lib{}.a{}", crate_name, extra_filename)),
        _ => unreachable!(
            "gccrs cannot handle other crate types than bin, dylib or staticlib at the moment"
        ),
    }

    Ok((output_file, crate_type))
}

/// Structure used to represent arguments passed to `gccrs`. Convert them from `rustc`
/// arguments using [`GccrsArg::from_rustc_arg`]
pub struct GccrsArgs {
    source_files: Vec<String>,
    crate_type: CrateType,
    output_file: PathBuf,
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

    /// Get the corresponding `gccrs` argument from a given `rustc` argument
    pub fn from_rustc_args(rustc_args: &[String]) -> Result<Vec<GccrsArgs>> {
        let options = GccrsArgs::generate_parser();

        // Parse arguments, skipping `cargo-gccrs` and `rustc` in the invocation
        let matches = options.parse(&rustc_args[2..])?;

        matches
            .opt_strs("crate-type")
            .iter()
            .map(|type_str| CrateType::from_str(&type_str))
            .map(|crate_type| format_output_filename(&matches, crate_type))
            .map(|result_tuple| {
                result_tuple.map(|(output_file, crate_type)| GccrsArgs {
                    source_files: matches.free.clone(),
                    crate_type,
                    output_file,
                })
            })
            .collect()
    }

    /// Convert a `GccrsArgs` structure into arguments usable to spawn a process
    pub fn into_args(self) -> Vec<String> {
        let mut args = self.source_files;

        // FIXME: How does gccrs behave with non-unicode filenames? Is gcc[rs] available
        // on the OSes that support non-unicode filenames?
        let output_file = self
            .output_file
            .to_str()
            .expect("Cannot handle non-unicode filenames yet")
            .to_owned();

        match self.crate_type {
            CrateType::Bin => args.append(&mut vec![String::from("-o"), output_file]),
            CrateType::DyLib => args.append(&mut vec![
                String::from("-shared"),
                String::from("-o"),
                output_file,
            ]),
            CrateType::StaticLib => unreachable!("Cannot generate static libraries yet"),
            _ => {}
        }

        args
    }
}
