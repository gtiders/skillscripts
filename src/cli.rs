use crate::picker::run_skim_picker;
use crate::registry::{display_path, global_config_dir, init_global_config, load_skills};
use crate::run_command;
use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "sks")]
#[command(about = "Registry-driven script launcher and picker")]
#[command(
    after_help = "Special command:\n  run <id> [args...]  Run a registered script and pass through all remaining args"
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a configuration file")]
    Init {
        #[arg(short = 'f', long, help = "Overwrite the existing configuration file")]
        force: bool,
    },
    #[command(about = "List all registered scripts as YAML")]
    List,
    #[command(about = "Interactive TUI selector with preview")]
    Pick,
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
    }
}

fn run_init(force: bool) -> Result<()> {
    init_global_config(force)?;
    println!("Created {}/sks.yaml", global_config_dir().display());
    println!("This config supports only:");
    println!("- imports");
    println!("- scripts[].id");
    println!("- scripts[].path");
    println!("- scripts[].command");
    println!("- scripts[].comment");
    Ok(())
}

fn run_list() -> Result<()> {
    print_yaml(&load_skills()?)
}

fn run_picker_command() -> Result<()> {
    match run_skim_picker(load_skills()?)? {
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
