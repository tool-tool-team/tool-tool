use crate::config::ToolConfiguration;

pub fn get_download_url(tool_configuration: &ToolConfiguration) -> Option<&str> {
    tool_configuration.download.linux.as_deref().or(tool_configuration.download.default.as_deref())
}

pub const APPLICATION_EXTENSIONS: &[&str] = &["", ".sh"];
