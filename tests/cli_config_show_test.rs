mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_config_prints_default_local_and_effective_config() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-config-show");
    let local_skills_dir = workspace.join("local-skills");

    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&local_skills_dir).expect("failed to create local skills dir");
    fs::create_dir_all(env.global_config_dir()).expect("failed to create global config dir");

    fs::write(
        env.global_config_file(),
        r"
scan_paths:
  - skills
search_limit: 9
",
    )
    .expect("failed to write global config");

    fs::write(
        workspace.join("skillscripts.yaml"),
        r"
scan_paths:
  - ./local-skills
ignore_patterns:
  - dist
max_file_size: 2MB
",
    )
    .expect("failed to write local config");

    let assert = env
        .command(&workspace)
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("=== BUILT-IN DEFAULTS ==="))
        .stdout(predicate::str::contains("=== GLOBAL CONFIG FILE ==="))
        .stdout(predicate::str::contains(
            "=== LOCAL CONFIG (CURRENT DIRECTORY) ===",
        ))
        .stdout(predicate::str::contains("=== EFFECTIVE CONFIG ==="));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");

    // Effective config should contain local scan path and global default skills path.
    assert!(
        stdout.contains("local-skills"),
        "effective config should contain local scan path"
    );
    let has_global_skills_path = stdout.contains("/.config/skillscripts/skills")
        || stdout.contains("\\.config\\skillscripts\\skills")
        || stdout.contains(".config/skillscripts/skills")
        || stdout.contains(".config\\skillscripts\\skills");
    assert!(
        has_global_skills_path,
        "effective config should contain injected global skills path"
    );
}
