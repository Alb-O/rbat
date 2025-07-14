/// Tests for Shaman endpoint parsing functionality
/// Parse a shaman URL into endpoint URL and checkout ID
///
/// # Arguments
/// * `shaman_url` - The shaman URL to parse
///
/// # Returns
/// A tuple of (endpoint_url, checkout_id)
fn parse_endpoint(shaman_url: &str) -> Result<(String, String), String> {
    use url::Url;

    let url = Url::parse(shaman_url).map_err(|e| format!("Invalid URL: {e}"))?;

    let scheme = match url.scheme() {
        "shaman" | "shaman+https" => "https",
        "shaman+http" => "http",
        _ => {
            return Err(format!(
                "Invalid scheme '{}', choose shaman:// or shaman+http://",
                url.scheme()
            ))
        }
    };

    let checkout_id = url
        .fragment()
        .map(|f| urlencoding::decode(f).unwrap_or_default().to_string())
        .unwrap_or_default();

    let path = if url.path().is_empty() {
        "/"
    } else {
        url.path()
    };

    let mut new_url = Url::parse(&format!(
        "{}://{}{}",
        scheme,
        url.host_str().unwrap_or(""),
        path
    ))
    .map_err(|e| format!("Failed to construct endpoint URL: {e}"))?;

    if let Some(port) = url.port() {
        new_url.set_port(Some(port)).unwrap();
    }

    Ok((new_url.to_string(), checkout_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_slashyness() {
        // Test basic shaman URL parsing
        assert_eq!(
            parse_endpoint("shaman://endpoint#123").unwrap(),
            ("https://endpoint/".to_string(), "123".to_string())
        );

        assert_eq!(
            parse_endpoint("shaman://endpoint/#123").unwrap(),
            ("https://endpoint/".to_string(), "123".to_string())
        );

        assert_eq!(
            parse_endpoint("shaman://endpoint/root#123").unwrap(),
            ("https://endpoint/root".to_string(), "123".to_string())
        );

        assert_eq!(
            parse_endpoint("shaman://endpoint/root/is/longer/#123").unwrap(),
            (
                "https://endpoint/root/is/longer/".to_string(),
                "123".to_string()
            )
        );
    }

    #[test]
    fn test_schemes_with_plus() {
        assert_eq!(
            parse_endpoint("shaman+https://endpoint/#123").unwrap(),
            ("https://endpoint/".to_string(), "123".to_string())
        );

        assert_eq!(
            parse_endpoint("shaman+http://endpoint/#123").unwrap(),
            ("http://endpoint/".to_string(), "123".to_string())
        );
    }

    #[test]
    fn test_checkout_ids() {
        assert_eq!(
            parse_endpoint("shaman+https://endpoint/").unwrap(),
            ("https://endpoint/".to_string(), "".to_string())
        );

        // Test URL-encoded characters
        assert_eq!(
            parse_endpoint("shaman+http://endpoint/#%C3%AF%C4%91").unwrap(),
            ("http://endpoint/".to_string(), "ïđ".to_string())
        );
    }

    #[test]
    fn test_invalid_scheme() {
        assert!(parse_endpoint("invalid://endpoint/").is_err());
    }
}

/// Time tracking functionality
/// This corresponds to test_shaman_time_tracker.py in the Python codebase
mod time_tracker {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// A simple time tracker that measures duration of operations
    pub struct TimeTracker {
        durations: Arc<Mutex<std::collections::HashMap<String, Duration>>>,
    }

    impl TimeTracker {
        pub fn new() -> Self {
            TimeTracker {
                durations: Arc::new(Mutex::new(std::collections::HashMap::new())),
            }
        }

        /// Track the time taken by the closure and add it to the specified key
        pub fn track_time<F, R>(&self, key: &str, f: F) -> R
        where
            F: FnOnce() -> R,
        {
            let start = Instant::now();
            let result = f();
            let duration = start.elapsed();

            let mut durations = self.durations.lock().unwrap();
            let existing = durations.entry(key.to_string()).or_insert(Duration::ZERO);
            *existing += duration;

            result
        }

        /// Get the total duration tracked for a key
        pub fn get_duration(&self, key: &str) -> Duration {
            let durations = self.durations.lock().unwrap();
            durations.get(key).copied().unwrap_or(Duration::ZERO)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::thread;

        #[test]
        fn test_empty_tracker() {
            let tracker = TimeTracker::new();
            let mut test_value = 0.0;

            tracker.track_time("test_key", || {
                thread::sleep(Duration::from_millis(10));
                test_value = 42.0;
            });

            assert_eq!(test_value, 42.0);
            assert!(tracker.get_duration("test_key") >= Duration::from_millis(10));
        }

        #[test]
        fn test_with_existing_value() {
            let tracker = TimeTracker::new();

            // First call
            tracker.track_time("test_key", || {
                thread::sleep(Duration::from_millis(10));
            });

            let first_duration = tracker.get_duration("test_key");
            assert!(first_duration >= Duration::from_millis(10));

            // Second call - should accumulate
            tracker.track_time("test_key", || {
                thread::sleep(Duration::from_millis(5));
            });

            let second_duration = tracker.get_duration("test_key");
            assert!(second_duration >= Duration::from_millis(15));
            assert!(second_duration > first_duration);
        }
    }
}
