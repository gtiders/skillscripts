use crate::cli::output::print_skill_yaml;
use crate::cli::picker::run_skim_picker;
use crate::model::{ParseError, Skill};
use crate::services::{ScanOutput, SkillEngine};
use anyhow::Result;
use arboard::Clipboard;

pub(crate) fn perform_scan(engine: &SkillEngine) -> Result<ScanOutput> {
    let cwd = std::env::current_dir()?;
    engine.scan(&cwd)
}

pub(crate) fn report_errors(errors: &[ParseError]) {
    if errors.is_empty() {
        return;
    }

    eprintln!();
    eprintln!("⚠ Parsed files with {} error(s):", errors.len());
    for error in errors {
        eprintln!("  {} - {}", error.path, error.reason);
    }
}

pub(crate) fn run_picker(skills: Vec<Skill>, copy_to_clipboard: bool) -> Result<()> {
    match run_skim_picker(skills)? {
        Some(skill) => {
            print_skill_yaml(&skill)?;
            println!("\nSkill Path: {}", skill.path);

            if copy_to_clipboard && let Ok(mut cb) = Clipboard::new() {
                let _ = cb.set_text(&skill.path);
                println!("(Path copied to clipboard)");
            }
        }
        None => eprintln!("No skill selected."),
    }

    Ok(())
}
