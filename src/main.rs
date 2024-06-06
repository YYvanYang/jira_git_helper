use jira_git_helper::{
    config::{read_config, write_config, reset_config},
    extract_jira_id, get_current_branch, get_jira_title, login_to_jira, prompt_for_commit_message,
    prompt_for_config, run_git_commit,
};
use tokio;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "reset" {
        reset_config().expect("Failed to reset config");
        println!("Configuration reset successfully.");
        return;
    }

    let mut config = read_config().unwrap_or_else(|_| {
        let config = prompt_for_config();
        write_config(&config).expect("Failed to write config");
        config
    });

    if config.jsessionid.is_none() || config.xsrf_token.is_none() {
        login_to_jira(&mut config).await.expect("Failed to login to JIRA");
    }

    let branch_name = get_current_branch()
        .await
        .expect("Failed to get current branch");
    let jira_id = extract_jira_id(&branch_name, &config.jira_id_prefix).expect("JIRA ID not found in branch name");

    let jira_title = get_jira_title(jira_id, &config)
        .await
        .expect("Failed to get JIRA title");

    println!("JIRA ID: {}", jira_id);
    println!("JIRA Title: {}", jira_title);

    let additional_message = prompt_for_commit_message();
    let commit_message = format!("[{}] {} {}", jira_id, jira_title, additional_message);

    run_git_commit(&commit_message).expect("Failed to run git commit");
}
