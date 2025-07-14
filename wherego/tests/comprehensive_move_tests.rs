use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;
use wherego::*;

#[tokio::test]
async fn test_move_detector_integration() {
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path();

    let old_file = test_path.join("original.txt");
    let new_file = test_path.join("renamed.txt");

    // Create test file
    File::create(&old_file)
        .unwrap()
        .write_all(b"test content")
        .unwrap();

    // Test MoveDetector directly
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    // Simulate delete and create events
    detector.record_delete(old_file.clone(), false);
    let move_event = detector.match_create(new_file.clone(), false);

    assert!(move_event.is_some());
    let mv = move_event.unwrap();
    assert_eq!(mv.from, old_file);
    assert_eq!(mv.to, new_file);
    assert!(!mv.is_directory);

    // Check move map
    assert_eq!(detector.get_new_path(&old_file), Some(new_file));
}

#[tokio::test]
async fn test_directory_move_detection() {
    let test_dir = tempdir().unwrap();
    let test_path = test_dir.path();

    let old_dir = test_path.join("old_folder");
    let new_dir = test_path.join("new_folder");

    fs::create_dir(&old_dir).unwrap();
    File::create(old_dir.join("file.txt"))
        .unwrap()
        .write_all(b"test")
        .unwrap();

    let mut detector = MoveDetector::new(Duration::from_secs(2));

    // Simulate directory move
    detector.record_delete(old_dir.clone(), true);
    let move_event = detector.match_create(new_dir.clone(), true);

    assert!(move_event.is_some());
    let mv = move_event.unwrap();
    assert_eq!(mv.from, old_dir);
    assert_eq!(mv.to, new_dir);
    assert!(mv.is_directory);
}

#[tokio::test]
async fn test_move_confidence_scoring() {
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    // Test high confidence match (same filename, same directory)
    let old_path = PathBuf::from("/test/file.txt");
    let new_path = PathBuf::from("/test/file.txt");

    detector.record_delete(old_path.clone(), false);
    let move_event = detector.match_create(new_path.clone(), false);

    assert!(move_event.is_some());
}

#[tokio::test]
async fn test_move_map_consistency() {
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    let old_path = PathBuf::from("/test/old.txt");
    let new_path = PathBuf::from("/test/new.txt");

    detector.record_delete(old_path.clone(), false);
    detector.match_create(new_path.clone(), false);

    assert_eq!(detector.get_new_path(&old_path), Some(new_path));

    // Test directory move updates
    let old_dir = PathBuf::from("/old");
    let new_dir = PathBuf::from("/new");

    detector.record_delete(old_dir.clone(), true);
    detector.match_create(new_dir.clone(), true);

    // Add a file in the old directory
    let old_file = old_dir.join("file.txt");
    let new_file = new_dir.join("file.txt");

    detector.record_delete(old_file.clone(), false);
    detector.match_create(new_file.clone(), false);

    assert_eq!(detector.get_new_path(&old_file), Some(new_file));
}

#[tokio::test]
async fn test_move_detector_cleanup() {
    let mut detector = MoveDetector::new(Duration::from_millis(100));

    let old_path = PathBuf::from("/test/old.txt");
    let new_path = PathBuf::from("/test/new.txt");

    detector.record_delete(old_path.clone(), false);

    // Wait for expiration
    std::thread::sleep(Duration::from_millis(200));

    // Should not match expired delete
    let move_event = detector.match_create(new_path, false);
    assert!(move_event.is_none());
}

#[tokio::test]
async fn test_multiple_moves() {
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    let moves = vec![
        (
            PathBuf::from("/test/file1.txt"),
            PathBuf::from("/test/renamed1.txt"),
        ),
        (
            PathBuf::from("/test/file2.txt"),
            PathBuf::from("/test/renamed2.txt"),
        ),
        (
            PathBuf::from("/test/dir1"),
            PathBuf::from("/test/dir1_renamed"),
        ),
    ];

    for (old, new) in &moves {
        detector.record_delete(old.clone(), old.ends_with("dir1"));
        detector.match_create(new.clone(), new.ends_with("dir1_renamed"));
    }

    for (old, expected_new) in &moves {
        assert_eq!(detector.get_new_path(old), Some(expected_new.clone()));
    }
}
