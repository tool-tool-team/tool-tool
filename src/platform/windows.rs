use crate::config::ToolConfiguration;
use crate::platform::Platform;

pub struct Windows;

#[cfg(target_os = "windows")]
impl crate::platform::PlatformFunctions for Windows {
    fn rename_atomically(src: &std::path::Path, dst: &std::path::Path) -> crate::Result<()> {
        if atomicwrites::move_atomic(src, dst).is_err() {
            report!("WARNING: Atomic move failed, falling back to plain move");
            std::fs::rename(src, dst)?;
        }
        Ok(())
    }
}

impl Platform for Windows {
    fn get_download_url<'a>(&self, tool_configuration: &'a ToolConfiguration) -> Option<&'a str> {
        tool_configuration
            .download
            .windows
            .as_deref()
            .or_else(|| tool_configuration.download.default.as_deref())
    }

    fn get_application_extensions(&self) -> &'static [&'static str] {
        &[".exe", ".cmd", ".bat", ""]
    }

    fn get_name(&self) -> &'static str {
        "windows"
    }
}
