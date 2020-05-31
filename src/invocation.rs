use crate::args::Invocation;
use crate::Result;
use std::process::{Command, exit};
use crate::config::{get_config, Configuration};
use crate::cache::Cache;
use crate::{NAME, VERSION};
use anyhow::Context;

pub fn run_invocation(invocation: Invocation, configuration: Configuration) -> Result<()> {
    verbose!("{} {}", NAME, VERSION);
    let cache = Cache::create(configuration)?;
    cache.init()?;
    let command_path = cache.get_command_path(&invocation.command_name)?;
    let mut command = Command::new(command_path);
    command.args(invocation.args);
    let status = command.status()?;
    let exitcode = status.code().unwrap_or(0);
    exit(exitcode);
}