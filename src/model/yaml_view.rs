use crate::model::Skill;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillYamlView {
    pub(crate) name: String,
    pub(crate) tags: Vec<String>,
    pub(crate) description: String,
    #[serde(serialize_with = "serialize_path")]
    pub(crate) path: PathBuf,
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&crate::model::display_path(path))
}

impl From<&Skill> for SkillYamlView {
    fn from(skill: &Skill) -> Self {
        Self {
            name: skill.name.clone(),
            tags: skill.tags.clone(),
            description: skill.description.clone(),
            path: skill.path.clone(),
        }
    }
}
