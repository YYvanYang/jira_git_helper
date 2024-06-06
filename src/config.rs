use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use dirs::home_dir;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub jira_url: String,
    pub jira_id_prefix: String,
    pub jsessionid: Option<String>,
    pub xsrf_token: Option<String>,
}

impl Config {
    pub fn new(username: String, password: String, jira_url: String, jira_id_prefix: String) -> Self {
        Config {
            username,
            password,
            jira_url,
            jira_id_prefix,
            jsessionid: None,
            xsrf_token: None
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let mut config_path = home_dir().expect("Could not find home directory");
    config_path.push(".jira_git_helper");
    config_path
}

pub fn read_config() -> Result<Config, &'static str> {
    let config_path = get_config_path();
    if config_path.exists() {
        let config_data = fs::read_to_string(config_path).map_err(|_| "Failed to read config file")?;
        let config: Config = serde_json::from_str(&config_data).map_err(|_| "Failed to parse config file")?;
        Ok(config)
    } else {
        Err("Config file not found")
    }
}

pub fn write_config(config: &Config) -> Result<(), &'static str> {
    let config_data = serde_json::to_string(config).map_err(|_| "Failed to serialize config data")?;
    let config_path = get_config_path();
    fs::write(config_path, config_data).map_err(|_| "Failed to write config file")
}

pub fn reset_config() -> Result<(), &'static str> {
    let config_path = get_config_path();
    if config_path.exists() {
        fs::remove_file(config_path).map_err(|_| "Failed to delete config file")
    } else {
        Ok(())
    }
}
