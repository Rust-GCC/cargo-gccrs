mod test_harness;

use test_harness::{FileType, Harness};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_project_compilation() {
        Harness::check_folder("binary_project", FileType::Bin).unwrap();
        Harness::check_folder("static_lib", FileType::Static).unwrap();
        Harness::check_folder("shared_library", FileType::Dyn).unwrap();

        // FIXME: As of right now, this just fails on rustc compilation which is not what
        // we want to check
        assert!(Harness::check_folder("invalid_code", FileType::Bin).is_err());

        std::env::set_var("GCCRS_EXTRA_ARGS", "-Werror");
        assert!(Harness::check_folder("warning_project", FileType::Bin).is_err());
        std::env::set_var("GCCRS_EXTRA_ARGS", "");
    }
}
