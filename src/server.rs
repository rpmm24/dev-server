use std::path::PathBuf;

use tokio::{fs, process};

use crate::Result;

pub struct Server {
    root: PathBuf,
    jar: PathBuf,
}

impl Server {
    pub fn open(jar: impl Into<PathBuf>) -> Server {
        let jar = jar.into();
        let root = jar.parent().unwrap().to_owned();
        Server { root, jar }
    }

    pub async fn run(&mut self, jvm: Option<&str>, mods: &[PathBuf]) -> Result<()> {
        self.setup_mods(mods).await?;

        let mut command = process::Command::new("java");
        command.arg("-jar");

        if let Some(jvm) = jvm {
            command.args(jvm.split_ascii_whitespace());
        }

        let jar_name = self.jar.file_name().unwrap().to_str().unwrap();
        command.arg(jar_name);

        command.current_dir(&self.root);

        command.spawn()?.await?;

        Ok(())
    }

    async fn setup_mods(&mut self, mods: &[PathBuf]) -> Result<()> {
        let mods_path = self.mods_path();

        fs::remove_dir_all(&mods_path).await?;
        fs::create_dir_all(&mods_path).await?;

        for jar in mods {
            let name = jar.file_name().and_then(|name| name.to_str());
            if let Some(name) = name {
                let target = mods_path.join(name);
                fs::copy(jar, target).await?;
            } else {
                eprintln!("mod jar path missing name!");
            }
        }

        Ok(())
    }

    fn mods_path(&self) -> PathBuf {
        self.root.join("mods")
    }
}
