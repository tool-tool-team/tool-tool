use crate::Result;
use std::collections::HashMap;
use std::fs::File;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Configuration {
    pub tools: Vec<ToolConfiguration>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct ToolConfiguration {
    pub name: String,
    pub version: String,
    pub download: DownloadUrls,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct DownloadUrls {
    pub linux: String,
    pub windows: String,
}


impl ToolConfiguration {
    #[cfg(target_os = "linux")]
    pub fn download_url(&self) -> &str {
        &self.download.linux
    }
    #[cfg(target_os = "windows")]
    pub fn download_url(&self) -> &str {
        &self.download.windows
    }
}

pub fn get_config() -> Result<Configuration> {
    Ok(serde_yaml::from_reader(File::open(".tool-tool.v1.yaml")?)?)
}