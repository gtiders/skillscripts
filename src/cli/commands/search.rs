use crate::cli::output::print_yaml;
use crate::model::SkillYamlView;
use crate::services::SkillEngine;
use anyhow::Result;

use super::shared::{perform_scan, report_errors};

pub(crate) fn run_search(engine: &SkillEngine, query: &str, limit: Option<usize>) -> Result<()> {
    let output = perform_scan(engine)?;

    let search_limit = engine.resolve_search_limit(limit);
    let results = output.search(query);
    let limited_results: Vec<_> = results.into_iter().take(search_limit).collect();

    let skills: Vec<_> = limited_results
        .iter()
        .map(|(skill, _)| SkillYamlView::from(*skill))
        .collect();
    print_yaml(&skills)?;

    if engine.report_parse_errors() {
        report_errors(&output.errors);
    }

    Ok(())
}
