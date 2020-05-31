use crate::config::ToolConfiguration;

pub fn get_download_url(tool_configuration: &ToolConfiguration) -> &str {
    &tool_configuration.download.windows
}

pub const APPLICATION_EXTENSIONS: &[&str] = &[".exe", ".cmd", ""];