use jira_git_helper::{App, AppError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "reset" {
        jira_git_helper::config::reset_config()?;
        println!("Configuration reset successfully.");
        return Ok(());
    }

    let mut app = App::new().await?;
    app.run().await
}
