use crate::config::ToolConfiguration;

pub fn get_download_url(tool_configuration: &ToolConfiguration) -> &str {
    &tool_configuration.download.linux
}

pub const APPLICATION_EXTENSIONS: &[&str] = &["", ".sh"];
