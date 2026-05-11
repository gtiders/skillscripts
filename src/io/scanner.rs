use crate::model::Config;
use anyhow::Result;
use ignore::{
    WalkBuilder, WalkParallel, WalkState,
    overrides::{Override, OverrideBuilder},
};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::path_utils::normalize_path;

const NUL_SNIFF_SIZE: usize = 512;

struct FileScanner<'a> {
    config: &'a Config,
}

impl<'a> FileScanner<'a> {
    fn new(config: &'a Config) -> Self {
        Self { config }
    }

    fn scan(&self) -> Result<Vec<PathBuf>> {
        let files = Arc::new(Mutex::new(Vec::new()));
        let max_size = self.config.max_file_size;

        for scan_path in &self.config.scan_paths {
            let root = Path::new(scan_path);
            if !root.exists() {
                eprintln!("Skipped scan path that does not exist: {scan_path}");
                continue;
            }

            let walker = build_walker(root, &self.config.ignore_patterns)?;
            let shared_files = Arc::clone(&files);

            walker.run(|| {
                let mut collector = LocalCollector::new(Arc::clone(&shared_files));

                Box::new(move |entry| {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_file() && is_safe_text_file(path, max_size) {
                                collector.push(normalize_path(path));
                            }
                        }
                        Err(error) => {
                            eprintln!("Skipped an unreadable path: {error}");
                        }
                    }

                    WalkState::Continue
                })
            });
        }

        Ok(take_scanned_files(files))
    }
}

struct LocalCollector {
    shared: Arc<Mutex<Vec<PathBuf>>>,
    local: Vec<PathBuf>,
}

impl LocalCollector {
    const FLUSH_THRESHOLD: usize = 256;

    fn new(shared: Arc<Mutex<Vec<PathBuf>>>) -> Self {
        Self {
            shared,
            local: Vec::with_capacity(Self::FLUSH_THRESHOLD),
        }
    }

    fn push(&mut self, path: PathBuf) {
        self.local.push(path);
        if self.local.len() >= Self::FLUSH_THRESHOLD {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if self.local.is_empty() {
            return;
        }

        if let Ok(mut shared) = self.shared.lock() {
            shared.append(&mut self.local);
        } else {
            self.local.clear();
        }
    }
}

impl Drop for LocalCollector {
    fn drop(&mut self) {
        self.flush();
    }
}

fn build_walker(root: &Path, ignore_patterns: &[String]) -> Result<WalkParallel> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .follow_links(false);

    if let Some(overrides) = build_overrides(root, ignore_patterns)? {
        builder.overrides(overrides);
    }

    Ok(builder.build_parallel())
}

fn build_overrides(root: &Path, ignore_patterns: &[String]) -> Result<Option<Override>> {
    if ignore_patterns.is_empty() {
        return Ok(None);
    }

    let mut builder = OverrideBuilder::new(root);
    let mut has_valid_pattern = false;

    for pattern in ignore_patterns.iter().map(String::as_str).map(str::trim) {
        if pattern.is_empty() {
            continue;
        }

        has_valid_pattern = true;
        let override_pattern = pattern
            .strip_prefix('!')
            .map_or_else(|| format!("!{pattern}"), |raw| format!("!{raw}"));

        if let Err(error) = builder.add(&override_pattern) {
            eprintln!("Ignored invalid ignore pattern '{pattern}'. Reason: {error}");
        }
    }

    if !has_valid_pattern {
        return Ok(None);
    }

    builder.build().map(Some).map_err(Into::into)
}

fn take_scanned_files(files: Arc<Mutex<Vec<PathBuf>>>) -> Vec<PathBuf> {
    match Arc::try_unwrap(files) {
        Ok(mutex) => match mutex.into_inner() {
            Ok(files) => files,
            Err(poisoned) => poisoned.into_inner(),
        },
        Err(shared) => shared.lock().map(|files| files.clone()).unwrap_or_default(),
    }
}

#[inline]
fn passes_size_limit(length: u64, max_size: u64) -> bool {
    length != 0 && length <= max_size
}

#[inline]
fn sniff_contains_nul(sniffed: &[u8], file_len: u64) -> bool {
    let sniff_len = usize::try_from(file_len)
        .unwrap_or(NUL_SNIFF_SIZE)
        .min(NUL_SNIFF_SIZE)
        .min(sniffed.len());
    sniffed[..sniff_len].contains(&0x00)
}

#[inline]
fn is_safe_text_file(path: &Path, max_size: u64) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };

    let length = metadata.len();
    if !passes_size_limit(length, max_size) {
        return false;
    }

    let Ok(mut file) = File::open(path) else {
        return false;
    };

    let sniff_size = usize::try_from(length)
        .unwrap_or(NUL_SNIFF_SIZE)
        .min(NUL_SNIFF_SIZE);
    let mut buffer = [0u8; NUL_SNIFF_SIZE];

    let Ok(bytes_read) = file.read(&mut buffer[..sniff_size]) else {
        return false;
    };

    !sniff_contains_nul(&buffer[..bytes_read], length)
}

pub(crate) fn scan_files(config: &Config) -> Result<Vec<PathBuf>> {
    FileScanner::new(config).scan()
}
