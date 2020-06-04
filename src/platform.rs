use crate::config::ToolConfiguration;
use crate::Result;
use std::path;

pub trait PlatformFunctions {
    fn get_download_url(tool_configuration: &ToolConfiguration) -> Option<&str>;
    fn rename_atomically(src: &path::Path, dst: &path::Path) -> Result<()>;

    const APPLICATION_EXTENSIONS: &'static [&'static str];
}


#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::Windows as Platform;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;



