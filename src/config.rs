use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub const CONFIG_FILENAME: &str = ".tool-tool.v1.yaml";

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuration {
    pub cache_dir: Option<String>,
    pub tools: Vec<ToolConfiguration>,
    #[serde(skip_deserializing)]
    pub configuration_files: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadUrls {
    pub default: Option<String>,
    pub linux: Option<String>,
    pub windows: Option<String>,
}

pub fn get_config(binary_name: &str) -> Result<Configuration> {
    let binary_path = PathBuf::from(binary_name);
    let mut parent_directory = binary_path;
    let mut config_path: PathBuf;
    loop {
        config_path = parent_directory.join(CONFIG_FILENAME);
        if config_path.exists() {
            break;
        }
        parent_directory = parent_directory
            .parent()
            .unwrap_or_else(|| {
                panic!(
                    "Could not find config file {}, search from directory {:?}",
                    CONFIG_FILENAME,
                    PathBuf::from(binary_name).parent().unwrap()
                )
            })
            .to_path_buf();
    }
    let config_path = parent_directory.join(CONFIG_FILENAME);
    verbose!("Reading configuration from {:?}", config_path);
    read_config(
        Box::new(File::open(&config_path)?),
        &config_path.to_string_lossy(),
    )
}

fn read_config(mut reader: Box<dyn Read>, path: &str) -> Result<Configuration> {
    let mut configuration: Configuration = serde_yaml::from_reader(reader.as_mut())?;
    for tool in &mut configuration.tools {
        replace_templates(&mut tool.download.default, &tool.version);
        replace_templates(&mut tool.download.linux, &tool.version);
        replace_templates(&mut tool.download.windows, &tool.version);
        // Add default command
        if tool.commands.is_empty() {
            tool.commands.insert(tool.name.clone(), tool.name.clone());
        }
        // Add missing dirs
        for value in &mut tool.commands.values_mut() {
            if !value.contains("${dir}") {
                *value = format!("${{dir}}/{}", value);
            }
        }
    }
    configuration.cache_dir.get_or_insert(
        PathBuf::from(path)
            .parent()
            .expect("config parent")
            .join(".tool-tool")
            .join("v1")
            .as_path()
            .to_str()
            .expect("Tool dir")
            .to_string()
            .replace('\\', "/"),
    );
    configuration.configuration_files.push(path.to_string());
    Ok(configuration)
}

fn replace_templates(string: &mut Option<String>, version: &str) {
    if let Some(inner) = string {
        *string = Some(inner.replace("${version}", version));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn verify_config(string: &'static str) {
        let cursor = Cursor::new(string.as_bytes());
        let config = read_config(Box::new(cursor), "root/foo.yaml").unwrap();
        let mut settings = insta::Settings::clone_current();
        settings.set_sort_maps(true);
        settings.bind(|| {
            insta::assert_yaml_snapshot!(config);
        });
    }

    #[test]
    fn simple() {
        verify_config(
            r#"
tools:
  - name: lsd
    version: 0.17.0
    download:
      linux: https://github.com/Peltoche/lsd/releases/download/${version}/lsd-${version}-x86_64-unknown-linux-gnu.tar.gz
      windows: https://github.com/Peltoche/lsd/releases/download/${version}/lsd-${version}-x86_64-pc-windows-msvc.zip
        "#,
        );
    }

    #[test]
    fn with_commands() {
        verify_config(
            r#"
tools:
  - name: xyz
    version: 0.17.0
    strip_directories: 0
    download:
      default: https://default.tar.gz
      windows: https://windows.tar.gz
    commands:
      foo: bar
      fizz: ${dir}/buzz
        "#,
        );
    }

    #[test]
    fn test_get_config() {
        let config = get_config(
            std::env::current_dir()
                .unwrap()
                .join("src/config.rs")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            config.configuration_files,
            vec![std::env::current_dir()
                .unwrap()
                .join(".tool-tool.v1.yaml")
                .to_str()
                .unwrap()
                .to_string()]
        );
    }
}
