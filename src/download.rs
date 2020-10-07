use crate::Result;
use anyhow::{bail, Context};
use http_req::response::Response;
use std::fs::File;
use std::path::Path;
use crate::util::retry;

const MAX_REDIRECTS: i32 = 10;

pub fn download(url: &str, path: &Path) -> Result<()> {
    let mut download_url = url.to_string();
    for _ in 0..MAX_REDIRECTS {
        let mut file = File::create(path)?;
        let res: Response = http_req::request::get(&download_url, &mut file).unwrap();
        if res.status_code().is_success() {
            return Ok(());
        }
        if res.status_code().is_redirect() {
            download_url = res
                .headers()
                .get("location")
                .expect("No location in HTTP redirect")
                .clone();
            verbose!("Download redirected to {}", download_url);
            retry(||std::fs::remove_file(&path)).with_context(|| format!("Unable to remove temp file at {:?}", path))?;
            continue;
        }
        let code: u16 = res.status_code().into();
        bail!("HTTP Error {} downloading {} ({})", code, url, res.reason())
    }
    Err(anyhow::anyhow!(
        "Failed to download {} after {} redirects",
        url,
        MAX_REDIRECTS
    ))
}

#[cfg(test)]
mod tests {
    use mockito::mock;

    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn simple_download() {
        let path = "/download1";
        let _m = mock("GET", path)
            .with_status(200)
            .with_body("world")
            .create();

        let file = create_temp_path();
        download(&(mockito::server_url() + path), &file).unwrap();
        let result = read_to_string(&file).unwrap();
        assert_eq!("world", result);
    }

    #[test]
    fn redirected_download() {
        let path = "/download2";
        let redirect_path = "/redirect2";
        let _m = mock("GET", path)
            .with_status(200)
            .with_body("world")
            .create();
        let _m2 = mock("GET", redirect_path)
            .with_status(301)
            .with_header("location", &(mockito::server_url() + path))
            .create();

        let file = create_temp_path();
        download(&(mockito::server_url() + redirect_path), &file).unwrap();
        let result = read_to_string(&file).unwrap();
        assert_eq!(result, "world");
    }

    fn create_temp_path() -> tempfile::TempPath {
        tempfile::NamedTempFile::new().unwrap().into_temp_path()
    }

    #[test]
    fn download_fail() {
        let path = "/download3";
        let _m = mock("GET", path).with_status(500).create();

        let file = create_temp_path();
        let result = download(&(mockito::server_url() + path), &file);
        let error_message = get_error_message(result);
        assert_eq!(
            error_message,
            format!(
                "HTTP Error 500 downloading http://127.0.0.1:9999{} (Internal Server Error)",
                path
            )
        );
    }

    fn get_error_message(result: Result<()>) -> String {
        let error_message = result.unwrap_err().to_string();
        error_message.replace(&format!("{}", mockito::server_address().port()), "9999")
    }

    #[test]
    fn download_redirect_loop() {
        let path = "/download4";
        let _m = mock("GET", path)
            .with_status(301)
            .with_header("location", &(mockito::server_url() + path))
            .create();

        let file = create_temp_path();
        let result = download(&(mockito::server_url() + path), &file);
        let error_message = get_error_message(result);
        assert_eq!(
            error_message,
            "Failed to download http://127.0.0.1:9999/download4 after 10 redirects"
        );
    }
}
