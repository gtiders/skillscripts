use crate::model::Skill;
use anyhow::Result;
use serde::Serialize;

pub(crate) fn print_skill_yaml(skill: &Skill) -> Result<()> {
    let content = serde_yaml::to_string(skill)?;
    print!("{content}");
    Ok(())
}

pub(crate) fn print_yaml<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    let content = serde_yaml::to_string(value)?;
    print!("{content}");
    Ok(())
}
