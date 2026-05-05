use crate::model::Skill;
use anyhow::{Result, anyhow};
use skim::prelude::*;
use skim::tui::BorderType;
use std::borrow::Cow;
use std::sync::Arc;

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
}

impl PickerItem {
    fn new(skill: Skill) -> Self {
        let search_text = skill.description.clone();

        Self { skill, search_text }
    }
}

impl SkimItem for PickerItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.search_text)
    }

    fn output(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.skill.path)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let yaml = match serde_yaml::to_string(&self.skill) {
            Ok(content) => content,
            Err(error) => {
                return ItemPreview::Text(format!("render failed: {error}\npath: {}", self.skill.path));
            }
        };
        ItemPreview::Text(yaml)
    }
}
