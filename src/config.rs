use crate::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

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
    pub default: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

pub fn get_config() -> Result<Configuration> {
    let mut configuration: Configuration = serde_yaml::from_reader(File::open(".tool-tool.v1.yaml")?)?;
    for tool in &mut configuration.tools {
        replace_templates(&mut tool.download.default, &tool.version);
        replace_templates(&mut tool.download.linux, &tool.version);
        replace_templates(&mut tool.download.windows, &tool.version);
    }
    Ok(configuration)
}

fn replace_templates(string: &mut Option<String>, version: &str) {
    if let Some(inner) = string {
        *string = Some(inner.replace("${version}", version));
    }
}
