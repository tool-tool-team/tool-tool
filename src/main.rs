pub use anyhow::{Result, bail};

pub mod args;
pub mod invocation;
pub mod help;

use crate::args::{parse_args, Args};
use crate::help::print_help;
use crate::invocation::run_invocation;

pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

fn main() -> Result<()> {
    let args = parse_args(&mut std::env::args().skip(1))?;
    match args {
        Args::Help => print_help(),
        Args::Invocation(invocation) => {
            run_invocation(invocation)?
        }
    }
    Ok(())
}
