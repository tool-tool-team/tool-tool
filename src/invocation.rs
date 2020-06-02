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
    let command_paths = cache.get_command_paths(&invocation.command_name)?;
    let mut command = Command::new(command_paths.first().expect("at least one command"));
    for subcommand in command_paths.iter().skip(1) {
        command.arg(subcommand);
    }
    command.args(invocation.args);
    for tool in &configuration.tools {
        if let Some(name) = &tool.export_directory {
            let tool_dir = cache.get_tool_dir(tool);
            command.env(OsStr::new(name), tool_dir.as_os_str().to_os_string());
        }
    }
    verbose!("Executing {:?}", command);
    let status = command
        .status()
        .with_context(|| format!("Unable to run invocation {:?}", command))?;
    let exitcode = status.code().unwrap_or(0);
    exit(exitcode);
}
