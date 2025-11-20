use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_tasks: u64,
    pub pending_tasks: u64,
    pub processing_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub cancelled_tasks: u64,
    pub queue_lengths: QueueLengths,
}

#[derive(Debug, Serialize)]
pub struct QueueLengths {
    pub video: i64,
    pub audio: i64,
    pub image: i64,
}

#[derive(Debug, Serialize)]
pub struct TaskSummary {
    pub id: String,
    pub task_type: String,
    pub status: String,
    pub progress: f32,
    pub created_at: String,
    pub updated_at: String,
    pub error: Option<String>,
}
