//! This module interprets arguments given to `rustc` and transforms them into valid
//! arguments for `gccrs`.

use std::convert::TryFrom;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use getopts::Matches;

use super::{EnvArgs, Error, Result, RustcArgs};

/// A collection containing multiple instances of `Args`. This is necessary in order
/// to circumvent the fact that `rustc` can currently generate multiple types of binaries
/// with a single invokation.
///
/// For example,
/// `rustc --crate-type=static --crate-type=dyn --crate-type=bin src/<file>.rs` will attempt
/// to generate an executable, a dynamic library and a static one.
///
/// When using `gccrs` to compile things, we must also take into account what `gcc` can
/// and cannot do. And `gcc` cannot currently generate multiple outputs in a single invokation.
///
/// Let's look at the previous example:
///
/// * To generate an executable from `src/<file>.rs`,
/// we'd need to do the following: `gccrs src/<file>.rs` (with `-o <file>` if we don't
/// want an executable named a.out, but that's not important).
///
/// * For a shared library, we need to add the `-shared` flag. On top of this, `rustc`
/// generates libraries named `lib<name>.[so|a]` on Linux, while `gcc` will happily generate
/// a shared library without any extension or prefix. This amounts to the following command:
/// `gccrs -shared src/<file>.rs -o lib<file>.so`.
///
/// * Finally, `gcc` is not able to generate a static library at all. We *need* to use a
/// different command, `ar`, in order to bundle up object files previously created by
/// `gcc`. Therefore, we actually need *two* commands:
/// `gccrs -c src/<file>.rs && ar csr src/<file>.o`
///
/// Multiple flags, such as `shared`, conflict with the generation of other binaries. On
/// top of that, we cannot use the `-o` option precisely enough to control the output name
/// based on the binary file produced.
///
/// Therefore, we need to wrap an unknown amount of sets of `gccrs` arguments in order to
/// mimic a single `rustc` invokation. Later on, we need to iterate over those sets and
/// spawn a new `gccrs` command for each of them.
pub struct ArgsCollection {
    args_set: Vec<Args>,
}

/// Get the corresponding set of `gccrs` arguments from a single set of `rustc` arguments
impl TryFrom<&RustcArgs> for ArgsCollection {
    type Error = Error;

    fn try_from(rustc_args: &RustcArgs) -> Result<ArgsCollection> {
        let matches = rustc_args.matches();

        let args_set: Result<Vec<Args>> = matches
            .opt_strs("crate-type")
            .iter()
            .map(|type_str| CrateType::from(type_str.as_str()))
            .map(|crate_type| format_output_filename(&matches, crate_type))
            .map(|result_tuple| {
                result_tuple.map(|(output_file, crate_type)| {
                    Args::new(&matches.free, crate_type, output_file)
                })
            })
            .collect();

        Ok(ArgsCollection {
            args_set: args_set?,
        })
    }
}

/// Implement deref on the collection so we can easily iterate on it
impl Deref for ArgsCollection {
    type Target = Vec<Args>;

    fn deref(&self) -> &Self::Target {
        &self.args_set
    }
}

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

/// Structure used to represent arguments passed to `gccrs`. Convert them from `rustc`
/// arguments using [`Args::from_rustc_arg`]
pub struct Args {
    source_files: Vec<String>,
    crate_type: CrateType,
    output_file: PathBuf,
}

impl Args {
    fn new(source_files: &[String], crate_type: CrateType, output_file: PathBuf) -> Args {
        Args {
            source_files: Vec::from(source_files),
            crate_type,
            output_file,
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

    /// Get a reference to the set of arguments' type of binary produced
    pub fn crate_type(&self) -> CrateType {
        self.crate_type
    }

    /// Create arguments usable when spawning a process from an instance of [`Args`]
    pub fn as_args(&self) -> Result<Vec<String>> {
        // `rustc` generates position independant code
        let mut args = vec![String::from("-fPIE"), String::from("-pie")];
        args.append(&mut self.source_files.clone());

        if let Some(mut user_compiler_args) = EnvArgs::Gcc.as_args() {
            args.append(&mut user_compiler_args);
        }

        let output_file = self.output_file().as_os_str().to_owned().into_string()?;

        match self.crate_type {
            CrateType::Bin => args.append(&mut vec![String::from("-o"), output_file]),
            CrateType::DyLib => args.append(&mut vec![
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
