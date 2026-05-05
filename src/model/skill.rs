use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillHeader {
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
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    pub(crate) command_template: Option<String>,
    pub(crate) parameters: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) checksum: Option<String>,
}

impl From<(SkillHeader, String)> for Skill {
    fn from((header, path): (SkillHeader, String)) -> Self {
        let parameters = (!header.args.is_empty()).then(|| build_parameters_schema(&header.args));

        Self {
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
