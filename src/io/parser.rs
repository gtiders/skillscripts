use crate::model::SkillHeader;
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::Path;

const HEADER_SEPARATOR: &str = "---";
const MAX_SCAN_LINES: usize = 50;

pub(crate) struct HeaderParser;

impl HeaderParser {
    pub(crate) fn parse_file(path: &Path) -> Result<Option<SkillHeader>> {
        let content = fs::read(path)
            .with_context(|| format!("failed to read skill file {}", path.display()))?;
        let content = String::from_utf8_lossy(&content);
        Self::parse_content(&content)
    }

    pub(crate) fn parse_content(content: &str) -> Result<Option<SkillHeader>> {
        let lines: Vec<&str> = content.lines().take(MAX_SCAN_LINES).collect();
        let Some((start_index, prefix)) = find_header_start(&lines) else {
            return Ok(None);
        };

        let mut yaml = String::new();
        let mut found_end = false;

        for line in lines.iter().skip(start_index + 1).copied() {
            if is_header_boundary(line, prefix) {
                found_end = true;
                break;
            }

            yaml.push_str(strip_comment_prefix(line, prefix));
            yaml.push('\n');
        }

        if !found_end || yaml.trim().is_empty() {
            return Ok(None);
        }

        serde_yaml::from_str(&yaml)
            .map(Some)
            .map_err(|error| anyhow!("YAML parse error: {error}"))
    }
}

fn find_header_start<'a>(lines: &'a [&'a str]) -> Option<(usize, &'a str)> {
    lines.iter().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim_start();
        if index == 0 && trimmed.starts_with("#!") {
            return None;
        }

        let separator = trimmed.find(HEADER_SEPARATOR)?;
        Some((index, trimmed[..separator].trim_end()))
    })
}

fn is_header_boundary(line: &str, prefix: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix(prefix)
        .is_some_and(|suffix| suffix.trim_start().starts_with(HEADER_SEPARATOR))
}

fn strip_comment_prefix<'a>(line: &'a str, prefix: &str) -> &'a str {
    let trimmed = line.trim_start();

    if prefix.is_empty() {
        return line;
    }

    trimmed.strip_prefix(prefix).map_or(trimmed, |stripped| {
        stripped.strip_prefix(' ').unwrap_or(stripped)
    })
}
