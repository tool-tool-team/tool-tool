use crate::config::ToolConfiguration;
use crate::platform::{PlatformFunctions, Platform};
use crate::Result;
use std::path::Path;

pub struct Linux;

impl PlatformFunctions for Linux {
    fn rename_atomically(src: &Path, dst: &Path) -> Result<()> {
        Ok(std::fs::rename(src, dst)?)
    }
}


impl Platform for Linux {
    fn get_download_url<'a>(&self, tool_configuration: &'a ToolConfiguration) -> Option<&'a str> {
        tool_configuration
            .download
            .linux
            .as_deref()
            .or_else(|| tool_configuration.download.default.as_deref())
    }

    fn get_application_extensions(&self) -> &'static [&'static str] {
        &["", ".sh"]
    }

    fn get_name(&self) -> &'static str {
        "linux"
    }
}
