use super::Result;
/// This module implements `rustc`'s options parser. Ultimately, this should be directly
/// taken from `rustc`'s implementation
use getopts::{Matches, Options};

pub struct RustcArgs {
    options: Options,
}

impl RustcArgs {
    /// Generate a new options parser according to `rustc`'s command line options
    pub fn new() -> RustcArgs {
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

        RustcArgs { options }
    }

    pub fn parse(&self, args: &[String]) -> Result<Matches> {
        // Parse arguments, skipping `cargo-gccrs` and `rustc` in the invocation
        Ok(self.options.parse(&args[2..])?)
    }
}
