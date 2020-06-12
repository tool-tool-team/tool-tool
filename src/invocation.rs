use std::ffi::OsStr;
use std::process::Command;

use anyhow::Context;

use crate::cache::CommandLine;
use crate::Result;

pub fn run_invocation(command_line: CommandLine) -> Result<i32> {
    let mut command = Command::new(command_line.binary);
    command.args(command_line.arguments);
    for (key, value) in command_line.env {
        command.env(OsStr::new(&key), OsStr::new(&value));
    }
    verbose!("Executing {:?}", command);
    let status = command
        .status()
        .with_context(|| format!("Unable to run invocation {:?}", command))?;
    let exitcode = status.code().unwrap_or(0);
    Ok(exitcode)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn assert_invocation(command_line: CommandLine, expected_exit_code: i32) {
        assert_eq!(run_invocation(command_line).unwrap(), expected_exit_code);
    }

    #[test]
    fn assert_simple_bash() {
        assert_invocation(
            CommandLine {
                binary: "bash".to_string(),
                arguments: vec!["-c".to_string(), "echo foo".to_string()],
                env: Default::default(),
            },
            0,
        );
    }

    #[test]
    fn assert_exit_code() {
        assert_invocation(
            CommandLine {
                binary: "bash".to_string(),
                arguments: vec!["-c".to_string(), "exit 123".to_string()],
                env: Default::default(),
            },
            123,
        );
    }

    #[test]
    fn assert_env_unset() {
        assert_invocation(
            CommandLine {
                binary: "bash".to_string(),
                arguments: vec!["-c".to_string(), "exit ${CODE}".to_string()],
                env: Default::default(),
            },
            0,
        );
    }

    #[test]
    fn assert_env_set() {
        let mut env: HashMap<String, String> = HashMap::new();
        env.insert("CODE".to_string(), "42".to_string());
        assert_invocation(
            CommandLine {
                binary: "bash".to_string(),
                arguments: vec!["-c".to_string(), "exit ${CODE}".to_string()],
                env,
            },
            42,
        );
    }
}
