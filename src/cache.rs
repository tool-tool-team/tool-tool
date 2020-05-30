use crate::config::get_config;
use crate::Result;
use anyhow::bail;
use anyhow::Context;
use std::fs::File;
use crate::download::download;
use std::ffi::OsStr;
use std::path::PathBuf;

pub struct Cache {
    tool_dir: PathBuf,
}

impl Cache {
    pub fn create() -> Result<Self> {
        let cache_dir = dirs::cache_dir().context("Unable to locate cache dir")?.join("tool-tool-v1");
        verbose!("Using cache_dir {:?}", cache_dir);
        let tmp_dir = cache_dir.join(format!(".tmp/{}-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos(), std::process::id()));
        verbose!("Using tmp_dir {:?}", tmp_dir);
        std::fs::create_dir_all(&tmp_dir)?;
        let configuration = get_config().context("Unable to load configuration")?;
        dbg!(&configuration);
        for tool in &configuration.tools {
            let url = tool.download_url();
            let file_name = url.rsplitn(2, "/").next().unwrap();
            verbose!("Downloading <{}> ({}) from <{}>", tool.name, file_name, url);
            let file_path = tmp_dir.join(file_name);
            download(url, &file_path);
            let tool_dir = tmp_dir.join(&tool.name);
            dbg!(&file_path);
            let extension = file_path.extension();
            if extension == Some(OsStr::new("zip")) {

                std::fs::create_dir_all(&tool_dir)?;
                let mut file = File::open(file_path)?;
                let mut archive = zip::ZipArchive::new(file).unwrap();
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let file_name: PathBuf = file.sanitized_name().components().skip(1).collect();
                    let outpath = tool_dir.join(file_name);

                    if (&*file.name()).ends_with('/') {
                        verbose!("File {} extracted to \"{}\"", i, outpath.as_path().display());
                        //std::fs::create_dir_all(&outpath).unwrap();
                    } else {
                        verbose!("File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
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
        }
        dbg!(&configuration);
        Ok(Cache {
            tool_dir: tmp_dir,
        })
    }

    pub fn get_command_path(&self, command: &str) -> Result<PathBuf> {
        Ok(self.tool_dir.join(format!("{}/{}.exe", command, command)))
    }
}
