use serde::{Deserialize, Serialize};

/// DTO pour créer une nouvelle tâche
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTaskDto {
    pub task_type: String,  // "video", "audio", "image"
    pub file_path: String,
    pub file_size: u64,
    pub original_name: String,
    pub mime_type: String,
    pub options: TaskOptionsDto,
}

/// Options de traitement
#[derive(Debug, Deserialize, Serialize)]
pub struct TaskOptionsDto {
    // Video options
    pub video_codec: Option<String>,
    pub resolution: Option<String>,
    pub bitrate: Option<String>,
    
    // Audio options
    pub audio_format: Option<String>,
    pub sample_rate: Option<u32>,
    
    // Image options
    pub image_format: Option<String>,
    pub quality: Option<u8>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

/// Réponse pour une tâche
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub task_type: String,
    pub status: String,
    pub progress: f32,
    pub error: Option<String>,
    pub output_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Réponse pour la création d'une tâche
#[derive(Debug, Serialize)]
pub struct CreateTaskResponse {
    pub success: bool,
    pub task_id: String,
    pub message: String,
}

/// Réponse API générique
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}
