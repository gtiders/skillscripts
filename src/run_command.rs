use crate::registry::{PATH_PLACEHOLDER, ScriptId, display_path, load_skills};
use anyhow::{Context, Result, anyhow, bail};
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;

pub(crate) struct RunInvocation {
    pub(crate) id: ScriptId,
    pub(crate) args: Vec<String>,
}

pub(crate) fn parse(args: &[OsString]) -> Result<Option<RunInvocation>> {
    let Some(command) = args.get(1).and_then(|value| value.to_str()) else {
        return Ok(None);
    };
    if command != "run" {
        return Ok(None);
    }

    let Some(id_raw) = args.get(2).and_then(|value| value.to_str()) else {
        bail!("Usage: sks run <id> [args...]");
    };

    let id = ScriptId::from_str(id_raw).map_err(|_| anyhow!("Invalid script id: {id_raw}"))?;
    let args = args
        .iter()
        .skip(3)
        .map(|value| value.to_string_lossy().into_owned())
        .collect();

    Ok(Some(RunInvocation { id, args }))
}

pub(crate) fn execute(invocation: RunInvocation) -> Result<()> {
    let skills = load_skills()?;
    let skill = skills
        .iter()
        .find(|skill| skill.id == invocation.id)
        .ok_or_else(|| anyhow!("No script found for id {}.", invocation.id))?;

    let mut parts = split_command(&skill.command)?
        .into_iter()
        .map(|part| part.replace(PATH_PLACEHOLDER, &display_path(&skill.path)))
        .collect::<Vec<_>>();
    if parts.is_empty() {
        bail!(
            "Registered script {} command is empty after parsing.",
            invocation.id
        );
    }

    let program = parts.remove(0);
    println!(
        "Running: {}",
        format_command_line(&program, &parts, &invocation.args)
    );

    let child = Command::new(&program)
        .args(&parts)
        .args(&invocation.args)
        .output()
        .with_context(|| {
            format!(
                "failed to launch command for script {}: {}",
                invocation.id, program
            )
        })?;

    io::stdout().write_all(&child.stdout)?;
    io::stderr().write_all(&child.stderr)?;

    if !child.status.success() {
        bail!(
            "Registered script {} command exited with status {}.",
            invocation.id,
            child.status
        );
    }

    Ok(())
}

fn split_command(command: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        match quote {
            Some(active) if ch == active => quote = None,
            Some(_) if ch == '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            Some(_) => current.push(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            None if matches!(ch, '"' | '\'') => quote = Some(ch),
            None if ch == '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        bail!("Command string contains an unterminated quote.");
    }
    if !current.is_empty() {
        parts.push(current);
    }

    Ok(parts)
}

fn format_command_line(program: &str, args: &[String], extra_args: &[String]) -> String {
    std::iter::once(program)
        .chain(args.iter().map(String::as_str))
        .chain(extra_args.iter().map(String::as_str))
        .map(format_command_part)
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_command_part(part: &str) -> String {
    if part.is_empty() {
        return "\"\"".to_string();
    }

    if part.contains([' ', '\t', '"']) {
        format!("\"{}\"", part.replace('"', "\\\""))
    } else {
        part.to_string()
    }
}
