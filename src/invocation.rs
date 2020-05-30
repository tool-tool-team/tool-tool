use crate::args::Invocation;
use crate::Result;
use std::process::{Command, exit};
use crate::config::get_config;
use crate::cache::Cache;
use crate::{NAME, VERSION};

pub fn run_invocation(invocation: Invocation) -> Result<()> {
    verbose!("{} {}", NAME, VERSION);
    dbg!(&invocation);
    let cache = Cache::create()?;
    let command_path = cache.get_command_path(&invocation.command_name)?;
    dbg!(&command_path);
    let mut command = Command::new(command_path);
    command.args(invocation.args);
    let status = command.status()?;
    let exitcode = status.code().unwrap_or(0);
    exit(exitcode);
}