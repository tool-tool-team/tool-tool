use crate::config::{Configuration, ToolConfiguration};
use crate::download::download;
use crate::platform::{get_download_url, APPLICATION_EXTENSIONS};
use crate::Result;
use anyhow::bail;
use anyhow::Context;
use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use tar::Archive;

pub struct Cache {
    configuration: Configuration,
    tools_dir: PathBuf,
}

impl Cache {
    pub fn create(configuration: Configuration) -> Result<Self> {
        let cache_dir = PathBuf::from(configuration.cache_dir.as_deref().expect("cache dir"));
        verbose!("Using cache_dir {:?}", cache_dir);
        let tools_dir = cache_dir.join("tools");
        Ok(Cache {
            configuration,
            tools_dir,
        })
    }
    pub fn init(&self) -> Result<()> {
        let tools_dir = &self.tools_dir;
        let configuration = &self.configuration;
        let tmp_dir = tools_dir.join(format!(
            ".tmp/{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            std::process::id()
        ));

        for tool in &configuration.tools {
            let tool_dir = self.get_tool_dir(tool);
            if tool_dir.exists() {
                verbose!(
                    "Tool Directory for {} v{} found, skipping download",
                    tool.name,
                    tool.version
                );
                continue;
            }
            verbose!("Using tmp_dir {:?}", tmp_dir);
            std::fs::create_dir_all(&tmp_dir)?;
            std::fs::create_dir_all(tool_dir.parent().expect("Parent should exist"))?;
            let url = get_download_url(tool).context("No download url configured")?;
            let file_name = url.rsplitn(2, "/").next().unwrap();
            verbose!("Downloading <{}> ({}) from <{}>", tool.name, file_name, url);
            let file_path = tmp_dir.join(file_name);
            download(url, &file_path)?;
            let extract_dir = tmp_dir.join(&tool.name);
            let extension = file_path.extension();
            std::fs::create_dir_all(&extract_dir)?;
            let zip_extension = Some(OsStr::new("zip"));
            let gz_extension = Some(OsStr::new("gz"));
            if extension == zip_extension {
                let file = File::open(file_path)?;
                let mut archive = zip::ZipArchive::new(file).unwrap();
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let file_name: PathBuf = file.sanitized_name().components().skip(1).collect();
                    let outpath = extract_dir.join(file_name);

                    if (&*file.name()).ends_with('/') {
                        std::fs::create_dir_all(&outpath).unwrap();
                    } else {
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                std::fs::create_dir_all(&p).unwrap();
                            }
                        }
                        let mut outfile = std::fs::File::create(&outpath).unwrap();
                        std::io::copy(&mut file, &mut outfile).unwrap();
                    }
                }
            } else if extension == gz_extension {
                let file = File::open(file_path)?;
                let tar = GzDecoder::new(file);
                let mut archive = Archive::new(tar);
                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let path: PathBuf = entry.path()?.components().skip(1).collect();
                    let outpath = extract_dir.join(path);
                    std::fs::create_dir_all(outpath.parent().expect("parent"))?;
                    entry.unpack(&outpath)?;
                }
            } else {
                bail!(
                    "Unsupported file extension for file {}: {:?}",
                    file_name,
                    extension
                );
            }
            atomicwrites::move_atomic(&extract_dir, &tool_dir)?
        }
        Ok(())
    }

    pub fn get_tool_dir(&self, tool: &ToolConfiguration) -> PathBuf {
        self.tools_dir.join(&tool.name).join(&tool.version)
    }

    pub fn get_command_paths(&self, command: &str) -> Result<Vec<PathBuf>> {
        let tool_configuration = self
            .configuration
            .tools
            .iter()
            .find(|tool| tool.name == command || tool.commands.contains_key(command))
            .context("Tool not found")?;
        let mut binary: &str = tool_configuration
            .commands
            .get(command)
            .map(Deref::deref)
            .unwrap_or(command);
        let tool_dir = self.get_tool_dir(tool_configuration);
        let mut result: Vec<PathBuf> = if binary.contains(" ") {
            // handle composite commands
            let split: Vec<&str> = binary.rsplitn(2, " ").collect();
            binary = split[0];
            self.get_command_paths(split[1])?
        } else {
            vec![]
        };
        let mut command_candidates = APPLICATION_EXTENSIONS
            .iter()
            .map(|extension| tool_dir.join(format!("{}{}", binary, extension)));
        let command_path = command_candidates
            .find(|tool_path| tool_path.exists())
            .with_context(|| format!("Tool executable {} not found", command))?;
        result.push(command_path);
        Ok(result)
    }
}
