use crate::error::{GitorgError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DefaultsConfig {
    pub orgs: Option<Vec<String>>,
}

impl Config {
    pub fn token(&self) -> Result<&str> {
        self.auth
            .token
            .as_deref()
            .ok_or(GitorgError::NotAuthenticated)
    }
}

pub fn config_path() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg).join("gitorg").join("config.toml");
        return Ok(path);
    }

    let home =
        dirs::home_dir().ok_or_else(|| GitorgError::Config("Cannot find home directory".into()))?;
    Ok(home.join(".config").join("gitorg").join("config.toml"))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(config)?;
    fs::write(&path, &contents)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip() {
        let config = Config {
            auth: AuthConfig {
                token: Some("ghp_test123".to_string()),
            },
            defaults: DefaultsConfig {
                orgs: Some(vec!["myorg".to_string(), "other".to_string()]),
            },
        };

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.auth.token.as_deref(), Some("ghp_test123"));
        assert_eq!(
            deserialized.defaults.orgs,
            Some(vec!["myorg".to_string(), "other".to_string()])
        );
    }

    #[test]
    fn config_default_has_no_token() {
        let config = Config::default();
        assert!(config.token().is_err());
    }

    #[test]
    fn config_token_accessor() {
        let config = Config {
            auth: AuthConfig {
                token: Some("ghp_abc".to_string()),
            },
            defaults: DefaultsConfig::default(),
        };
        assert_eq!(config.token().unwrap(), "ghp_abc");
    }

    #[test]
    fn config_deserialize_empty() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.auth.token.is_none());
        assert!(config.defaults.orgs.is_none());
    }

    #[test]
    fn config_path_uses_xdg() {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/test_xdg");
        let path = config_path().unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test_xdg/gitorg/config.toml"));
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
