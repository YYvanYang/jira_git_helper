use jira_git_helper::config::{write_config, reset_config};
use jira_git_helper::{
    extract_jira_id, get_jira_title, login_to_jira, prompt_for_config, run_git_commit,
};

#[tokio::test]
async fn test_integration() {
    // Assuming JIRA credentials and URLs are correct
    let mut config = prompt_for_config();
    write_config(&config).expect("Failed to write config");

    login_to_jira(&mut config)
        .await
        .expect("Failed to login to JIRA");

    let branch_name = "feature/JIRA-1234-add-login-feature";
    let jira_id = extract_jira_id(branch_name, &config.jira_id_prefix).expect("Failed to extract JIRA ID");

    let jira_title = get_jira_title(jira_id, &config)
        .await
        .expect("Failed to get JIRA title");
    assert!(!jira_title.is_empty(), "JIRA title should not be empty");

    let commit_message = format!("[{}] {} {}", jira_id, jira_title, "Test commit message");
    run_git_commit(&commit_message).expect("Failed to run git commit");

    // Clean up
    reset_config().expect("Failed to reset config");
}
