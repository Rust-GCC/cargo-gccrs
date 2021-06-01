//! The Config module aims to parse gccrs target options and translate them to an output
//! similar to what rustc does. This module parses a file previously created by the
//! [`Gccrs::dump_config()`] function. This corresponds to invoking gccrs with the
//! `-frust-dump-target_options` argument.

pub struct GccrsConfig;

use super::Result;

/// Different kinds of options dumped by `gccrs -frust-dump-target_options`
#[derive(Debug, PartialEq)]
enum DumpedOption {
    /// Corresponds to options dumped as multiple values of the following format:
    /// `target_<...>: <...>`
    TargetSpecific(String, String),
    /// Corresponds to options dumped as a singular value: `<...>`
    OsInfo(String),
}

impl DumpedOption {
    fn parse_multi_value(input: Vec<&str>) -> Option<DumpedOption> {
        let key = input.get(0)?.to_string();
        let value = input.get(1)?.trim_start().to_string();

        Some(DumpedOption::TargetSpecific(key, value))
    }

    // FIXME: Should we use the convert::From<&str> trait?
    pub fn from_str(input: &str) -> Option<DumpedOption> {
        let splitted_input: Vec<&str> = input.split(':').collect();
        match splitted_input.len() {
            // If no colon is found, then we are parsing a singular value
            1 => Some(DumpedOption::OsInfo(input.to_owned())),
            // If just one colon is found, then we're in a mutivalue. This is valid
            2 => DumpedOption::parse_multi_value(splitted_input),
            // Invalid input: Multiple colons in line
            // TODO: Is that correct?
            // FIXME: What do we do in that case? We should return an Err rather than None
            _ => None,
        }
    }

    // FIXME: Should we use the fmt::Display trait?
    pub fn display(&self) {
        // rustc displays os informations in the same vein as gccrs: `<info>`
        // For target specific options however, rustc uses an equal sign and no space
        // `target_<...>: <...>` becomes `target_<...>=<...>`
        match self {
            DumpedOption::OsInfo(s) => println!("{}", s),
            DumpedOption::TargetSpecific(k, v) => println!("{}={}", k, v),
        }
    }
}

impl GccrsConfig {
    const CONFIG_FILENAME: &'static str = "gccrs.target-options.dump";

    fn read_options() -> Result<String> {
        std::fs::read_to_string(GccrsConfig::CONFIG_FILENAME)
    }

    fn parse(input: String) -> Result<Vec<Option<DumpedOption>>> {
        Ok(input.lines().map(|line| DumpedOption::from_str(line)).collect())
    }

    /// Display the gccrs target options on stdout, in a format that cargo understands
    pub fn display() -> Result {
        let lines = GccrsConfig::read_options()?;
        let options = GccrsConfig::parse(lines)?;

        // FIXME: Ugly
        options.iter().for_each(|opt| opt.as_ref().unwrap().display());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FIXME: Useful for tests but really ugly, keep it?
    macro_rules! s {
        ($hamster:expr) => { $hamster.to_string() }
    }

    #[test]
    fn os_info_valid() {
        assert_eq!(DumpedOption::from_str("unix"), Some(DumpedOption::OsInfo(s!("unix"))))
    }

    #[test]
    fn target_kv_valid() {
        assert_eq!(DumpedOption::from_str("target_k: v"), Some(DumpedOption::TargetSpecific(s!("target_k"), s!("v"))))
    }

    #[test]
    fn option_invalid() {
        assert_eq!(DumpedOption::from_str("k: v0: v1"), None)
    }
}
