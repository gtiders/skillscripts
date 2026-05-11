use crate::services::SkillEngine;
use anyhow::Result;

/// Handle `skillscripts config`.
pub(crate) fn run_config(engine: &SkillEngine) -> Result<()> {
    let snapshot = engine.resolve_config_snapshot()?;

    println!("=== BUILT-IN DEFAULTS ===");
    println!("{}", serde_yaml::to_string(&snapshot.built_in_defaults)?);

    println!("=== GLOBAL CONFIG FILE ===");
    match snapshot.global_config {
        Some(global) => println!("{}", serde_yaml::to_string(&global)?),
        None => println!("null"),
    }

    println!("=== LOCAL CONFIG (CURRENT DIRECTORY) ===");
    match snapshot.local_config {
        Some(local) => println!("{}", serde_yaml::to_string(&local)?),
        None => println!("null"),
    }

    println!("=== EFFECTIVE CONFIG ===");
    println!("{}", serde_yaml::to_string(&snapshot.effective_config)?);

    Ok(())
}
