use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct TestEnv {
    root: TempDir,
    config_root: PathBuf,
    home: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("failed to create temp dir");
        let home = root.path().join("home");
        let cache_dir = home.join(".cache");
        let config_root = home.join(".config");

        fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        fs::create_dir_all(&config_root).expect("failed to create config dir");
        fs::create_dir_all(&home).expect("failed to create home dir");

        Self {
            root,
            config_root,
            home,
        }
    }

    fn root(&self) -> &Path {
        self.root.path()
    }

    fn global_config_dir(&self) -> PathBuf {
        self.config_root.join("sks")
    }

    fn global_config_file(&self) -> PathBuf {
        self.global_config_dir().join("sks.yaml")
    }

    fn imported_scripts_file(&self) -> PathBuf {
        self.global_config_dir().join("scripts.yaml")
    }

    fn installed_skill_file(&self, name: &str) -> PathBuf {
        self.home
            .join(".agents")
            .join("skills")
            .join(name)
            .join("SKILL.md")
    }

    fn write_global_config(&self, content: &str) {
        fs::create_dir_all(self.global_config_dir()).expect("failed to create global config dir");
        fs::write(self.global_config_file(), content).expect("failed to write global config");
    }

    fn command(&self, workspace: &Path) -> Command {
        let mut cmd = Command::cargo_bin("sks").expect("binary should build");
        cmd.current_dir(workspace);
        cmd.env("HOME", &self.home);
        cmd.env("USERPROFILE", &self.home);
        cmd.env("LANG", "en_US.UTF-8");
        cmd.env("LC_ALL", "en_US.UTF-8");
        cmd
    }
}

#[test]
fn init_creates_global_config() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"))
        .stdout(predicate::str::contains("imports"))
        .stdout(predicate::str::contains("scripts[].command"));

    let yaml = fs::read_to_string(env.global_config_file()).expect("failed to read config");
    let config: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("generated config should be valid YAML");

    assert_eq!(config["imports"][0].as_str(), Some("scripts.yaml"));
    assert_eq!(
        config["picker"]["theme"].as_str(),
        Some("Catppuccin Frappe")
    );
    assert!(config["scripts"].is_sequence());
    assert_eq!(
        fs::read_to_string(env.imported_scripts_file()).unwrap(),
        "scripts: []\n"
    );
    let create_skill = fs::read_to_string(env.installed_skill_file("sks-script-create"))
        .expect("authoring skill should be installed");
    assert!(create_skill.contains("name: sks-script-create"));
    assert!(create_skill.contains("sks run <name> [args...]"));
    let use_skill = fs::read_to_string(env.installed_skill_file("sks-script-use"))
        .expect("discovery skill should be installed");
    assert!(use_skill.contains("name: sks-script-use"));
    assert!(use_skill.contains("every request to use a script"));
    env.command(&workspace).arg("list").assert().success();
}

#[test]
fn themes_lists_library_presets_without_config() {
    let env = TestEnv::new();
    let output = env.command(env.root()).arg("themes").output().unwrap();
    assert!(output.status.success());
    let themes: serde_yaml::Value = serde_yaml::from_slice(&output.stdout).unwrap();
    let names: Vec<&str> = themes
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|entry| entry["name"].as_str())
        .collect();
    assert!(names.contains(&"Github Dark"));
    assert!(names.contains(&"Dark Plus"));
    assert!(names.contains(&"Catppuccin Latte"));
    assert!(names.contains(&"Catppuccin Frappe"));
}

#[test]
fn environment_cannot_override_picker_theme() {
    let env = TestEnv::new();
    env.write_global_config("picker:\n  theme: Catppuccin Frappe\nscripts: []\n");
    env.command(env.root())
        .arg("pick")
        .env("SKS_THEME", "not a theme")
        .assert()
        .success()
        .stderr(predicate::str::contains("No script selected."));
}

#[test]
fn picker_theme_config_accepts_library_name_and_rejects_unknown_name() {
    let env = TestEnv::new();
    env.write_global_config("picker:\n  theme: Dark Plus\nscripts: []\n");
    env.command(env.root()).arg("list").assert().success();
    env.write_global_config("picker:\n  theme: missing theme\nscripts: []\n");
    env.command(env.root())
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("run `sks themes`"));
}

#[test]
fn imported_config_cannot_override_picker_theme() {
    let env = TestEnv::new();
    env.write_global_config("imports: [scripts.yaml]\n");
    fs::write(
        env.imported_scripts_file(),
        "picker:\n  theme: Dark Plus\nscripts: []\n",
    )
    .unwrap();
    env.command(env.root())
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot declare picker options"));
}

#[test]
fn init_keeps_existing_config_and_installs_missing_skill() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-exists");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config("scripts: []\n");

    env.command(&workspace)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Kept"))
        .stdout(predicate::str::contains("Installed"));
    assert_eq!(
        fs::read_to_string(env.global_config_file()).unwrap(),
        "scripts: []\n"
    );
    assert!(env.installed_skill_file("sks-script-create").is_file());
    assert!(env.installed_skill_file("sks-script-use").is_file());
}

#[test]
fn init_updates_old_mcp_skill_guidance_without_replacing_custom_content() {
    let env = TestEnv::new();
    env.write_global_config("scripts: []\n");
    let path = env.installed_skill_file("sks-script-use");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "Search once with the sks MCP `search_scripts` tool.\nRead the source resource.\nCustom guidance stays.\n",
    )
    .unwrap();

    env.command(env.root()).arg("init").assert().success();
    let updated = fs::read_to_string(path).unwrap();
    assert!(updated.contains("sks search"));
    assert!(updated.contains("result's `path`"));
    assert!(updated.contains("Custom guidance stays."));
    assert!(!updated.contains("MCP"));
}

#[test]
fn init_force_overwrites_existing_config() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-force");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config("scripts: []\n");

    env.command(&workspace)
        .args(["init", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let yaml = fs::read_to_string(env.global_config_file()).expect("failed to read config");
    let config: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("overwritten config should be valid YAML");

    assert_eq!(config["imports"][0].as_str(), Some("scripts.yaml"));
}

#[test]
fn init_force_does_not_overwrite_the_script_registry() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-force-registry");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.command(&workspace).arg("init").assert().success();
    fs::write(env.imported_scripts_file(), "scripts:\n  # keep me\n").unwrap();

    env.command(&workspace)
        .args(["init", "--force"])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(env.imported_scripts_file()).unwrap(),
        "scripts:\n  # keep me\n"
    );
}

#[test]
fn init_uses_userprofile_dot_directories_when_home_is_unset() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-userprofile");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .env_remove("HOME")
        .arg("init")
        .assert()
        .success();

    assert!(env.global_config_file().is_file());
    assert!(env.installed_skill_file("sks-script-create").is_file());
    assert!(env.installed_skill_file("sks-script-use").is_file());
    assert!(!env.home.join("AppData").exists());
}

#[test]
fn list_outputs_registered_script_array() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-list");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    fs::write(scripts_dir.join("alpha.py"), "print('alpha')\n").expect("failed to write alpha");
    fs::write(scripts_dir.join("beta.py"), "print('beta')\n").expect("failed to write beta");
    env.write_global_config(
        r"
scripts:
  - name: script_101
    path: scripts/alpha.py
    command: python {{path}}
  - name: script_102
    path: scripts/beta.py
    command: python {{path}}
",
    );

    let assert = env
        .command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("- name:"));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("list should emit a valid YAML array");

    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0]["name"].as_str(), Some("script_101"));
    assert_eq!(skills[0]["command"].as_str(), Some("python {{path}}"));
    assert!(
        skills[0]["path"]
            .as_str()
            .expect("script path should be a string")
            .ends_with("scripts/alpha.py")
    );
}

#[test]
fn list_preserves_registered_script_comment() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-list-comment");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    fs::write(scripts_dir.join("commented.py"), "print('commented')\n")
        .expect("failed to write script");
    env.write_global_config(
        r"
scripts:
  - name: script_103
    path: scripts/commented.py
    command: python {{path}}
    comment: Run the commented test script
",
    );

    let assert = env.command(&workspace).arg("list").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("list should emit a valid YAML array");

    assert_eq!(
        skills[0]["comment"].as_str(),
        Some("Run the commented test script")
    );
}

#[test]
fn list_normalizes_optional_tags_without_breaking_old_entries() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-list-tags");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");
    fs::write(scripts_dir.join("tagged.py"), "print('tagged')\n").unwrap();
    fs::write(scripts_dir.join("legacy.py"), "print('legacy')\n").unwrap();
    env.write_global_config(
        r"
scripts:
  - name: script_110
    path: scripts/tagged.py
    command: python {{path}}
    tags: [' PDF ', document, pdf]
  - name: script_111
    path: scripts/legacy.py
    command: python {{path}}
",
    );

    let assert = env.command(&workspace).arg("list").assert().success();
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(skills[0]["tags"][0].as_str(), Some("PDF"));
    assert_eq!(skills[0]["tags"][1].as_str(), Some("document"));
    assert_eq!(skills[0]["tags"].as_sequence().unwrap().len(), 2);
    assert!(skills[1].get("tags").is_none());
}

#[test]
fn list_reads_registered_scripts_immediately() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-live-list");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    fs::write(scripts_dir.join("hello.py"), "print('hello')\n").expect("failed to write script");
    env.write_global_config(
        r"
scripts:
  - name: script_1
    path: scripts/hello.py
    command: python {{path}}
",
    );

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello.py"));
}

#[test]
fn list_detects_new_imported_scripts_without_cache() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-import-refresh");
    let imported_dir = env.global_config_dir().join("imports");
    let scripts_dir = imported_dir.join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    env.write_global_config(
        r"
imports:
  - imports/scripts.yaml
",
    );

    fs::write(scripts_dir.join("one.py"), "print('one')\n").expect("failed to write first");
    fs::write(
        imported_dir.join("scripts.yaml"),
        r"
scripts:
  - name: script_1
    path: scripts/one.py
    command: python {{path}}
",
    )
    .expect("failed to write imported config");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("one.py"));

    fs::write(scripts_dir.join("two.py"), "print('two')\n").expect("failed to write second");
    fs::write(
        imported_dir.join("scripts.yaml"),
        r"
scripts:
  - name: script_1
    path: scripts/one.py
    command: python {{path}}
  - name: script_2
    path: scripts/two.py
    command: python {{path}}
",
    )
    .expect("failed to update imported config");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("one.py"))
        .stdout(predicate::str::contains("two.py"));
}

#[test]
fn list_normalizes_registered_script_paths() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-normalized-script-path");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    fs::write(scripts_dir.join("normalized.py"), "print('normalized')\n")
        .expect("failed to write script");
    env.write_global_config(
        r"
scripts:
  - name: script_301
    path: scripts/../scripts/normalized.py
    command: python {{path}}
",
    );

    let assert = env.command(&workspace).arg("list").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("list should emit valid YAML");

    let path = skills[0]["path"]
        .as_str()
        .expect("script path should be a string");
    assert!(path.ends_with("scripts/normalized.py"));
    assert!(!path.contains("/../"));
    assert!(!path.contains("\\..\\"));
}

#[test]
fn default_command_handles_empty_registry() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-default");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config("scripts: []\n");

    env.command(&workspace)
        .assert()
        .success()
        .stderr(predicate::str::contains("No script selected"));
}

#[test]
fn pick_uses_registered_scripts_without_scan_headers() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-pick");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    fs::write(scripts_dir.join("echo.py"), "print('echo')\n").expect("failed to write script");
    env.write_global_config(
        r"
scripts:
  - name: script_1
    path: scripts/echo.py
    command: python {{path}} echo
",
    );

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("echo.py"));
}

#[test]
fn reports_missing_global_config_with_init_hint() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-missing-config");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Global config not found"))
        .stderr(predicate::str::contains("sks init"));
}

#[test]
fn list_rejects_missing_registered_script_files() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-missing-script");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config(
        r"
scripts:
  - name: script_1
    path: scripts/missing.py
    command: python {{path}}
",
    );

    env.command(&workspace)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("points to a missing file"));
}

#[test]
fn list_rejects_absolute_import_paths() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-absolute-import");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config(
        r"
imports:
  - C:/absolute/scripts.yaml
",
    );

    env.command(&workspace)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "import must be a relative Unix-style path",
        ));
}

#[test]
fn list_rejects_imported_configs_with_nested_imports() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-nested-import");
    let config_dir = env.global_config_dir();
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(config_dir.join("nested")).expect("failed to create nested config dir");
    env.write_global_config(
        r"
imports:
  - nested/python.yaml
",
    );

    fs::write(
        config_dir.join("nested").join("python.yaml"),
        r"
imports:
  - nope.yaml
scripts: []
",
    )
    .expect("failed to write imported config");

    env.command(&workspace)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot declare imports"));
}

#[test]
fn list_resolves_cleaned_import_paths() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-clean-import");
    let imported_dir = env.global_config_dir().join("imports");
    let scripts_dir = imported_dir.join("tools");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create tools dir");

    env.write_global_config(
        r"
imports:
  - ./imports/../imports/scripts.yaml
",
    );

    fs::write(scripts_dir.join("clean.py"), "print('clean')\n").expect("failed to write script");
    fs::write(
        imported_dir.join("scripts.yaml"),
        r"
scripts:
  - name: script_401
    path: ./tools/../tools/clean.py
    command: python {{path}}
",
    )
    .expect("failed to write imported config");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("clean.py"));
}

#[test]
fn run_replaces_path_placeholder_and_appends_extra_args() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-run");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    #[cfg(unix)]
    let (script_name, script_body, command) = (
        "echo_args.sh",
        "printf 'PATH=%s\\n' \"$1\"\nshift\nfor arg in \"$@\"; do printf 'ARG=%s\\n' \"$arg\"; done\n",
        "sh {{path}} \"{{path}}\"",
    );
    #[cfg(windows)]
    let (script_name, script_body, command) = (
        "echo_args.cmd",
        "@echo off\necho PATH=%1\nshift\n:loop\nif \"%1\"==\"\" goto end\necho ARG=%1\nshift\ngoto loop\n:end\n",
        "cmd /C {{path}} \"{{path}}\"",
    );
    fs::write(scripts_dir.join(script_name), script_body).expect("failed to write script");

    env.write_global_config(&format!(
        r#"
scripts:
  - name: script_501
    path: scripts/{script_name}
    command: {command}
"#,
    ));

    env.command(&workspace)
        .args(["run", "script_501", "one", "--flag", "three"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Running:"))
        .stdout(predicate::str::contains(script_name))
        .stdout(predicate::str::contains("ARG=one"))
        .stdout(predicate::str::contains("ARG=--flag"))
        .stdout(predicate::str::contains("ARG=three"));
    assert!(workspace.join(".sks").join(script_name).is_file());
}

#[test]
fn skill_commands_print_the_embedded_guides() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-skill");
    fs::create_dir_all(&workspace).unwrap();

    env.command(&workspace)
        .args(["skill", "use"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: sks-script-use"))
        .stdout(predicate::str::contains("sks search"));
    env.command(&workspace)
        .args(["skill", "create"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: sks-script-create"))
        .stdout(predicate::str::contains("sks search"));
}

#[test]
fn run_preserves_unquoted_placeholder_path_with_spaces() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-run-path-spaces");
    let scripts_dir = env.global_config_dir().join("scripts with spaces");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    #[cfg(unix)]
    let (script_name, script_body, command) = (
        "echo path.sh",
        "printf 'PATH=%s\\n' \"$1\"\n",
        "sh {{path}} {{path}}",
    );
    #[cfg(windows)]
    let (script_name, script_body, command) = (
        "echo path.cmd",
        "@echo off\necho PATH=%1\n",
        "cmd /C {{path}} {{path}}",
    );
    fs::write(scripts_dir.join(script_name), script_body).expect("failed to write script");

    env.write_global_config(&format!(
        r"
scripts:
  - name: script_503
    path: scripts with spaces/{script_name}
    command: {command}
",
    ));

    env.command(&workspace)
        .args(["run", "script_503"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scripts with spaces"))
        .stdout(predicate::str::contains("PATH="))
        .stdout(predicate::str::contains(script_name));
}

#[test]
fn run_requires_path_placeholder() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-run-missing-placeholder");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    fs::write(scripts_dir.join("echo.ps1"), "Write-Output 'ok'\n").expect("failed to write script");
    env.write_global_config(
        r"
scripts:
  - name: script_502
    path: scripts/echo.ps1
    command: powershell -NoProfile -ExecutionPolicy Bypass -File
",
    );

    env.command(&workspace)
        .args(["run", "script_502"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must contain {{path}}"));
}

#[test]
fn run_reports_missing_id() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-run-missing-name");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config("scripts: []\n");

    env.command(&workspace)
        .args(["run", "missing_script"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No script found for name missing_script.",
        ));
}

#[test]
fn run_reports_usage_without_id() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-run-usage");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config("scripts: []\n");

    env.command(&workspace)
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: sks run <name> [args...]"));
}

#[test]
fn search_returns_ranked_yaml_with_source_paths() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-search");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&scripts_dir).unwrap();
    fs::write(scripts_dir.join("markdown_pdf.py"), "print('pdf')\n").unwrap();
    fs::write(scripts_dir.join("notes.py"), "print('notes')\n").unwrap();
    env.write_global_config(
        r"
scripts:
  - { name: markdown_pdf, path: scripts/markdown_pdf.py, command: 'python {{path}}', comment: Convert Markdown to PDF, tags: [markdown, pdf] }
  - { name: notes, path: scripts/notes.py, command: 'python {{path}}', comment: Manage notes, tags: [text] }
",
    );

    let output = env
        .command(&workspace)
        .args(["search", "markdown pdf"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let matches: Vec<serde_yaml::Value> = serde_yaml::from_slice(&output.stdout).unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["name"].as_str(), Some("markdown_pdf"));
    assert!(
        matches[0]["path"]
            .as_str()
            .unwrap()
            .ends_with("scripts/markdown_pdf.py")
    );
    assert_eq!(matches[0]["command"].as_str(), Some("python {{path}}"));
    assert_eq!(matches[0]["tags"][0].as_str(), Some("markdown"));

    let output = env
        .command(&workspace)
        .args(["search", "nonexistent xyz"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let empty: Vec<serde_yaml::Value> = serde_yaml::from_slice(&output.stdout).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn search_limit_and_tags_rank_matches() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-search-limit");
    let scripts_dir = env.global_config_dir().join("scripts");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&scripts_dir).unwrap();
    for id in 1..=4 {
        fs::write(scripts_dir.join(format!("pdf-{id}.py")), "print('pdf')\n").unwrap();
    }
    env.write_global_config(
        r"
scripts:
  - { name: script_1, path: scripts/pdf-1.py, command: 'python {{path}}', comment: Create PDF, tags: [document] }
  - { name: script_2, path: scripts/pdf-2.py, command: 'python {{path}}', comment: Create PDF, tags: [image] }
  - { name: script_3, path: scripts/pdf-3.py, command: 'python {{path}}', comment: Create PDF }
  - { name: script_4, path: scripts/pdf-4.py, command: 'python {{path}}', comment: Create PDF }
",
    );

    let output = env
        .command(&workspace)
        .args(["search", "pdf", "--tag", "image", "--limit", "2"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let matches: Vec<serde_yaml::Value> = serde_yaml::from_slice(&output.stdout).unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0]["name"].as_str(), Some("script_2"));
    env.command(&workspace)
        .args(["search", "pdf", "--limit", "0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "search limit must be greater than zero",
        ));
}
