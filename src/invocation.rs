use crate::args::Invocation;
use crate::Result;
use std::process::{Command, exit};

pub fn run_invocation(invocation: Invocation) -> Result<()> {
    dbg!(&invocation);
    let mut command = Command::new(invocation.command_name);
    command.args(invocation.args);
    let status = command.status()?;
    let exitcode = status.code().unwrap_or(0);
    exit(exitcode);
}