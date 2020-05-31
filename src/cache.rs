use crate::config::{get_config, Configuration, ToolConfiguration};
use crate::Result;
use anyhow::bail;
use anyhow::Context;
use std::fs::File;
use crate::download::download;
use std::ffi::OsStr;
use std::path::PathBuf;
use crate::platform::{get_download_url, APPLICATION_EXTENSION};

pub struct Cache {
    configuration: Configuration,
    tools_dir: PathBuf,
}

impl Cache {
    pub fn create(configuration: Configuration) -> Result<Self> {
        let cache_dir = dirs::cache_dir().context("Unable to locate cache dir")?.join("tool-tool-v1");
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
        let tmp_dir = tools_dir.join(format!(".tmp/{}-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos(), std::process::id()));

        for tool in &configuration.tools {
            let tool_dir = self.get_tool_dir(tool);;
            if tool_dir.exists() {
                verbose!("Tool Directory for {} v{} found, skipping download", tool.name, tool.version);
                continue;
            }
            verbose!("Using tmp_dir {:?}", tmp_dir);
            std::fs::create_dir_all(&tmp_dir)?;
            std::fs::create_dir_all(tool_dir.parent().expect("Parent should exist"))?;
            let url = get_download_url(tool);
            let file_name = url.rsplitn(2, "/").next().unwrap();
            verbose!("Downloading <{}> ({}) from <{}>", tool.name, file_name, url);
            let file_path = tmp_dir.join(file_name);
            download(url, &file_path)?;
            let extract_dir = tmp_dir.join(&tool.name);
            dbg!(&file_path);
            let extension = file_path.extension();
            if extension == Some(OsStr::new("zip")) {

                std::fs::create_dir_all(&extract_dir)?;
                let file = File::open(file_path)?;
                let mut archive = zip::ZipArchive::new(file).unwrap();
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let file_name: PathBuf = file.sanitized_name().components().skip(1).collect();
                    let outpath = extract_dir.join(file_name);

                    if (&*file.name()).ends_with('/') {
                        //verbose!("File {} extracted to \"{}\"", i, outpath.as_path().display());
                        //std::fs::create_dir_all(&outpath).unwrap();
                    } else {
                        //verbose!("File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                std::fs::create_dir_all(&p).unwrap();
                            }
                        }
                        let mut outfile = std::fs::File::create(&outpath).unwrap();
                        std::io::copy(&mut file, &mut outfile).unwrap();
                    }
                }
            } else {
                bail!("Unsupported file extension for file {}", file_name);
            }
            atomicwrites::move_atomic(&extract_dir, &tool_dir)?
        }
        Ok(())
    }

    fn get_tool_dir(&self, tool: &ToolConfiguration) -> PathBuf {
        self.tools_dir.join(&tool.name).join(&tool.version)
    }

    pub fn get_command_path(&self, command: &str) -> Result<PathBuf> {
        let tool_configuration = self.configuration.tools.iter().find(|tool| tool.name == command).context("Tool not found")?;
        Ok(self.get_tool_dir(tool_configuration).join(format!("{}{}", command, APPLICATION_EXTENSION)))
    }
}
