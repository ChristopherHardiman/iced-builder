//! Layout file loading and saving.
//!
//! Supports both RON and JSON formats.

use crate::model::LayoutDocument;
use std::path::Path;
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
}

/// Load a layout document from a file.
pub fn load_layout(path: &Path) -> Result<LayoutDocument, LayoutFileError> {
    let content = std::fs::read_to_string(path)?;

    let format = LayoutFormat::from_path(path)
        .ok_or_else(|| LayoutFileError::UnknownFormat(path.display().to_string()))?;

    match format {
        LayoutFormat::Ron => {
            let doc: LayoutDocument = ron::from_str(&content)?;
            Ok(doc)
        }
        LayoutFormat::Json => {
            let doc: LayoutDocument = serde_json::from_str(&content)?;
            Ok(doc)
        }
    }
}

/// Save a layout document to a file.
pub fn save_layout(path: &Path, layout: &LayoutDocument) -> Result<(), LayoutFileError> {
    let format = LayoutFormat::from_path(path)
        .ok_or_else(|| LayoutFileError::UnknownFormat(path.display().to_string()))?;

    // Create backup if file exists
    if path.exists() {
        let backup_path = path.with_extension("bak");
        let _ = std::fs::copy(path, backup_path);
    }

    let content = match format {
        LayoutFormat::Ron => {
            let pretty = ron::ser::PrettyConfig::default()
                .struct_names(true)
                .enumerate_arrays(true);
            ron::ser::to_string_pretty(layout, pretty)?
        }
        LayoutFormat::Json => serde_json::to_string_pretty(layout)?,
    };

    std::fs::write(path, content)?;
    Ok(())
}

/// Create a new layout file with default content.
pub fn create_default_layout(path: &Path) -> Result<(), LayoutFileError> {
    let layout = LayoutDocument::default();
    save_layout(path, &layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
        assert_eq!(
            LayoutFormat::from_path(&PathBuf::from("test.txt")),
            None
        );
    }
}
