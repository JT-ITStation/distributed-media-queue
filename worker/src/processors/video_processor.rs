use super::{TaskProcessor, ProgressCallback, CancelFlag};
use anyhow::Result;
use shared::{Task, TaskStatus};
use std::sync::atomic::Ordering;

pub struct VideoProcessor {
    output_dir: String,
}

impl VideoProcessor {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }
}

#[async_trait::async_trait]
impl TaskProcessor for VideoProcessor {
    async fn process(
        &self,
        task: &mut Task,
        progress_callback: ProgressCallback,
        cancel_flag: CancelFlag,
    ) -> Result<()> {
        tracing::info!(task_id = %task.id, "Starting video compression");
        
        task.update_status(TaskStatus::Processing);
        
        // Récupérer les options
        let codec = task.media.metadata.get("video_codec")
            .map(|s| s.as_str())
            .unwrap_or("libx264");
        
        let preset = task.media.metadata.get("preset")
            .map(|s| s.as_str())
            .unwrap_or("medium");
        
        let crf = task.media.metadata.get("crf")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(23);
        
        let output_filename = format!("{}_compressed.mp4", task.id);
        let output_path = format!("{}/{}", self.output_dir, output_filename);
        
        tracing::debug!(
            task_id = %task.id,
            codec = codec,
            preset = preset,
            crf = crf,
            "Video compression parameters"
        );
        
        // Simuler le traitement avec vérification de cancellation
        simulate_video_processing(task, progress_callback, cancel_flag).await?;
        
        task.output_path = Some(output_path);
        task.update_status(TaskStatus::Completed);
        
        tracing::info!(task_id = %task.id, "Video compression completed");
        
        Ok(())
    }
}

async fn simulate_video_processing(
    task: &mut Task,
    progress_callback: ProgressCallback,
    cancel_flag: CancelFlag,
) -> Result<()> {
    use tokio::time::{sleep, Duration};
    
    for i in 0..=100 {
        // Vérifier cancellation à chaque itération
        if cancel_flag.load(Ordering::Relaxed) {
            tracing::warn!(task_id = %task.id, "Task cancelled");
            task.update_status(TaskStatus::Cancelled);
            return Err(anyhow::anyhow!("Task cancelled"));
        }
        
        let progress = i as f32 / 100.0;
        task.update_progress(progress);
        progress_callback(progress);
        
        tracing::debug!(task_id = %task.id, progress = progress, "Processing progress");
        sleep(Duration::from_millis(200)).await;
    }
    
    Ok(())
}
