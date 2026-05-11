use crate::io::{
    ConfigSnapshot, InitScope, get_global_config_dir, init_config, resolve_config,
    resolve_config_snapshot, scan_files,
};
use crate::model::{Config, ParseError, Skill, TaskId};
use crate::services::builder::build_skills;
use crate::services::search::fuzzy_search;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct SkillIndex {
    pub(crate) skills: Vec<Skill>,
    pub(crate) errors: Vec<ParseError>,
}

pub(crate) struct SkillEngine {
    local_dir: PathBuf,
    config: Config,
}

impl SkillEngine {
    pub(crate) fn from_current_dir() -> Result<Self> {
        let local_dir = std::env::current_dir()?;
        Self::for_dir(local_dir)
    }

    pub(crate) fn for_dir(local_dir: PathBuf) -> Result<Self> {
        let config = resolve_config(&local_dir)?;
        Ok(Self { local_dir, config })
    }

    pub(crate) fn local_dir(&self) -> &Path {
        &self.local_dir
    }

    pub(crate) fn scan(&self) -> Result<SkillIndex> {
        let files = scan_files(&self.config)?;

        let (skills, errors) = build_skills(&files, self.config.report_parse_errors)?;

        Ok(SkillIndex { skills, errors })
    }

    pub(crate) fn init_global_config(force: bool) -> Result<Config> {
        init_config(InitScope::Global, force)
    }

    pub(crate) fn init_local_config(&self, force: bool) -> Result<Config> {
        init_config(InitScope::Local(self.local_dir.clone()), force)
    }

    pub(crate) fn global_config_dir() -> PathBuf {
        get_global_config_dir()
    }

    pub(crate) fn resolve_config_snapshot(&self) -> Result<ConfigSnapshot> {
        resolve_config_snapshot(&self.local_dir)
    }

    pub(crate) fn resolve_search_limit(&self, requested_limit: Option<usize>) -> usize {
        requested_limit.unwrap_or(self.config.search_limit)
    }

    pub(crate) fn copy_to_clipboard_on_pick(&self) -> bool {
        self.config.copy_to_clipboard_on_pick
    }

    pub(crate) fn report_parse_errors(&self) -> bool {
        self.config.report_parse_errors
    }
}

impl SkillIndex {
    pub(crate) fn search(&self, query: &str) -> Vec<(&Skill, i64)> {
        fuzzy_search(&self.skills, query)
    }

    pub(crate) fn find_by_task_id(&self, task_id: TaskId) -> Option<&Skill> {
        self.skills
            .iter()
            .find(|skill| skill.task_id == Some(task_id))
    }
}
