mod common;

use common::TestEnv;
use std::fs;

#[test]
fn cli_list_reads_global_and_local_config_together() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-config-resolution");
    let local_skills_dir = workspace.join("local-skills");
    let global_skills_dir = env.global_config_dir().join("skills");

    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&local_skills_dir).expect("failed to create local skills dir");
    fs::create_dir_all(&global_skills_dir).expect("failed to create global skills dir");

    fs::write(
        env.global_config_file(),
        r"
scan_paths:
  - skills
",
    )
    .expect("failed to write global config");

    fs::write(
        workspace.join("skillscripts.yaml"),
        r"
scan_paths:
  - ./local-skills
",
    )
    .expect("failed to write local config");

    fs::write(
        global_skills_dir.join("global.py"),
        r#"# ---
# name: global_skill
# description: From global config
# ---
print("global")
"#,
    )
    .expect("failed to write global skill");

    fs::write(
        local_skills_dir.join("local.py"),
        r#"# ---
# name: local_skill
# description: From local config
# ---
print("local")
"#,
    )
    .expect("failed to write local skill");

    let output = env
        .command(&workspace)
        .arg("list")
        .output()
        .expect("failed to run list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("output should be valid YAML");

    assert_eq!(skills.len(), 2);
    assert!(skills.iter().any(|skill| skill["name"] == "global_skill"));
    assert!(skills.iter().any(|skill| skill["name"] == "local_skill"));
}
