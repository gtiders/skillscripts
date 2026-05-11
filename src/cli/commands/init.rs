use crate::services::SkillEngine;
use anyhow::Result;

/// Handle `skillscripts init`.
pub(crate) fn run_init(engine: &SkillEngine, force: bool, local: bool) -> Result<()> {
    let path = if local {
        engine.local_dir().to_path_buf()
    } else {
        SkillEngine::global_config_dir()
    };
    let config = if local {
        engine.init_local_config(force)?
    } else {
        SkillEngine::init_global_config(force)?
    };

    println!("Created {}/skillscripts.yaml", path.display());
    println!("Scan Paths: {:?}", config.scan_paths);
    println!("Max File Size: {}", config.max_file_size);
    println!("Search Limit: {}", config.search_limit);
    println!(
        "Copy to Clipboard on Pick: {}",
        config.copy_to_clipboard_on_pick
    );

    Ok(())
}
