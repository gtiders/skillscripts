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
    "Type to filter  |  Enter run  |  Esc cancel  |  Preview: script content";
const COLUMN_GAP: &str = "  ";
const ID_WIDTH: usize = 6;
const MIN_PATH_WIDTH: usize = 16;
const MIN_COMMAND_WIDTH: usize = 20;
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
        Self {
            search_text: format!("{} {} {}", skill.id, path, skill.command),
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
            &display_path(&self.skill.path),
            &self.skill.command,
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
    let syntax_set = syntax_set();
    let syntax = syntax_set
        .find_syntax_for_file(&skill.path)
        .ok()
        .flatten()
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, preview_theme());

    let mut rendered = format!(
        "\x1b[38;2;92;99;112m# {}\n# command: {}\x1b[0m\n\n",
        display_path(&skill.path),
        skill.command
    );

    for line in LinesWithEndings::from(&body) {
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

fn format_row(total_width: usize, id: u32, path: &str, command: &str) -> String {
    let widths = column_widths(total_width);
    let id_text = format!("{id:<width$}", width = widths.id);
    let path_text = truncate_left(path, widths.path);
    let command_text = truncate_right(command, widths.command);
    format!("{id_text}{COLUMN_GAP}{path_text}{COLUMN_GAP}{command_text}")
}

struct ColumnWidths {
    id: usize,
    path: usize,
    command: usize,
}

fn column_widths(total_width: usize) -> ColumnWidths {
    let available = total_width.saturating_sub(COLUMN_GAP.len() * 2);
    if available <= ID_WIDTH + MIN_PATH_WIDTH + MIN_COMMAND_WIDTH {
        let remaining = available.saturating_sub(ID_WIDTH);
        let path = remaining / 2;
        return ColumnWidths {
            id: ID_WIDTH.min(available),
            path,
            command: remaining.saturating_sub(path),
        };
    }

    let content_width = available - ID_WIDTH;
    let command = content_width
        .saturating_sub((content_width * 35 / 100).max(MIN_PATH_WIDTH))
        .max(MIN_COMMAND_WIDTH);

    ColumnWidths {
        id: ID_WIDTH,
        path: content_width.saturating_sub(command),
        command,
    }
}

fn truncate_left(value: &str, width: usize) -> String {
    truncate(value, width, true)
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
