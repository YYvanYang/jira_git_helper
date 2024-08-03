use config::Config;

pub mod error;
pub mod app_config;
pub mod git;
pub mod jira;
pub mod input;

pub use error::AppError;
pub use crate::app_config::AppConfig;
pub use crate::jira::JiraClient;
pub use crate::git::GitOperations;

pub struct App {
    config: Config,
    jira_client: jira::JiraClient,
    git_ops: git::GitOperations,
}

impl App {

    pub async fn new() -> Result<Self, AppError> {
        let config = app_config::load_config()?;
        
        // 基本验证
        if config.get_string("jira_url").is_err() || config.get_string("username").is_err() {
            return Err(AppError::ConfigMissing);
        }

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
            println!("Commit successful!");
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