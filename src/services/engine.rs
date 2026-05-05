use crate::io::{ConfigSnapshot, scan_files};
use crate::model::{Config, ParseError, Skill};
use crate::services::builder::build_skills;
use crate::services::{config_service::ConfigService, search::fuzzy_search};
use anyhow::Result;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct ScanOutput {
    pub(crate) skills: Vec<Skill>,
    pub(crate) errors: Vec<ParseError>,
}

pub(crate) struct SkillEngine {
    config_service: ConfigService,
    cache: RefCell<Option<ScanOutput>>,
}

impl SkillEngine {
    pub(crate) fn new() -> Self {
        Self {
            config_service: ConfigService::new(),
            cache: RefCell::new(None),
        }
    }

    pub(crate) fn scan(&self, local_dir: &Path) -> Result<ScanOutput> {
        if let Some(cached) = self.cache.borrow().as_ref() {
            return Ok(cached.clone());
        }

        let config = self.config_service.resolve(local_dir)?;
        let files = scan_files(&config)?;

        let (skills, errors) = build_skills(&files, config.report_parse_errors);

        let output = ScanOutput {
            skills,
            errors,
        };

        *self.cache.borrow_mut() = Some(output.clone());
        Ok(output)
    }

    pub(crate) fn init_global_config(&self, force: bool) -> Result<Config> {
        self.config_service.init_global(force)
    }

    pub(crate) fn init_local_config(&self, local_dir: &Path, force: bool) -> Result<Config> {
        self.config_service.init_local(local_dir, force)
    }

    pub(crate) fn global_config_dir(&self) -> PathBuf {
        self.config_service.global_config_dir()
    }

    pub(crate) fn resolve_config_snapshot(&self, local_dir: &Path) -> Result<ConfigSnapshot> {
        self.config_service.resolve_snapshot(local_dir)
    }

    pub(crate) fn resolve_search_limit(
        &self,
        local_dir: &Path,
        requested_limit: Option<usize>,
    ) -> Result<usize> {
        self.config_service
            .resolve_search_limit(local_dir, requested_limit)
    }

    pub(crate) fn search<'a>(&self, skills: &'a [Skill], query: &str) -> Vec<(&'a Skill, i64)> {
        fuzzy_search(skills, query)
    }

    pub(crate) fn copy_to_clipboard_on_pick(&self, local_dir: &Path) -> Result<bool> {
        let config = self.config_service.resolve(local_dir)?;
        Ok(config.copy_to_clipboard_on_pick)
    }

    pub(crate) fn report_parse_errors(&self, local_dir: &Path) -> Result<bool> {
        let config = self.config_service.resolve(local_dir)?;
        Ok(config.report_parse_errors)
    }
}
