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

    pub fn pull(&self) -> Result<()> {
        let output = Command::new("git")
            .args(["pull", "origin", "main"])
            .current_dir(&self.repo_dir)
            .output()
            .context("Could not launch Git. Is it installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Git add failed: {stderr}");
        }

        Ok(())
    }

    pub fn origin_matches_local(&self) -> Result<bool> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
            .current_dir(&self.repo_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("No upstream branch configured: {stderr}");
        }

        let output = Command::new("git")
            .arg("fetch")
            .current_dir(&self.repo_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Origin check failed: {stderr}");
        }

        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Origin check failed: {stderr}");
        }

        let local_id = output.stdout.trim_ascii();

        let output = Command::new("git")
            .args(["rev-parse", "@{u}"])
            .current_dir(&self.repo_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Origin check failed: {stderr}");
        }

        let upstream_id = output.stdout.trim_ascii();

        Ok(local_id == upstream_id)
    }
}