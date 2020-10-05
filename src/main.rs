pub use anyhow::{bail, Result};

macro_rules! verbose {
     ($($arg:tt)+) => ({
        if(crate::VERBOSE.load(std::sync::atomic::Ordering::Relaxed)) {
            eprint!("🔧 ");
            eprintln!($($arg)+);
        }
    });
}

macro_rules! report {
     ($($arg:tt)+) => ({
        eprint!("🔧 ");
        eprintln!($($arg)+);
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
use crate::cache::{Cache, CommandNotFoundError};
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

const EXIT_CODE_NOT_FOUND: i32 = 125;

fn main() -> Result<()> {
    let verbose_env = std::env::var("TOOL_TOOL_VERBOSE").is_ok();
    let args = parse_args(&mut std::env::args().skip(1), verbose_env)?;
    let binary = std::env::args().next().unwrap();
    match args {
        Args::Help => {
            let configuration = get_config(&binary).unwrap_or_default();
            print_help(&configuration, &mut std::io::stdout().lock())?;
        }
        Args::Download => {
            VERBOSE.store(true, Ordering::Relaxed);
            init_cache(&binary)?;
            report!("Download complete!");
        }
        Args::GetBinaryPath { command_name } => {
            VERBOSE.store(false, Ordering::Relaxed);
            let cache = create_cache(&binary)?;
            let command_line = cache.get_command_line(&command_name)?;
            println!(
                "{}",
                make_absolute(&std::path::Path::new(&command_line.binary))?
            );
        }
        Args::GetToolPath { tool_name } => {
            VERBOSE.store(false, Ordering::Relaxed);
            let cache = create_cache(&binary)?;
            let tool_configuration = cache
                .configuration
                .tools
                .iter()
                .find(|tool| tool.name == tool_name)
                .with_context(|| format!("Tool '{}' not found", tool_name))?;
            let tool_dir = cache.get_tool_dir(tool_configuration);
            println!("{}", make_absolute(tool_dir.as_path())?);
        }
        Args::Invocation(mut invocation) => {
            VERBOSE.store(invocation.verbose, Ordering::Relaxed);
            let cache = init_cache(&binary)?;
            let command_result = cache.get_command_line(&invocation.command_name);
            if invocation.from_shim {
                if let Err(err) = &command_result {
                    if err.is::<CommandNotFoundError>() {
                        // Emit special exit code and suppress error output when invoked from shim to let it know
                        exit(EXIT_CODE_NOT_FOUND);
                    }
                }
            }
            let mut command_line = command_result
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

fn init_cache(binary_name: &str) -> Result<Cache> {
    verbose!("{} {}", NAME, VERSION);
    let cache = create_cache(binary_name)?;
    verbose!("Cache initialized");
    Ok(cache)
}

fn create_cache(binary_name: &str) -> Result<Cache> {
    let configuration = get_config(&binary_name).with_context(|| format!("Unable to load configuration, please ensure that a file called {} exists, either in the current directory or an ancestor", CONFIG_FILENAME))?;
    let mut cache = Cache::create(configuration)?;
    cache.init().context("Could not initialize cache")?;
    Ok(cache)
}

fn make_absolute(path: &std::path::Path) -> Result<String> {
    Ok(dunce::canonicalize(path)?
        .to_string_lossy()
        .replace("\\", "/"))
}
