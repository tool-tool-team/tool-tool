#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    const TT_FILENAME: &str = "tt";

    #[cfg(target_os = "windows")]
    const TT_FILENAME: &str = "tt.exe";

    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    const TMP_DIR: &str = ".test";

    struct Runner {
        test_binary: PathBuf,
        test_directory: PathBuf,
    }

    impl Runner {
        fn verify_execution(&self, command: &str) -> () {
            let output = Command::new(&self.test_binary)
                .env("PATH", &self.test_directory)
                .env("RUST_BACKTRACE", &"1")
                .stdin(Stdio::null())
                .current_dir(&self.test_directory)
                .args(command.split_ascii_whitespace())
                .output()
                .expect("failed to execute process");
            let mut settings = insta::Settings::clone_current();
            settings.set_sort_maps(true);
            let result = format!(
                "Exit status: {}\nSTDOUT:\n{}\nSTDERR:\n{}\n",
                output.status,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
            dbg!(&result);
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
        let tt_binary = PathBuf::from(format!("../target/release/{}", TT_FILENAME));
        std::fs::copy(
            Path::new(&format!("test-configurations/{}.yaml", config_name)),
            &test_directory.join(".tool-tool.v1.yaml"),
        )
        .unwrap();
        let test_binary = test_directory.join(tt_binary.file_name().unwrap());
        std::fs::copy(&tt_binary, &test_binary).unwrap();

        #[cfg(target_os = "linux")]
        let test_binary = Path::new(".").join(tt_binary.file_name().unwrap());

        #[cfg(target_os = "windows")]
        let test_binary = PathBuf::from(tt_binary.file_name().unwrap());

        Runner {
            test_binary,
            test_directory,
        }
    }

    #[test]
    fn basic_operation() {
        let runner = prepare_test("basic");
        runner.verify_execution("bat --version");
        runner.verify_execution("--help");
    }

    #[test]
    fn coreutils() {
        let runner = prepare_test("coreutils");
        runner.verify_execution("--download");
        runner.verify_execution("coreutils echo foo");
        runner.verify_execution("echo bar");
        runner.verify_execution("replace_version");
        runner.verify_execution("--getToolVersion coreutils");
        runner.verify_execution("--getToolPath coreutils");
        runner.verify_execution("--getBinaryPath coreutils");
        runner.verify_execution("--getBinaryPath echo");
    }
}
