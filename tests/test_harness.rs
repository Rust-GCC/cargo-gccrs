use tempdir::TempDir;

use std::{
    env::{self, join_paths},
    io::{Error, ErrorKind, Result},
    path::PathBuf,
    process::Command,
};

pub struct Harness;

impl Harness {
    /// Build the project present in the current directory using `rustc` or `gccrs`
    fn cargo_build(use_gccrs: bool) -> Result<()> {
        let mut cmd = Command::new("cargo");

        if use_gccrs {
            // Add `gccrs` argument so that `cargo build` becomes `cargo gccrs build`
            cmd.arg("gccrs");

            // Tweak the path so that the most recently compiled *debug* version of
            // cargo-gccrs is available as a subcommand.
            // Create this target path in a way that's compatible with Windows.
            let target_path = PathBuf::from("..").join("..").join("target").join("debug");

            let mut paths = vec![target_path];
            paths.append(&mut env::split_paths(&env::var("PATH").unwrap()).collect::<Vec<_>>());

            let new_path = join_paths(paths.iter()).unwrap();

            cmd.env("PATH", new_path);
        }

        cmd.arg("build").status().and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    "command did not exit successfully",
                ))
            }
        })
    }

    /// Copy a folder to a set destination
    fn copy_folder(src: &str, dest: &str) -> Result {
        Command::new("cp")
            .arg("-r")
            .arg(src)
            .arg(dest)
            .status()
            .into_result()
    }

    /// Runs the folder generic test suite on a give folder. This test suite
    /// makes sure that the project compiles using `rustc` as well as `gccrs`,
    /// before verifying that both compilers output create binaires with the
    /// correct file name and correct location.
    pub fn check_folder(folder_path: &str) -> Result<()> {
        let old_path = env::current_dir()?;
        let mut test_dir = PathBuf::from("tests");
        test_dir.push(folder_path);

        env::set_current_dir(&test_dir)?;

        // Build the project using rustc
        Harness::cargo_build(false)?;

        // Copy the rustc target folder to a temporary directory
        let rustc_target_tmpdir = TempDir::new("target-rustc")?;
        Harness::copy_folder("target", rustc_target_tmpdir.path().to_str().unwrap())?;

        // Build the project using gccrs
        Harness::cargo_build(true)?;

        env::set_current_dir(old_path)?;

        Ok(())
    }
}
