use std::process::Command;

#[test]
fn cli_filters_and_serializes_json() {
    let binary = std::env::var("CARGO_BIN_EXE_data_transformer")
        .expect("Cargo should provide the binary path");

    let output = Command::new(binary)
        .arg("tests/fixtures/sample.csv")
        .args(["--filter", "department=Engineering"])
        .args(["--format", "json"])
        .output()
        .expect("binary should execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("valid UTF-8 output");

    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn cli_rejects_unsupported_format() {
    let binary = std::env::var("CARGO_BIN_EXE_data_transformer")
        .expect("Cargo should provide the binary path");

    let output = Command::new(binary)
        .arg("tests/fixtures/sample.csv")
        .args(["--format", "xml"])
        .output()
        .expect("binary should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("valid UTF-8 error");

    assert!(stderr.contains("Unsupported output format"));
}

#[test]
fn cli_rejects_missing_input() {
    let binary = std::env::var("CARGO_BIN_EXE_data_transformer")
        .expect("Cargo should provide the binary path");

    let output = Command::new(binary)
        .output()
        .expect("binary should execute");

    assert!(!output.status.success());
}
