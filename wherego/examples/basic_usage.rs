use std::time::Duration;
use wherego::*;

#[tokio::main]
async fn main() {
    // Example 1: Using MoveDetector directly
    println!("=== MoveDetector Example ===");
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    // Simulate file rename
    let old_file = "/home/user/documents/report.txt";
    let new_file = "/home/user/documents/report_final.txt";

    detector.record_delete(old_file.into(), false);
    if let Some(mv) = detector.match_create(new_file.into(), false) {
        println!("Detected move: {:?} -> {:?}", mv.from, mv.to);
    }

    // Check move map
    if let Some(new_path) = detector.get_new_path(old_file.into()) {
        println!("File moved to: {:?}", new_path);
    }

    // Example 2: Directory move
    println!("\n=== Directory Move Example ===");
    let old_dir = "/home/user/projects/old_project";
    let new_dir = "/home/user/projects/new_project";

    detector.record_delete(old_dir.into(), true);
    if let Some(mv) = detector.match_create(new_dir.into(), true) {
        println!("Directory moved: {:?} -> {:?}", mv.from, mv.to);
    }

    // Example 3: Multiple files
    println!("\n=== Multiple Files Example ===");
    let files = vec![
        ("/test/file1.txt", "/test/renamed1.txt"),
        ("/test/file2.txt", "/test/renamed2.txt"),
        ("/test/file3.txt", "/test/renamed3.txt"),
    ];

    for (old, new) in files {
        detector.record_delete(old.into(), false);
        detector.match_create(new.into(), false);
    }

    let move_map = detector.get_move_map();
    println!("Move map contains {} entries", move_map.len());

    for (old, new) in move_map {
        println!("  {:?} -> {:?}", old, new);
    }
}
