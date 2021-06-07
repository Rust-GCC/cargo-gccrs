//! This module interprets arguments given to `rustc` and transforms them into valid
//! arguments for `gccrs`.

/// Structure used to represent arguments passed to `gccrs`. Convert them from `rustc`
/// arguments using [`GccrsArg::from_rustc_arg`]
#[derive(Debug)]
pub struct GccrsArg(String);

impl GccrsArg {
    fn new(s: &str) -> GccrsArg {
        GccrsArg(String::from(s))
    }

    /// Get the corresponding `gccrs` argument from a given `rustc` argument
    pub fn from_rustc_arg(rustc_arg: String) -> GccrsArg {
        match rustc_arg.as_str() {
            "--crate-name" => GccrsArg::new("-o"),
            "basic_rust_project" => GccrsArg::new("basic_rust_project"),
            "src/main.rs" => GccrsArg::new("src/main.rs"),
            _ => GccrsArg::new("")
        }
    }
}

use std::{convert::AsRef, ffi::OsStr};

impl AsRef<OsStr> for GccrsArg {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}
