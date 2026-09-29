use crate::portable_path::agent_skills_dir;
use crate::skill::{CREATE_SKILL, USE_SKILL};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const BUILTIN_SKILLS: &[(&str, &str)] = &[
    ("sks-script-use", USE_SKILL),
    ("sks-script-create", CREATE_SKILL),
];

pub(crate) struct SkillInstall {
    pub(crate) path: PathBuf,
    pub(crate) changed: bool,
}

pub(crate) fn install_agent_skills(force: bool) -> Result<Vec<SkillInstall>> {
    let skills_directory = agent_skills_dir()?;
    BUILTIN_SKILLS
        .iter()
        .map(|(name, content)| install_agent_skill(&skills_directory, name, content, force))
        .collect()
}

fn install_agent_skill(
    skills_directory: &std::path::Path,
    name: &str,
    content: &str,
    force: bool,
) -> Result<SkillInstall> {
    let directory = skills_directory.join(name);
    let path = directory.join("SKILL.md");
    if path.exists() && !force {
        let existing = fs::read_to_string(&path)
            .with_context(|| format!("failed to read skill {}", path.display()))?;
        let updated = migrate_mcp_instructions(&existing);
        if updated != existing {
            fs::write(&path, updated)
                .with_context(|| format!("failed to update skill {}", path.display()))?;
            return Ok(SkillInstall {
                path,
                changed: true,
            });
        }
        return Ok(SkillInstall {
            path,
            changed: false,
        });
    }

    fs::create_dir_all(&directory)
        .with_context(|| format!("failed to create directory {}", directory.display()))?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write skill {}", path.display()))?;
    Ok(SkillInstall {
        path,
        changed: true,
    })
}

fn migrate_mcp_instructions(content: &str) -> String {
    content
        .replace(
            "Search once with the sks MCP `search_scripts` tool",
            "Run `sks search \"<capability query>\"` once",
        )
        .replace(
            "search once with the sks MCP `search_scripts` tool",
            "run `sks search \"<capability query>\"` once",
        )
        .replace(
            "call the sks MCP `search_scripts` tool",
            "run `sks search \"<capability query>\"`",
        )
        .replace(
            "Call the sks MCP `search_scripts` tool",
            "Run `sks search \"<capability query>\"`",
        )
        .replace("sks MCP registry", "sks YAML registry")
        .replace(
            "Read the source resource",
            "Read the file at the result's `path`",
        )
        .replace(
            "Read the script resource",
            "Read the file at the result's `path`",
        )
}
