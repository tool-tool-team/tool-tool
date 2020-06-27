use crate::config::ToolConfiguration;
use crate::Result;
use std::path;

pub trait PlatformFunctions {
    fn rename_atomically(src: &path::Path, dst: &path::Path) -> Result<()>;
}

pub trait Platform {
    fn get_download_url<'a>(&self, tool_configuration: &'a ToolConfiguration) -> Option<&'a str>;
    fn get_application_extensions(&self) -> &'static [&'static str];
    fn get_name(&self) -> &'static str;
}

mod linux;
mod windows;

pub use linux::Linux;
pub use windows::Windows;

#[cfg(target_os = "linux")]
pub use linux::Linux as PlatformFns;

#[cfg(target_os = "windows")]
pub use windows::Windows as PlatformFns;
