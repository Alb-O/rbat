use std::time::Duration;
use wherego::*;

#[test]
fn test_move_detector_basic_functionality() {
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    let old_path = std::path::PathBuf::from("/test/old.txt");
    let new_path = std::path::PathBuf::from("/test/new.txt");

    detector.record_delete(old_path.clone(), false);
    let move_event = detector.match_create(new_path.clone(), false);

    assert!(move_event.is_some());
    let mv = move_event.unwrap();
    assert_eq!(mv.from, old_path);
    assert_eq!(mv.to, new_path);
    assert!(!mv.is_directory);
}

#[test]
fn test_move_detector_directory_moves() {
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    let old_dir = std::path::PathBuf::from("/old/folder");
    let new_dir = std::path::PathBuf::from("/new/folder");

    detector.record_delete(old_dir.clone(), true);
    let move_event = detector.match_create(new_dir.clone(), true);

    assert!(move_event.is_some());
    let mv = move_event.unwrap();
    assert_eq!(mv.from, old_dir);
    assert_eq!(mv.to, new_dir);
    assert!(mv.is_directory);
}

#[test]
fn test_move_map_updates() {
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    let old_path = std::path::PathBuf::from("/test/file.txt");
    let new_path = std::path::PathBuf::from("/test/renamed.txt");

    detector.record_delete(old_path.clone(), false);
    detector.match_create(new_path.clone(), false);

    assert_eq!(detector.get_new_path(&old_path), Some(new_path));
}

#[test]
fn test_move_detector_cleanup() {
    let mut detector = MoveDetector::new(Duration::from_millis(100));

    let old_path = std::path::PathBuf::from("/test/old.txt");
    let new_path = std::path::PathBuf::from("/test/new.txt");

    detector.record_delete(old_path.clone(), false);

    // Wait for expiration
    std::thread::sleep(Duration::from_millis(200));

    // Should not match expired delete
    let move_event = detector.match_create(new_path, false);
    assert!(move_event.is_none());
}

#[test]
fn test_multiple_moves() {
    let mut detector = MoveDetector::new(Duration::from_secs(2));

    let moves = vec![
        (
            std::path::PathBuf::from("/test/file1.txt"),
            std::path::PathBuf::from("/test/renamed1.txt"),
        ),
        (
            std::path::PathBuf::from("/test/file2.txt"),
            std::path::PathBuf::from("/test/renamed2.txt"),
        ),
        (
            std::path::PathBuf::from("/test/dir1"),
            std::path::PathBuf::from("/test/dir1_renamed"),
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
