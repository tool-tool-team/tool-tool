use crate::config::{Configuration, ToolConfiguration};
use crate::download::download;
use crate::platform::{Platform, PlatformFns, PlatformFunctions};
use crate::template::template;
use crate::Result;
use anyhow::bail;
use anyhow::Context;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use tar::Archive;

pub struct Cache {
    configuration: Configuration,
    tools_dir: PathBuf,
    platform: Box<dyn Platform>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommandLine {
    pub binary: String,
    pub arguments: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CommandNotFoundError {
    pub command: String,
}

impl fmt::Display for CommandNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command '{}' not found", self.command)
    }
}

impl Cache {
    pub fn create(configuration: Configuration) -> Result<Self> {
        for configuration_file in &configuration.configuration_files {
            verbose!("Loaded configuration from {}", configuration_file);
        }
        let cache_dir = configuration.cache_dir.as_deref().expect("cache dir");
        verbose!("Using cache_dir {}", cache_dir);
        let cache_dir = PathBuf::from(cache_dir);
        let tools_dir = cache_dir.join("tools");
        Ok(Cache {
            configuration,
            tools_dir,
            platform: Box::new(PlatformFns {}),
        })
    }
    pub fn init(&mut self) -> Result<()> {
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
                    "Tool found, skipping download for {} v{}",
                    tool.name,
                    tool.version
                );
                continue;
            }
            std::fs::create_dir_all(&tmp_dir).with_context(|| format!("Unable to create tmp dir {:?}", tmp_dir))?;
            std::fs::create_dir_all(tool_dir.parent().expect("Parent should exist"))?;
            let url = self
                .platform
                .get_download_url(tool)
                .with_context(|| format!("No download url configured for {}", tool.name))?;
            let file_name = url.rsplitn(2, '/').next().unwrap();
            report!(
                "Downloading <{} {}> ({}) from <{}>",
                tool.name,
                tool.version,
                file_name,
                url
            );
            verbose!("Using tmp_dir {:?}", tmp_dir);
            let file_path = tmp_dir.join(file_name);
            download(url, &file_path)
                .with_context(|| format!("Unable to download {} to {:?}", url, file_path))?;
            let extract_dir = tmp_dir.join(&tool.name);
            let extension = file_path.extension();
            std::fs::create_dir_all(&extract_dir).with_context(|| format!("Unable to extract directory {:?}", extract_dir))?;
            let zip_extension = Some(OsStr::new("zip"));
            let gz_extension = Some(OsStr::new("gz"));
            let tgz_extension = Some(OsStr::new("tgz"));
            if extension == zip_extension {
                let file = File::open(&file_path)?;
                let mut archive = zip::ZipArchive::new(file).with_context(|| format!("Unable to open zip file {:?}", file_path))?;
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).with_context(|| format!("Unable to open zip entry {:?}", i))?;
                    let file_name: PathBuf = file
                        .sanitized_name()
                        .components()
                        .skip(tool.strip_directories)
                        .collect();
                    let outpath = extract_dir.join(file_name);

                    if (&*file.name()).ends_with('/') {
                        std::fs::create_dir_all(&outpath).unwrap();
                    } else {
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                std::fs::create_dir_all(&p).with_context(|| format!("Unable to zip path {:?}", p))?;
                            }
                        }
                        let mut outfile = std::fs::File::create(&outpath).with_context(|| {
                            format!("Could not create output file '{:?}'", outpath)
                        })?;
                        std::io::copy(&mut file, &mut outfile).with_context(|| format!("Unable to extract file {:?} to path {:?}", outpath, outfile))?;
                    }
                }
            } else if extension == gz_extension || extension == tgz_extension {
                let file = File::open(&file_path)?;
                let tar = GzDecoder::new(file);
                let mut archive = Archive::new(tar);
                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let path: PathBuf = entry
                        .path()?
                        .components()
                        .skip(tool.strip_directories)
                        .collect();
                    let outpath = extract_dir.join(path);
                    std::fs::create_dir_all(outpath.parent().expect("parent"))?;
                    entry.unpack(&outpath)?;
                }
            } else {
                // save as tool name
                let from = file_path.as_os_str();
                let to = extract_dir.join(&tool.name);
                std::fs::rename(from, &to)
                    .with_context(|| format!("Unable to rename from {:?} to {:?}", from, to))?;
            }
            PlatformFns::rename_atomically(&extract_dir, &tool_dir).with_context(|| format!("Unable to atomically rename from {:?} to {:?}", extract_dir, tool_dir))?;
        }

        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir)
                .with_context(|| format!("Could not remove temp dir {:?}", tmp_dir))?;
        }
        Ok(())
    }

    pub fn get_tool_dir(&self, tool: &ToolConfiguration) -> PathBuf {
        self.tools_dir.join(&tool.name).join(&tool.version)
    }

    pub fn get_command_line(&self, command: &str) -> Result<CommandLine> {
        let tool_configuration = self
            .configuration
            .tools
            .iter()
            .find(|tool| tool.commands.contains_key(command))
            .with_context(|| CommandNotFoundError {
                command: command.to_string(),
            })?;
        let command_line: &str = tool_configuration
            .commands
            .get(command)
            .map(Deref::deref)
            .unwrap_or(command);
        let tool_dir = self.get_tool_dir(tool_configuration);
        let replace_fn = |name: &str| match name {
            "dir" => Ok(tool_dir.to_string_lossy().to_string()),
            name => {
                if name.starts_with("cmd:") {
                    let command_name = &name[4..];
                    let command_line = self.get_command_line(command_name).with_context(|| {
                        format!("Could not find tool command '{}'", command_name)
                    })?;
                    Ok(command_line.binary)
                } else if name.starts_with("dir:") {
                    let tool_name = &name[4..];
                    let tool = self
                        .configuration
                        .tools
                        .iter()
                        .find(|tool| tool.name == tool_name)
                        .with_context(|| {
                            format!("Could not find tool '{}' in tools list", tool_name)
                        })?;
                    let tool_dir = self.get_tool_dir(tool);
                    Ok(tool_dir.to_string_lossy().to_string())
                } else if name.starts_with("linux:") {
                    if self.platform.get_name() == "linux" {
                        Ok(name[6..].to_string())
                    } else {
                        Ok("".to_string())
                    }
                } else if name.starts_with("windows:") {
                    if self.platform.get_name() == "windows" {
                        Ok(name[8..].to_string())
                    } else {
                        Ok("".to_string())
                    }
                } else {
                    bail!("Unsupported template: '{}'", name)
                }
            }
        };
        let command_results: Vec<Result<String>> = command_line
            .split(' ')
            .map(|part| template(part, replace_fn))
            .collect();
        let command_parts: Result<Vec<String>> = command_results.into_iter().collect();
        let mut command_parts: Vec<String> = command_parts?;
        let command = command_parts.remove(0);
        let mut command_candidates = self
            .platform
            .get_application_extensions()
            .iter()
            .map(|extension| PathBuf::from(format!("{}{}", command, extension)));
        let command_path = command_candidates
            .find(|tool_path| tool_path.exists())
            .with_context(|| format!("Tool executable {} not found", command))?;

        let mut env = tool_configuration.env.clone();
        for (key, value) in &mut env {
            *value = template(value, replace_fn).with_context(|| {
                format!(
                    "Could not replace template string for env '{}' to '{}'",
                    key, value
                )
            })?;
        }

        Ok(CommandLine {
            binary: command_path.to_string_lossy().to_string(),
            arguments: command_parts,
            env,
        })
    }
}

#[cfg(test)]
mod tests {
    use mockito::mock;
    use std::io::{Cursor, Write};

    use super::*;
    use crate::config::DownloadUrls;
    use std::fs::read_to_string;
    use tempfile::TempDir;

    #[test]
    fn empty_tool_list() {
        let (configuration, temp_dir) = create_configuration();
        let mut cache = Cache::create(configuration).unwrap();
        cache.init().unwrap();
        assert_eq!(cache.tools_dir, temp_dir.path().join("tools"))
    }

    fn create_configuration() -> (Configuration, tempfile::TempDir) {
        let mut configuration: Configuration = Default::default();
        let temp_dir = tempfile::tempdir().unwrap();
        configuration.cache_dir = Some(temp_dir.path().to_str().unwrap().to_string());
        (configuration, temp_dir)
    }

    #[test]
    fn download_single_file_tool() {
        let path = "/cache/tool1";
        let mut commands = HashMap::new();
        commands.insert(
            "foo".to_string(),
            "${dir}/foo bar ${linux:LLL}${windows:WWW}".to_string(),
        );
        commands.insert("reframe".to_string(), "${dir:foo} ${cmd:foo}".to_string());
        let _m = mock("GET", path)
            .with_status(200)
            .with_body("world")
            .create();
        let (mut configuration, temp_dir) = create_configuration();
        let mut env = HashMap::new();
        env.insert("XPATH".to_string(), "${dir:foo}".to_string());
        configuration.tools.push(ToolConfiguration {
            name: "foo".to_string(),
            version: "1.2.3".to_string(),
            download: DownloadUrls {
                default: Some(mockito::server_url() + path),
                linux: None,
                windows: None,
            },
            commands,
            env: env.clone(),
            strip_directories: 0,
        });
        let mut cache = Cache::create(configuration).unwrap();
        cache.init().unwrap();
        let dir = temp_dir
            .path()
            .join("tools")
            .join("foo")
            .join("1.2.3")
            .to_str()
            .unwrap()
            .to_string();
        env.insert("XPATH".to_string(), dir.clone());
        cache.platform = Box::new(crate::platform::Linux {});
        assert_eq!(
            cache.get_command_line("foo").unwrap(),
            CommandLine {
                binary: dir.clone() + "/foo",
                arguments: vec!["bar".to_string(), "LLL".to_string()],
                env: env.clone(),
            }
        );
        cache.platform = Box::new(crate::platform::Windows {});
        assert_eq!(
            cache.get_command_line("foo").unwrap(),
            CommandLine {
                binary: dir.clone() + "/foo",
                arguments: vec!["bar".to_string(), "WWW".to_string()],
                env: env.clone(),
            }
        );
        assert_eq!(
            cache.get_command_line("reframe").unwrap(),
            CommandLine {
                binary: dir.clone(),
                arguments: vec![dir + "/foo"],
                env: env.clone(),
            }
        );
    }

    #[test]
    fn do_not_download_existing_tool() {
        let (mut configuration, temp_dir) = create_configuration();
        configuration.tools.push(ToolConfiguration {
            name: "foo".to_string(),
            version: "1.2.3".to_string(),
            download: DownloadUrls {
                default: Some("http://url.invalid/tool".to_string()),
                linux: None,
                windows: None,
            },
            commands: HashMap::new(),
            env: HashMap::new(),
            strip_directories: 0,
        });
        std::fs::create_dir_all(temp_dir.path().join("tools").join("foo").join("1.2.3")).unwrap();
        let mut cache = Cache::create(configuration).unwrap();
        cache.init().unwrap();
    }

    #[test]
    fn download_zip() {
        let path = "/cache/tool.zip";

        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("foo/hello_world.txt", options).unwrap();
        zip.write(b"Hello, World!").unwrap();
        zip.add_directory("foo/bar", options).unwrap();

        let buf = zip.finish().unwrap().into_inner();

        let _m = mock("GET", path).with_status(200).with_body(buf).create();
        let temp_dir = verify_hello_world_txt(path);
        let path = temp_dir
            .path()
            .join("tools")
            .join("foo")
            .join("1.2.3")
            .join("bar");
        assert!(path.exists(), "directory should exist");
        assert!(path.is_dir(), "directory should be a directory");
    }

    fn verify_hello_world_txt(path: &str) -> TempDir {
        let (mut configuration, temp_dir) = create_configuration();
        configuration.tools.push(ToolConfiguration {
            name: "foo".to_string(),
            version: "1.2.3".to_string(),
            download: DownloadUrls {
                default: Some(mockito::server_url() + path),
                linux: None,
                windows: None,
            },
            commands: HashMap::new(),
            env: HashMap::new(),
            strip_directories: 1,
        });
        let mut cache = Cache::create(configuration).unwrap();
        cache.init().unwrap();
        let path = temp_dir
            .path()
            .join("tools")
            .join("foo")
            .join("1.2.3")
            .join("hello_world.txt");
        let content = read_to_string(path).expect("File hello_world.txt should exist");
        assert_eq!(content, "Hello, World!");
        temp_dir
    }

    #[test]
    fn download_tar_gz() {
        let path = "/cache/tool.tar.gz";

        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut ar = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_path("foo/hello_world.txt").unwrap();
        let content = b"Hello, World!";
        header.set_size(content.len() as u64);
        header.set_cksum();
        ar.append(&header, Cursor::new(content)).unwrap();
        let data = ar.into_inner().unwrap().finish().unwrap();

        let _m = mock("GET", path).with_status(200).with_body(data).create();
        verify_hello_world_txt(path);
    }
}
