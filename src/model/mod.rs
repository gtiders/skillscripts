mod config;
mod skill;
mod yaml_view;

pub(crate) use config::Config;
pub(crate) use skill::{Skill, SkillHeader};
pub(crate) use yaml_view::SkillYamlView;

use std::fmt;

#[derive(Debug, Clone)]
pub(crate) struct ParseError {
    pub(crate) path: String,
    pub(crate) reason: String,
}

impl ParseError {
    pub(crate) fn new(path: String, reason: String) -> Self {
        Self { path, reason }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.path, self.reason)
    }
}

impl std::error::Error for ParseError {}
