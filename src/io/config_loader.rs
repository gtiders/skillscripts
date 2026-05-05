use crate::model::Config;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "skillscripts.yaml";
const SKILLS_DIR_NAME: &str = "skills";

pub(crate) enum InitScope {
    Global,
    Local(PathBuf),
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConfigSnapshot {
    pub(crate) default_config: Config,
    pub(crate) local_config: Option<Config>,
    pub(crate) effective_config: Config,
}

pub(crate) struct ConfigResolver {
    global_config_dir: PathBuf,
    local_config_dir: PathBuf,
}

impl ConfigResolver {
    pub(crate) fn new(local_dir: &Path) -> Self {
        let global_config_dir = global_config_dir();

        Self {
            global_config_dir,
            local_config_dir: local_dir.to_path_buf(),
        }
    }

    pub(crate) fn resolve(&self) -> Result<Config> {
        let global_config =
            self.load_optional_config(&self.global_config_dir.join(CONFIG_FILE_NAME))?;
        let local_config =
            self.load_optional_config(&self.local_config_dir.join(CONFIG_FILE_NAME))?;

        let mut config = match (global_config, local_config) {
            (Some(global), Some(local)) => global.merge(&local),
            (Some(global), None) => global,
            (None, Some(local)) => local,
            (None, None) => Config::default(),
        };

        self.add_default_skills_path(&mut config);

        Ok(config)
    }

    fn load_optional_config(&self, path: &Path) -> Result<Option<Config>> {
        if !path.exists() {
            return Ok(None);
        }

        load_config_file(path).map(Some)
    }

    fn add_default_skills_path(&self, config: &mut Config) {
        let default_skills_path = self.global_config_dir.join("skills");
        let default_skills_path = default_skills_path.to_string_lossy().into_owned();

        if !config.scan_paths.contains(&default_skills_path) {
            config.scan_paths.insert(0, default_skills_path);
        }
    }
}

fn global_config_dir() -> PathBuf {
    home_dir().join(".config").join("skillscripts")
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

fn default_global_config() -> Config {
    Config {
        scan_paths: vec![SKILLS_DIR_NAME.to_string()],
        ..Config::default()
    }
}

fn default_local_config() -> Config {
    Config::default()
}

fn load_config_file(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse YAML {}", path.display()))?;
    absolutize_scan_paths(path.parent().unwrap_or_else(|| Path::new(".")), &mut config);
    Ok(config)
}

fn absolutize_scan_paths(base_dir: &Path, config: &mut Config) {
    let home = home_dir();
    for scan_path in &mut config.scan_paths {
        *scan_path = resolve_scan_path(base_dir, &home, scan_path);
    }
}

fn resolve_scan_path(base_dir: &Path, home: &Path, scan_path: &str) -> String {
    if scan_path == "~" {
        return home.to_string_lossy().into_owned();
    }
    if let Some(rest) = scan_path
        .strip_prefix("~/")
        .or_else(|| scan_path.strip_prefix("~\\"))
    {
        return home.join(rest).to_string_lossy().into_owned();
    }

    let candidate = Path::new(scan_path);
    if candidate.is_relative() {
        return base_dir.join(candidate).to_string_lossy().into_owned();
    }

    scan_path.to_string()
}

pub(crate) fn get_global_config_dir() -> PathBuf {
    global_config_dir()
}

pub(crate) fn init_config(scope: InitScope, force: bool) -> Result<Config> {
    let (config_dir, config) = match scope {
        InitScope::Global => {
            let config_dir = get_global_config_dir();
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("failed to create directory {}", config_dir.display()))?;
            let global_skills_dir = config_dir.join(SKILLS_DIR_NAME);
            let skills_dir_preexisted = global_skills_dir.exists();
            fs::create_dir_all(&global_skills_dir).with_context(|| {
                format!("failed to create skills directory {}", config_dir.display())
            })?;
            if !skills_dir_preexisted {
                seed_global_skills_from_workspace(&global_skills_dir)?;
            }
            (config_dir, default_global_config())
        }
        InitScope::Local(config_dir) => {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("failed to create directory {}", config_dir.display()))?;
            (config_dir, default_local_config())
        }
    };

    let config_path = config_dir.join(CONFIG_FILE_NAME);
    if config_path.exists() && !force {
        bail!(
            "A configuration file already exists at {}\nUse --force to overwrite it.",
            config_path.display()
        );
    }

    let content = serde_yaml::to_string(&config).context("failed to serialize default config")?;
    fs::write(&config_path, content)
        .with_context(|| format!("failed to write config {}", config_path.display()))?;

    Ok(config)
}

fn seed_global_skills_from_workspace(global_skills_dir: &Path) -> Result<()> {
    let Some(source_skills_dir) = find_seed_skills_source()? else {
        return Ok(());
    };

    copy_dir_contents_if_missing(&source_skills_dir, global_skills_dir)
}

fn copy_dir_contents_if_missing(source: &Path, destination: &Path) -> Result<()> {
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", source.display()))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", source_path.display()))?;

        if file_type.is_dir() {
            fs::create_dir_all(&destination_path)
                .with_context(|| format!("failed to create {}", destination_path.display()))?;
            copy_dir_contents_if_missing(&source_path, &destination_path)?;
            continue;
        }

        if file_type.is_file() && !destination_path.exists() {
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn find_seed_skills_source() -> Result<Option<PathBuf>> {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let binary_adjacent = exe_dir.join(SKILLS_DIR_NAME);
        if binary_adjacent.is_dir() {
            return Ok(Some(binary_adjacent));
        }
    }

    let workspace_skills_dir = std::env::current_dir()
        .context("failed to resolve current working directory")?
        .join(SKILLS_DIR_NAME);
    if workspace_skills_dir.is_dir() {
        return Ok(Some(workspace_skills_dir));
    }

    Ok(None)
}

pub(crate) fn resolve_config(local_dir: &Path) -> Result<Config> {
    ConfigResolver::new(local_dir).resolve()
}

pub(crate) fn resolve_config_snapshot(local_dir: &Path) -> Result<ConfigSnapshot> {
    let default_config = Config::default();
    let local_config_path = local_dir.join(CONFIG_FILE_NAME);
    let local_config = if local_config_path.exists() {
        Some(load_config_file(&local_config_path)?)
    } else {
        None
    };
    let effective_config = resolve_config(local_dir)?;

    Ok(ConfigSnapshot {
        default_config,
        local_config,
        effective_config,
    })
}
