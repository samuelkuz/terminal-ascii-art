use assert_cmd::Command;
use image::{GrayImage, Luma, Rgb, RgbImage};
use predicates::prelude::*;
use tempfile::tempdir;

fn write_png_fixture() -> std::path::PathBuf {
    let directory = tempdir().unwrap();
    let path = directory.path().join("fixture.png");
    let mut image = GrayImage::new(4, 2);

    for y in 0..2 {
        for x in 0..4 {
            let intensity = if x < 2 { 0 } else { 255 };
            image.put_pixel(x, y, Luma([intensity]));
        }
    }

    image.save(&path).unwrap();
    let _ = directory.keep();
    path
}

fn write_jpeg_fixture() -> std::path::PathBuf {
    let directory = tempdir().unwrap();
    let path = directory.path().join("fixture.jpg");
    let mut image = RgbImage::new(4, 2);

    for y in 0..2 {
        for x in 0..4 {
            let intensity = if x < 2 { 32 } else { 224 };
            image.put_pixel(x, y, Rgb([intensity, intensity, intensity]));
        }
    }

    image
        .save_with_format(&path, image::ImageFormat::Jpeg)
        .unwrap();
    let _ = directory.keep();
    path
}

fn write_corrupt_fixture() -> std::path::PathBuf {
    let directory = tempdir().unwrap();
    let path = directory.path().join("fixture.png");
    std::fs::write(&path, b"not-a-real-image").unwrap();
    let _ = directory.keep();
    path
}

#[test]
fn renders_ascii_art_to_stdout() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .args(["text", "Hi"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#####"));
}

#[test]
fn rejects_empty_input() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .args(["text", ""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("text input cannot be empty"));
}

#[test]
fn clap_rejects_invalid_alignment() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .args(["text", "--align", "diagonal", "Hi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn width_error_is_reported_for_center_alignment() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .args(["text", "--align", "center", "--width", "3", "Hi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("exceeds available width"));
}

#[test]
fn defaults_to_detected_terminal_width_when_width_is_missing() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .env("COLUMNS", "12")
        .args(["text", "--align", "center", "A"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("    ### "));
}

#[test]
fn rejects_explicit_width_larger_than_terminal() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .env("COLUMNS", "10")
        .args(["text", "--width", "12", "A"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "requested width 12 exceeds terminal width 10",
        ));
}

#[test]
fn renders_png_image_to_stdout() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();
    let image = write_png_fixture();

    command
        .args(["image", image.to_str().unwrap(), "--width", "4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("@@"));
}

#[test]
fn renders_jpeg_image_to_stdout() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();
    let image = write_jpeg_fixture();

    command
        .args(["image", image.to_str().unwrap(), "--width", "4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("%"));
}

#[test]
fn missing_image_path_is_reported() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();

    command
        .args(["image", "does-not-exist.png"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read image"));
}

#[test]
fn corrupt_image_data_is_reported() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();
    let image = write_corrupt_fixture();

    command
        .args(["image", image.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to decode image"));
}

#[test]
fn image_mode_defaults_to_detected_terminal_width_when_width_is_missing() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();
    let image = write_png_fixture();

    command
        .env("COLUMNS", "4")
        .args(["image", image.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("@@"));
}

#[test]
fn image_mode_rejects_explicit_width_larger_than_terminal() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();
    let image = write_png_fixture();

    command
        .env("COLUMNS", "3")
        .args(["image", image.to_str().unwrap(), "--width", "4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "requested width 4 exceeds terminal width 3",
        ));
}

#[test]
fn image_mode_can_emit_ansi_color() {
    let mut command = Command::cargo_bin("terminal-ascii-art").unwrap();
    let image = write_jpeg_fixture();

    command
        .args(["image", image.to_str().unwrap(), "--width", "4", "--color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}[38;2;"));
}
