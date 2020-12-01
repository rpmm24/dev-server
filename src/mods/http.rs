use std::path::{Path, PathBuf};

use tokio::fs::{self, File};
use tokio::prelude::*;

use crate::{CACHE_ROOT, Result};

use super::HttpMod;

pub async fn get(http: &HttpMod) -> Result<PathBuf> {
    let cache_root = Path::new(CACHE_ROOT);
    if !cache_root.exists() {
        fs::create_dir_all(cache_root).await?;
    }

    let name = name(http);

    let cache_path = cache_root.join(name);
    if cache_path.exists() {
        return Ok(cache_path);
    }

    println!("downloading {}", http.url);
    let response = reqwest::get(&http.url).await?;
    let bytes = response.bytes().await?;

    let mut file = File::create(&cache_path).await?;
    file.write_all(&bytes).await?;

    Ok(cache_path)
}

pub fn name(http: &HttpMod) -> String {
    let name_pattern = fancy_regex::Regex::new(r#".*\/(.*\.jar)"#).unwrap();
    let captures = name_pattern.captures(&http.url).unwrap().unwrap();
    captures.get(1).unwrap().as_str().to_owned()
}
