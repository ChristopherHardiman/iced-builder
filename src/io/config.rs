//! Project configuration loading and saving.
//!
//! Handles parsing `iced_builder.toml` files with backup support.

use crate::model::ProjectConfig;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The default config file name.
pub const CONFIG_FILENAME: &str = "iced_builder.toml";

/// Errors that can occur when loading config.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Config file not found: {0}")]
    NotFound(String),

    #[error("Failed to create backup: {0}")]
    BackupError(String),
}

/// Load project configuration from a TOML file.
pub fn load_config(path: &Path) -> Result<ProjectConfig, ConfigError> {
    tracing::info!(target: "iced_builder::io", path = %path.display(), "Loading config file");

    if !path.exists() {
        return Err(ConfigError::NotFound(path.display().to_string()));
    }

    let content = std::fs::read_to_string(path)?;
    let config: ProjectConfig = toml::from_str(&content)?;

    tracing::info!(target: "iced_builder::io", "Config loaded successfully");
    Ok(config)
}

/// Save project configuration to a TOML file.
pub fn save_config(path: &Path, config: &ProjectConfig) -> Result<(), ConfigError> {
    save_config_with_backup(path, config, true)
}

/// Save project configuration with optional backup.
pub fn save_config_with_backup(
    path: &Path,
    config: &ProjectConfig,
    create_backup: bool,
) -> Result<(), ConfigError> {
    tracing::info!(target: "iced_builder::io", path = %path.display(), "Saving config file");

    // Create backup if file exists
    if create_backup && path.exists() {
        let backup_path = path.with_extension("toml.bak");
        tracing::debug!(target: "iced_builder::io", 
            backup = %backup_path.display(), 
            "Creating config backup"
        );
        std::fs::copy(path, &backup_path).map_err(|e| {
            ConfigError::BackupError(format!("Failed to create backup: {}", e))
        })?;
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;

    tracing::info!(target: "iced_builder::io", "Config saved successfully");
    Ok(())
}

/// Create a default configuration file.
pub fn create_default_config(path: &Path) -> Result<(), ConfigError> {
    let config = ProjectConfig::default();
    save_config_with_backup(path, &config, false)
}

/// Find the config file in a project directory.
pub fn find_config(project_dir: &Path) -> Option<PathBuf> {
    let config_path = project_dir.join(CONFIG_FILENAME);
    if config_path.exists() {
        Some(config_path)
    } else {
        None
    }
}

/// Get the default config file path for a project directory.
pub fn config_path(project_dir: &Path) -> PathBuf {
    project_dir.join(CONFIG_FILENAME)
}

/// Check if a directory is a valid Iced Builder project.
pub fn is_valid_project(project_dir: &Path) -> bool {
    find_config(project_dir).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_serializes() {
        let config = ProjectConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("output_file"));
        assert!(toml_str.contains("message_type"));
    }

    #[test]
    fn test_config_filename() {
        assert_eq!(CONFIG_FILENAME, "iced_builder.toml");
    }

    #[test]
    fn test_config_path() {
        let dir = PathBuf::from("/home/user/project");
        assert_eq!(
            config_path(&dir),
            PathBuf::from("/home/user/project/iced_builder.toml")
        );
    }
}
