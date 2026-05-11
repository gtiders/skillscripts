mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_task_prints_only_the_skill_path_for_matching_task_id() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-task");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("agent.py"),
        r#"# ---
# task_id: 902
# name: agent_skill
# description: Match by task id
# ---
print("agent")
"#,
    )
    .expect("failed to write skill file");

    let assert = env
        .command(&workspace)
        .args(["task", "902"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let expected_path = workspace
        .join("agent.py")
        .to_string_lossy()
        .replace('\\', "/");

    assert_eq!(stdout.trim(), expected_path);
    assert!(!stdout.contains("name:"));
    assert!(!stdout.contains("description:"));
}

#[test]
fn cli_task_reports_when_task_id_does_not_exist() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-task-missing");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("agent.py"),
        r#"# ---
# task_id: 902
# name: agent_skill
# description: Match by task id
# ---
print("agent")
"#,
    )
    .expect("failed to write skill file");

    env.command(&workspace)
        .args(["task", "999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No skill found for task_id 999."));
}

#[test]
fn cli_rejects_duplicate_task_ids() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-task-duplicate");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("alpha.py"),
        r#"# ---
# task_id: 902
# name: alpha_skill
# description: First duplicate
# ---
print("alpha")
"#,
    )
    .expect("failed to write first skill file");

    fs::write(
        workspace.join("beta.py"),
        r#"# ---
# task_id: 902
# name: beta_skill
# description: Second duplicate
# ---
print("beta")
"#,
    )
    .expect("failed to write second skill file");

    env.command(&workspace)
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Duplicate task_id 902"));
}
