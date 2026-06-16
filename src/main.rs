mod cli;
mod picker;
mod registry;
mod run_command;

use anyhow::Result;

fn run() -> Result<()> {
    cli::run()
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Operation failed: {error}");
        for cause in error.chain().skip(1) {
            eprintln!("Caused by: {cause}");
        }
        std::process::exit(1);
    }
}
