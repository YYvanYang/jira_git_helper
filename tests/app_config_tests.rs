use jira_git_helper::app_config::{AppConfig, load_config, save_config};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{env, panic};
use tempfile::TempDir;

lazy_static! {
    static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
}

struct EnvGuard<'a> {
    _lock: std::sync::MutexGuard<'a, ()>,
    old_vars: Vec<(String, Option<String>)>,
}

impl<'a> Drop for EnvGuard<'a> {
    fn drop(&mut self) {
        for (name, value) in &self.old_vars {
            match value {
                Some(v) => env::set_var(name, v),
                None => env::remove_var(name),
            }
        }
    }
}

fn with_env_vars<T, F>(test: F) -> T
where
    F: FnOnce() -> T + panic::UnwindSafe,
{
    let vars_to_manage = vec![
        "JIRA_GIT_HOME",
        "JIRA_GIT_JIRA_URL",
        "JIRA_GIT_USERNAME",
        "JIRA_GIT_JIRA_ID_PREFIX",
    ];

    let guard = EnvGuard {
        _lock: ENV_MUTEX.lock().unwrap(),
        old_vars: vars_to_manage
            .iter()
            .map(|&name| (name.to_string(), env::var(name).ok()))
            .collect(),
    };

    for var in &vars_to_manage {
        env::remove_var(var);
    }

    let result = panic::catch_unwind(test);
    drop(guard);
    result.unwrap_or_else(|e| panic::resume_unwind(e))
}

#[test]
fn test_load_and_save_config() {
    with_env_vars(|| {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".jira_git_helper.toml");
        
        env::set_var("JIRA_GIT_HOME", temp_dir.path().to_str().unwrap());

        let test_config = AppConfig {
            jira_url: "https://test.atlassian.net".to_string(),
            username: "testuser".to_string(),
            encrypted_password: Some("encrypted_password".to_string()),
            jira_id_prefix: "TEST".to_string(),
        };

        save_config(&test_config).unwrap();
        assert!(config_path.exists());

        let loaded_config = load_config().unwrap();
        assert_eq!(loaded_config.get_string("jira_url").unwrap(), "https://test.atlassian.net");
        assert_eq!(loaded_config.get_string("username").unwrap(), "testuser");
        assert_eq!(loaded_config.get_string("jira_id_prefix").unwrap(), "TEST");
    });
}

#[test]
fn test_encrypt_and_decrypt_password() {
    with_env_vars(|| {
        let original_password = "my_secret_password";
        let app_config = AppConfig::new(
            "https://test.atlassian.net".to_string(),
            "testuser".to_string(),
            original_password.to_string(),
            "TEST".to_string()
        ).unwrap();

        assert!(app_config.encrypted_password.is_some());
        let decrypted_password = app_config.get_password().unwrap();
        assert_eq!(decrypted_password, original_password);
    });
}

#[test]
fn test_config_from_env() {
    with_env_vars(|| {
        let temp_dir = TempDir::new().unwrap();
        
        env::set_var("JIRA_GIT_HOME", temp_dir.path());
        env::set_var("JIRA_GIT_JIRA_URL", "https://env.atlassian.net");
        env::set_var("JIRA_GIT_USERNAME", "envuser");
        env::set_var("JIRA_GIT_JIRA_ID_PREFIX", "ENV");

        let config = load_config().unwrap();
        assert_eq!(config.get_string("jira_url").unwrap(), "https://env.atlassian.net");
        assert_eq!(config.get_string("username").unwrap(), "envuser");
        assert_eq!(config.get_string("jira_id_prefix").unwrap(), "ENV");
    });
}