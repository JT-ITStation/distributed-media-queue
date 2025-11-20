use super::{TaskProcessor, ProgressCallback, CancelFlag};
use anyhow::Result;
use shared::{Task, TaskStatus};
use std::sync::atomic::Ordering;

pub struct ImageProcessor {
    output_dir: String,
}

impl ImageProcessor {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }
}

#[async_trait::async_trait]
impl TaskProcessor for ImageProcessor {
    async fn process(
        &self,
        task: &mut Task,
        progress_callback: ProgressCallback,
        cancel_flag: CancelFlag,
    ) -> Result<()> {
        tracing::info!(task_id = %task.id, "Starting image optimization");
        
        task.update_status(TaskStatus::Processing);
        
        // Récupérer les options
        let format = task.media.metadata.get("image_format")
            .map(|s| s.as_str())
            .unwrap_or("jpg");
        
        let quality = task.media.metadata.get("quality")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(85);
        
        let max_width = task.media.metadata.get("max_width")
            .and_then(|s| s.parse::<u32>().ok());
        
        let max_height = task.media.metadata.get("max_height")
            .and_then(|s| s.parse::<u32>().ok());
        
        let output_filename = format!("{}_optimized.{}", task.id, format);
        let output_path = format!("{}/{}", self.output_dir, output_filename);
        
        tracing::debug!(
            task_id = %task.id,
            format = format,
            quality = quality,
            max_width = ?max_width,
            max_height = ?max_height,
            "Image optimization parameters"
        );
        
        // Simuler le traitement avec vérification de cancellation
        simulate_image_processing(task, progress_callback, cancel_flag).await?;
        
        task.output_path = Some(output_path);
        task.update_status(TaskStatus::Completed);
        
        tracing::info!(task_id = %task.id, "Image optimization completed");
        
        Ok(())
    }
}

async fn simulate_image_processing(
    task: &mut Task,
    progress_callback: ProgressCallback,
    cancel_flag: CancelFlag,
) -> Result<()> {
    use tokio::time::{sleep, Duration};
    
    for i in 0..=20 {
        // Vérifier cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            tracing::warn!(task_id = %task.id, "Task cancelled");
            task.update_status(TaskStatus::Cancelled);
            return Err(anyhow::anyhow!("Task cancelled"));
        }
        
        let progress = i as f32 / 20.0;
        task.update_progress(progress);
        progress_callback(progress);
        
        tracing::debug!(task_id = %task.id, progress = progress, "Processing progress");
        sleep(Duration::from_millis(200)).await;
    }
    
    Ok(())
}
