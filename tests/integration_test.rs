use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn manifest_path(fixture: &str) -> PathBuf {
    fixture_path(fixture).join("Cargo.toml")
}

fn config_path(fixture: &str) -> PathBuf {
    fixture_path(fixture).join("depgraph-rules.toml")
}

fn cmd() -> Command {
    Command::cargo_bin("cargo-depgraph-check").expect("binary should exist")
}

// ---- valid-workspace ----

#[test]
fn valid_workspace_passes() {
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("valid-workspace").to_str().unwrap(),
            "--config",
            config_path("valid-workspace").to_str().unwrap(),
            "--color",
            "never",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("All dependency rules pass"));
}

// ---- violation-workspace ----

#[test]
fn violation_workspace_fails() {
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("violation-workspace").to_str().unwrap(),
            "--config",
            config_path("violation-workspace").to_str().unwrap(),
            "--color",
            "never",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("api depends on server"))
        .stderr(predicate::str::contains("allowed deps are: [core]"));
}

#[test]
fn violation_workspace_json_output() {
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("violation-workspace").to_str().unwrap(),
            "--config",
            config_path("violation-workspace").to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "\"kind\": \"forbidden_dependency\"",
        ))
        .stderr(predicate::str::contains("\"dependency\": \"server\""));
}

// ---- unknown-crate ----

#[test]
fn unknown_crate_fails_strict() {
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("unknown-crate").to_str().unwrap(),
            "--config",
            config_path("unknown-crate").to_str().unwrap(),
            "--color",
            "never",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("beta"))
        .stderr(predicate::str::contains("not listed in the config"));
}

// ---- virtual-workspace ----

#[test]
fn virtual_workspace_passes() {
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("virtual-workspace").to_str().unwrap(),
            "--config",
            config_path("virtual-workspace").to_str().unwrap(),
            "--color",
            "never",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("All dependency rules pass"));
}

// ---- stale-config ----

#[test]
fn stale_config_warns() {
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("stale-config").to_str().unwrap(),
            "--config",
            config_path("stale-config").to_str().unwrap(),
            "--color",
            "never",
        ])
        .assert()
        .success() // warnings don't fail
        .stderr(predicate::str::contains("ghost-crate"))
        .stderr(predicate::str::contains("no matching workspace member"));
}

// ---- generate subcommand ----

#[test]
fn generate_produces_valid_toml() {
    let output = cmd()
        .args([
            "generate",
            "--manifest-path",
            manifest_path("valid-workspace").to_str().unwrap(),
        ])
        .output()
        .expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("valid utf8");

    // Should be parseable as TOML
    let parsed: toml::Value = toml::from_str(&stdout).expect("should parse as valid TOML");

    // Should have rules for all 3 crates
    let rules = parsed.get("rules").expect("should have [rules]");
    let rules_table = rules.as_table().expect("rules should be a table");
    assert_eq!(rules_table.len(), 3);
    assert!(rules_table.contains_key("crate-a"));
    assert!(rules_table.contains_key("crate-b"));
    assert!(rules_table.contains_key("crate-c"));

    // Should have options section
    assert!(parsed.get("options").is_some());
}

#[test]
fn generate_topological_order() {
    let output = cmd()
        .args([
            "generate",
            "--manifest-path",
            manifest_path("valid-workspace").to_str().unwrap(),
        ])
        .output()
        .expect("should run");

    let stdout = String::from_utf8(output.stdout).expect("valid utf8");

    // crate-a (depth 0) should appear before crate-b (depth 1) which should appear before crate-c (depth 2)
    let pos_a = stdout.find("crate-a = ").expect("should contain crate-a");
    let pos_b = stdout.find("crate-b = ").expect("should contain crate-b");
    let pos_c = stdout.find("crate-c = ").expect("should contain crate-c");
    assert!(pos_a < pos_b, "crate-a should appear before crate-b");
    assert!(pos_b < pos_c, "crate-b should appear before crate-c");
}

// ---- flag tests ----

#[test]
fn manifest_path_flag_works() {
    // Running check with explicit --manifest-path should work
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("valid-workspace").to_str().unwrap(),
            "--config",
            config_path("valid-workspace").to_str().unwrap(),
            "--color",
            "never",
        ])
        .assert()
        .success();
}

#[test]
fn missing_config_exits_with_code_2() {
    cmd()
        .args([
            "check",
            "--manifest-path",
            manifest_path("valid-workspace").to_str().unwrap(),
            "--config",
            "/nonexistent/depgraph-rules.toml",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("failed to read config"));
}

#[test]
fn no_subcommand_exits_with_code_2() {
    // Invoking with no subcommand should print help and exit 2
    cmd().arg("depgraph-check").assert().code(2);
}
