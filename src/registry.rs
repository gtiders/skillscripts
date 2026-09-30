use crate::portable_path;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const CONFIG_FILE_NAME: &str = "sks.yaml";
pub(crate) const PATH_PLACEHOLDER: &str = "{{path}}";

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct ScriptName(String);

impl fmt::Display for ScriptName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ScriptName {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ScriptName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match chars.next() {
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
            _ => return Err(format!("invalid script name `{s}`")),
        }
        if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
            return Err(format!("invalid script name `{s}`"));
        }
        Ok(Self(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for ScriptName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) picker: Option<PickerConfig>,
    #[serde(default)]
    pub(crate) imports: Vec<String>,
    #[serde(default)]
    pub(crate) scripts: Vec<ScriptRegistration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PickerConfig {
    pub(crate) theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScriptRegistration {
    pub(crate) name: ScriptName,
    pub(crate) path: String,
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) comment: Option<String>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Skill {
    pub(crate) name: ScriptName,
    #[serde(serialize_with = "serialize_path")]
    pub(crate) path: PathBuf,
    /// Path text as written in the registry. Search ranks on this instead of `path`,
    /// so unrelated absolute prefixes cannot create or weaken matches.
    #[serde(skip_serializing)]
    pub(crate) registered_path: String,
    pub(crate) command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConfigSource {
    path: PathBuf,
    config: ConfigFile,
}

pub(crate) struct LoadedRegistry {
    pub(crate) skills: Vec<Skill>,
    pub(crate) picker_theme: &'static chromata::Theme,
}

#[derive(Debug, Clone, Copy)]
struct PathResolver<'a> {
    base_dir: &'a Path,
}

pub(crate) fn global_config_dir() -> Result<PathBuf> {
    portable_path::config_dir()
}

pub(crate) fn global_config_path() -> Result<PathBuf> {
    Ok(global_config_dir()?.join(CONFIG_FILE_NAME))
}

pub(crate) fn init_global_config(force: bool) -> Result<bool> {
    let config_dir = global_config_dir()?;
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("failed to create directory {}", config_dir.display()))?;

    let config_path = config_dir.join(CONFIG_FILE_NAME);
    let changed = !config_path.exists() || force;
    if changed {
        let content = serde_yaml::to_string(&default_global_config())
            .context("failed to serialize default config")?;
        fs::write(&config_path, content)
            .with_context(|| format!("failed to write config {}", config_path.display()))?;
    }

    let scripts_path = config_dir.join("scripts.yaml");
    if !scripts_path.exists() {
        fs::write(&scripts_path, "scripts: []\n")
            .with_context(|| format!("failed to write config {}", scripts_path.display()))?;
    }

    Ok(changed)
}

pub(crate) fn load_skills() -> Result<Vec<Skill>> {
    Ok(load_registry()?.skills)
}

pub(crate) fn load_registry() -> Result<LoadedRegistry> {
    let sources = load_config_sources()?;
    let picker_theme = crate::theme::configured(
        sources[0]
            .config
            .picker
            .as_ref()
            .map(|picker| picker.theme.as_str()),
    )?;
    let mut skills = build_skills(&sources)?;
    skills.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.path.cmp(&right.path))
    });
    validate_unique_names(&skills)?;
    Ok(LoadedRegistry {
        skills,
        picker_theme,
    })
}

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn load_config_sources() -> Result<Vec<ConfigSource>> {
    let global_path = global_config_path()?;
    let global = load_global_config_source(global_path)?;
    let global_base_dir = parent_dir(&global.path)?;
    let imports = global.config.imports.clone();
    let resolver = PathResolver {
        base_dir: global_base_dir.as_path(),
    };
    let mut sources = vec![global];

    for import in &imports {
        let import_path = resolver.resolve(import, "import")?;
        sources.push(load_imported_config_source(import_path)?);
    }

    Ok(sources)
}

fn load_global_config_source(path: PathBuf) -> Result<ConfigSource> {
    if !path.exists() {
        bail!(
            "Global config not found at {}. Run `sks init` first.",
            path.display()
        );
    }

    Ok(ConfigSource {
        config: load_config_file(&path)?,
        path,
    })
}

fn load_imported_config_source(path: PathBuf) -> Result<ConfigSource> {
    let config = load_config_file(&path)?;
    if !config.imports.is_empty() {
        bail!("Imported config {} cannot declare imports.", path.display());
    }
    if config.picker.is_some() {
        bail!(
            "Imported config {} cannot declare picker options.",
            path.display()
        );
    }

    Ok(ConfigSource { path, config })
}

fn build_skills(sources: &[ConfigSource]) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();
    for source in sources {
        skills.extend(build_skills_from_source(source)?);
    }
    Ok(skills)
}

fn build_skills_from_source(source: &ConfigSource) -> Result<Vec<Skill>> {
    let base_dir = parent_dir(&source.path)?;
    let resolver = PathResolver {
        base_dir: base_dir.as_path(),
    };

    source
        .config
        .scripts
        .iter()
        .map(|entry| build_skill(entry, &source.path, resolver))
        .collect()
}

fn build_skill(
    entry: &ScriptRegistration,
    config_path: &Path,
    resolver: PathResolver<'_>,
) -> Result<Skill> {
    let path = resolver
        .resolve(&entry.path, "script path")
        .with_context(|| format!("in {}", config_path.display()))?;

    if !path.is_file() {
        bail!(
            "Registered script {} points to a missing file: {}",
            entry.name,
            path.display()
        );
    }

    if entry.command.trim().is_empty() {
        bail!("Registered script {} has an empty command.", entry.name);
    }
    if !entry.command.contains(PATH_PLACEHOLDER) {
        bail!(
            "Registered script {} command must contain {}.",
            entry.name,
            PATH_PLACEHOLDER
        );
    }

    Ok(Skill {
        name: entry.name.clone(),
        path,
        registered_path: entry.path.clone(),
        command: entry.command.clone(),
        comment: entry.comment.clone(),
        tags: normalize_tags(&entry.name, &entry.tags)?,
    })
}

fn load_config_file(path: &Path) -> Result<ConfigFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse YAML {}", path.display()))
}

impl PathResolver<'_> {
    fn resolve(self, value: &str, label: &str) -> Result<PathBuf> {
        let joined = portable_path::resolve_unix_relative(self.base_dir, value, label)?;
        if joined.exists() {
            return dunce::canonicalize(&joined)
                .with_context(|| format!("failed to canonicalize {}", joined.display()));
        }

        Ok(joined)
    }
}

fn normalize_tags(name: &ScriptName, tags: &[String]) -> Result<Vec<String>> {
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            bail!("Registered script {name} contains an empty tag.");
        }
        if !normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(tag))
        {
            normalized.push(tag.to_string());
        }
    }
    Ok(normalized)
}

fn validate_unique_names(skills: &[Skill]) -> Result<()> {
    let mut seen: HashMap<&ScriptName, &Path> = HashMap::new();
    for skill in skills {
        if let Some(existing) = seen.insert(&skill.name, skill.path.as_path()) {
            bail!(
                "Duplicate name {} found in {} and {}",
                skill.name,
                display_path(existing),
                display_path(&skill.path)
            );
        }
    }
    Ok(())
}

fn parent_dir(path: &Path) -> Result<PathBuf> {
    path.parent()
        .map(Path::to_path_buf)
        .context("config file must have a parent directory")
}

fn default_global_config() -> ConfigFile {
    ConfigFile {
        picker: Some(PickerConfig {
            theme: crate::theme::DEFAULT_NAME.to_string(),
        }),
        imports: vec!["scripts.yaml".to_string()],
        scripts: Vec::new(),
    }
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&display_path(path))
}

#[cfg(test)]
mod tests {
    use super::ScriptName;
    use std::str::FromStr;

    #[test]
    fn accepts_python_ascii_identifier_names() {
        for value in ["foo", "_internal", "ConvertCSV2"] {
            assert!(ScriptName::from_str(value).is_ok());
        }
    }

    #[test]
    fn rejects_invalid_script_names() {
        for value in [
            "",
            "123script",
            "foo-bar",
            "foo.bar",
            "foo/bar",
            "foo bar",
            "中文",
        ] {
            assert!(ScriptName::from_str(value).is_err(), "{value}");
        }
    }
}
