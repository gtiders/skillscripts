use serde::{Deserialize, Serialize};

const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024;
const DEFAULT_SEARCH_LIMIT: usize = 5;

fn default_scan_paths() -> Vec<String> {
    vec![".".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(default = "default_scan_paths")]
    pub(crate) scan_paths: Vec<String>,
    #[serde(default)]
    pub(crate) ignore_patterns: Vec<String>,
    #[serde(
        default = "default_max_file_size",
        serialize_with = "serialize_file_size",
        deserialize_with = "deserialize_file_size"
    )]
    pub(crate) max_file_size: u64,
    #[serde(default = "default_search_limit")]
    pub(crate) search_limit: usize,
    #[serde(default)]
    pub(crate) report_parse_errors: bool,
    #[serde(default)]
    pub(crate) copy_to_clipboard_on_pick: bool,
}

fn default_max_file_size() -> u64 {
    DEFAULT_MAX_FILE_SIZE
}

fn default_search_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_paths: default_scan_paths(),
            ignore_patterns: Vec::new(),
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            search_limit: DEFAULT_SEARCH_LIMIT,
            report_parse_errors: false,
            copy_to_clipboard_on_pick: false,
        }
    }
}

impl Config {
    pub(crate) fn merge(&self, other: &Config) -> Config {
        let mut merged = self.clone();

        extend_unique(&mut merged.scan_paths, &other.scan_paths);
        extend_unique(&mut merged.ignore_patterns, &other.ignore_patterns);

        if other.max_file_size != DEFAULT_MAX_FILE_SIZE {
            merged.max_file_size = other.max_file_size;
        }

        if other.search_limit != DEFAULT_SEARCH_LIMIT {
            merged.search_limit = other.search_limit;
        }

        if other.report_parse_errors {
            merged.report_parse_errors = true;
        }

        if other.copy_to_clipboard_on_pick {
            merged.copy_to_clipboard_on_pick = other.copy_to_clipboard_on_pick;
        }

        merged
    }
}

fn extend_unique(target: &mut Vec<String>, source: &[String]) {
    for item in source {
        if !target.contains(item) {
            target.push(item.clone());
        }
    }
}

fn parse_file_size(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(bytes) = trimmed.parse::<u64>() {
        return Some(bytes);
    }

    let upper = trimmed.to_ascii_uppercase();
    let (number, multiplier) = if let Some(number) = upper.strip_suffix("GB") {
        (number, 1024u64 * 1024 * 1024)
    } else if let Some(number) = upper.strip_suffix("MB") {
        (number, 1024u64 * 1024)
    } else if let Some(number) = upper.strip_suffix("KB") {
        (number, 1024u64)
    } else if let Some(number) = upper.strip_suffix('G') {
        (number, 1024u64 * 1024 * 1024)
    } else if let Some(number) = upper.strip_suffix('M') {
        (number, 1024u64 * 1024)
    } else if let Some(number) = upper.strip_suffix('K') {
        (number, 1024u64)
    } else {
        return None;
    };

    number
        .trim()
        .parse::<u64>()
        .ok()
        .map(|size| size * multiplier)
}

fn serialize_file_size<S>(size: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if size.is_multiple_of(1024 * 1024) {
        serializer.serialize_str(&format!("{}MB", size / (1024 * 1024)))
    } else {
        serializer.serialize_u64(*size)
    }
}

fn deserialize_file_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FileSizeValue {
        Number(u64),
        String(String),
    }

    match FileSizeValue::deserialize(deserializer)? {
        FileSizeValue::Number(size) => Ok(size),
        FileSizeValue::String(size) => parse_file_size(&size)
            .ok_or_else(|| D::Error::custom(format!("Invalid file size format: {size}"))),
    }
}
