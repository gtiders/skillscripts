use crate::io::HeaderParser;
use crate::model::{ParseError, Skill};
use rayon::prelude::*;
use std::path::Path;

pub(crate) fn build_skills(files: &[String], report_errors: bool) -> (Vec<Skill>, Vec<ParseError>) {
    let results: Vec<_> = files
        .par_iter()
        .map(|file_path| parse_skill_file(file_path, report_errors))
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

    (skills, errors)
}

enum ScanEntry {
    Skill(Skill),
    Error(ParseError),
    Skipped,
}

fn parse_skill_file(file_path: &str, report_errors: bool) -> ScanEntry {
    match HeaderParser::parse_file(Path::new(file_path)) {
        Ok(Some(header)) => ScanEntry::Skill(Skill::from((header, file_path.to_string()))),
        Ok(None) => ScanEntry::Skipped,
        Err(error) if report_errors => {
            ScanEntry::Error(ParseError::new(file_path.to_string(), error.to_string()))
        }
        Err(_) => ScanEntry::Skipped,
    }
}
