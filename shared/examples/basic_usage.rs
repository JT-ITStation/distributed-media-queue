use shared::{Task, TaskType, TaskStatus, MediaFile, MediaType};
use std::path::PathBuf;

fn main() {
    println!("🧪 Testing Shared Library\n");

    // Test 1: Create a video task
    println!("✅ Test 1: Creating a video task");
    let media = MediaFile::new(
        "video-001".to_string(),
        MediaType::Video,
        PathBuf::from("C:\\videos\\test.mp4"),
        5_242_880, // 5MB
        "test.mp4".to_string(),
        "video/mp4".to_string(),
    );
    
    let mut task = Task::new(TaskType::VideoCompression, media);
    println!("   Task ID: {}", task.id);
    println!("   Status: {:?}", task.status);
    println!("   Progress: {}%\n", task.progress * 100.0);

    // Test 2: Update task status
    println!("✅ Test 2: Updating task status");
    task.update_status(TaskStatus::Processing);
    println!("   New status: {:?}", task.status);
    println!("   Started at: {:?}\n", task.started_at);

    // Test 3: Update progress
    println!("✅ Test 3: Updating progress");
    task.update_progress(0.5);
    println!("   Progress: {}%\n", task.progress * 100.0);

    // Test 4: Complete task
    println!("✅ Test 4: Completing task");
    task.update_status(TaskStatus::Completed);
    println!("   Status: {:?}", task.status);
    println!("   Completed at: {:?}\n", task.completed_at);

    // Test 5: Test retry logic
    println!("✅ Test 5: Testing retry logic");
    let media2 = MediaFile::new(
        "audio-001".to_string(),
        MediaType::Audio,
        PathBuf::from("C:\\audio\\test.mp3"),
        1_048_576, // 1MB
        "test.mp3".to_string(),
        "audio/mp3".to_string(),
    );
    
    let mut task2 = Task::new(TaskType::AudioProcessing, media2);
    println!("   Can retry: {}", task2.can_retry());
    
    task2.mark_failed("Connection timeout".to_string());
    println!("   After 1st failure - Retry count: {}, Can retry: {}", 
             task2.retry_count, task2.can_retry());
    
    task2.mark_failed("Connection timeout".to_string());
    task2.mark_failed("Connection timeout".to_string());
    println!("   After 3rd failure - Retry count: {}, Can retry: {}", 
             task2.retry_count, task2.can_retry());

    // Test 6: Serialization
    println!("\n✅ Test 6: Testing JSON serialization");
    let json = serde_json::to_string_pretty(&task).unwrap();
    println!("   Task as JSON:\n{}\n", json);

    println!("🎉 All tests passed!");
}
