use crate::model::{TaskId, display_path};
use crate::services::SkillEngine;
use anyhow::{Result, bail};

use super::shared::perform_scan;

pub(crate) fn run_task(engine: &SkillEngine, task_id: TaskId) -> Result<()> {
    let output = perform_scan(engine)?;

    match output.find_by_task_id(task_id) {
        Some(skill) => println!("{}", display_path(&skill.path)),
        None => bail!("No skill found for task_id {task_id}."),
    }

    Ok(())
}
