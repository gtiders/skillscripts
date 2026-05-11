use crate::cli::output::print_yaml;
use crate::model::SkillYamlView;
use crate::services::SkillEngine;
use anyhow::Result;

use super::shared::{perform_scan, report_errors};

pub(crate) fn run_list(engine: &SkillEngine) -> Result<()> {
    let output = perform_scan(engine)?;

    let skills: Vec<_> = output.skills.iter().map(SkillYamlView::from).collect();
    print_yaml(&skills)?;

    if engine.report_parse_errors() {
        report_errors(&output.errors);
    }

    Ok(())
}
