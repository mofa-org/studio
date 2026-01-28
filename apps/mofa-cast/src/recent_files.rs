//! Recent files manager for mofa-cast
//!
//! Tracks the last 5 imported script files and persists them to disk.

use crate::transcript_parser::Transcript;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Maximum number of recent files to track
const MAX_RECENT_FILES: usize = 5;

/// Recent file entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    /// File path
    pub path: PathBuf,
    /// File name (for display)
    pub name: String,
    /// Message count
    pub message_count: usize,
    /// Speaker count
    pub speaker_count: usize,
    /// Last opened timestamp
    pub last_opened: u64,
}

impl RecentFile {
    /// Create a new recent file entry
    pub fn new(path: PathBuf, transcript: &Transcript) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            path,
            name,
            message_count: transcript.message_count(),
            speaker_count: transcript.metadata.participants.len(),
            last_opened: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Format for display in UI
    pub fn format_display(&self) -> String {
        format!(
            "{} • {} msgs • {} speakers",
            self.name, self.message_count, self.speaker_count
        )
    }
}

/// Manager for recent files list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFilesManager {
    /// List of recent files (most recent first)
    files: Vec<RecentFile>,
}

impl Default for RecentFilesManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecentFilesManager {
    /// Create a new empty manager
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?;
        path.push("mofa-studio");
        path.push("recent_cast_scripts.json");
        Some(path)
    }

    /// Load from disk
    pub fn load() -> Self {
        let config_path = match Self::config_path() {
            Some(path) => path,
            None => {
                ::log::warn!("Cannot determine config directory, starting with empty recent files");
                return Self::new();
            }
        };

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    ::log::warn!("Failed to create config directory: {}", e);
                    return Self::new();
                }
            }
        }

        // Read and parse file
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<RecentFilesManager>(&content) {
                    Ok(manager) => {
                        ::log::info!("Loaded {} recent files from disk", manager.files.len());
                        manager
                    }
                    Err(e) => {
                        ::log::warn!("Failed to parse recent files: {}, starting fresh", e);
                        Self::new()
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                ::log::info!("Recent files cache not found, starting fresh");
                Self::new()
            }
            Err(e) => {
                ::log::warn!("Failed to read recent files: {}, starting fresh", e);
                Self::new()
            }
        }
    }

    /// Save to disk
    pub fn save(&self) {
        let config_path = match Self::config_path() {
            Some(path) => path,
            None => {
                ::log::warn!("Cannot determine config directory, skipping save");
                return;
            }
        };

        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write(&config_path, json) {
                    ::log::warn!("Failed to save recent files: {}", e);
                } else {
                    ::log::debug!("Saved {} recent files to disk", self.files.len());
                }
            }
            Err(e) => {
                ::log::warn!("Failed to serialize recent files: {}", e);
            }
        }
    }

    /// Add a new recent file (or update existing)
    pub fn add(&mut self, file: RecentFile) {
        // Remove existing entry with same path (if any)
        self.files.retain(|f| f.path != file.path);

        // Add to front of list
        self.files.insert(0, file);

        // Trim to max size
        if self.files.len() > MAX_RECENT_FILES {
            self.files.truncate(MAX_RECENT_FILES);
        }

        // Auto-save after modification
        self.save();
    }

    /// Get all recent files (most recent first)
    pub fn get_all(&self) -> &[RecentFile] {
        &self.files
    }

    /// Clear all recent files
    pub fn clear(&mut self) {
        self.files.clear();
        self.save();
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get count
    pub fn len(&self) -> usize {
        self.files.len()
    }
}
