use crate::Result;
use anyhow::bail;
use http_req::response::Response;
use std::fs::File;
use std::path::Path;

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
            std::fs::remove_file(path)?;
            continue;
        }
        let code: u16 = res.status_code().into();
        bail!(
            "HTTP Error {} downloading {} ({:?})",
            code,
            url,
            res.status_code().reason()
        )
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
        let _m = mock("GET", "/download")
            .with_status(200)
            .with_body("world")
            .create();

        let file = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        download(&(mockito::server_url() + "/download"), &file).unwrap();
        let result = read_to_string(&file).unwrap();
        assert_eq!("world", result);
    }

    #[test]
    fn redirected_download() {
        let _m = mock("GET", "/download")
            .with_status(200)
            .with_body("world")
            .create();
        let _m2 = mock("GET", "/redirect")
            .with_status(301)
            .with_header("location", &(mockito::server_url() + "/download"))
            .create();

        let file = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        download(&(mockito::server_url() + "/redirect"), &file).unwrap();
        let result = read_to_string(&file).unwrap();
        assert_eq!(result, "world");
    }

    #[test]
    fn download_fail() {
        let _m = mock("GET", "/download").with_status(500).create();

        let file = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let result = download(&(mockito::server_url() + "/download"), &file);
        let error_message = result.unwrap_err().to_string();
        let normalized_error_message =
            error_message.replace(&format!("{}", mockito::server_address().port()), "9999");
        assert_eq!(normalized_error_message, "HTTP Error 500 downloading http://127.0.0.1:9999/download (Some(\"Internal Server Error\"))");
    }
}
