use crate::Result;
use std::collections::HashMap;
use std::fs::File;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Configuration {
    pub tools: Vec<ToolConfiguration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ToolConfiguration {
    pub name: String,
    pub version: String,
    pub download: DownloadUrls,
    #[serde(default)]
    pub commands: HashMap<String, String>,
    #[serde(default)]
    pub export_directory: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DownloadUrls {
    pub linux: String,
    pub windows: String,
}

pub fn get_config() -> Result<Configuration> {
    Ok(serde_yaml::from_reader(File::open(".tool-tool.v1.yaml")?)?)
}