use crate::io::HeaderParser;
use crate::model::{ParseError, Skill, TaskId};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

pub(crate) fn build_skills(
    files: &[PathBuf],
    report_errors: bool,
) -> Result<(Vec<Skill>, Vec<ParseError>)> {
    let results: Vec<_> = files
        .par_iter()
        .map(|file_path| parse_skill_file(file_path.as_path(), report_errors))
        .collect();

    let mut skills = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            ScanEntry::Skill(skill) => skills.push(skill),
            ScanEntry::Error(error) => errors.push(error),
            ScanEntry::Skipped => {}
        }
    }

    skills.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.path.cmp(&right.path))
    });

    validate_unique_task_ids(&skills)?;

    Ok((skills, errors))
}

enum ScanEntry {
    Skill(Skill),
    Error(ParseError),
    Skipped,
}

fn parse_skill_file(file_path: &Path, report_errors: bool) -> ScanEntry {
    match HeaderParser::parse_file(file_path) {
        Ok(Some(header)) => ScanEntry::Skill(Skill::from((header, file_path.to_path_buf()))),
        Err(error) if report_errors => {
            ScanEntry::Error(ParseError::new(file_path.to_path_buf(), error.to_string()))
        }
        Ok(None) | Err(_) => ScanEntry::Skipped,
    }
}

fn validate_unique_task_ids(skills: &[Skill]) -> Result<()> {
    let mut seen: HashMap<TaskId, &Path> = HashMap::new();

    for skill in skills {
        let Some(task_id) = skill.task_id else {
            continue;
        };

        if let Some(existing_path) = seen.insert(task_id, skill.path.as_path()) {
            return Err(
                DuplicateTaskIdError::new(task_id, existing_path, skill.path.as_path()).into(),
            );
        }
    }

    Ok(())
}

#[derive(Debug)]
struct DuplicateTaskIdError {
    task_id: TaskId,
    first_path: PathBuf,
    second_path: PathBuf,
}

impl DuplicateTaskIdError {
    fn new(task_id: TaskId, first_path: &Path, second_path: &Path) -> Self {
        Self {
            task_id,
            first_path: first_path.to_path_buf(),
            second_path: second_path.to_path_buf(),
        }
    }
}

impl fmt::Display for DuplicateTaskIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Duplicate task_id {} found in {} and {}",
            self.task_id,
            crate::model::display_path(&self.first_path),
            crate::model::display_path(&self.second_path)
        )
    }
}

impl Error for DuplicateTaskIdError {}
