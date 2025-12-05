//! Layout file loading and saving.
//!
//! Supports both RON and JSON formats with backup creation.

use crate::model::LayoutDocument;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when loading/saving layouts.
#[derive(Debug, Error)]
pub enum LayoutFileError {
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse RON: {0}")]
    RonParseError(#[from] ron::error::SpannedError),

    #[error("Failed to serialize RON: {0}")]
    RonSerializeError(#[from] ron::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Unknown file format: {0}")]
    UnknownFormat(String),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Failed to create backup: {0}")]
    BackupError(String),
}

/// Detected file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutFormat {
    Ron,
    Json,
}

impl LayoutFormat {
    /// Detect format from file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("ron") => Some(Self::Ron),
            Some("json") => Some(Self::Json),
            _ => None,
        }
    }

    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Ron => "ron",
            Self::Json => "json",
        }
    }

    /// Get the display name for this format.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ron => "RON",
            Self::Json => "JSON",
        }
    }
}

/// Load a layout document from a file.
pub fn load_layout(path: &Path) -> Result<LayoutDocument, LayoutFileError> {
    tracing::info!(target: "iced_builder::io", path = %path.display(), "Loading layout file");

    if !path.exists() {
        return Err(LayoutFileError::NotFound(path.display().to_string()));
    }

    let content = std::fs::read_to_string(path)?;

    let format = LayoutFormat::from_path(path)
        .ok_or_else(|| LayoutFileError::UnknownFormat(path.display().to_string()))?;

    let doc = match format {
        LayoutFormat::Ron => {
            tracing::debug!(target: "iced_builder::io", "Parsing RON format");
            ron::from_str(&content)?
        }
        LayoutFormat::Json => {
            tracing::debug!(target: "iced_builder::io", "Parsing JSON format");
            serde_json::from_str(&content)?
        }
    };

    tracing::info!(target: "iced_builder::io", "Layout loaded successfully");
    Ok(doc)
}

/// Save a layout document to a file with optional backup.
pub fn save_layout(path: &Path, layout: &LayoutDocument) -> Result<(), LayoutFileError> {
    save_layout_with_backup(path, layout, true)
}

/// Save a layout document to a file.
/// 
/// If `create_backup` is true and the file exists, creates a `.bak` backup first.
pub fn save_layout_with_backup(
    path: &Path,
    layout: &LayoutDocument,
    create_backup: bool,
) -> Result<(), LayoutFileError> {
    tracing::info!(target: "iced_builder::io", path = %path.display(), "Saving layout file");

    let format = LayoutFormat::from_path(path)
        .ok_or_else(|| LayoutFileError::UnknownFormat(path.display().to_string()))?;

    // Create backup if file exists and backup is requested
    if create_backup && path.exists() {
        create_backup_file(path)?;
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let content = match format {
        LayoutFormat::Ron => {
            tracing::debug!(target: "iced_builder::io", "Serializing to RON format");
            let pretty = ron::ser::PrettyConfig::default()
                .struct_names(true)
                .enumerate_arrays(true);
            ron::ser::to_string_pretty(layout, pretty)?
        }
        LayoutFormat::Json => {
            tracing::debug!(target: "iced_builder::io", "Serializing to JSON format");
            serde_json::to_string_pretty(layout)?
        }
    };

    std::fs::write(path, content)?;
    tracing::info!(target: "iced_builder::io", "Layout saved successfully");
    Ok(())
}

/// Create a backup of an existing file.
fn create_backup_file(path: &Path) -> Result<PathBuf, LayoutFileError> {
    let backup_path = path.with_extension(format!(
        "{}.bak",
        path.extension().and_then(|e| e.to_str()).unwrap_or("bak")
    ));

    tracing::debug!(target: "iced_builder::io", 
        original = %path.display(), 
        backup = %backup_path.display(), 
        "Creating backup file"
    );

    std::fs::copy(path, &backup_path).map_err(|e| {
        LayoutFileError::BackupError(format!("Failed to create backup: {}", e))
    })?;

    Ok(backup_path)
}

/// Create a new layout file with default content.
pub fn create_default_layout(path: &Path) -> Result<(), LayoutFileError> {
    let layout = LayoutDocument::default();
    save_layout_with_backup(path, &layout, false)
}

/// Find layout files in a directory.
/// 
/// Returns a list of paths to `.ron` and `.json` files.
pub fn find_layout_files(dir: &Path) -> Vec<PathBuf> {
    let mut layouts = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(format) = LayoutFormat::from_path(&path) {
                    tracing::debug!(target: "iced_builder::io", 
                        path = %path.display(), 
                        format = format.name(),
                        "Found layout file"
                    );
                    layouts.push(path);
                }
            }
        }
    }

    // Check for layouts subdirectory
    let layouts_dir = dir.join("layouts");
    if layouts_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&layouts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && LayoutFormat::from_path(&path).is_some() {
                    layouts.push(path);
                }
            }
        }
    }

    layouts
}

/// Get the default layout file path for a project directory.
pub fn default_layout_path(project_dir: &Path) -> PathBuf {
    project_dir.join("layout.ron")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            LayoutFormat::from_path(&PathBuf::from("test.ron")),
            Some(LayoutFormat::Ron)
        );
        assert_eq!(
            LayoutFormat::from_path(&PathBuf::from("test.json")),
            Some(LayoutFormat::Json)
        );
        assert_eq!(LayoutFormat::from_path(&PathBuf::from("test.txt")), None);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(LayoutFormat::Ron.extension(), "ron");
        assert_eq!(LayoutFormat::Json.extension(), "json");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(LayoutFormat::Ron.name(), "RON");
        assert_eq!(LayoutFormat::Json.name(), "JSON");
    }

    #[test]
    fn test_default_layout_path() {
        let dir = PathBuf::from("/home/user/project");
        assert_eq!(
            default_layout_path(&dir),
            PathBuf::from("/home/user/project/layout.ron")
        );
    }
}
