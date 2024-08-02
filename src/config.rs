use std::path::PathBuf;
use std::fs;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, File};
use ring::{aead, rand};
use ring::rand::SecureRandom;
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub jira_url: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_password: Option<String>,
    pub jira_id_prefix: String,
}

impl AppConfig {
    pub fn new(jira_url: String, username: String, password: String, jira_id_prefix: String) -> Result<Self, ConfigError> {
        let encrypted_password = Some(encrypt_password(&password)?);
        Ok(Self {
            jira_url,
            username,
            encrypted_password,
            jira_id_prefix,
        })
    }

    pub fn get_password(&self) -> Result<String, ConfigError> {
        self.encrypted_password
            .as_ref()
            .ok_or_else(|| ConfigError::Message("Password not set".to_string()))
            .and_then(|enc_pass| decrypt_password(enc_pass))
    }
}

pub fn load_config() -> Result<Config, ConfigError> {
    let config_path = get_config_path();
    let config = Config::builder()
        .add_source(File::with_name(config_path.to_str().unwrap()).required(false))
        .add_source(config::Environment::with_prefix("JIRA_GIT"))
        .build()?;

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<(), ConfigError> {
    let config_path = get_config_path();
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| ConfigError::Message(format!("Failed to serialize config: {}", e)))?;
    fs::write(config_path, config_str)
        .map_err(|e| ConfigError::Message(format!("Failed to write config file: {}", e)))
}

pub fn get_config_path() -> PathBuf {
    let mut config_path = home_dir().expect("Could not find home directory");
    config_path.push(".jira_git_helper.toml");
    config_path
}

fn encrypt_password(password: &str) -> Result<String, ConfigError> {
    let mut key = [0; 32];
    let mut nonce = [0; 12];
    let rng = rand::SystemRandom::new();

    rng.fill(&mut key)
        .map_err(|_| ConfigError::Message("Failed to generate encryption key".to_string()))?;
    rng.fill(&mut nonce)
        .map_err(|_| ConfigError::Message("Failed to generate nonce".to_string()))?;

    let nonce = aead::Nonce::assume_unique_for_key(nonce);
    let aad = aead::Aad::empty();

    let mut in_out = password.as_bytes().to_vec();
    let sealing_key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::AES_256_GCM, &key)
            .map_err(|_| ConfigError::Message("Failed to create encryption key".to_string()))?
    );

    sealing_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
        .map_err(|_| ConfigError::Message("Failed to encrypt password".to_string()))?;

    let mut result = Vec::with_capacity(key.len() + nonce.as_ref().len() + in_out.len());
    result.extend_from_slice(&key);
    result.extend_from_slice(nonce.as_ref());
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
    let (nonce, ciphertext) = rest.split_at(12);

    let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
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

pub fn reset_config() -> Result<(), ConfigError> {
    let config_path = get_config_path();
    if config_path.exists() {
        fs::remove_file(config_path)
            .map_err(|e| ConfigError::Message(format!("Failed to remove config file: {}", e)))
    } else {
        Ok(())
    }
}
