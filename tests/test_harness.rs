use std::{
    env,
    io::{Error, ErrorKind},
    process::{Command, ExitStatus},
};

pub struct Harness;

type Result<T = ()> = std::io::Result<T>;

/// Maps an exit status to a result, meaning that a non-successful exit status
/// will get mapped to an `Err` variant
trait ExitStatusConverter {
    fn into_result(self) -> Result;
}

impl ExitStatusConverter for Result<ExitStatus> {
    fn into_result(self) -> Result {
        match self {
            Ok(exit_code) => match exit_code.success() {
                true => Ok(()),
                false => Err(Error::new(
                    ErrorKind::Other,
                    "command did not exit successfully",
                )),
            },
            Err(e) => Err(e),
        }
    }
}

impl Harness {
    /// Build the project present in the current directory using `rustc` or `gccrs`
    fn cargo_build(use_gccrs: bool) -> Result {
        let mut cmd = Command::new("cargo");

        if use_gccrs {
            cmd.arg("gccrs");
        }

        cmd.arg("build").status().into_result()
    }

    /// Runs the folder generic test suite on a give folder. This test suite
    /// makes sure that the project compiles using `rustc` as well as `gccrs`,
    /// before verifying that both compilers output create binaires with the
    /// correct file name and correct location.
    pub fn check_folder(folder_path: &str) -> Result {
        let old_path = env::current_dir()?;
        let folder_path = format!("tests/{}", folder_path);

        env::set_current_dir(&folder_path)?;

        // Build the project using rustc
        Harness::cargo_build(false)?;

        // Build the project using gccrs
        Harness::cargo_build(true)?;

        env::set_current_dir(old_path)?;

        Ok(())
    }
}
