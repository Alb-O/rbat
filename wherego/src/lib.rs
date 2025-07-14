use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::RwLock;

/// Represents a detected move/rename operation
#[derive(Debug, Clone, PartialEq)]
pub struct MoveEvent {
    pub from: PathBuf,
    pub to: PathBuf,
    pub is_directory: bool,
    pub timestamp: Instant,
}

/// Structure to correlate delete/create events for rename/move detection
#[derive(Debug)]
pub struct MoveDetector {
    /// Recent delete events waiting for matching creates
    pending_deletes: VecDeque<(PathBuf, Instant, bool)>, // (path, timestamp, is_directory)
    /// Window of time to consider events as related
    correlation_window: Duration,
    /// Maps old paths to new paths for completed moves
    move_map: HashMap<PathBuf, PathBuf>,
    /// Recently completed moves
    recent_moves: VecDeque<MoveEvent>,
    /// Maximum number of pending events to track
    max_pending: usize,
}

impl MoveDetector {
    pub fn new(correlation_window: Duration) -> Self {
        Self {
            pending_deletes: VecDeque::new(),
            correlation_window,
            move_map: HashMap::new(),
            recent_moves: VecDeque::new(),
            max_pending: 1000,
        }
    }

    /// Record a delete event
    pub fn record_delete(&mut self, path: PathBuf, is_directory: bool) {
        self.cleanup_expired();

        // Remove any existing pending delete for this path
        self.pending_deletes.retain(|(p, _, _)| p != &path);

        self.pending_deletes
            .push_back((path, Instant::now(), is_directory));

        // Limit the size of pending deletes
        while self.pending_deletes.len() > self.max_pending {
            self.pending_deletes.pop_front();
        }
    }

    /// Try to match a create event with a pending delete
    pub fn match_create(&mut self, path: PathBuf, is_directory: bool) -> Option<MoveEvent> {
        self.cleanup_expired();

        // Find the best matching delete event
        let best_match = self.find_best_match(&path, is_directory)?;

        let (old_path, _, _) = self.pending_deletes.remove(best_match).unwrap();
        let move_event = MoveEvent {
            from: old_path.clone(),
            to: path.clone(),
            is_directory,
            timestamp: Instant::now(),
        };

        // Update move map
        self.move_map.insert(old_path.clone(), path.clone());
        self.recent_moves.push_back(move_event.clone());

        // Limit recent moves
        while self.recent_moves.len() > 100 {
            self.recent_moves.pop_front();
        }

        Some(move_event)
    }

    /// Find the best matching delete for a create event
    fn find_best_match(&self, new_path: &Path, is_directory: bool) -> Option<usize> {
        let mut best_score = 0;
        let mut best_idx = None;

        for (idx, (old_path, _, old_is_dir)) in self.pending_deletes.iter().enumerate() {
            // Must match directory/file type
            if *old_is_dir != is_directory {
                continue;
            }

            let score = self.calculate_match_score(old_path, new_path);
            if score > best_score {
                best_score = score;
                best_idx = Some(idx);
            }
        }

        // Require a minimum confidence score
        if best_score >= 50 {
            best_idx
        } else {
            None
        }
    }

    /// Calculate a confidence score for matching paths
    fn calculate_match_score(&self, old_path: &Path, new_path: &Path) -> u32 {
        let mut score = 0;

        // Same filename gives high score
        if old_path.file_name() == new_path.file_name() {
            score += 40;
        }

        // Same extension gives some score
        if old_path.extension() == new_path.extension() {
            score += 20;
        }

        // Same parent directory gives some score
        if old_path.parent() == new_path.parent() {
            score += 30;
        }

        // Similar path structure
        let old_components: Vec<_> = old_path.components().collect();
        let new_components: Vec<_> = new_path.components().collect();

        let min_len = old_components.len().min(new_components.len());
        let mut matching_components = 0;

        for i in 0..min_len {
            if old_components[i] == new_components[i] {
                matching_components += 1;
            }
        }

        score += matching_components * 10;

        score
    }

    /// Clean up expired events
    fn cleanup_expired(&mut self) {
        let now = Instant::now();

        self.pending_deletes
            .retain(|(_, ts, _)| now.duration_since(*ts) <= self.correlation_window);

        self.recent_moves
            .retain(|mv| now.duration_since(mv.timestamp) <= self.correlation_window);
    }

    /// Get the current move map
    pub fn get_move_map(&self) -> HashMap<PathBuf, PathBuf> {
        self.move_map.clone()
    }

    /// Get recent moves
    pub fn get_recent_moves(&self) -> Vec<MoveEvent> {
        self.recent_moves.iter().cloned().collect()
    }

    /// Check if a path was moved
    pub fn get_new_path(&self, old_path: &Path) -> Option<PathBuf> {
        self.move_map.get(old_path).cloned()
    }

    /// Handle directory moves by updating all affected paths
    pub fn handle_directory_move(&mut self, old_dir: &Path, new_dir: &Path) {
        let mut updates = Vec::new();

        // Update all paths that start with the old directory
        for (old_path, new_path) in &self.move_map {
            if old_path.starts_with(old_dir) {
                let relative_path = old_path.strip_prefix(old_dir).unwrap();
                let updated_new_path = new_dir.join(relative_path);
                updates.push((old_path.clone(), updated_new_path));
            }
        }

        // Apply updates
        for (old_path, new_path) in updates {
            self.move_map.insert(old_path, new_path);
        }
    }

    /// Get all moves affecting a specific directory
    pub fn get_moves_in_directory(&self, dir: &Path) -> Vec<MoveEvent> {
        self.recent_moves
            .iter()
            .filter(|mv| mv.from.starts_with(dir) || mv.to.starts_with(dir))
            .cloned()
            .collect()
    }
}

/// Enhanced watcher with move detection
pub struct PathWatcher {
    watcher: RecommendedWatcher,
    move_detector: Arc<RwLock<MoveDetector>>,
    event_tx: mpsc::UnboundedSender<WatcherEvent>,
}

#[derive(Debug, Clone)]
pub enum WatcherEvent {
    Move(MoveEvent),
    Create(PathBuf, bool),
    Delete(PathBuf, bool),
    Modify(PathBuf),
    Error(String),
}

impl PathWatcher {
    pub fn new(correlation_window: Duration) -> (Self, mpsc::UnboundedReceiver<WatcherEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let move_detector = Arc::new(RwLock::new(MoveDetector::new(correlation_window)));

        let tx = event_tx.clone();
        let md = move_detector.clone();

        let watcher = RecommendedWatcher::new(
            move |res| {
                handle_enhanced_event(res, &tx, &md);
            },
            notify::Config::default(),
        )
        .expect("Failed to create watcher");

        let path_watcher = PathWatcher {
            watcher,
            move_detector,
            event_tx,
        };

        (path_watcher, event_rx)
    }

    pub fn watch(&mut self, path: &Path, recursive: bool) -> notify::Result<()> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        self.watcher.watch(path, mode)
    }

    pub fn unwatch(&mut self, path: &Path) -> notify::Result<()> {
        self.watcher.unwatch(path)
    }

    pub async fn get_move_map(&self) -> HashMap<PathBuf, PathBuf> {
        self.move_detector.read().await.get_move_map()
    }

    pub async fn get_recent_moves(&self) -> Vec<MoveEvent> {
        self.move_detector.read().await.get_recent_moves()
    }
}

fn handle_enhanced_event(
    res: notify::Result<Event>,
    event_tx: &mpsc::UnboundedSender<WatcherEvent>,
    move_detector: &Arc<RwLock<MoveDetector>>,
) {
    match res {
        Ok(event) => {
            let kind = event.kind;
            let paths = event.paths;

            for path in paths {
                let is_dir = path.is_dir();

                match kind {
                    EventKind::Remove(_) => {
                        let mut md = move_detector.blocking_write();
                        md.record_delete(path.clone(), is_dir);

                        // Special handling for directory moves
                        if is_dir {
                            // Check if this directory was moved
                            if let Some(new_path) = md.get_new_path(&path) {
                                let _ = event_tx.send(WatcherEvent::Move(MoveEvent {
                                    from: path,
                                    to: new_path,
                                    is_directory: true,
                                    timestamp: Instant::now(),
                                }));
                                continue;
                            }
                        }

                        let _ = event_tx.send(WatcherEvent::Delete(path, is_dir));
                    }
                    EventKind::Create(_) => {
                        let mut md = move_detector.blocking_write();

                        if let Some(move_event) = md.match_create(path.clone(), is_dir) {
                            let from_path = move_event.from.clone();
                            let to_path = move_event.to.clone();
                            let _ = event_tx.send(WatcherEvent::Move(move_event));

                            // Handle directory move implications
                            if is_dir {
                                md.handle_directory_move(&from_path, &to_path);
                            }
                        } else {
                            let _ = event_tx.send(WatcherEvent::Create(path, is_dir));
                        }
                    }
                    EventKind::Modify(_) => {
                        let _ = event_tx.send(WatcherEvent::Modify(path));
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            let _ = event_tx.send(WatcherEvent::Error(e.to_string()));
        }
    }
}

/// Start watching a directory with enhanced move detection
pub async fn watch_directory(path: &str, correlation_window: Duration) {
    let (mut watcher, mut rx) = PathWatcher::new(correlation_window);

    watcher
        .watch(Path::new(path), true)
        .expect("Failed to watch directory");

    println!("Watching directory: {path}");

    // Handle events in current task
    while let Some(event) = rx.recv().await {
        match event {
            WatcherEvent::Move(mv) => {
                println!("Move detected: {:?} -> {:?}", mv.from, mv.to);
            }
            WatcherEvent::Create(path, is_dir) => {
                println!("Create: {:?} (dir: {})", path, is_dir);
            }
            WatcherEvent::Delete(path, is_dir) => {
                println!("Delete: {:?} (dir: {})", path, is_dir);
            }
            WatcherEvent::Modify(path) => {
                println!("Modify: {:?}", path);
            }
            WatcherEvent::Error(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

/// Simple function to get move information for a path
pub async fn get_path_moves(_path: &str, correlation_window: Duration) -> Vec<MoveEvent> {
    let (watcher, _) = PathWatcher::new(correlation_window);
    watcher.get_recent_moves().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_detector_basic() {
        let mut detector = MoveDetector::new(Duration::from_secs(2));

        let old_path = PathBuf::from("/test/old.txt");
        let new_path = PathBuf::from("/test/new.txt");

        detector.record_delete(old_path.clone(), false);
        let move_event = detector.match_create(new_path.clone(), false);

        assert!(move_event.is_some());
        assert_eq!(move_event.unwrap().from, old_path);
        assert_eq!(detector.get_new_path(&old_path), Some(new_path));
    }

    #[test]
    fn test_move_detector_directory() {
        let mut detector = MoveDetector::new(Duration::from_secs(2));

        let old_dir = PathBuf::from("/old/dir");
        let new_dir = PathBuf::from("/new/dir");

        detector.record_delete(old_dir.clone(), true);
        let move_event = detector.match_create(new_dir.clone(), true);

        assert!(move_event.is_some());
        assert!(move_event.unwrap().is_directory);
    }

    #[test]
    fn test_move_detector_no_match() {
        let mut detector = MoveDetector::new(Duration::from_secs(2));

        let old_path = PathBuf::from("/test/old.txt");
        let new_path = PathBuf::from("/different/new.txt");

        // Don't record delete
        let move_event = detector.match_create(new_path, false);
        assert!(move_event.is_none());
    }
}
