use goblin::{elf64::header::ET_DYN, Object};
use is_executable::IsExecutable;
use tempdir::TempDir;

use std::{
    env::{self, join_paths},
    ffi::{OsStr, OsString},
    fs::{File, ReadDir},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    process::Command,
};

// FIXME: Needs to be refactored once we clean up the handling of file extension on
// the cargo-gccrs side
/// Compiler output kinds that the test harness can compare
#[derive(Debug)]
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

    fn check_archive(file: &Path) -> Result<()> {
        // For now, check that an archive is not empty. We can add more checks later as
        // per the functional testing tracking issue #12
        assert_ne!(ar::Archive::new(File::open(file)?).count_entries()?, 0);

        Ok(())
    }

    fn check_correct_filetype(file: &Path, expected_type: &FileType) -> Result<()> {
        let e_type = match expected_type {
            FileType::Static => return Harness::check_archive(file),
            FileType::Dyn | FileType::Bin => ET_DYN,
        };

        let elf_data = std::fs::read(file)?;
        match Object::parse(&elf_data).unwrap() {
            Object::Elf(elf) => assert_eq!(elf.header.e_type, e_type),
            _ => unreachable!("Invalid ELF file: {:?}", file),
        }

        Ok(())
    }

    fn get_output_filename(dir_iter: ReadDir, file_type: &FileType) -> Result<Option<OsString>> {
        // FIXME: Wrong on windows
        let predicate = match file_type {
            FileType::Static => |p: &Path| p.extension() == Some(OsString::from("a")).as_deref(),
            FileType::Dyn => |p: &Path| p.extension() == Some(OsString::from("so")).as_deref(),
            FileType::Bin => |p: &Path| p.is_executable(),
        };

        for dir_entry in dir_iter.into_iter() {
            let current_path = dir_entry?.path();
            // https://rust-lang.github.io/rust-clippy/master/index.html#filetype_is_file
            if predicate(&current_path) && !current_path.is_dir() {
                Harness::check_correct_filetype(&current_path, file_type)?;
                return Ok(current_path.file_name().map(OsStr::to_owned));
            }
        }

        Ok(None)
    }

    /// Check that temporary output files (The binaries in target/debug/deps) have the
    /// same name and type
    fn check_deps_output(gccrs_target: &Path, file_type: &FileType) -> Result<()> {
        let rustc_deps_dir = PathBuf::new().join("target").join("debug").join("deps");
        let gccrs_deps_dir = PathBuf::from(gccrs_target).join("debug").join("deps");

        let rustc_output_file =
            Harness::get_output_filename(rustc_deps_dir.read_dir()?, file_type)?;
        let gccrs_output_file =
            Harness::get_output_filename(gccrs_deps_dir.read_dir()?, file_type)?;

        assert!(
            rustc_output_file.is_some(),
            "Couldn't find a file fitting the given type in rustc target/debug/deps"
        );
        assert!(
            gccrs_output_file.is_some(),
            "Couldn't find a file fitting the given type in gccrs target/debug/deps"
        );
        assert_eq!(rustc_output_file, gccrs_output_file);

        Ok(())
    }

    /// Check that the final output (The binary in target/debug/) has the same name
    /// and type
    fn check_final_output(gccrs_target: &Path, file_type: &FileType) -> Result<()> {
        let rustc_target_dir = PathBuf::new().join("target").join("debug");
        let gccrs_target_dir = PathBuf::from(gccrs_target).join("debug");

        let rustc_output_file =
            Harness::get_output_filename(rustc_target_dir.read_dir()?, file_type)?;
        let gccrs_output_file =
            Harness::get_output_filename(gccrs_target_dir.read_dir()?, file_type)?;

        assert!(
            rustc_output_file.is_some(),
            "Couldn't find a file fitting the given type in rustc target/debug"
        );
        assert!(
            gccrs_output_file.is_some(),
            "Couldn't find a file fitting the given type in gccrs target/debug"
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
        let gccrs_target_tmpdir = TempDir::new(&format!("{}-target-gccrs", folder_path))?;

        // Build the project using gccrs
        Harness::cargo_build(true, Some(&gccrs_target_tmpdir))?;

        Harness::check_deps_output(gccrs_target_tmpdir.path(), &file_type)?;
        Harness::check_final_output(gccrs_target_tmpdir.path(), &file_type)?;

        env::set_current_dir(old_path)?;

        Ok(())
    }
}
