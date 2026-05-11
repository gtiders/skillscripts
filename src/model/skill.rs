use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct TaskId(pub(crate) u32);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u32> for TaskId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl FromStr for TaskId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(Self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillHeader {
    #[serde(default)]
    pub(crate) task_id: Option<TaskId>,
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) version: Option<String>,
    #[serde(default)]
    pub(crate) command_template: Option<String>,
    #[serde(default)]
    pub(crate) args: HashMap<String, ArgDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArgDef {
    #[serde(rename = "type")]
    pub(crate) arg_type: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Skill {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) task_id: Option<TaskId>,
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(serialize_with = "serialize_path")]
    pub(crate) path: PathBuf,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    pub(crate) command_template: Option<String>,
    pub(crate) parameters: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) checksum: Option<String>,
}

impl From<(SkillHeader, PathBuf)> for Skill {
    fn from((header, path): (SkillHeader, PathBuf)) -> Self {
        let parameters = (!header.args.is_empty()).then(|| build_parameters_schema(&header.args));

        Self {
            task_id: header.task_id,
            name: header.name,
            description: header.description,
            path,
            tags: header.tags,
            command_template: header.command_template,
            parameters,
            checksum: None,
        }
    }
}

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&display_path(path))
}

fn build_parameters_schema(args: &HashMap<String, ArgDef>) -> serde_yaml::Value {
    let mut sorted_args: Vec<_> = args.iter().collect();
    sorted_args.sort_by_key(|(name, _)| *name);

    let mut properties = serde_yaml::Mapping::new();
    let mut required = Vec::new();

    for (name, definition) in sorted_args {
        let mut prop = serde_yaml::Mapping::new();
        prop.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String(definition.arg_type.clone()),
        );
        prop.insert(
            serde_yaml::Value::String("description".to_string()),
            serde_yaml::Value::String(definition.description.clone()),
        );
        properties.insert(
            serde_yaml::Value::String(name.clone()),
            serde_yaml::Value::Mapping(prop),
        );

        if definition.required {
            required.push(serde_yaml::Value::String(name.clone()));
        }
    }

    let mut schema = serde_yaml::Mapping::new();
    schema.insert(
        serde_yaml::Value::String("type".to_string()),
        serde_yaml::Value::String("object".to_string()),
    );
    schema.insert(
        serde_yaml::Value::String("properties".to_string()),
        serde_yaml::Value::Mapping(properties),
    );
    schema.insert(
        serde_yaml::Value::String("required".to_string()),
        serde_yaml::Value::Sequence(required),
    );

    serde_yaml::Value::Mapping(schema)
}
