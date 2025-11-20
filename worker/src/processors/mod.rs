use shared::Task;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Type pour le callback de progression
pub type ProgressCallback = Arc<dyn Fn(f32) + Send + Sync>;

/// Type pour le flag de cancellation
pub type CancelFlag = Arc<AtomicBool>;

/// Trait pour tous les processeurs de tâches
#[async_trait::async_trait]
pub trait TaskProcessor: Send + Sync {
    async fn process(
        &self,
        task: &mut Task,
        progress_callback: ProgressCallback,
        cancel_flag: CancelFlag,
    ) -> Result<()>;
}

pub mod video_processor;
pub mod audio_processor;
pub mod image_processor;

pub use video_processor::VideoProcessor;
pub use audio_processor::AudioProcessor;
pub use image_processor::ImageProcessor;
