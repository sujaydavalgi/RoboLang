//! CLI integration tests for package commands.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn spanda_bin() -> String {
    std::env::var("CARGO_BIN_EXE_spanda").expect("CARGO_BIN_EXE_spanda not set")
}

#[test]
fn init_creates_manifest_and_sources() {
    let dir = TempDir::new().unwrap();
    let output = Command::new(spanda_bin())
        .arg("init")
        .arg("test_pkg")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("spanda.toml").is_file());
    assert!(dir.path().join("src/main.sd").is_file());
}

#[test]
fn registry_search_finds_ros2() {
    let output = Command::new(spanda_bin())
        .args(["registry", "search", "ros2"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("spanda-ros2"));
}

#[test]
fn install_writes_lockfile() {
    let dir = TempDir::new().unwrap();
    let manifest = r#"
[package]
name = "test_app"
version = "0.1.0"

[dependencies]
spanda-ros2 = "0.1.0"
"#;
    fs::write(dir.path().join("spanda.toml"), manifest).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.sd"), "module test;\n").unwrap();

    let output = Command::new(spanda_bin())
        .arg("install")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("spanda.lock").is_file());
}

#[test]
fn add_and_remove_dependency() {
    let dir = TempDir::new().unwrap();
    let manifest = r#"
[package]
name = "test_app"
version = "0.1.0"
"#;
    fs::write(dir.path().join("spanda.toml"), manifest).unwrap();

    let add = Command::new(spanda_bin())
        .args(["add", "spanda-vision", "--version", "0.1.0"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(add.status.success());
    let content = fs::read_to_string(dir.path().join("spanda.toml")).unwrap();
    assert!(content.contains("spanda-vision"));

    let remove = Command::new(spanda_bin())
        .args(["remove", "spanda-vision"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(remove.status.success());
    let content = fs::read_to_string(dir.path().join("spanda.toml")).unwrap();
    assert!(!content.contains("spanda-vision"));
}
