//! This module interprets arguments given to `rustc` and transforms them into valid
//! arguments for `gccrs`.

use getopts::{Matches, Options};

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

/// Structure used to represent arguments passed to `gccrs`. Convert them from `rustc`
/// arguments using [`GccrsArg::from_rustc_arg`]
pub struct GccrsArgs {
    source_files: Vec<String>,
    crate_type: CrateType,
    output_file: String,
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

    fn format_output_filename(matches: &Matches, crate_type: &CrateType) -> String {
        // FIXME: No unwraps here
        let crate_name = matches.opt_str("crate-name").unwrap();
        let out_dir = matches.opt_str("out-dir").unwrap();

        // FIXME: Figure out a way to return multiple output filenames. Just handle `bin`
        // for now
        let c_options = matches.opt_strs("C");

        // FIXME: This is probably different on Windows, we should use Paths and PathStrs
        // instead
        let mut output_file = match *crate_type {
            CrateType::Bin => format!("{}/{}", out_dir, crate_name),
            CrateType::DyLib => format!("{}/lib{}.so", out_dir, crate_name),
            CrateType::StaticLib => format!("{}/lib{}.a", out_dir, crate_name),
            _ => unreachable!(
                "gccrs cannot handle other crate types than bin, dylib or staticlib at the moment"
            ),
        };

        // FIXME: Horrendous. We need to create a separate "C options" parser since we'll
        // probably use more than just `extra-filename`.
        c_options.iter().for_each(|c_opt| {
            let mut split = c_opt.split('=');

            if let Some("extra-filename") = split.next().as_deref() {
                output_file.push_str(&split.next().unwrap())
            }
        });

        dbg!(output_file)
    }

    /// Get the corresponding `gccrs` argument from a given `rustc` argument
    pub fn from_rustc_args(rustc_args: &[String]) -> Vec<GccrsArgs> {
        let options = GccrsArgs::generate_parser();

        // Parse arguments, skipping `cargo-gccrs` and `rustc` in the invocation
        let matches = match options.parse(&rustc_args[2..]) {
            Ok(m) => m,
            Err(err) => unreachable!("{:?}", err),
        };

        matches
            .opt_strs("crate-type")
            .iter()
            .map(|type_str| CrateType::from_str(&type_str))
            .map(|crate_type| {
                (
                    // We need to keep the crate_type as well, in order to spawn the
                    // correct command. Return it in a tuple alongside the generated
                    // output filename.
                    crate_type,
                    GccrsArgs::format_output_filename(&matches, &crate_type),
                )
            })
            .map(|(crate_type, output_file)| GccrsArgs {
                source_files: matches.free.clone(),
                crate_type,
                output_file,
            })
            .collect()
    }

    /// Convert a `GccrsArgs` structure into arguments usable to spawn a process
    pub fn into_args(mut self) -> Vec<String> {
        let mut args = vec![];

        // Add all the source files to the command line
        self.source_files
            .drain(..)
            .for_each(|source| args.push(source));

        match self.crate_type {
            CrateType::Bin => args.append(&mut vec![String::from("-o"), self.output_file]),
            CrateType::DyLib => args.append(&mut vec![
                String::from("-shared"),
                String::from("-o"),
                self.output_file,
            ]),
            CrateType::StaticLib => unreachable!("Cannot generate static libraries yet"),
            _ => {}
        }

        args
    }
}
