mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_list_yaml_outputs_plain_machine_readable_skill_array() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("alpha.py"),
        r#"# ---
# name: alpha_skill
# description: First skill
# tags: [alpha, beta]
# ---
print("alpha")
"#,
    )
    .expect("failed to write first skill");

    fs::write(
        workspace.join("beta.py"),
        r#"# ---
# name: beta_skill
# description: Second skill
# ---
print("beta")
"#,
    )
    .expect("failed to write second skill");

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
    assert_eq!(skills[0]["name"], "alpha_skill");
    assert_eq!(skills[1]["name"], "beta_skill");
    assert_eq!(skills[0]["description"], "First skill");
    assert!(skills[0]["tags"].is_sequence());
    assert!(skills[0]["path"].is_string());
}
