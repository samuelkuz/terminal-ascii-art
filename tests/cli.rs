use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn renders_ascii_art_to_stdout() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command.arg("Hi").assert().success().stdout(predicate::str::contains("#####"));
}

#[test]
fn rejects_empty_input() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("text input cannot be empty"));
}

#[test]
fn clap_rejects_invalid_alignment() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .args(["--align", "diagonal", "Hi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn width_error_is_reported_for_center_alignment() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .args(["--align", "center", "--width", "3", "Hi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("exceeds available width"));
}

#[test]
fn defaults_to_detected_terminal_width_when_width_is_missing() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .env("COLUMNS", "12")
        .args(["--align", "center", "A"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("    ### "));
}

#[test]
fn rejects_explicit_width_larger_than_terminal() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .env("COLUMNS", "10")
        .args(["--width", "12", "A"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requested width 12 exceeds terminal width 10"));
}
