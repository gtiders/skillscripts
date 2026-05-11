use path_clean::PathClean;
use std::path::{Path, PathBuf};

pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let absolute = make_absolute(path);
    simplify_windows_path(&absolute)
}

fn make_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    std::fs::canonicalize(path).unwrap_or_else(|_| {
        std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path).clean())
    })
}

fn simplify_windows_path(path: &Path) -> PathBuf {
    dunce::simplified(path).to_path_buf()
}
