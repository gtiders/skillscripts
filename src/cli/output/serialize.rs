use crate::model::Skill;
use anyhow::Result;
use serde::Serialize;

pub(crate) fn print_skill_yaml(skill: &Skill) -> Result<()> {
    print_serialized(|| serde_yaml::to_string(skill).map_err(|error| error.to_string()), "YAML")
}

pub(crate) fn print_yaml<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    print_serialized(
        || serde_yaml::to_string(value).map_err(|error| error.to_string()),
        "YAML",
    )
}

fn print_serialized<F>(serialize: F, label: &str) -> Result<()>
where
    F: FnOnce() -> std::result::Result<String, String>,
{
    let content = match serialize() {
        Ok(content) => content,
        Err(error) => {
            println!("Failed to render {label} output. Reason: {error}");
            return Ok(());
        }
    };

    print!("{content}");
    Ok(())
}
