mod config;
mod skill;
mod yaml_view;

pub(crate) use config::Config;
pub(crate) use skill::{Skill, SkillHeader, TaskId, display_path};
pub(crate) use yaml_view::SkillYamlView;

use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct ParseError {
    pub(crate) path: PathBuf,
    pub(crate) reason: String,
}

impl ParseError {
    pub(crate) fn new(path: PathBuf, reason: String) -> Self {
        Self { path, reason }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", display_path(&self.path), self.reason)
    }
}

impl std::error::Error for ParseError {}
