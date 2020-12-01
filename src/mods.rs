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

    pub async fn collect_jars(&mut self) -> Result<Vec<PathBuf>> {
        let mut jars = Vec::new();

        for m in &self.mods {
            match m {
                Mod::Http(http) => jars.push(http::get(&http).await?),
                Mod::Git(git) => jars.push(git::get(&git).await?),
                Mod::File(file) => jars.push(file.path.clone())
            }
        }

        Ok(jars)
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
