use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use dirs::home_dir;
use ring::{aead, rand};
use ring::rand::SecureRandom;
use base64::{encode, decode};
use ring::error::Unspecified;

#[derive(Serialize, Deserialize, Clone)]
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
            password: encrypt_password(&password).unwrap(),
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
        let mut config: Config = serde_json::from_str(&config_data).map_err(|_| "Failed to parse config file")?;
        config.password = decrypt_password(&config.password).map_err(|_| "Failed to decrypt password")?;
        Ok(config)
    } else {
        Err("Config file not found")
    }
}

pub fn write_config(config: &Config) -> Result<(), &'static str> {
    let mut config = config.clone();
    config.password = encrypt_password(&config.password).unwrap();
    let config_data = serde_json::to_string(&config).map_err(|_| "Failed to serialize config data")?;
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

fn encrypt_password(password: &str) -> Result<String, Unspecified> {
    let mut key = [0; 32];
    let rng = rand::SystemRandom::new();
    rng.fill(&mut key)?;

    let mut nonce_bytes = [0; 12];
    rng.fill(&mut nonce_bytes)?;
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key)?;
    let sealing_key = aead::LessSafeKey::new(unbound_key);

    let mut in_out = password.as_bytes().to_vec();
    sealing_key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)?;

    let mut result = Vec::new();
    result.extend_from_slice(&key);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);

    Ok(encode(&result))
}

fn decrypt_password(encoded: &str) -> Result<String, Unspecified> {
    let decoded = decode(encoded).map_err(|_| Unspecified)?;

    if decoded.len() < 44 {
        return Err(Unspecified);
    }

    let key = &decoded[0..32];
    let nonce = aead::Nonce::assume_unique_for_key(decoded[32..44].try_into().map_err(|_| Unspecified)?);
    let ciphertext = &decoded[44..];

    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key).map_err(|_| Unspecified)?;
    let opening_key = aead::LessSafeKey::new(unbound_key);

    let mut in_out = ciphertext.to_vec();
    let plaintext = opening_key.open_in_place(nonce, aead::Aad::empty(), &mut in_out).map_err(|_| Unspecified)?;

    Ok(String::from_utf8(plaintext.to_vec()).map_err(|_| Unspecified)?)
}
