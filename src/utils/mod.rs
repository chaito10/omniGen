//! Utility functions and helper modules.

use anyhow::Result;
use std::path::Path;

/// Get the project cache directory.
pub fn cache_dir() -> Result<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "omnigen")
        .ok_or_else(|| anyhow::anyhow!("failed to determine cache directory"))?;
    Ok(dirs.cache_dir().to_path_buf())
}

/// Get the models directory.
pub fn models_dir() -> Result<std::path::PathBuf> {
    Ok(std::path::PathBuf::from("models"))
}

/// Get the config directory.
pub fn config_dir() -> Result<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "omnigen")
        .ok_or_else(|| anyhow::anyhow!("failed to determine config directory"))?;
    Ok(dirs.config_dir().to_path_buf())
}

/// Get the data directory.
pub fn data_dir() -> Result<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "omnigen")
        .ok_or_else(|| anyhow::anyhow!("failed to determine data directory"))?;
    Ok(dirs.data_dir().to_path_buf())
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get the file size in bytes.
pub fn file_size(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

/// Format a duration as a human-readable string.
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Format a number with commas.
pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
