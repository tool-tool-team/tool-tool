use crate::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Configuration {
    pub cache_dir: Option<String>,
    pub tools: Vec<ToolConfiguration>,
    #[serde(skip_deserializing)]
    pub configuration_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ToolConfiguration {
    pub name: String,
    pub version: String,
    pub download: DownloadUrls,
    #[serde(default)]
    pub commands: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    // strip directories when unpacking zip/tar.gz downloads
    #[serde(default = "default_strip_directories")]
    pub strip_directories: usize,
}

fn default_strip_directories() -> usize {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DownloadUrls {
    pub default: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

pub fn get_config() -> Result<Configuration> {
    // TODO: resolve upwards
    let config_path = std::env::current_dir()?.join(".tool-tool.v1.yaml");
    verbose!("Reading configuration from {:?}", config_path);
    let mut configuration: Configuration = serde_yaml::from_reader(File::open(&config_path)?)?;
    for tool in &mut configuration.tools {
        replace_templates(&mut tool.download.default, &tool.version);
        replace_templates(&mut tool.download.linux, &tool.version);
        replace_templates(&mut tool.download.windows, &tool.version);
        // Add default command
        if tool.commands.is_empty() {
            tool.commands.insert(tool.name.clone(), tool.name.clone());
        }
        // Add missing dirs
        for (_, value) in &mut tool.commands {
            if !value.contains("${dir}") {
                *value = format!("${{dir}}{}{}", std::path::MAIN_SEPARATOR, value);
            }
        }
    }
    configuration.cache_dir.get_or_insert(
        config_path
            .parent()
            .expect("config parent")
            .join(".tool-tool")
            .join("v1")
            .as_path()
            .to_str()
            .expect("Tool dir")
            .to_string(),
    );
    configuration
        .configuration_files
        .push(config_path.to_string_lossy().to_string());
    Ok(configuration)
}

fn replace_templates(string: &mut Option<String>, version: &str) {
    if let Some(inner) = string {
        *string = Some(inner.replace("${version}", version));
    }
}
