use path_clean::PathClean;
use std::path::{Path, PathBuf};

pub(crate) fn normalize_path(path: &Path) -> String {
    let absolute = make_absolute(path);
    let simplified = simplify_windows_path(&absolute);
    to_unix_style(&simplified)
}

fn make_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    std::fs::canonicalize(path).unwrap_or_else(|_| {
        std::env::current_dir()
            .map(|cwd| cwd.join(path).clean())
            .unwrap_or_else(|_| path.to_path_buf())
    })
}

fn simplify_windows_path(path: &Path) -> PathBuf {
    dunce::simplified(path).to_path_buf()
}

fn to_unix_style(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
