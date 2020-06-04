pub use anyhow::{bail, Result};

macro_rules! verbose {
     ($($arg:tt)+) => ({
        if(crate::VERBOSE.load(std::sync::atomic::Ordering::Relaxed)) {
            print!("TT> ");
            println!($($arg)+);
        }
    });
}
pub mod args;
pub mod cache;
pub mod config;
pub mod download;
pub mod help;
pub mod invocation;
pub mod platform;
pub mod template;

use crate::args::{parse_args, Args};
use crate::config::get_config;
use crate::help::print_help;
use crate::invocation::run_invocation;
use anyhow::Context;
use std::sync::atomic::{AtomicBool, Ordering};

pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

pub static VERBOSE: AtomicBool = AtomicBool::new(false);

fn main() -> Result<()> {
    let verbose_env = std::env::var("TOOL_TOOL_VERBOSE").is_ok();
    let args = parse_args(&mut std::env::args().skip(1), verbose_env)?;

    let configuration = get_config().context("Unable to load configuration")?;

    match args {
        Args::Help => print_help(),
        Args::Invocation(invocation) => {
            VERBOSE.store(invocation.verbose, Ordering::Relaxed);
            run_invocation(invocation, configuration)?
        }
    }
    Ok(())
}
