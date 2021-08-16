//! This module implements `rustc`'s options parser. Ultimately, this should be directly
//! taken from `rustc`'s implementation

use super::{Error, Result};
use getopts::{Matches, Options};
use std::convert::TryFrom;

pub struct RustcArgs {
    matches: Matches,
}

impl TryFrom<&[String]> for RustcArgs {
    type Error = Error;

    fn try_from(args: &[String]) -> Result<Self> {
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
        options.optmulti(
            "",
            "cap-lints",
            "Set the most restrictive lint level",
            "LEVEL",
        );
        options.optmulti("", "cfg", "Configure the compilation environment", "SPEC");

        // Parse arguments, skipping `cargo-gccrs` and `rustc` in the invocation
        Ok(RustcArgs {
            matches: options.parse(&args[2..])?,
        })
    }
}

impl RustcArgs {
    pub fn matches(&self) -> &Matches {
        &self.matches
    }
}
