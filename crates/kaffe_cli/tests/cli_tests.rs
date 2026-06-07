use std::fs;
use std::process::Command;

#[test]
fn test_cli_build_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_kaffe"))
        .args(["build", "examples/main.kaf"])
        .current_dir("../..")
        .output()
        .expect("failed to run kaffe build");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("export function greet"));
    assert!(stdout.contains("includes"));
}

#[test]
fn test_cli_build_out_file() {
    let out_path = "target/test-output/main.ts";
    let repo_root = "../..";
    let _ = fs::remove_file(format!("{repo_root}/{out_path}"));

    let output = Command::new(env!("CARGO_BIN_EXE_kaffe"))
        .args(["build", "examples/main.kaf", "--out", out_path])
        .current_dir(repo_root)
        .output()
        .expect("failed to run kaffe build --out");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let contents = fs::read_to_string(format!("{repo_root}/{out_path}")).expect("missing output file");
    assert!(contents.contains("type User = {"));
    assert!(contents.contains("export function canEdit"));
}

#[test]
fn test_cli_build_missing_out_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_kaffe"))
        .args(["build", "examples/main.kaf", "--out"])
        .current_dir("../..")
        .output()
        .expect("failed to run kaffe build --out");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Missing output path after --out"));
}

#[test]
fn test_cli_build_rejects_unexpected_args() {
    let output = Command::new(env!("CARGO_BIN_EXE_kaffe"))
        .args(["build", "examples/main.kaf", "--wat"])
        .current_dir("../..")
        .output()
        .expect("failed to run kaffe build with unexpected args");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unexpected arguments"));
}
