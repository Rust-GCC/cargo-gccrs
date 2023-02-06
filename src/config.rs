//! The Config module aims to parse gccrs target options and translate them to an output
//! similar to what rustc does. This module parses a file previously created by the
//! [`Gccrs::dump_config()`] function. This corresponds to invoking gccrs with the
//! `-frust-dump-target_options` argument.

use super::{Error, Result};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Different kinds of options dumped by `gccrs -frust-dump-target_options`
#[derive(Debug, PartialEq, Eq, Clone)]
enum DumpedOption {
    /// Corresponds to options dumped as multiple values of the following format:
    /// `target_<...>: <...>`
    TargetSpecific(String, String),
    /// Corresponds to options dumped as a singular value: `<...>`
    OsInfo(String),
}

impl DumpedOption {
    fn parse_multi_value(input: Vec<&str>) -> Option<DumpedOption> {
        let key = input.first()?.to_string();
        let value = input.get(1)?.trim_start().to_string();

        Some(DumpedOption::TargetSpecific(key, value))
    }

    // FIXME: Should we use the convert::From<&str> trait?
    /// Attempt to parse a [`DumpedOption`] from a given input. The input should
    /// correspond to a singular line of the `gccrs.target-options.dump` file
    pub fn from_str(input: &str) -> Result<DumpedOption> {
        let invalid_input = Error::InvalidCfgDump;

        let splitted_input: Vec<&str> = input.split(':').collect();

        match splitted_input.len() {
            // If no colon is found, then we are parsing a singular value
            1 => Ok(DumpedOption::OsInfo(input.to_owned())),
            // If just one colon is found, then we're in a mutivalue. This is valid
            2 => DumpedOption::parse_multi_value(splitted_input).ok_or(invalid_input),
            // Invalid input: Multiple colons in line
            // TODO: Is that correct?
            _ => Err(invalid_input),
        }
    }
}

impl Display for DumpedOption {
    /// Display a parsed [`DumpedOption`] on stdout according to the format used by `rustc`
    /// `rustc` displays OS information in the same way as gccrs: `<info>`
    /// For target specific options however, `rustc` uses an equal sign and no space between
    /// the key and value. Thus, `target_<0>: <1> becomes `target_<0>=<1>`
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DumpedOption::OsInfo(s) => write!(f, "{s}"),
            DumpedOption::TargetSpecific(k, v) => write!(f, "{k}={v}"),
        }
    }
}

impl PartialOrd for DumpedOption {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Sort DumpedArgs based on syntax printing rules. `rustc` prints target options in
/// alphabetical order, before printing OS information
impl Ord for DumpedOption {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (DumpedOption::TargetSpecific(_, _), DumpedOption::OsInfo(_)) => Ordering::Less,
            (DumpedOption::OsInfo(_), DumpedOption::TargetSpecific(_, _)) => Ordering::Greater,
            // FIXME: This is the same behavior twice. There might be a way to simply match the
            // tuple and call it `s` or `o` without unfolding it
            (DumpedOption::OsInfo(s), DumpedOption::OsInfo(o)) => s.cmp(o),
            (DumpedOption::TargetSpecific(s_k, s_v), DumpedOption::TargetSpecific(o_k, o_v)) => {
                (s_k, s_v).cmp(&(o_k, o_v))
            }
        }
    }
}

pub struct GccrsConfig {
    options: Vec<DumpedOption>,
}

impl GccrsConfig {
    const CONFIG_FILENAME: &'static str = "gccrs.target-options.dump";

    fn read_options() -> Result<String> {
        let content = std::fs::read_to_string(GccrsConfig::CONFIG_FILENAME)?;

        Ok(content)
    }

    fn parse(input: String) -> Result<Vec<DumpedOption>> {
        input.lines().map(DumpedOption::from_str).collect()
    }

    /// Create a new instance `GccrsConfig` with a call to the `gccrs` compiler
    pub fn new() -> Result<GccrsConfig> {
        let lines = GccrsConfig::read_options()?;
        let mut options = GccrsConfig::parse(lines)?;

        // Sort the vector according to the syntax printing rules
        options.sort();

        Ok(GccrsConfig { options })
    }
}

impl Display for GccrsConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.options.iter().try_for_each(|opt| write!(f, "{opt}"))
    }
}

#[cfg(test)]
mod tests {
    use super::DumpedOption;

    // FIXME: Useful for tests but really ugly, keep it?
    macro_rules! s {
        ($hamster:expr) => {
            $hamster.to_string()
        };
    }

    #[test]
    fn os_info_valid() {
        assert_eq!(
            DumpedOption::from_str("unix").unwrap(),
            DumpedOption::OsInfo(s!("unix"))
        )
    }

    #[test]
    fn target_kv_valid() {
        assert_eq!(
            DumpedOption::from_str("target_k: v").unwrap(),
            DumpedOption::TargetSpecific(s!("target_k"), s!("v"))
        )
    }

    #[test]
    fn option_invalid() {
        assert!(DumpedOption::from_str("k: v0: v1").is_err())
    }

    #[test]
    fn sorting() {
        let c0 = DumpedOption::from_str(r#"target_os="linux""#).unwrap();
        let c3 = DumpedOption::from_str(r#"unix"#).unwrap();
        let c2 = DumpedOption::from_str(r#"target_vendor="unknown""#).unwrap();
        let c1 = DumpedOption::from_str(r#"target_pointer_width="64""#).unwrap();

        let mut v = vec![c0.clone(), c3.clone(), c2.clone(), c1.clone()];
        v.sort();

        assert_eq!(v, vec![c0, c1, c2, c3]);
    }
}
