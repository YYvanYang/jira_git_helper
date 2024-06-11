pub mod config;

use std::process::Command;
use std::io::{self, Write};
use reqwest::Client;
use reqwest::header::{COOKIE, SET_COOKIE};
use regex::Regex;
use config::{write_config, Config};
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

pub fn extract_jira_id<'a>(branch_name: &'a str, jira_id_prefix: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?i){}-\d+", regex::escape(jira_id_prefix))).unwrap();
    re.find(branch_name).map(|m| m.as_str().to_uppercase())
}

pub async fn login_to_jira(config: &mut Config) -> Result<(), &'static str> {
    let login_url = format!("{}/rest/gadget/1.0/login", config.jira_url);
    let client = Client::new();

    let params = [
        ("os_username", &config.username),
        ("os_password", &config.password),
    ];
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

pub async fn get_jira_title(jira_id: &str, config: &mut Config) -> Result<String, &'static str> {
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

    if response.status() == reqwest::StatusCode::UNAUTHORIZED || response.status() == reqwest::StatusCode::FORBIDDEN {
        login_to_jira(config).await?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(COOKIE, config.jsessionid.as_ref().unwrap().parse().unwrap());
        headers.insert("X-Atlassian-Token", config.xsrf_token.as_ref().unwrap().parse().unwrap());
        let response = client
            .get(&jira_api_url)
            .headers(headers)
            .send()
            .await
            .map_err(|_| "Failed to send request")?;
        if !response.status().is_success() {
            return Err("Failed to get JIRA issue");
        }
        let issue: JiraIssue = response.json().await.map_err(|_| "Failed to parse JSON response")?;
        Ok(issue.fields.summary)
    } else if response.status().is_success() {
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

pub fn prompt_for_input(prompt: &str, default: Option<&str>) -> String {
    print!("{} [{}]: ", prompt, default.unwrap_or(""));
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_string();
    if input.is_empty() {
        default.unwrap_or("").to_string()
    } else {
        input
    }
}

pub fn prompt_for_config() -> Config {
    let jira_url = prompt_for_input("Enter your JIRA URL", None);
    let username = prompt_for_input("Enter your domain username", None);
    print!("Enter your domain password: ");
    io::stdout().flush().unwrap();
    let password = rpassword::read_password().expect("Failed to read password");
    let jira_id_prefix = prompt_for_input("Enter your JIRA ID prefix", Some("JIRA"));

    Config { username, password, jira_url, jira_id_prefix, jsessionid: None, xsrf_token: None }
}

pub fn prompt_for_commit_message() -> String {
    prompt_for_input("Enter additional commit message", None)
}

pub fn confirm_commit(commit_message: &str) -> bool {
    println!("Git commit command: git commit -m \"{}\"", commit_message);
    let input = prompt_for_input("Do you want to proceed? (y/n)", Some("y"));
    matches!(input.to_lowercase().as_str(), "y" | "yes" | "")
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
