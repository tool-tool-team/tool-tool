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
    let command_line = cache.get_command_line(&invocation.command_name)?;
    let mut command = Command::new(command_line.binary);
    for arg in command_line.arguments {
        command.arg(arg);
    }
    command.args(invocation.args);
    for tool in &configuration.tools {
        if let Some(env_name) = &tool.export_directory_as {
            let tool_dir = cache.get_tool_dir(tool);
            command.env(OsStr::new(env_name), tool_dir.as_os_str().to_os_string());
        }
    }
    verbose!("Executing {:?}", command);
    let status = command
        .status()
        .with_context(|| format!("Unable to run invocation {:?}", command))?;
    let exitcode = status.code().unwrap_or(0);
    exit(exitcode);
}
