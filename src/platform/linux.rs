use crate::config::ToolConfiguration;
use crate::platform::PlatformFunctions;
use crate::Result;
use std::path::Path;

pub struct Linux;

impl PlatformFunctions for Linux {
    fn get_download_url(tool_configuration: &ToolConfiguration) -> Option<&str> {
        tool_configuration
            .download
            .linux
            .as_deref()
            .or(tool_configuration.download.default.as_deref())
    }

    fn rename_atomically(src: &Path, dst: &Path) -> Result<()> {
        Ok(std::fs::rename(src, dst)?)
    }

    const APPLICATION_EXTENSIONS: &'static [&'static str] = &["", ".sh"];
}
