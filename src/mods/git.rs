use std::path::{Path, PathBuf};
use std::process::Stdio;

use futures::StreamExt;
use tokio::{fs, process};
use tokio::io::BufReader;
use tokio::prelude::*;

use crate::{Error, Result};
use crate::CACHE_ROOT;

use super::GitMod;
use super::sources::ModSource;

pub async fn get(git: &GitMod) -> Result<PathBuf> {
    let mut source = open_source(&git).await?;

    match source.build().await? {
        Some(jar) => Ok(jar),
        None => Err(Error::MissingArtifact),
    }
}

pub async fn reset(git: &GitMod) -> Result<()> {
    let repository_path = get_repository_cache(git)?;
    if repository_path.exists() {
        ModSource::unchanged(repository_path).reset().await
    } else {
        Ok(())
    }
}

async fn open_source(git: &GitMod) -> Result<ModSource> {
    let repository_path = get_repository_cache(git)?;
    if let Some(parent) = repository_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    if repository_path.exists() {
        open_and_pull_source(repository_path, git).await
    } else {
        clone_source(repository_path, name, git).await
    }
}

fn get_repository_cache(git: &GitMod) -> Result<PathBuf> {
    let cache_root = Path::new(CACHE_ROOT);

    let name_pattern = fancy_regex::Regex::new(r#".*\/(.*)\.git"#)?;
    let captures = name_pattern.captures(&git.url)?.unwrap();
    let name = captures.get(1).unwrap().as_str();

    Ok(cache_root.join(name))
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
