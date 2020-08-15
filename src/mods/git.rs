use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::{fs, process};
use tokio::io::BufReader;
use tokio::prelude::*;

use crate::{Error, Result};
use crate::CACHE_ROOT;

use super::GitMod;
use super::sources::ModSource;
use futures::StreamExt;

pub async fn get(git: &GitMod) -> Result<PathBuf> {
    let mut source = open_source(&git).await?;

    match source.build().await? {
        Some(jar) => Ok(jar),
        None => Err(Error::MissingArtifact),
    }
}

async fn open_source(git: &GitMod) -> Result<ModSource> {
    let cache_root = Path::new(CACHE_ROOT);
    if !cache_root.exists() {
        fs::create_dir_all(cache_root).await?;
    }

    let name_pattern = fancy_regex::Regex::new(r#".*\/(.*)\.git"#)?;
    let captures = name_pattern.captures(&git.url)?.unwrap();
    let name = captures.get(1).unwrap().as_str();

    let repository_path = cache_root.join(name);
    if repository_path.exists() {
        open_and_pull_source(repository_path, git).await
    } else {
        clone_source(repository_path, name, git).await
    }
}

async fn open_and_pull_source(root: PathBuf, git: &GitMod) -> Result<ModSource> {
    println!("pulling repository {} @ {:?}", git.url, git.branch);

    let mut command = process::Command::new("git");
    command.stdout(Stdio::piped());
    command.current_dir(&root);
    command.arg("pull");

    if let Some(branch) = &git.branch {
        command.arg("origin").arg(branch);
    }

    let mut child = command.spawn()?;

    let mut changed = true;

    if let Some(stdout) = child.stdout.take() {
        let mut lines = BufReader::new(stdout).lines();

        // hacky solution to detect git changes
        while let Some(line) = lines.next().await {
            let line = line?;
            if line.contains("Already up to date.") {
                changed = false;
            }
        }
    }

    child.await?;

    if changed {
        Ok(ModSource::changed(root))
    } else {
        Ok(ModSource::unchanged(root))
    }
}

async fn clone_source(root: PathBuf, name: &str, git: &GitMod) -> Result<ModSource> {
    println!("cloning repository {} @ {:?}", git.url, git.branch);

    let mut command = process::Command::new("git");
    command.current_dir(root.parent().unwrap());

    command.arg("clone").arg(&git.url);

    if let Some(branch) = &git.branch {
        command.args(&["-b", branch]);
    }

    command.arg(name);

    command.spawn()?.await?;

    Ok(ModSource::changed(root))
}
