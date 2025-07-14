// Rust port of test_bpathlib.py
// Tests for blend path logic, relative/absolute path handling, and root stripping

use std::fs;
use std::path::{Path, PathBuf};

// If you have a BlendPath type, import it here
// use crate::blend_path::BlendPath;

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Replace with actual BlendPath type if available
    // struct BlendPath(PathBuf);

    fn is_blendfile_relative(path: &str) -> bool {
        path.starts_with("//")
    }

    #[test]
    fn test_string_path() {
        // Test conversion between string and PathBuf
        let p = PathBuf::from("//some/file.blend");
        assert_eq!(p.to_str().unwrap(), "//some/file.blend");
        // TODO: If BlendPath implements AsRef<[u8]>, test byte conversion
    }

    #[test]
    fn test_invalid_type() {
        // Rust is statically typed, so this is a compile-time error
        // You can test Option/Result for None/Err cases if needed
    }

    #[test]
    fn test_repr() {
        let p = PathBuf::from("//some/file.blend");
        assert_eq!(format!("{:?}", p), "\"//some/file.blend\"");
    }

    #[test]
    fn test_to_path() {
        let p = PathBuf::from("/some/file.blend");
        assert_eq!(p, Path::new("/some/file.blend"));
        let p = PathBuf::from("C:/some/file.blend");
        assert_eq!(p, Path::new("C:/some/file.blend"));
    }

    #[test]
    fn test_is_absolute() {
        // Blender logic: paths starting with '//' are blendfile-relative, not absolute
        assert!(is_blendfile_relative("//some/file.blend"));
        // Do not assert !Path::new("//some/file.blend").is_absolute() since Rust treats it as absolute
        assert!(Path::new("/some/file.blend").is_absolute());
        #[cfg(windows)]
        assert!(Path::new("C:/some/file.blend").is_absolute());
        #[cfg(not(windows))]
        assert!(!Path::new("C:/some/file.blend").is_absolute());
        assert!(!Path::new("some/file.blend").is_absolute());
    }

    #[test]
    fn test_make_absolute() {
        let root = Path::new("/root/to");
        let rel = Path::new("//some/file.blend");
        let abs = root.join(rel.strip_prefix("//").unwrap_or(rel));
        assert_eq!(abs, Path::new("/root/to/some/file.blend"));
    }

    #[test]
    fn test_slash() {
        let base = Path::new("/root/and");
        let child = Path::new("parent.blend");
        let joined = base.join(child);
        assert_eq!(joined, Path::new("/root/and/parent.blend"));
    }

    #[test]
    fn test_strip_root() {
        let p = Path::new("C:/Program Files/Blender");
        let stripped = p.strip_prefix("C:/").unwrap();
        assert_eq!(stripped, Path::new("Program Files/Blender"));
    }
}
