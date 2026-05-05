use crate::model::Skill;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillYamlView {
    pub(crate) name: String,
    pub(crate) tags: Vec<String>,
    pub(crate) description: String,
    pub(crate) path: String,
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
