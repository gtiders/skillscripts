mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_list_skips_invalid_files() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace");
    let global_skills_dir = env.global_config_dir().join("skills");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&global_skills_dir).expect("failed to create global skills dir");

    fs::write(
        workspace.join("hello.py"),
        r#"# ---
# name: hello_skill
# description: Echo hello
# tags: [shell]
# ---
print("hello")
"#,
    )
    .expect("failed to write skill file");

    fs::write(
        workspace.join("README.md"),
        "This file has no skillscripts header.\n",
    )
    .expect("failed to write plain text file");

    fs::write(workspace.join("binary.bin"), b"abc\0def").expect("failed to write binary file");

    let assert = env.command(&workspace).arg("list").assert().success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("output should be valid YAML");

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["name"], "hello_skill");
    assert!(
        skills[0]["path"]
            .as_str()
            .expect("skill path should be a string")
            .ends_with("hello.py")
    );
}

#[test]
fn cli_list_warns_when_scan_path_does_not_exist() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-missing-path");
    let global_skills_dir = env.global_config_dir().join("skills");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&global_skills_dir).expect("failed to create global skills dir");

    fs::write(
        workspace.join("skillscripts.yaml"),
        r"
scan_paths:
  - ./missing-skills
",
    )
    .expect("failed to write local config");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Skipped scan path that does not exist",
        ))
        .stderr(predicate::str::contains("missing-skills"));
}

#[test]
fn cli_list_reports_parse_errors_when_configured() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-report-errors");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("skillscripts.yaml"),
        r"
report_parse_errors: true
",
    )
    .expect("failed to write local config");

    fs::write(
        workspace.join("good.py"),
        r#"# ---
# name: good_skill
# description: Valid skill
# ---
print("good")
"#,
    )
    .expect("failed to write valid skill");

    fs::write(
        workspace.join("broken.py"),
        r#"# ---
# name: broken_skill
# description: [unterminated
# ---
print("broken")
"#,
    )
    .expect("failed to write broken skill");

    let assert = env.command(&workspace).arg("list").assert().success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let skills: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&stdout).expect("output should be valid YAML");

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["name"], "good_skill");

    let stderr = String::from_utf8(assert.get_output().stderr.clone())
        .expect("stderr should be valid UTF-8");
    assert!(
        stderr.contains("error"),
        "should report parse errors when configured: {stderr}"
    );
}
