use crate::init::install_agent_skills;
use crate::picker::run_picker;
use crate::registry::{
    display_path, global_config_dir, init_global_config, load_registry, load_skills,
};
use crate::run_command;
use crate::search::search_skills;
use crate::skill;
use crate::theme;
use crate::update;
use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "sks")]
#[command(about = "Registry-driven script launcher and picker")]
#[command(
    after_help = "Special command:\n  run <name> [args...]  Run a registered script and pass through all remaining args"
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize the config and Agent Skills")]
    Init {
        #[arg(short = 'f', long, help = "Overwrite the existing configuration file")]
        force: bool,
    },
    #[command(about = "List all registered scripts as YAML")]
    List,
    #[command(about = "Search registered scripts and print ranked matches as YAML")]
    Search {
        #[arg(help = "Capability or script to search for")]
        query: String,
        #[arg(long = "tag", help = "Optional relevance tag; may be repeated")]
        tags: Vec<String>,
        #[arg(long, default_value_t = 5, help = "Maximum number of matches")]
        limit: usize,
    },
    #[command(about = "List available picker themes as YAML")]
    Themes,
    #[command(about = "Interactive TUI selector with preview")]
    Pick,
    #[command(about = "Print instructions for using or creating sks scripts")]
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
    #[command(about = "Update sks from the latest GitHub release")]
    Update {
        #[arg(long, help = "Only check for an update without installing it")]
        check: bool,
        #[arg(
            long,
            help = "Install latest release even if version comparison is inconclusive"
        )]
        force: bool,
    },
}

#[derive(Subcommand)]
enum SkillCommands {
    #[command(about = "Print how to use registered scripts")]
    Use,
    #[command(about = "Print how to create and register a script")]
    Create,
}

pub(crate) fn run() -> Result<()> {
    let raw_args: Vec<OsString> = std::env::args_os().collect();
    if let Some(invocation) = run_command::parse(&raw_args)? {
        return run_command::execute(invocation);
    }

    match Cli::parse().command {
        None | Some(Commands::Pick) => run_picker_command(),
        Some(Commands::Init { force }) => run_init(force),
        Some(Commands::List) => run_list(),
        Some(Commands::Search { query, tags, limit }) => run_search(&query, &tags, limit),
        Some(Commands::Themes) => print_yaml(&theme::available()),
        Some(Commands::Skill { command }) => match command {
            SkillCommands::Use => skill::print_use(),
            SkillCommands::Create => skill::print_create(),
        },
        Some(Commands::Update { check, force }) => update::run(check, force),
    }
}

fn run_init(force: bool) -> Result<()> {
    let config_path = global_config_dir()?.join("sks.yaml");
    let config_changed = init_global_config(force)?;
    let skills = install_agent_skills(force)?;
    println!(
        "{} {}",
        if config_changed { "Created" } else { "Kept" },
        config_path.display()
    );
    for skill in skills {
        println!(
            "{} {}",
            if skill.changed { "Installed" } else { "Kept" },
            skill.path.display()
        );
    }
    println!("This config supports only:");
    println!("- imports");
    println!("- scripts[].name");
    println!("- scripts[].path");
    println!("- scripts[].command");
    println!("- scripts[].comment");
    println!("- scripts[].tags");
    println!("- picker.theme");
    Ok(())
}

fn run_list() -> Result<()> {
    print_yaml(&load_skills()?)
}

fn run_search(query: &str, tags: &[String], limit: usize) -> Result<()> {
    ensure!(limit > 0, "search limit must be greater than zero");
    ensure!(
        !query.trim().is_empty() || !tags.is_empty(),
        "search query or --tag is required"
    );
    let skills = load_skills()?;
    let matches = search_skills(&skills, Some(query), tags, limit)
        .into_iter()
        .map(|entry| entry.skill)
        .collect::<Vec<_>>();
    print_yaml(&matches)
}

fn run_picker_command() -> Result<()> {
    let registry = load_registry()?;
    match run_picker(registry.skills, registry.picker_theme)? {
        Some(skill) => {
            print_yaml(&skill)?;
            println!("\nScript Path: {}", display_path(&skill.path));
        }
        None => eprintln!("No script selected."),
    }
    Ok(())
}

fn print_yaml<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    print!("{}", serde_yaml::to_string(value)?);
    Ok(())
}
