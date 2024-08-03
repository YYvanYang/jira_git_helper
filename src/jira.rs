use crate::AppError;
use config::Config;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use regex::Regex;
use lazy_static::lazy_static;

pub struct JiraClient {
    client: Client,
    config: Config,
}

#[derive(Deserialize)]
struct JiraIssue {
    fields: JiraFields,
}

#[derive(Deserialize)]
struct JiraFields {
    summary: String,
}

#[derive(Serialize)]
struct LoginCredentials {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    session: SessionInfo,
}

#[derive(Deserialize)]
struct SessionInfo {
    name: String,
    value: String,
}

impl JiraClient {
    pub fn new(config: &Config) -> Self {
        JiraClient {
            client: Client::new(),
            config: config.clone(),
        }
    }

    async fn login(&mut self) -> Result<(), AppError> {
        let login_url = format!("{}/rest/auth/1/session", self.config.get_string("jira_url")?);
        let credentials = LoginCredentials {
            username: self.config.get_string("username")?,
            password: self.config.get_string("password")?,
        };

        let response = self.client
            .post(&login_url)
            .json(&credentials)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::JiraApi(format!("Failed to login to JIRA: {}", response.status())));
        }

        let login_response: LoginResponse = response.json().await?;
        let session_cookie = format!("{}={}", login_response.session.name, login_response.session.value);

        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str(&session_cookie).unwrap());

        self.client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AppError::JiraApi(e.to_string()))?;

        Ok(())
    }

    pub async fn get_issue_title(&mut self, jira_id: &str) -> Result<String, AppError> {
        let jira_api_url = format!("{}/rest/api/2/issue/{}", self.config.get_string("jira_url")?, jira_id);

        let mut response = self.client.get(&jira_api_url).send().await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED || response.status() == reqwest::StatusCode::FORBIDDEN {
            // Session might be expired, try to login again
            self.login().await?;
            // Retry the request
            response = self.client.get(&jira_api_url).send().await?;
        }

        if !response.status().is_success() {
            return Err(AppError::JiraApi(format!("Failed to get JIRA issue: {}", response.status())));
        }

        let issue: JiraIssue = response.json().await?;
        Ok(issue.fields.summary)
    }
}

pub fn extract_jira_id(branch_name: &str, jira_id_prefix: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)([A-Z]+-\d+)").unwrap();
    }
    RE.find(branch_name)
        .map(|m| m.as_str().to_uppercase())
        .filter(|id| id.starts_with(jira_id_prefix))
}