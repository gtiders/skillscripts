use crate::registry::{Skill, display_path};
use anyhow::{Result, anyhow};
use ratatui::text::Line;
use skim::prelude::*;
use skim::tui::BorderType;
use std::borrow::Cow;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};

const SKIM_COLORS: &str = concat!(
    "dark,",
    "fg:#abb2bf,",
    "current:#e5c07b,",
    "matched:#61afef,",
    "current_match:#98c379,",
    "query:#abb2bf,",
    "prompt:#c678dd,",
    "spinner:#56b6c2,",
    "info:#5c6370,",
    "border:#3e4452,",
    "header:#5c6370,",
    "cursor:#e06c75,",
    "selected:#98c379"
);
const PICKER_HEADER: &str =
    "Type to filter  |  Enter run  |  Esc cancel  |  Preview: YAML metadata + script content";
const COLUMN_GAP: &str = "  ";
const ID_WIDTH: usize = 6;
const GITHUB_DARK_THEME: &[u8] = include_bytes!("themes/github-dark.tmTheme");

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static PREVIEW_THEME: OnceLock<Theme> = OnceLock::new();

pub(crate) fn run_skim_picker(items: Vec<Skill>) -> Result<Option<Skill>> {
    if items.is_empty() {
        return Ok(None);
    }

    let options = SkimOptionsBuilder::default()
        .no_height(true)
        .border(BorderType::Rounded)
        .highlight_line(true)
        .color(SKIM_COLORS)
        .header(PICKER_HEADER.to_string())
        .preview("")
        .multi(false)
        .prompt("🔎 ")
        .preview_window("right:35%:wrap")
        .build()?;

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    let entries = items
        .into_iter()
        .map(PickerItem::new)
        .map(|item| Arc::new(item) as Arc<dyn SkimItem>)
        .collect::<Vec<_>>();
    tx.send(entries)?;
    drop(tx);

    let output = Skim::run_with(options, Some(rx)).map_err(|error| anyhow!(error.to_string()))?;
    Ok(output
        .selected_items
        .into_iter()
        .next()
        .and_then(|item| item.downcast_item::<PickerItem>().cloned())
        .map(|item| item.skill))
}

#[derive(Clone)]
struct PickerItem {
    skill: Skill,
    search_text: String,
}

impl PickerItem {
    fn new(skill: Skill) -> Self {
        let path = display_path(&skill.path);
        let comment = skill.comment.as_deref().unwrap_or_default();
        Self {
            search_text: format!("{} {} {} {}", skill.id, path, skill.command, comment),
            skill,
        }
    }
}

impl SkimItem for PickerItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.search_text)
    }

    fn display(&self, context: DisplayContext) -> Line<'_> {
        Line::from(format_row(
            context.container_width,
            self.skill.id.0,
            self.skill.comment.as_deref(),
        ))
    }

    fn output(&self) -> Cow<'_, str> {
        Cow::Owned(display_path(&self.skill.path))
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        match fs::read(&self.skill.path) {
            Ok(bytes) => ItemPreview::AnsiText(render_preview(&self.skill, &bytes)),
            Err(error) => ItemPreview::AnsiText(format!(
                "preview failed: {error}\npath: {}",
                display_path(&self.skill.path)
            )),
        }
    }
}

fn render_preview(skill: &Skill, bytes: &[u8]) -> String {
    let body = String::from_utf8_lossy(bytes);
    let metadata =
        serde_yaml::to_string(skill).unwrap_or_else(|error| format!("metadata: {error}\n"));
    let syntax_set = syntax_set();
    let file_syntax = syntax_set
        .find_syntax_for_file(&skill.path)
        .ok()
        .flatten()
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let content = format!("# YAML\n{metadata}\n---\n\n{body}");
    let mut rendered = render_highlighted(&content, file_syntax, syntax_set);
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push_str("\x1b[0m");

    rendered
}

fn render_highlighted(
    body: &str,
    syntax: &syntect::parsing::SyntaxReference,
    syntax_set: &SyntaxSet,
) -> String {
    let mut highlighter = HighlightLines::new(syntax, preview_theme());
    let mut rendered = String::new();

    for line in LinesWithEndings::from(body) {
        let highlighted = highlighter
            .highlight_line(line, syntax_set)
            .map(|ranges| as_24_bit_terminal_escaped(&ranges[..], false))
            .unwrap_or_else(|_| line.to_string());
        rendered.push_str(&highlighted);
    }

    rendered
}

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn preview_theme() -> &'static Theme {
    PREVIEW_THEME.get_or_init(|| {
        let mut reader = Cursor::new(GITHUB_DARK_THEME);
        let mut theme = ThemeSet::load_from_reader(&mut reader).unwrap_or_else(|_| {
            let themes = ThemeSet::load_defaults();
            themes.themes.values().next().cloned().unwrap_or_default()
        });
        theme.settings.background = None;
        theme
    })
}

fn format_row(total_width: usize, id: u32, comment: Option<&str>) -> String {
    let id_width = ID_WIDTH.min(total_width);
    let id_text = format!("{id:<id_width$}");
    let comment_width = total_width.saturating_sub(id_width + COLUMN_GAP.len());
    let comment_text = truncate_right(comment.unwrap_or_default(), comment_width);
    format!("{id_text}{COLUMN_GAP}{comment_text}")
}

fn truncate_right(value: &str, width: usize) -> String {
    truncate(value, width, false)
}

fn truncate(value: &str, width: usize, left: bool) -> String {
    if width == 0 {
        return String::new();
    }
    let len = value.chars().count();
    if len <= width {
        return format!("{value:<width$}");
    }
    if width <= 3 {
        return ".".repeat(width);
    }

    let keep = width - 3;
    let trimmed = if left {
        let suffix: String = value
            .chars()
            .rev()
            .take(keep)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("...{suffix}")
    } else {
        format!("{}...", value.chars().take(keep).collect::<String>())
    };

    format!("{trimmed:<width$}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ScriptId;
    use std::path::PathBuf;

    #[test]
    fn preview_starts_with_yaml_metadata() {
        let skill = Skill {
            id: ScriptId(7),
            path: PathBuf::from("scripts/example.py"),
            command: "python {{path}}".to_string(),
            comment: Some("example script".to_string()),
        };

        let preview = render_preview(&skill, b"print('ok')\n");
        let plain = strip_ansi(&preview);

        assert!(plain.contains("# YAML"));
        assert!(plain.contains("id: 7"));
        assert!(plain.contains("path: scripts/example.py"));
        assert!(plain.contains("command: python {{path}}"));
        assert!(plain.contains("comment: example script"));
        assert!(plain.contains("---"));
        assert!(!plain.contains("# FILE: scripts/example.py"));
        assert!(plain.contains("print"));
    }

    #[test]
    fn preview_keeps_final_line_without_trailing_newline() {
        let skill = Skill {
            id: ScriptId(8),
            path: PathBuf::from("scripts/no_newline.py"),
            command: "python {{path}}".to_string(),
            comment: None,
        };

        let preview = render_preview(&skill, b"first\nsecond\nfinal line");
        let plain = strip_ansi(&preview);

        assert!(plain.contains("first"));
        assert!(plain.contains("second"));
        assert!(plain.contains("final line\n"));
    }

    fn strip_ansi(value: &str) -> String {
        let mut stripped = String::new();
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' && chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                stripped.push(ch);
            }
        }
        stripped
    }
}
