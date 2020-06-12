use crate::Result;
use anyhow::bail;
use std::fs::File;
use std::path::Path;

const MAX_REDIRECTS: i32 = 10;

pub fn download(url: &str, path: &Path) -> Result<()> {
    let mut download_url = url.to_string();
    for _ in 0..MAX_REDIRECTS {
        let mut file = File::create(path)?;
        let res = http_req::request::get(&download_url, &mut file).unwrap();
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
        bail!("Error {:?} downloading {}", res.status_code().reason(), url)
    }
    Err(anyhow::anyhow!(
        "Failed to download {} after {} redirects",
        url,
        MAX_REDIRECTS
    ))
}
