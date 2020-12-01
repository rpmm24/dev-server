use std::path::PathBuf;

use crate::{Config, config, Result};

mod artifact;
mod git;
mod http;
mod sources;

#[derive(Debug)]
pub struct Mods {
    pub mods: Vec<Mod>,
}

impl Mods {
    pub fn parse(config: &Config) -> Mods {
        let mods = config.mods.values()
            .filter_map(|m| Mod::parse(m))
            .collect();

        Mods { mods }
    }

    pub async fn collect_jars(&mut self) -> Vec<PathBuf> {
        let mut jars = Vec::new();

        for m in &self.mods {
            match Mods::retry_build_jar(m).await {
                Ok(jar) => jars.push(jar),
                Err(err) => eprintln!("failed to build jar! excluding... {:?}", err),
            }
        }

        jars
    }

    async fn retry_build_jar(m: &Mod) -> Result<PathBuf> {
        const MAX_RETRIES: u8 = 2;

        let mut retries = 0;

        loop {
            match Mods::build_jar(m).await {
                Ok(jar) => break Ok(jar),
                Err(err) => {
                    retries += 1;
                    if retries < MAX_RETRIES {
                        eprintln!("failed to build jar! retrying... {:?}", err);
                        Mods::reset_build(m).await?
                    } else {
                        break Err(err);
                    }
                }
            }
        }
    }

    async fn build_jar(m: &Mod) -> Result<PathBuf> {
        match m {
            Mod::Http(http) => http::get(&http).await,
            Mod::Git(git) => git::get(&git).await,
            Mod::File(file) => Ok(file.path.clone())
        }
    }

    async fn reset_build(m: &Mod) -> Result<()> {
        match m {
            Mod::Git(git) => git::reset(&git).await,
            _ => Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mod {
    Http(HttpMod),
    Git(GitMod),
    File(FileMod),
}

impl Mod {
    pub fn parse(config: &config::Mod) -> Option<Mod> {
        match config {
            config::Mod { url: Some(url), path: None, git: None, branch: None } => {
                Some(Mod::Http(HttpMod { url: url.clone() }))
            }
            config::Mod { url: None, path: None, git: Some(url), branch } => {
                Some(Mod::Git(GitMod { url: url.clone(), branch: branch.clone() }))
            }
            config::Mod { url: None, path: Some(path), git: None, branch: None } => {
                Some(Mod::File(FileMod { path: path.clone() }))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpMod {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct GitMod {
    pub url: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileMod {
    pub path: PathBuf,
}
