use crate::services::SkillEngine;
use anyhow::Result;

use super::shared::{perform_scan, report_errors, run_picker};

pub(crate) fn run_default_command(engine: &SkillEngine) -> Result<()> {
    let output = perform_scan(engine)?;
    let copy_to_clipboard = engine.copy_to_clipboard_on_pick();

    run_picker(output.skills, copy_to_clipboard)?;

    if engine.report_parse_errors() {
        report_errors(&output.errors);
    }

    Ok(())
}

pub(crate) fn run_pick(engine: &SkillEngine) -> Result<()> {
    let output = perform_scan(engine)?;
    let copy_to_clipboard = engine.copy_to_clipboard_on_pick();

    run_picker(output.skills, copy_to_clipboard)?;

    if engine.report_parse_errors() {
        report_errors(&output.errors);
    }

    Ok(())
}
