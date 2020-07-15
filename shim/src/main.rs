use anyhow::Context;
use std::path::{PathBuf, Path};
use std::process::{exit, Command};

pub type Result<T> = anyhow::Result<T>;

const TOOL_TOOL_NAME: &str = "tt.exe";

fn main() -> Result<()> {
    let mut args = std::env::args();

    // Determine tool name
    let binary = args.next().with_context(|| format!("Could not determine first argument"))?;
    let args: Vec<_> = args.collect();
    let mut binary_path = PathBuf::from(&binary);
    binary_path.set_extension("");
    let tool_name = binary_path.file_name().with_context(|| format!("Could not determine tool name from path {:?}", binary_path))?;
    dbg!(&tool_name);

    // Find tool tool binary
    let directory = std::env::current_dir().with_context(|| format!("Could not determine current directory"))?;
    let directories = parent_directories(&directory);
    for directory in directories {
        let tool_path = directory.join(TOOL_TOOL_NAME);
        if tool_path.exists() {
            let mut command = Command::new(tool_path);
            command.arg("--from-shim");
            command.arg(tool_name);
            command.args(&args);
            println!("Executing {:?}", command);
            let status = command
                .status()
                .with_context(|| format!("Unable to run invocation {:?}", command))?;
            let exitcode = status.code().unwrap_or(0);
            if exitcode == 404 {
                println!("Tool {:?} not found, continue from the top", tool_name);
                continue;
            }
            exit(exitcode);
        }
    }
    println!("Tool {:?} not found as tool nor in PATH", tool_name);
/*    let tool_path = loop {

        if let Some(parent) = directory.parent() {
            directory = parent.to_path_buf();
        } else {
            // No tool tool found on path try using PATH
            let env_path = std::env::var("PATH").context("Unable to read PATH environment variable")?;
            dbg!(env_path);
            println!("Tool {:?} not found in PATH", tool_name);
            exit(127)
        }
    };*/
    Ok(())
}

fn parent_directories(start_directory: &Path) -> impl Iterator<Item = PathBuf> {
    ParentPathIterator {
        next_path: Some(start_directory.to_path_buf()),
    }
}
/*
fn path_directories() -> impl Iterator<Item = PathBuf> {
    ParentPathIterator {
        next_path: Some(start_directory.to_path_buf()),
    }
}
*/

struct ParentPathIterator {
    next_path: Option<PathBuf>,
}

impl Iterator for ParentPathIterator {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let next_path = &mut self.next_path;
        if next_path.is_some() {
            let mut parent_path = next_path.as_ref().unwrap().parent().map(|path| path.to_path_buf());
            std::mem::swap(&mut parent_path, next_path);
            parent_path
        } else {
            None
        }
    }
}