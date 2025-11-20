use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub file_id: String,
    pub file_type: MediaType,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub original_name: String,
    pub mime_type: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Video,
    Audio,
    Image,
}

impl MediaFile {
    pub fn new(
        file_id: String,
        file_type: MediaType,
        file_path: PathBuf,
        file_size: u64,
        original_name: String,
        mime_type: String,
    ) -> Self {
        Self {
            file_id,
            file_type,
            file_path,
            file_size,
            original_name,
            mime_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Video => write!(f, "video"),
            MediaType::Audio => write!(f, "audio"),
            MediaType::Image => write!(f, "image"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_file_creation() {
        let media = MediaFile::new(
            "test-123".to_string(),
            MediaType::Video,
            PathBuf::from("/path/to/video.mp4"),
            1024000,
            "video.mp4".to_string(),
            "video/mp4".to_string(),
        );

        assert_eq!(media.file_id, "test-123");
        assert_eq!(media.file_type, MediaType::Video);
        assert_eq!(media.file_size, 1024000);
        assert!(media.metadata.is_empty());
    }

    #[test]
    fn test_media_file_with_metadata() {
        let media = MediaFile::new(
            "test-123".to_string(),
            MediaType::Video,
            PathBuf::from("/path/to/video.mp4"),
            1024000,
            "video.mp4".to_string(),
            "video/mp4".to_string(),
        )
        .with_metadata("duration".to_string(), "120".to_string())
        .with_metadata("resolution".to_string(), "1920x1080".to_string());

        assert_eq!(media.metadata.len(), 2);
        assert_eq!(media.get_metadata("duration"), Some(&"120".to_string()));
        assert_eq!(media.get_metadata("resolution"), Some(&"1920x1080".to_string()));
    }

    #[test]
    fn test_add_metadata() {
        let mut media = MediaFile::new(
            "test-123".to_string(),
            MediaType::Audio,
            PathBuf::from("/path/to/audio.mp3"),
            512000,
            "audio.mp3".to_string(),
            "audio/mp3".to_string(),
        );

        media.add_metadata("bitrate".to_string(), "320kbps".to_string());
        media.add_metadata("sample_rate".to_string(), "48000".to_string());

        assert_eq!(media.metadata.len(), 2);
        assert_eq!(media.get_metadata("bitrate"), Some(&"320kbps".to_string()));
    }
}
