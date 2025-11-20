use super::{TaskProcessor, ProgressCallback, CancelFlag};
use anyhow::Result;
use shared::{Task, TaskStatus};
use std::sync::atomic::Ordering;

pub struct AudioProcessor {
    output_dir: String,
}

impl AudioProcessor {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }
}

#[async_trait::async_trait]
impl TaskProcessor for AudioProcessor {
    async fn process(
        &self,
        task: &mut Task,
        progress_callback: ProgressCallback,
        cancel_flag: CancelFlag,
    ) -> Result<()> {
        tracing::info!(task_id = %task.id, "Starting audio processing");
        
        task.update_status(TaskStatus::Processing);
        
        // Récupérer les options
        let format = task.media.metadata.get("audio_format")
            .map(|s| s.as_str())
            .unwrap_or("mp3");
        
        let bitrate = task.media.metadata.get("bitrate")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(128);
        
        let sample_rate = task.media.metadata.get("sample_rate")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(44100);
        
        let output_filename = format!("{}_processed.{}", task.id, format);
        let output_path = format!("{}/{}", self.output_dir, output_filename);
        
        tracing::debug!(
            task_id = %task.id,
            format = format,
            bitrate = bitrate,
            sample_rate = sample_rate,
            "Audio processing parameters"
        );
        
        // Simuler le traitement avec vérification de cancellation
        simulate_audio_processing(task, progress_callback, cancel_flag).await?;
        
        task.output_path = Some(output_path);
        task.update_status(TaskStatus::Completed);
        
        tracing::info!(task_id = %task.id, "Audio processing completed");
        
        Ok(())
    }
}

async fn simulate_audio_processing(
    task: &mut Task,
    progress_callback: ProgressCallback,
    cancel_flag: CancelFlag,
) -> Result<()> {
    use tokio::time::{sleep, Duration};
    
    for i in 0..=100 {
        // Vérifier cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            tracing::warn!(task_id = %task.id, "Task cancelled");
            task.update_status(TaskStatus::Cancelled);
            return Err(anyhow::anyhow!("Task cancelled"));
        }
        
        let progress = i as f32 / 100.0;
        task.update_progress(progress);
        progress_callback(progress);
        
        tracing::debug!(task_id = %task.id, progress = progress, "Processing progress");
        sleep(Duration::from_millis(150)).await;
    }
    
    Ok(())
}
