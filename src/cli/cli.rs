use crate::cli::commands;
use crate::services::SkillEngine;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skillscripts")]
#[command(about = "Fast script search and skill retrieval CLI")]
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

        #[arg(long, help = "Create the configuration file in the current directory")]
        local: bool,
    },

    #[command(about = "Print current configuration")]
    Config,

    #[command(about = "List all scripts as YAML")]
    List,

    #[command(about = "Interactive TUI selector with preview")]
    Pick,

    #[command(about = "Fuzzy search scripts")]
    Search {
        #[arg(required = true)]
        query: String,

        #[arg(short = 'l', long, help = "Limit the number of results")]
        limit: Option<usize>,
    },
}

pub(crate) fn run() -> Result<()> {
    let engine = SkillEngine::new();

    match Cli::parse().command {
        None => commands::run_default_command(&engine),
        Some(Commands::Init { force, local }) => commands::run_init(&engine, force, local),
        Some(Commands::Config) => commands::run_config(&engine),
        Some(Commands::List) => commands::run_list(&engine),
        Some(Commands::Pick) => commands::run_pick(&engine),
        Some(Commands::Search { query, limit }) => commands::run_search(&engine, &query, limit),
    }
}
