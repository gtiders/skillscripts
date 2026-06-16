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
    assert!(config["scripts"].is_sequence());
}

#[test]
fn init_rejects_existing_config_without_force() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-exists");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config("scripts: []\n");

    env.command(&workspace)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "A configuration file already exists",
        ));
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
  - id: 101
    path: scripts/alpha.py
    command: python {{path}}
  - id: 102
    path: scripts/beta.py
    command: python {{path}}
",
    );

    let assert = env
        .command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("- id:"));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("list should emit a valid YAML array");

    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0]["id"].as_i64(), Some(101));
    assert_eq!(skills[0]["command"].as_str(), Some("python {{path}}"));
    assert!(
        skills[0]["path"]
            .as_str()
            .expect("script path should be a string")
            .ends_with("scripts/alpha.py")
    );
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
  - id: 1
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
  - id: 1
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
  - id: 1
    path: scripts/one.py
    command: python {{path}}
  - id: 2
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
  - id: 301
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
  - id: 1
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
  - id: 1
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
        .stderr(predicate::str::contains("import must be relative"));
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
  - id: 401
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

    fs::write(
        scripts_dir.join("echo_args.ps1"),
        r#"
param(
    [string]$PathArg,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Rest
)
Write-Output "PATH=$PathArg"
foreach ($item in $Rest) {
    Write-Output "ARG=$item"
}
"#,
    )
    .expect("failed to write script");

    env.write_global_config(
        r#"
scripts:
  - id: 501
    path: scripts/echo_args.ps1
    command: powershell -NoProfile -ExecutionPolicy Bypass -File {{path}} "{{path}}"
"#,
    );

    env.command(&workspace)
        .args(["run", "501", "one", "--flag", "three"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: powershell"))
        .stdout(predicate::str::contains("echo_args.ps1"))
        .stdout(predicate::str::contains("ARG=one"))
        .stdout(predicate::str::contains("ARG=--flag"))
        .stdout(predicate::str::contains("ARG=three"));
}

#[test]
fn run_preserves_unquoted_placeholder_path_with_spaces() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-run-path-spaces");
    let scripts_dir = env.global_config_dir().join("scripts with spaces");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&scripts_dir).expect("failed to create scripts dir");

    fs::write(
        scripts_dir.join("echo path.ps1"),
        r#"
param([string]$PathArg)
Write-Output "PATH=$PathArg"
"#,
    )
    .expect("failed to write script");

    env.write_global_config(
        r"
scripts:
  - id: 503
    path: scripts with spaces/echo path.ps1
    command: powershell -NoProfile -ExecutionPolicy Bypass -File {{path}} {{path}}
",
    );

    env.command(&workspace)
        .args(["run", "503"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scripts with spaces"))
        .stdout(predicate::str::contains("PATH="))
        .stdout(predicate::str::contains("echo path.ps1"));
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
  - id: 502
    path: scripts/echo.ps1
    command: powershell -NoProfile -ExecutionPolicy Bypass -File
",
    );

    env.command(&workspace)
        .args(["run", "502"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must contain {{path}}"));
}

#[test]
fn run_reports_missing_id() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-run-missing-id");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    env.write_global_config("scripts: []\n");

    env.command(&workspace)
        .args(["run", "999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No script found for id 999."));
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
        .stderr(predicate::str::contains("Usage: sks run <id> [args...]"));
}
