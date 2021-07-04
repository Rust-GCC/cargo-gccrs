use tempdir::TempDir;

use std::{
    env::{self, join_paths},
    ffi::{OsStr, OsString},
    fs::ReadDir,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    process::Command,
};

// FIXME: Needs to be refactored once we clean up the handling of file extension on
// the cargo-gccrs side
/// Compiler output kinds that the test harness can compare
pub enum FileType {
    /// Static libraries
    Static,
    /// Dynamic libraries
    Dyn,
    /// Binary executables
    Bin,
}

pub struct Harness;

impl Harness {
    /// Build the project present in the current directory using `rustc` or `gccrs`
    fn cargo_build(use_gccrs: bool, target_dir: Option<&TempDir>) -> Result<()> {
        let mut cmd = Command::new("cargo");

        // If a target dir is given, then run `cargo gccrs build`
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

        cmd.arg("build");

        if let Some(target_dir) = target_dir {
            cmd.arg("--target-dir");
            cmd.arg(target_dir.path());
        }

        cmd.status().and_then(|s| {
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

    fn get_output_filename(dir_iter: ReadDir, file_type: &FileType) -> Result<Option<OsString>> {
        // FIXME: Wrong on windows
        let extension = match file_type {
            FileType::Static => Some(OsString::from("a")),
            FileType::Dyn => Some(OsString::from("so")),
            FileType::Bin => None,
        };

        for dir_entry in dir_iter.into_iter() {
            let current_path = dir_entry?.path();
            if current_path.extension() == extension.as_deref() {
                return Ok(current_path.file_name().map(OsStr::to_owned));
            }
        }

        Ok(None)
    }

    fn compare_filenames(gccrs_target: &Path, file_type: FileType) -> Result<()> {
        let rustc_deps_dir = PathBuf::new().join("target").join("debug").join("deps");
        let gccrs_deps_dir = PathBuf::from(gccrs_target).join("debug").join("deps");

        dbg!(&gccrs_deps_dir);

        let rustc_output_file =
            Harness::get_output_filename(rustc_deps_dir.read_dir()?, &file_type)?;
        let gccrs_output_file =
            Harness::get_output_filename(gccrs_deps_dir.read_dir()?, &file_type)?;

        assert!(
            rustc_output_file.is_some(),
            "Couldn't find a file fitting the given type in rustc target"
        );
        assert!(
            gccrs_output_file.is_some(),
            "Couldn't find a file fitting the given type in gccrs target"
        );
        assert_eq!(rustc_output_file, gccrs_output_file);

        Ok(())
    }

    /// Runs the folder generic test suite on a give folder. This test suite
    /// makes sure that the project compiles using `rustc` as well as `gccrs`,
    /// before verifying that both compilers output create binaires with the
    /// correct file name and correct location.
    pub fn check_folder(folder_path: &str, file_type: FileType) -> Result<()> {
        let old_path = env::current_dir()?;

        let mut test_dir = PathBuf::from("tests");
        test_dir.push(folder_path);

        env::set_current_dir(&test_dir)?;

        // Clean existing artefacts
        Command::new("cargo").arg("clean").status()?;

        // Build the project using rustc
        Harness::cargo_build(false, None)?;

        // Copy the rustc target folder to a temporary directory
        let rustc_target_tmpdir = TempDir::new(&format!("{}-target-gccrs", folder_path))?;

        // Build the project using gccrs
        Harness::cargo_build(true, Some(&rustc_target_tmpdir))?;

        Harness::compare_filenames(rustc_target_tmpdir.path(), file_type)?;

        env::set_current_dir(old_path)?;

        Ok(())
    }
}
