use std::path::Path;

/// Normalizes a path for display in diagnostics.
///
/// - If `raw` is already relative, it is returned as-is (with normalized separators).
/// - If `raw` is absolute and within `base`, it returns the relative path from `base`.
/// - If `raw` is absolute and outside `base` (or no base provided), it returns just the filename.
///
/// This ensures emitted shell scripts do not leak absolute paths from the build environment.
pub fn display_path(raw: &str, base: Option<&Path>) -> String {
    let path = Path::new(raw);

    let display_str = if path.is_relative() {
        raw.to_string()
    } else if let Some(base_dir) = base {
        if let Ok(rel) = path.strip_prefix(base_dir) {
            rel.to_string_lossy().to_string()
        } else {
            fallback_filename(path, raw)
        }
    } else {
        fallback_filename(path, raw)
    };

    // Normalize separators to forward slashes for portability
    display_str.replace('\\', "/")
}

fn fallback_filename(path: &Path, raw: &str) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_relative_stays_relative() {
        assert_eq!(display_path("foo/bar.sh2", None), "foo/bar.sh2");
    }

    #[test]
    fn test_relative_normalizes_slashes() {
        assert_eq!(display_path("foo\\bar.sh2", None), "foo/bar.sh2");
    }

    #[test]
    fn test_absolute_under_base() {
        // Use unix-style paths for the generic test
        let base = PathBuf::from("/users/me/project");
        let raw = "/users/me/project/src/foo.sh2";
        assert_eq!(display_path(raw, Some(&base)), "src/foo.sh2");
    }

    #[test]
    fn test_absolute_outside_base() {
        let base = PathBuf::from("/users/me/project");
        let raw = "/opt/lib/helper.sh2";
        // Should return filename only
        assert_eq!(display_path(raw, Some(&base)), "helper.sh2");
    }

    #[test]
    fn test_absolute_no_base() {
        let raw = "/opt/lib/helper.sh2";
        assert_eq!(display_path(raw, None), "helper.sh2");
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_paths() {
        let base = PathBuf::from(r"C:\Users\me\project");
        
        // Under base
        let raw = r"C:\Users\me\project\src\foo.sh2";
        assert_eq!(display_path(raw, Some(&base)), "src/foo.sh2");

        // Outside base
        let raw_out = r"C:\Windows\System32\cmd.exe";
        assert_eq!(display_path(raw_out, Some(&base)), "cmd.exe");
        
        // No base
        assert_eq!(display_path(raw, None), "foo.sh2");
    }
}
