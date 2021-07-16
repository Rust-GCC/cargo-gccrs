//! This module interprets arguments given to `rustc` and transforms them into valid
//! arguments for `gccrs`.

use std::path::{Path, PathBuf};

use getopts::Matches;

use super::{EnvArgs, Error, Result, RustcOptions};

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
        .collect::<Vec<&str>>()[0];

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

/// Structure used to represent arguments passed to `gccrs`. Convert them from `rustc`
/// arguments using [`GccrsArg::from_rustc_arg`]
pub struct GccrsArgs {
    source_files: Vec<String>,
    crate_type: CrateType,
    output_file: PathBuf,
    callback: Option<&'static dyn Fn(&GccrsArgs) -> Result>,
}

impl GccrsArgs {
    fn new(source_files: &[String], crate_type: CrateType, output_file: PathBuf) -> GccrsArgs {
        GccrsArgs {
            source_files: Vec::from(source_files),
            crate_type,
            output_file,
            callback: None,
        }
    }

    /// Add `.tmp.o` to the expected output filename.
    pub fn object_file_name(&self) -> PathBuf {
        self.output_file.clone().join(".tmp.o")
    }

    /// Get a reference to the set of arguments' output file path
    pub fn output_file(&self) -> &Path {
        &self.output_file
    }

    /// Set the callback of an argument set
    pub fn set_callback(&mut self, function: &'static dyn Fn(&GccrsArgs) -> Result) {
        self.callback = Some(function);
    }

    /// Get a reference to the set of arguments' type of binary produced
    pub fn crate_type(&self) -> CrateType {
        self.crate_type
    }

    // Execute an argument set's callback if present. Returns Ok if no callback was
    // present
    pub fn callback(&self) -> Option<&'static dyn Fn(&GccrsArgs) -> Result> {
        self.callback
    }

    /// Get the corresponding `gccrs` argument from a given `rustc` argument
    pub fn from_rustc_args(rustc_args: &[String]) -> Result<Vec<GccrsArgs>> {
        let matches = RustcOptions::new().parse(rustc_args)?;

        matches
            .opt_strs("crate-type")
            .iter()
            .map(|type_str| CrateType::from(type_str.as_str()))
            .map(|crate_type| format_output_filename(&matches, crate_type))
            .map(|result_tuple| {
                result_tuple.map(|(output_file, crate_type)| {
                    GccrsArgs::new(&matches.free, crate_type, output_file)
                })
            })
            .collect()
    }

    /// Create arguments usable when spawning a process from an instance of [`GccrsArgs`]
    pub fn as_args(&self) -> Result<Vec<String>> {
        let mut args = self.source_files.clone();

        if let Some(mut user_compiler_args) = EnvArgs::Gcc.as_args() {
            args.append(&mut user_compiler_args);
        }

        let output_file = self.output_file().as_os_str().to_owned().into_string()?;

        match self.crate_type {
            CrateType::Bin => args.append(&mut vec![String::from("-o"), output_file]),
            CrateType::DyLib => args.append(&mut vec![
                String::from("-fPIC"),
                String::from("-shared"),
                String::from("-o"),
                output_file,
            ]),
            CrateType::StaticLib => args.append(&mut vec![
                String::from("-c"),
                String::from("-o"),
                self.object_file_name().into_os_string().into_string()?,
            ]),
            CrateType::Unknown => {}
        }

        Ok(args)
    }
}
