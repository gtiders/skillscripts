mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_list_scans_skills_instantly() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-instant-scan");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("hello.py"),
        r#"# ---
# name: hello_skill
# description: hello
# ---
print("hello")
"#,
    )
    .expect("failed to write skill file");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello_skill"));
}

#[test]
fn cli_list_detects_new_skills_without_cache() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-new-skill");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("one.py"),
        r#"# ---
# name: one_skill
# description: one
# ---
print("one")
"#,
    )
    .expect("failed to write first skill");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("one_skill"));

    fs::write(
        workspace.join("two.py"),
        r#"# ---
# name: two_skill
# description: two
# ---
print("two")
"#,
    )
    .expect("failed to write second skill");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("one_skill"))
        .stdout(predicate::str::contains("two_skill"));
}

#[test]
fn cli_search_scans_skills_instantly() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-search-scan");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("echo.py"),
        r#"# ---
# name: echo_skill
# description: Echo user input
# ---
print("echo")
"#,
    )
    .expect("failed to write skill file");

    env.command(&workspace)
        .args(["search", "echo"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("- name:"))
        .stdout(predicate::str::contains("echo_skill"));
}

#[test]
fn cli_default_command_scans_skills_instantly() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-default-scan");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .assert()
        .success()
        .stderr(predicate::str::contains("No skill selected"));
}
