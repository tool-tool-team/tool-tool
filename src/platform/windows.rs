use crate::config::ToolConfiguration;
use crate::platform::PlatformFunctions;
use crate::Result;
use std::path::Path;

pub struct Windows;

impl PlatformFunctions for Windows {
    fn get_download_url(tool_configuration: &ToolConfiguration) -> Option<&str> {
        tool_configuration
            .download
            .windows
            .as_deref()
            .or_else(|| tool_configuration.download.default.as_deref())
    }

    fn rename_atomically(src: &Path, dst: &Path) -> Result<()> {
        Ok(atomicwrites::move_atomic(src, dst)?)
    }

    const APPLICATION_EXTENSIONS: &'static [&'static str] = &[".exe", ".cmd", ".bat", ""];
}
