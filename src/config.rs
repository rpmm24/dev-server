use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io;
use tokio::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_jar: PathBuf,
    pub jvm: Option<String>,
    pub mods: HashMap<String, Mod>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server_jar: PathBuf::new(),
            jvm: None,
            mods: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub url: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
}

const CONFIG_PATH: &str = "dev_config.toml";

pub async fn config() -> Config {
    let path = Path::new(CONFIG_PATH);
    if path.exists() {
        read_config(path).await.expect("failed to read config")
    } else {
        let config = Config::default();
        write_config(path, &config).await.expect("failed to write default config");
        config
    }
}

async fn write_config(path: &Path, config: &Config) -> io::Result<()> {
    let mut file = File::create(path).await?;

    let bytes = toml::to_vec(config).expect("malformed config");
    file.write_all(&bytes).await?;

    Ok(())
}

async fn read_config(path: &Path) -> io::Result<Config> {
    let mut file = File::open(path).await?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;

    Ok(toml::from_slice::<Config>(&bytes).expect("malformed config"))
}
