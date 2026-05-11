use crate::model::{Skill, display_path};
use anyhow::{Result, anyhow};
use ratatui::text::Line;
use skim::prelude::*;
use skim::tui::BorderType;
use std::borrow::Cow;
use std::sync::Arc;

const SKIM_ONE_DARK_COLORS: &str = concat!(
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

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_KEY: &str = "\x1b[38;2;97;175;239m";
const ANSI_NESTED_KEY: &str = "\x1b[38;2;86;182;194m";
const ANSI_SCHEMA_KEY: &str = "\x1b[38;2;92;99;112m";
const ANSI_TASK_ID_KEY: &str = "\x1b[38;2;229;192;123m";
const ANSI_STRING: &str = "\x1b[38;2;152;195;121m";
const ANSI_NUMBER: &str = "\x1b[38;2;229;192;123m";
const ANSI_BOOL: &str = "\x1b[38;2;209;154;102m";
const ANSI_LIST: &str = "\x1b[38;2;224;108;117m";
const ANSI_COMMENT: &str = "\x1b[38;2;92;99;112m";
const ANSI_PATH: &str = "\x1b[38;2;198;120;221m";
const PICKER_HEADER: &str =
    "Enter accept  •  Esc close  •  Up/Down move  •  Type to filter  •  Preview shows YAML";
const LIST_SEPARATOR: &str = "  ✨  ";

pub(crate) fn run_skim_picker(items: Vec<Skill>) -> Result<Option<Skill>> {
    if items.is_empty() {
        return Ok(None);
    }

    let options = build_options()?;
    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let entries = items
        .into_iter()
        .map(PickerItem::new)
        .map(|item| Arc::new(item) as Arc<dyn SkimItem>)
        .collect::<Vec<_>>();

    tx.send(entries)?;
    drop(tx);

    let output = Skim::run_with(options, Some(rx)).map_err(|error| anyhow!(error.to_string()))?;

    let selected = output
        .selected_items
        .into_iter()
        .next()
        .and_then(|item| item.downcast_item::<PickerItem>().cloned())
        .map(|item| item.skill);

    Ok(selected)
}

fn build_options() -> Result<SkimOptions> {
    Ok(SkimOptionsBuilder::default()
        .no_height(true)
        .border(BorderType::Rounded)
        .highlight_line(true)
        .color(SKIM_ONE_DARK_COLORS)
        .header(PICKER_HEADER.to_string())
        .preview("")
        .multi(false)
        .prompt("🔎 ")
        .preview_window("right:35%:wrap")
        .build()?)
}

#[derive(Clone)]
struct PickerItem {
    skill: Skill,
    search_text: String,
    tags_text: String,
}

impl PickerItem {
    fn new(skill: Skill) -> Self {
        let tags_text = if skill.tags.is_empty() {
            "—".to_string()
        } else {
            skill.tags.join(", ")
        };
        let search_text = format!("{} {} {}", skill.name, tags_text, skill.description);

        Self {
            skill,
            search_text,
            tags_text,
        }
    }
}

impl SkimItem for PickerItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.search_text)
    }

    fn display(&self, context: DisplayContext) -> Line<'_> {
        let _ = context;
        Line::from(format_display_text(
            &self.skill.name,
            &self.tags_text,
            &self.skill.description,
        ))
    }

    fn output(&self) -> Cow<'_, str> {
        Cow::Owned(display_path(&self.skill.path))
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let yaml = match serde_yaml::to_string(&self.skill) {
            Ok(content) => content,
            Err(error) => {
                return ItemPreview::AnsiText(format!(
                    "render failed: {error}\npath: {}",
                    display_path(&self.skill.path)
                ));
            }
        };
        ItemPreview::AnsiText(highlight_yaml(&yaml))
    }
}

fn highlight_yaml(yaml: &str) -> String {
    yaml.lines()
        .map(highlight_yaml_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn highlight_yaml_line(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    if trimmed.is_empty() {
        return line.to_string();
    }

    if let Some(comment) = trimmed.strip_prefix('#') {
        return format!("{indent}{ANSI_COMMENT}#{comment}{ANSI_RESET}");
    }

    if let Some(rest) = trimmed.strip_prefix("- ") {
        return format!(
            "{indent}{ANSI_LIST}- {ANSI_RESET}{}",
            highlight_scalar(rest)
        );
    }

    if let Some((key, value)) = trimmed.split_once(':') {
        let key_color = key_color(key, indent_len);
        let rendered_value = if value.is_empty() {
            String::new()
        } else {
            format!(":{}", highlight_value(key, value))
        };

        return format!("{indent}{key_color}{key}{ANSI_RESET}{rendered_value}");
    }

    format!("{indent}{}", highlight_scalar(trimmed))
}

fn highlight_value(key: &str, value: &str) -> String {
    let leading_len = value.len() - value.trim_start().len();
    let leading = &value[..leading_len];
    let trimmed = value.trim_start();

    let rendered = match key {
        "path" => colorize(trimmed, ANSI_PATH),
        "task_id" => colorize(trimmed, ANSI_NUMBER),
        _ => highlight_scalar(trimmed),
    };

    format!("{leading}{rendered}")
}

fn highlight_scalar(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    if matches!(value, "true" | "false" | "null" | "~") {
        return colorize(value, ANSI_BOOL);
    }

    if value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok() {
        return colorize(value, ANSI_NUMBER);
    }

    if value.starts_with(['"', '\'']) || !value.ends_with(':') {
        return colorize(value, ANSI_STRING);
    }

    value.to_string()
}

fn key_color(key: &str, indent_len: usize) -> &'static str {
    if key == "task_id" {
        return ANSI_TASK_ID_KEY;
    }

    if matches!(key, "type" | "description" | "required" | "properties") {
        return ANSI_SCHEMA_KEY;
    }

    if indent_len >= 4 {
        return ANSI_NESTED_KEY;
    }

    ANSI_KEY
}

fn colorize(text: &str, color: &str) -> String {
    format!("{color}{text}{ANSI_RESET}")
}

fn format_display_text(name: &str, tags: &str, description: &str) -> String {
    format!("{name}{LIST_SEPARATOR}{tags}{LIST_SEPARATOR}{description}")
}
