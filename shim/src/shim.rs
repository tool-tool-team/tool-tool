use anyhow::Context;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

pub type Result<T> = anyhow::Result<T>;

const EXIT_CODE_NOT_FOUND: i32 = 125;

// TODO: use PATHEXT env var?
#[cfg(target_os = "windows")]
const EXECUTABLE_EXTENSIONS: &[&str] = &[".exe", ".cmd", ".bat", ""];

#[cfg(target_os = "windows")]
const TOOL_TOOL_NAME: &str = "tt.exe";

#[cfg(target_os = "linux")]
const EXECUTABLE_EXTENSIONS: &[&str] = &[""];

#[cfg(target_os = "linux")]
const TOOL_TOOL_NAME: &str = "tt";

fn main() -> Result<()> {
    let mut args = std::env::args();

    // Determine tool name
    let binary = args.next().context("Could not determine first argument")?;
    let mut binary_path = PathBuf::from(&binary);
    let canonical_binary_path = binary_path
        .canonicalize()
        .or_else(|_| find_path(&binary_path))
        .with_context(|| format!("Could not determine canonical binary path for '{}'", binary))?;
    binary_path.set_extension("");
    let mut tool_name = binary_path
        .file_name()
        .with_context(|| format!("Could not determine tool name from path {:?}", binary_path))?
        .to_str()
        .with_context(|| format!("Could not convert tool name from path {:?}", binary_path))?
        .to_string();
    if tool_name == "tt" || tool_name == "tool-tool-shim" {
        tool_name = args
            .next()
            .context("tt invoked, but no tool name was given")?;
    }
    let args: Vec<_> = args.collect();

    // Find tool tool binary
    let directory = std::env::current_dir().context("Could not determine current directory")?;
    for directory in parent_directories(&directory) {
        let tool_path = directory.join(TOOL_TOOL_NAME);
        if tool_path.exists() {
            let mut command = Command::new(tool_path);
            command.arg("--from-shim");
            command.arg(&tool_name);
            command.args(&args);
            let status = command
                .status()
                .with_context(|| format!("Unable to run invocation {:?}", command))?;
            let exitcode = status.code().unwrap_or(0);
            if exitcode == EXIT_CODE_NOT_FOUND {
                continue;
            }
            exit(exitcode);
        }
    }
    for directory in path_directories()? {
        for extension in EXECUTABLE_EXTENSIONS {
            let tool_path = directory.join(format!("{}{}", tool_name, extension));
            let canonical_tool_path = tool_path
                .canonicalize()
                .context("Could not canonicalize path");
            if let Ok(tool_path) = canonical_tool_path {
                if tool_path == canonical_binary_path {
                    // Prevent calling ourselves, which would lead to infinite recursion
                    continue;
                }
            }

            if tool_path.exists() {
                let mut command = Command::new(tool_path);
                command.args(&args);
                let status = command
                    .status()
                    .with_context(|| format!("Unable to run invocation {:?}", command))?;
                let exitcode = status.code().unwrap_or(0);
                exit(exitcode);
            }
        }
    }
    eprintln!("Tool {} not found as tool nor in PATH", tool_name);
    exit(127);
}

fn find_path(path: &Path) -> Result<PathBuf> {
    for directory in path_directories()?.chain(std::iter::once(
        current_dir().context("Could not determine current dir")?,
    )) {
        for extension in EXECUTABLE_EXTENSIONS {
            let tool_path = directory.join(format!(
                "{}{}",
                path.with_extension("")
                    .to_str()
                    .context("Could not transform path")?,
                extension
            ));
            if tool_path.exists() {
                return Ok(tool_path
                    .canonicalize()
                    .context("Could not canonicalize path in find_path()")?);
            }
        }
    }
    Err(anyhow::anyhow!("Could not find '{:?}' in path", path))
}

fn parent_directories(start_directory: &Path) -> impl Iterator<Item = PathBuf> {
    ParentPathIterator {
        next_path: Some(start_directory.to_path_buf()),
    }
}

fn path_directories() -> Result<impl Iterator<Item = PathBuf>> {
    let path_var = std::env::var("PATH").context("Could not extract PATH variable")?;
    let paths: Vec<_> = std::env::split_paths(&path_var)
        .map(PathBuf::from)
        .collect();
    Ok(paths.into_iter())
}

struct ParentPathIterator {
    next_path: Option<PathBuf>,
}

impl Iterator for ParentPathIterator {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let next_path = &mut self.next_path;
        if next_path.is_some() {
            let mut parent_path = next_path
                .as_ref()
                .unwrap()
                .parent()
                .map(|path| path.to_path_buf());
            std::mem::swap(&mut parent_path, next_path);
            parent_path
        } else {
            None
        }
    }
}
