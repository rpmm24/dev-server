use std::path::PathBuf;

use futures::StreamExt;
use tokio::{fs, process};

use crate::Result;

use super::artifact;

pub struct ModSource {
    root: PathBuf,
    changed: bool,
}

impl ModSource {
    pub fn unchanged(root: impl Into<PathBuf>) -> ModSource {
        ModSource { root: root.into(), changed: false }
    }

    pub fn changed(root: impl Into<PathBuf>) -> ModSource {
        ModSource { root: root.into(), changed: true }
    }

    pub async fn reset(&mut self) -> Result<()> {
        let mut command = self.command_gradle();
        command.args(&["clean"]);
        command.spawn()?.await?;
        Ok(())
    }

    pub async fn build(&mut self) -> Result<Option<PathBuf>> {
        let build = self.root.join("build/libs");

        if self.changed || !build.exists() {
            let mut command = self.command_gradle();
            command.args(&["clean", "build"]);

            command.spawn()?.await?;
        }

        let mut children = fs::read_dir(build).await?;
        while let Some(child) = children.next().await {
            let child = child?;
            if let Some(id) = artifact::parse(child.path()) {
                if id.classifier.is_none() {
                    return Ok(Some(child.path().into()));
                }
            }
        }

        Ok(None)
    }

    #[inline]
    fn command_gradle(&mut self) -> process::Command {
        let mut command = process::Command::new("gradle");
        command.current_dir(&self.root);
        command
    }
}
