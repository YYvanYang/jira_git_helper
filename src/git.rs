use crate::AppError;
use tokio::process::Command;

pub struct GitOperations;

impl GitOperations {
    pub fn new() -> Self {
        GitOperations
    }

    pub async fn get_current_branch(&self) -> Result<String, AppError> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()
            .await
            .map_err(|e| AppError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(AppError::Git("Failed to get current branch".to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub async fn commit(&self, message: &str) -> Result<(), AppError> {
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .status()
            .await
            .map_err(|e| AppError::Git(e.to_string()))?;

        if !status.success() {
            return Err(AppError::Git("git commit failed".to_string()));
        }

        Ok(())
    }
}
