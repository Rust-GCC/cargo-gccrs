mod test_harness;

use test_harness::Harness;

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FOLDERS: &[&str] = &["binary_project", "static_lib", "shared_library"];

    #[test]
    fn compile_projects() {
        TEST_FOLDERS.iter().for_each(|f| Harness::check_folder(*f).unwrap())
    }
}