use std::path::PathBuf;
use std::fs;
use std::env;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, File};
use ring::{aead, rand};
use ring::rand::SecureRandom;
use base64::{engine::general_purpose, Engine as _};
use crate::input;
use crate::AppError;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub jira_url: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_password: Option<String>,
    pub jira_id_prefix: String,
}

impl AppConfig {
    pub fn new(jira_url: String, username: String, password: String, jira_id_prefix: String) -> Result<Self, AppError> {
        let encrypted_password = Some(encrypt_password(&password)?);
        Ok(Self {
            jira_url,
            username,
            encrypted_password,
            jira_id_prefix,
        })
    }

    pub fn get_password(&self) -> Result<String, AppError> {
        self.encrypted_password
            .as_ref()
            .ok_or_else(|| AppError::Config(ConfigError::NotFound("Password not set".to_string())))
            .and_then(|enc_pass| decrypt_password(enc_pass).map_err(AppError::Config))
    }
}

pub fn load_config() -> Result<Config, AppError> {
    let config_path = get_config_path();
    
    // 创建初始配置
    let config = Config::builder()
        .add_source(File::with_name(config_path.to_str().unwrap()).required(false))
        .add_source(config::Environment::with_prefix("JIRA_GIT"))
        .build()
        .map_err(AppError::Config)?;

    // 检查必要的配置项
    if config.get_string("jira_url").is_err() || config.get_string("username").is_err() || config.get_string("encrypted_password").is_err() {
        return Err(AppError::ConfigMissing);
    }

    // 处理加密密码
    let encrypted_password = config.get_string("encrypted_password").map_err(AppError::Config)?;
    let decrypted_password = decrypt_password(&encrypted_password).map_err(AppError::Config)?;
    
    // 创建新的 builder，包含原始配置和解密后的密码
    let config = Config::builder()
        .add_source(config.clone())
        .set_override("password", decrypted_password) // 这里设置解密后的密码
        .map_err(AppError::Config)?
        .build()
        .map_err(AppError::Config)?;

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<(), AppError> {
    let config_path = get_config_path();
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| AppError::Config(ConfigError::Message(format!("Failed to serialize config: {}", e))))?;
    fs::write(config_path, config_str)
        .map_err(|e| AppError::Config(ConfigError::Message(format!("Failed to write config file: {}", e))))
}

pub fn get_config_path() -> PathBuf {
    let home = env::var("JIRA_GIT_HOME").unwrap_or_else(|_| home_dir().expect("Could not find home directory").to_string_lossy().into_owned());
    let mut config_path = PathBuf::from(home);
    config_path.push(".jira_git_helper.toml");
    config_path
}

fn encrypt_password(password: &str) -> Result<String, ConfigError> {
    let mut key = [0; 32];
    let mut nonce_bytes = [0; 12];
    let rng = rand::SystemRandom::new();

    rng.fill(&mut key)
        .map_err(|_| ConfigError::Message("Failed to generate encryption key".to_string()))?;
    rng.fill(&mut nonce_bytes)
        .map_err(|_| ConfigError::Message("Failed to generate nonce".to_string()))?;

    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
    let aad = aead::Aad::empty();

    let mut in_out = password.as_bytes().to_vec();
    let sealing_key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::AES_256_GCM, &key)
            .map_err(|_| ConfigError::Message("Failed to create encryption key".to_string()))?
    );

    sealing_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
        .map_err(|_| ConfigError::Message("Failed to encrypt password".to_string()))?;

    let mut result = Vec::with_capacity(key.len() + nonce_bytes.len() + in_out.len());
    result.extend_from_slice(&key);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);

    Ok(general_purpose::STANDARD_NO_PAD.encode(result))
}

fn decrypt_password(encrypted: &str) -> Result<String, ConfigError> {
    let decoded = general_purpose::STANDARD_NO_PAD.decode(encrypted)
        .map_err(|_| ConfigError::Message("Failed to decode encrypted password".to_string()))?;

    if decoded.len() < 44 {  // 32 (key) + 12 (nonce) = 44
        return Err(ConfigError::Message("Invalid encrypted password format".to_string()));
    }

    let (key, rest) = decoded.split_at(32);
    let (nonce_bytes, ciphertext) = rest.split_at(12);

    let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| ConfigError::Message("Invalid nonce".to_string()))?;
    let aad = aead::Aad::empty();

    let opening_key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|_| ConfigError::Message("Failed to create decryption key".to_string()))?
    );

    let mut in_out = ciphertext.to_vec();
    let plaintext = opening_key.open_in_place(nonce, aad, &mut in_out)
        .map_err(|_| ConfigError::Message("Failed to decrypt password".to_string()))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|_| ConfigError::Message("Failed to convert decrypted password to string".to_string()))
}

pub fn reset_config() -> Result<(), AppError> {
    let config_path = get_config_path();
    if config_path.exists() {
        fs::remove_file(config_path)
            .map_err(|e| AppError::Config(ConfigError::Message(format!("Failed to remove config file: {}", e))))
    } else {
        Ok(())
    }
}

pub async fn create_interactive_config(existing_config: Option<AppConfig>) -> Result<AppConfig, AppError> {
    println!("Welcome to JIRA Git Helper configuration!");
    
    let jira_url = input::prompt_for_input("Enter your JIRA URL:", existing_config.as_ref().map(|c| c.jira_url.as_str()))?;
    let username = input::prompt_for_input("Enter your JIRA username:", existing_config.as_ref().map(|c| c.username.as_str()))?;
    let password = input::prompt_for_password("Enter your JIRA password:")?;
    let jira_id_prefix = input::prompt_for_input("Enter your JIRA project ID prefix:", existing_config.as_ref().map(|c| c.jira_id_prefix.as_str()))?;

    let config = AppConfig::new(jira_url, username, password, jira_id_prefix)?;
    save_config(&config)?;

    Ok(config)
}