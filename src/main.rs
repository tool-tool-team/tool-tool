pub use anyhow::{bail, Result};

macro_rules! verbose {
     ($($arg:tt)+) => ({
        if(crate::VERBOSE.load(std::sync::atomic::Ordering::Relaxed)) {
            print!("🔧 ");
            println!($($arg)+);
        }
    });
}

macro_rules! report {
     ($($arg:tt)+) => ({
        print!("🔧 ");
        println!($($arg)+);
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
use crate::cache::Cache;
use crate::config::{get_config, CONFIG_FILENAME};
use crate::help::print_help;
use crate::invocation::run_invocation;
use anyhow::Context;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub static VERBOSE: AtomicBool = AtomicBool::new(false);

fn main() -> Result<()> {
    let verbose_env = std::env::var("TOOL_TOOL_VERBOSE").is_ok();
    let args = parse_args(&mut std::env::args().skip(1), verbose_env)?;

    match args {
        Args::Help => {
            let configuration = get_config().unwrap_or_default();
            print_help(&configuration, &mut std::io::stdout().lock())?
        }
        Args::Invocation(mut invocation) => {
            VERBOSE.store(invocation.verbose, Ordering::Relaxed);
            let configuration = get_config().with_context(|| format!("Unable to load configuration, please ensure that a file called {} exists, either in the current directory or an ancestor", CONFIG_FILENAME))?;
            verbose!("{} {}", NAME, VERSION);
            let mut cache = Cache::create(configuration)?;
            cache.init().context("Could not initialize cache")?;
            verbose!("Cache initialized");
            let mut command_line = cache
                .get_command_line(&invocation.command_name)
                .with_context(|| format!("Could not run command '{}'", invocation.command_name))?;
            command_line.arguments.append(&mut invocation.args);
            let exitcode = run_invocation(command_line)?;
            if exitcode != 0 {
                report!(
                    "Command '{}' terminated with exit code {}",
                    invocation.command_name,
                    exitcode
                );
            }
            exit(exitcode);
        }
    }
    Ok(())
}
