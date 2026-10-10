use std::path::PathBuf;
use anyhow::{Context, Result, bail};
use std::process::Command;

pub struct GitManager {
    pub repo_dir: PathBuf,
}

impl GitManager {
    pub fn clone(&self, url: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("clone")
            .arg(url)
            .arg(&self.repo_dir)
            .output()
            .context("Could not launch Git. Is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Git clone failed: {stderr}");
        }

        Ok(())
    }

    pub fn push(&self, msg: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&self.repo_dir)
            .output()
            .context("Could not launch Git. Is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Git add failed: {stderr}");
        }

        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(msg)
            .current_dir(&self.repo_dir)
            .output()
            .context("Could not launch Git. Is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Git commit failed: {stderr}");
        }

        let output = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("main")
            .current_dir(&self.repo_dir)
            .output()
            .context("Could not launch Git. Is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Git push failed: {stderr}");
        }

        Ok(())
    }
}