mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_search_yaml_outputs_lightweight_skill_array() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("echo.py"),
        r#"# ---
# name: echo_skill
# description: Echo user input
# args:
#   message:
#     type: string
#     description: Message to echo
#     required: true
# ---
print("echo")
"#,
    )
    .expect("failed to write skill file");

    fs::write(
        workspace.join("image.py"),
        r#"# ---
# name: image_skill
# description: Generate an image
# ---
print("image")
"#,
    )
    .expect("failed to write second skill file");

    let mut search = env.command(&workspace);

    let assert = search
        .args(["search", "echo"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("- name:"));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("search should emit a valid YAML array");

    assert!(
        !skills.is_empty(),
        "search should return at least one skill"
    );

    assert_eq!(skills[0]["name"], "echo_skill");
    assert_eq!(skills[0]["description"], "Echo user input");
    let expected_path = workspace
        .join("echo.py")
        .to_string_lossy()
        .replace('\\', "/");
    assert_eq!(skills[0]["path"], serde_yaml::Value::String(expected_path));
    assert!(skills[0]["tags"].is_sequence());
}

#[test]
fn cli_search_yaml_respects_limit_and_keeps_result_order_stable() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-search-limit");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("alpha.py"),
        r#"# ---
# name: alpha_skill
# description: shared description
# ---
print("alpha")
"#,
    )
    .expect("failed to write alpha skill");

    fs::write(
        workspace.join("beta.py"),
        r#"# ---
# name: beta_skill
# description: shared description
# ---
print("beta")
"#,
    )
    .expect("failed to write beta skill");

    let assert = env
        .command(&workspace)
        .args(["search", "shared", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("- name:"));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("search should emit valid YAML");

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["name"], "alpha_skill");
}

#[test]
fn cli_search_prioritizes_name_then_tags_then_description() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-search-priority");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("name.py"),
        r#"# ---
# name: alpha_beacon
# description: generic helper
# tags: [misc]
# ---
print("name")
"#,
    )
    .expect("failed to write name-priority skill");

    fs::write(
        workspace.join("tags.py"),
        r#"# ---
# name: utility_skill
# description: generic helper
# tags: [alpha, beacon]
# ---
print("tags")
"#,
    )
    .expect("failed to write tag-priority skill");

    fs::write(
        workspace.join("description.py"),
        r#"# ---
# name: helper_skill
# description: alpha workflow
# tags: [misc]
# ---
print("description")
"#,
    )
    .expect("failed to write description-priority skill");

    let assert = env
        .command(&workspace)
        .args(["search", "alpha"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("search should emit valid YAML");

    assert_eq!(skills[0]["name"], "alpha_beacon");
    assert_eq!(skills[1]["name"], "utility_skill");
    assert_eq!(skills[2]["name"], "helper_skill");
}
