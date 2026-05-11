use crate::cli::output::print_skill_yaml;
use crate::cli::picker::run_skim_picker;
use crate::model::{ParseError, Skill, display_path};
use crate::services::{SkillEngine, SkillIndex};
use anyhow::Result;
use arboard::Clipboard;

pub(crate) fn perform_scan(engine: &SkillEngine) -> Result<SkillIndex> {
    engine.scan()
}

pub(crate) fn report_errors(errors: &[ParseError]) {
    if errors.is_empty() {
        return;
    }

    eprintln!();
    eprintln!("⚠ Parsed files with {} error(s):", errors.len());
    for error in errors {
        eprintln!("  {} - {}", display_path(&error.path), error.reason);
    }
}

pub(crate) fn run_picker(skills: Vec<Skill>, copy_to_clipboard: bool) -> Result<()> {
    match run_skim_picker(skills)? {
        Some(skill) => {
            print_skill_yaml(&skill)?;
            println!("\nSkill Path: {}", display_path(&skill.path));

            if copy_to_clipboard && let Ok(mut cb) = Clipboard::new() {
                let _ = cb.set_text(display_path(&skill.path));
                println!("(Path copied to clipboard)");
            }
        }
        None => eprintln!("No skill selected."),
    }

    Ok(())
}
