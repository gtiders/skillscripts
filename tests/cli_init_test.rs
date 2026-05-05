mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_init_creates_default_config_in_current_workspace() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let config_path = env.global_config_file();
    assert!(
        config_path.exists(),
        "init should create the global config file"
    );
    assert!(
        env.global_config_dir().join("skills").exists(),
        "init should create the global skills directory"
    );

    let yaml = fs::read_to_string(config_path).expect("failed to read generated config");
    let config: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("generated config should be valid YAML");

    // Default config must be parseable and contain basic release defaults.
    assert_eq!(config["scan_paths"][0].as_str(), Some("skills"));
    assert_eq!(config["max_file_size"].as_str(), Some("1MB"));
    assert_eq!(config["search_limit"].as_i64(), Some(5));
}

#[test]
fn cli_init_rejects_existing_config_without_force() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-exists");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    let config_path = env.global_config_file();
    fs::create_dir_all(env.global_config_dir()).expect("failed to create global config dir");
    fs::write(&config_path, "search_limit: 99\n").expect("failed to seed config");

    env.command(&workspace)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "A configuration file already exists",
        ));

    // Without --force, must preserve user's existing config, no silent overwrite.
    let current = fs::read_to_string(config_path).expect("failed to read existing config");
    assert_eq!(current, "search_limit: 99\n");
}

#[test]
fn cli_init_force_overwrites_existing_config() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-force");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    let config_path = env.global_config_file();
    fs::create_dir_all(env.global_config_dir()).expect("failed to create global config dir");
    fs::write(&config_path, "search_limit: 99\n").expect("failed to seed config");

    env.command(&workspace)
        .args(["init", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let yaml = fs::read_to_string(config_path).expect("failed to read overwritten config");
    let config: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("overwritten config should be valid YAML");

    assert_eq!(config["scan_paths"][0].as_str(), Some("skills"));
    assert_eq!(config["max_file_size"].as_str(), Some("1MB"));
    assert_eq!(config["search_limit"].as_i64(), Some(5));
}

#[test]
fn cli_init_local_creates_config_in_current_directory() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-local");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .args(["init", "--local"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let local_config = workspace.join("skillscripts.yaml");
    assert!(
        local_config.exists(),
        "init --local should create a config file in the current directory"
    );
    assert!(
        !env.global_config_file().exists(),
        "init --local should not implicitly create the global config file"
    );

    let yaml = fs::read_to_string(local_config).expect("failed to read local config");
    let config: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("local config should be valid YAML");

    assert_eq!(config["scan_paths"][0].as_str(), Some("."));
}

#[test]
fn cli_init_global_seeds_workspace_skills_into_global_skills_dir() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-seed");
    let workspace_skills = workspace.join("skills");
    let workspace_nested = workspace_skills.join("nested");
    fs::create_dir_all(&workspace_nested).expect("failed to create workspace skills dir");

    fs::write(
        workspace_skills.join("agent.md"),
        "---\nname: seeded_agent\ndescription: seeded\n---\n# seeded\n",
    )
    .expect("failed to write root skill");
    fs::write(
        workspace_nested.join("guide.md"),
        "---\nname: seeded_guide\ndescription: seeded\n---\n# seeded\n",
    )
    .expect("failed to write nested skill");

    env.command(&workspace)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let global_skills = env.global_config_dir().join("skills");
    assert!(
        global_skills.join("agent.md").exists(),
        "init should seed workspace skills into global skills directory"
    );
    assert!(
        global_skills.join("nested").join("guide.md").exists(),
        "init should preserve nested skill directory structure"
    );
}
