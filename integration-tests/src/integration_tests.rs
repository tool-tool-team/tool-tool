#[cfg(target_os = "linux")]
const TT_FILENAME: &str = "tt";

#[cfg(target_os = "windows")]
const TT_FILENAME: &str = "tt.exe";

use regex::Regex;
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
            .env("FOO", &"BAR")
            .stdin(Stdio::null())
            .current_dir(&self.test_directory)
            .args(command.split_ascii_whitespace())
            .output()
            .expect("failed to execute process");
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        let result = format!(
            "Command: {}\nExit status: {}\nSTDOUT:\n{}\nSTDERR:\n{}\n",
            command,
            output.status,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
        let dir = dunce::canonicalize(&self.test_directory)
            .expect("canonicalize")
            .to_str()
            .expect("dir")
            .replace("\\", "/")
            .to_string();
        let result = result.replace(&dir, "<DIRECTORY>");
        let tmp_regex = Regex::new("/\\.tmp/[0-9-]+/").unwrap();
        let result = tmp_regex.replace(&result, "/<TMP>/");
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
    let tt_binary = PathBuf::from(format!("../target/x86_64-unknown-linux-musl/release/{}", TT_FILENAME));
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
    runner.verify_execution("--getToolVersion bat");
}

#[test]
fn coreutils() {
    let runner = prepare_test("coreutils");
    runner.verify_execution("--download");
    runner.verify_execution("coreutils echo foo");
    runner.verify_execution("echo bar");
    runner.verify_execution("replace_version");
    runner.verify_execution("--getToolPath coreutils");
    runner.verify_execution("--getBinaryPath coreutils");
    runner.verify_execution("--getBinaryPath echo");
    runner.verify_execution("cmd");
    runner.verify_execution("dir");
    runner.verify_execution("ver");
    runner.verify_execution("environ");
    runner.verify_execution("coreutils false");
    runner.verify_execution("coreutils true");

    runner.verify_execution("coreutils no_such_command");
    runner.verify_execution("unconfigured_tool");
    runner.verify_execution("--invalid-config");
    runner.verify_execution("env");
}

#[test]
fn invalid_url() {
    let runner = prepare_test("invalid_url");
    runner.verify_execution("no_such_tool");
    runner.verify_execution("--download");
}
