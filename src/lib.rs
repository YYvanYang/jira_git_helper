use thiserror::Error;
use config::Config as AppConfig;

pub mod config;
pub mod git;
pub mod jira;
pub mod input;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JIRA API error: {0}")]
    JiraApi(String),
    #[error("Git error: {0}")]
    Git(String),
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub struct App {
    config: AppConfig,
    jira_client: jira::JiraClient,
    git_ops: git::GitOperations,
}

impl App {
    pub async fn new() -> Result<Self, AppError> {
        let config = config::load_config()?;
        let jira_client = jira::JiraClient::new(&config);
        let git_ops = git::GitOperations::new();

        Ok(Self {
            config,
            jira_client,
            git_ops,
        })
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        let branch_name = self.git_ops.get_current_branch().await?;
        let jira_id = self.extract_jira_id(&branch_name)?;

        let jira_title = self.jira_client.get_issue_title(&jira_id).await?;

        println!("JIRA ID: {}", jira_id);
        println!("JIRA Title: {}", jira_title);

        let additional_message = input::prompt_for_commit_message();
        let commit_message = format!("[{}] {} {}", jira_id, jira_title, additional_message);

        if input::confirm_commit(&commit_message) {
            self.git_ops.commit(&commit_message).await?;
        } else {
            println!("Commit cancelled.");
        }

        Ok(())
    }

    fn extract_jira_id(&self, branch_name: &str) -> Result<String, AppError> {
        let jira_id_prefix = self.config.get_string("jira_id_prefix")?;
        jira::extract_jira_id(branch_name, &jira_id_prefix)
            .ok_or_else(|| AppError::Other("JIRA ID not found in branch name".to_string()))
    }
}
