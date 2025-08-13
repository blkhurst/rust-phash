use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Checks if a path has an allowed file extension (case-insensitive).
fn has_allowed_extension(path: &Path, allowed_exts: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext_str| {
            allowed_exts
                .iter()
                .any(|&allowed| ext_str.eq_ignore_ascii_case(allowed))
        })
}

/// Recursively scans the `root` directory for files with allowed extensions.
pub fn scan_files(root: &Path, allowed_exts: &[&str]) -> Vec<PathBuf> {
    let mut matching_files: Vec<PathBuf> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| has_allowed_extension(path, allowed_exts))
        .collect();

    matching_files.sort(); // Deterministic order
    matching_files
}
