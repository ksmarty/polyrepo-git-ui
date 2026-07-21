use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{GitHubAuth, RepoGroup, Repository};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub default_branch: String,
    pub default_repo_location: String,
    pub theme: String,
    pub auto_fetch_on_open: bool,
    pub fetch_interval_seconds: u32,
    pub sidebar_width: u32,
    #[serde(default = "default_pr_density")]
    pub pr_density: String,
}

fn default_pr_density() -> String {
    "compact".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_repo_location = dirs::home_dir()
            .map(|d| d.join("Projects").to_string_lossy().to_string())
            .unwrap_or_else(|| "~/Projects".to_string());
        Self {
            default_branch: "main".to_string(),
            default_repo_location,
            theme: "midnight".to_string(),
            auto_fetch_on_open: true,
            fetch_interval_seconds: 300,
            sidebar_width: 300,
            pr_density: "compact".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app_config: AppConfig,
    pub repos: Vec<Repository>,
    pub groups: Vec<RepoGroup>,
    pub github_auth: GitHubAuth,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_config: AppConfig::default(),
            repos: Vec::new(),
            groups: Vec::new(),
            github_auth: GitHubAuth::default(),
        }
    }
}

impl Config {
    pub fn config_dir() -> Option<PathBuf> {
        // Use ~/.config/polyrepo-git-ui on all Unix platforms (matches Linux XDG convention)
        // Windows still uses the standard config_dir (AppData/Roaming)
        #[cfg(unix)]
        {
            dirs::home_dir().map(|d| d.join(".config").join("polyrepo-git-ui"))
        }
        #[cfg(not(unix))]
        {
            dirs::config_dir().map(|d| d.join("polyrepo-git-ui"))
        }
    }

    fn config_file() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn config_path_string() -> Option<String> {
        Self::config_file().map(|p| p.to_string_lossy().to_string())
    }

    pub fn load() -> Result<Self, String> {
        let file = Self::config_file().ok_or("Could not determine config directory")?;

        if !file.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&file).map_err(|e| format!("Failed to read config: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let config_dir =
            Self::config_dir().ok_or("Could not determine config directory")?;

        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let file = config_dir.join("config.toml");
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(file, content).map_err(|e| format!("Failed to write config: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.default_branch, "main");
        assert!(!config.default_repo_location.is_empty());
        assert_eq!(config.theme, "midnight");
        assert!(config.auto_fetch_on_open);
        assert_eq!(config.fetch_interval_seconds, 300);
        assert_eq!(config.sidebar_width, 300);
        assert_eq!(config.pr_density, "compact");
    }

    #[test]
    fn test_app_config_serialize_deserialize() {
        let config = AppConfig {
            default_branch: "develop".to_string(),
            default_repo_location: "/Users/test/Projects".to_string(),
            theme: "midnight".to_string(),
            auto_fetch_on_open: false,
            fetch_interval_seconds: 600,
            sidebar_width: 350,
            pr_density: "relaxed".to_string(),
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.default_branch, "develop");
        assert_eq!(deserialized.default_repo_location, "/Users/test/Projects");
        assert_eq!(deserialized.theme, "midnight");
        assert!(!deserialized.auto_fetch_on_open);
        assert_eq!(deserialized.fetch_interval_seconds, 600);
        assert_eq!(deserialized.sidebar_width, 350);
        assert_eq!(deserialized.pr_density, "relaxed");
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app_config, AppConfig::default());
        assert!(config.repos.is_empty());
        assert!(config.groups.is_empty());
        assert!(config.github_auth.token.is_none());
    }

    #[test]
    fn test_config_serialize_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.app_config.theme, "midnight");
        assert!(deserialized.repos.is_empty());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let config = Config::default();
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_file, content).unwrap();

        let loaded_content = fs::read_to_string(&config_file).unwrap();
        let loaded: Config = toml::from_str(&loaded_content).unwrap();

        assert_eq!(loaded.app_config.default_branch, "main");
        assert!(loaded.repos.is_empty());
    }
}
