use crate::models::media::{MediaFile, MediaType};  
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "task_id")]
    pub id: String,
    pub task_type: TaskType,
    pub media: MediaFile,
    pub status: TaskStatus,
    pub progress: f32,
    pub error: Option<String>,
    pub output_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    VideoCompression,
    AudioProcessing,
    ImageOptimization,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl Task {
    pub fn new(task_type: TaskType, media: MediaFile) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            media,
            status: TaskStatus::Pending,
            progress: 0.0,
            error: None,
            output_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn update_status(&mut self, new_status: TaskStatus) {
        self.status = new_status.clone();
        self.updated_at = Utc::now();

        match new_status {
            TaskStatus::Processing => {
                if self.started_at.is_none() {
                    self.started_at = Some(Utc::now());
                }
            }
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                if self.completed_at.is_none() {
                    self.completed_at = Some(Utc::now());
                }
            }
            _ => {}
        }
    }

    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.retry_count += 1;
        self.updated_at = Utc::now();
        if self.completed_at.is_none() {
            self.completed_at = Some(Utc::now());
        }
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.updated_at = Utc::now();
    }
    
    pub fn should_retry(&self) -> bool {
        self.can_retry()
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::VideoCompression => write!(f, "video"),
            TaskType::AudioProcessing => write!(f, "audio"),
            TaskType::ImageOptimization => write!(f, "image"),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Processing => write!(f, "processing"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let media = MediaFile {
            file_id: "test-123".to_string(),
            file_type: MediaType::Video,
            file_path: PathBuf::from("/path/to/video.mp4"),
            file_size: 1024000,
            original_name: "video.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
            metadata: HashMap::new(),
        };

        let task = Task::new(TaskType::VideoCompression, media);

        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.progress, 0.0);
        assert_eq!(task.retry_count, 0);
        assert!(task.error.is_none());
    }

    #[test]
    fn test_task_status_update() {
        let media = MediaFile {
            file_id: "test-123".to_string(),
            file_type: MediaType::Video,
            file_path: PathBuf::from("/path/to/video.mp4"),
            file_size: 1024000,
            original_name: "video.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
            metadata: HashMap::new(),
        };

        let mut task = Task::new(TaskType::VideoCompression, media);
        
        task.update_status(TaskStatus::Processing);
        assert_eq!(task.status, TaskStatus::Processing);
        assert!(task.started_at.is_some());

        task.update_status(TaskStatus::Completed);
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_progress_update() {
        let media = MediaFile {
            file_id: "test-123".to_string(),
            file_type: MediaType::Video,
            file_path: PathBuf::from("/path/to/video.mp4"),
            file_size: 1024000,
            original_name: "video.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
            metadata: HashMap::new(),
        };

        let mut task = Task::new(TaskType::VideoCompression, media);
        
        task.update_progress(0.5);
        assert_eq!(task.progress, 0.5);

        // Test clamping
        task.update_progress(1.5);
        assert_eq!(task.progress, 1.0);

        task.update_progress(-0.5);
        assert_eq!(task.progress, 0.0);
    }

    #[test]
    fn test_task_retry_logic() {
        let media = MediaFile {
            file_id: "test-123".to_string(),
            file_type: MediaType::Video,
            file_path: PathBuf::from("/path/to/video.mp4"),
            file_size: 1024000,
            original_name: "video.mp4".to_string(),
            mime_type: "video/mp4".to_string(),
            metadata: HashMap::new(),
        };

        let mut task = Task::new(TaskType::VideoCompression, media);
        
        assert!(task.can_retry());
        
        task.mark_failed("Test error".to_string());
        assert_eq!(task.retry_count, 1);
        assert!(task.can_retry());

        task.mark_failed("Test error 2".to_string());
        task.mark_failed("Test error 3".to_string());
        assert_eq!(task.retry_count, 3);
        assert!(!task.can_retry());
    }
}
