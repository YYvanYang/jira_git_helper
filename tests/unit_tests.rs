use jira_git_helper::config::{Config, read_config, write_config, reset_config};
use jira_git_helper::extract_jira_id;

#[test]
fn test_extract_jira_id() {
    let branch_name = "feature/JIRA-1234-add-login-feature";
    assert_eq!(extract_jira_id(branch_name, "JIRA"), Some(String::from("JIRA-1234")));
}

#[tokio::test]
async fn test_config_read_write() {
    let config = Config::new(
        "user".to_string(),
        "pass".to_string(),
        "http://example.com".to_string(),
        "JIRA".to_string()
    );
    write_config(&config).expect("Failed to write config");

    let read_config = read_config().expect("Failed to read config");
    assert_eq!(config.username, read_config.username);
    assert_eq!(config.password, read_config.password);
    assert_eq!(config.jira_url, read_config.jira_url);
    assert_eq!(config.jira_id_prefix, read_config.jira_id_prefix);

    // Clean up
    reset_config().expect("Failed to reset config");
}

#[test]
fn test_reset_config() {
    let config = Config::new(
        "user".to_string(),
        "pass".to_string(),
        "http://example.com".to_string(),
        "JIRA".to_string()
    );
    write_config(&config).expect("Failed to write config");

    reset_config().expect("Failed to reset config");
    assert!(read_config().is_err());
}
