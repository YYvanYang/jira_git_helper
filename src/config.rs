use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use dirs::home_dir;
use ring::{aead, rand};
use ring::rand::SecureRandom;
use base64::{encode, decode};

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
            password: encrypt_password(&password),
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
    config.password = encrypt_password(&config.password);
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

fn encrypt_password(password: &str) -> String {
    let mut key = [0; 32];
    let rng = rand::SystemRandom::new();
    rng.fill(&mut key).unwrap();

    let nonce = aead::Nonce::assume_unique_for_key([0; 12]);
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key).unwrap();
    let sealing_key = aead::LessSafeKey::new(unbound_key);

    let mut in_out = password.as_bytes().to_vec();
    sealing_key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out).unwrap();

    let mut result = key.to_vec();
    result.extend(nonce.as_ref());
    result.extend(in_out);

    encode(&result)
}

fn decrypt_password(encoded: &str) -> Result<String, aead::Unspecified> {
    let decoded = decode(encoded).unwrap();

    let key = &decoded[0..32];
    let nonce = aead::Nonce::assume_unique_for_key(decoded[32..44].try_into().unwrap());
    let ciphertext = &decoded[44..];

    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key).unwrap();
    let opening_key = aead::LessSafeKey::new(unbound_key);

    let mut in_out = ciphertext.to_vec();
    let plaintext = opening_key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)?;

    Ok(String::from_utf8(plaintext.to_vec()).unwrap())
}
