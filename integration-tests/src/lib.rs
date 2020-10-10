#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    const PLATFORM: &str = "linux";

    #[cfg(target_os = "windows")]
    const PLATFORM: &str = "windows";

    use std::path::{Path, PathBuf};
    use std::process::Command;

    const TMP_DIR: &str = ".test";

    struct Runner {
        test_binary: PathBuf,
        test_directory: PathBuf,
    }

    impl Runner {
        fn verify_execution(&self, command: &str) -> () {
            let output = Command::new(&self.test_binary)
                .current_dir(&self.test_directory)
                .args(command.split_ascii_whitespace())
                .output()
                .expect("failed to execute process");
            let mut settings = insta::Settings::clone_current();
            settings.set_sort_maps(true);
            settings.set_snapshot_suffix(PLATFORM);
            let result = format!(
                "Exit status: {}\nSTDOUT:\n{}\nSTDERR:\n{}\n",
                output.status,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
            settings.bind(|| {
                insta::assert_snapshot!(result);
            });
        }
    }

    fn prepare_test(config_name: &str) -> Runner {
        let test_directory = Path::new(TMP_DIR).join(config_name);
        // Cleanup test directory
        if test_directory.exists() {
            std::fs::remove_dir_all(&test_directory).unwrap();
        }
        // Prepare test directory
        std::fs::create_dir_all(&test_directory).unwrap();
        let tt_binary = Path::new("../target/release/tt.exe");
        std::fs::copy(
            Path::new(&format!("test-configurations/{}.yaml", config_name)),
            &test_directory.join(".tool-tool.v1.yaml"),
        )
        .unwrap();
        let test_binary = test_directory.join(tt_binary.file_name().unwrap());
        std::fs::copy(&tt_binary, &test_binary).unwrap();

        Runner {
            test_binary,
            test_directory,
        }
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn basic_operation() {
        let runner = prepare_test("basic");
        runner.verify_execution("bat --version");
    }
}
