//! Project configuration loading.
//!
//! Handles parsing `iced_builder.toml` files.

use crate::model::ProjectConfig;
use std::path::Path;
use thiserror::Error;

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
}

/// Load project configuration from a TOML file.
pub fn load_config(path: &Path) -> Result<ProjectConfig, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound(path.display().to_string()));
    }

    let content = std::fs::read_to_string(path)?;
    let config: ProjectConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Save project configuration to a TOML file.
pub fn save_config(path: &Path, config: &ProjectConfig) -> Result<(), ConfigError> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Create a default configuration file.
pub fn create_default_config(path: &Path) -> Result<(), ConfigError> {
    let config = ProjectConfig::default();
    save_config(path, &config)
}

/// Find the config file in a project directory.
pub fn find_config(project_dir: &Path) -> Option<std::path::PathBuf> {
    let config_path = project_dir.join("iced_builder.toml");
    if config_path.exists() {
        Some(config_path)
    } else {
        None
    }
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
}
