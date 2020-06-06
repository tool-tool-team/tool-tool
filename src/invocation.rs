use crate::args::Invocation;
use crate::cache::Cache;
use crate::config::Configuration;
use crate::Result;
use crate::{NAME, VERSION};
use anyhow::Context;
use std::ffi::OsStr;
use std::process::{exit, Command};

pub fn run_invocation(invocation: Invocation, configuration: Configuration) -> Result<()> {
    verbose!("{} {}", NAME, VERSION);
    let cache = Cache::create(configuration.clone())?;
    cache.init()?;
    let command_line = cache
        .get_command_line(&invocation.command_name)
        .with_context(|| format!("Could not run command '{}'", invocation.command_name))?;
    let mut command = Command::new(command_line.binary);
    for arg in command_line.arguments {
        command.arg(arg);
    }
    command.args(invocation.args);
    for (key, value) in command_line.env {
        command.env(OsStr::new(&key), OsStr::new(&value));
    }
    verbose!("Executing {:?}", command);
    let status = command
        .status()
        .with_context(|| format!("Unable to run invocation {:?}", command))?;
    let exitcode = status.code().unwrap_or(0);
    if exitcode != 0 {
        report!(
            "Command '{}' terminated with exit code {}",
            invocation.command_name,
            exitcode
        );
    }
    exit(exitcode);
}
