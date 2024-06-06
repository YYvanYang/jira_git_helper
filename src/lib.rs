pub mod config;

use std::process::Command;
use std::io::{self, Write};
use reqwest::Client;
use reqwest::header::{COOKIE, SET_COOKIE};
use regex::Regex;
use config::{Config, write_config};
use serde::Deserialize;

pub async fn get_current_branch() -> Result<String, &'static str> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .map_err(|_| "Failed to execute git command")?;

    if !output.status.success() {
        return Err("Failed to get current branch");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn extract_jira_id<'a>(branch_name: &'a str, jira_id_prefix: &str) -> Option<&'a str> {
    let re = Regex::new(&format!(r"{}-\d+", regex::escape(jira_id_prefix))).unwrap();
    re.find(branch_name).map(|m| m.as_str())
}

pub async fn login_to_jira(config: &mut Config) -> Result<(), &'static str> {
    let login_url = format!("{}/rest/gadget/1.0/login", config.jira_url);
    let client = Client::new();

    let params = [("os_username", &config.username), ("os_password", &config.password)];
    let response = client
        .post(&login_url)
        .form(&params)
        .send()
        .await
        .map_err(|_| "Failed to send login request")?;

    if !response.status().is_success() {
        return Err("Failed to login to JIRA");
    }

    let cookies = response
        .headers()
        .get_all(SET_COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .collect::<Vec<_>>();

    for cookie in cookies {
        if cookie.starts_with("JSESSIONID") {
            config.jsessionid = Some(cookie.to_string());
        }
        if cookie.contains("atlassian.xsrf.token") {
            let token = cookie.split('=').nth(1).unwrap().split(';').next().unwrap();
            config.xsrf_token = Some(token.to_string());
        }
    }

    write_config(&config).map_err(|_| "Failed to write config")?;
    Ok(())
}

pub async fn get_jira_title(jira_id: &str, config: &Config) -> Result<String, &'static str> {
    let jira_api_url = format!("{}/rest/api/2/issue/{}", config.jira_url, jira_id);
    let client = Client::new();

    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(ref jsessionid) = config.jsessionid {
        headers.insert(COOKIE, jsessionid.parse().unwrap());
    }
    if let Some(ref xsrf_token) = config.xsrf_token {
        headers.insert("X-Atlassian-Token", xsrf_token.parse().unwrap());
    }

    let response = client
        .get(&jira_api_url)
        .headers(headers)
        .send()
        .await
        .map_err(|_| "Failed to send request")?;

    if response.status().is_success() {
        let issue: JiraIssue = response.json().await.map_err(|_| "Failed to parse JSON response")?;
        Ok(issue.fields.summary)
    } else {
        Err("Failed to get JIRA issue")
    }
}

#[derive(Deserialize)]
struct JiraIssue {
    fields: JiraFields,
}

#[derive(Deserialize)]
struct JiraFields {
    summary: String,
}

pub fn prompt_for_config() -> Config {
    let prompt = |msg: &str, default: &str| -> String {
        print!("{}", msg);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();
        if input.is_empty() {
            default.to_string()
        } else {
            input
        }
    };

    let jira_url = prompt("Enter your JIRA URL: ", "");
    let username = prompt("Enter your domain username: ", "");
    print!("Enter your domain password: ");
    io::stdout().flush().unwrap();
    let password = rpassword::read_password().expect("Failed to read password");
    let jira_id_prefix = prompt("Enter your JIRA ID prefix: ", "JIRA");

    Config { username, password, jira_url, jira_id_prefix, jsessionid: None, xsrf_token: None }
}

pub fn prompt_for_commit_message() -> String {
    print!("Enter additional commit message: ");
    io::stdout().flush().unwrap();

    let mut additional_message = String::new();
    io::stdin().read_line(&mut additional_message).unwrap();
    additional_message.trim().to_string()
}

pub fn run_git_commit(commit_message: &str) -> Result<(), &'static str> {
    let status = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(commit_message)
        .status()
        .map_err(|_| "Failed to execute git commit")?;

    if !status.success() {
        return Err("git commit failed");
    }

    Ok(())
}
