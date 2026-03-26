# terminal-ascii-art

`terminal-ascii-art` is a small Rust CLI and library for rendering text, images, and video as ASCII art in the terminal.

It supports:

- Text rendering with built-in ASCII-art fonts
- PNG and JPEG image rendering
- Optional truecolor ANSI output for image and video modes
- Terminal-aware auto-sizing
- Video playback in the terminal through `ffmpeg`

## Features

### Text mode

- Renders input text as 5-row ASCII art
- Supports `left`, `center`, and `right` alignment
- Supports `plain`, `outline`, and `block` themes

### Image mode

- Loads PNG and JPEG files
- Converts brightness to ASCII characters
- Can emit grayscale ASCII or ANSI-colored ASCII
- Automatically fits output to the current terminal when possible

### Video mode

- Uses `ffprobe` to inspect source dimensions
- Uses `ffmpeg` to decode frames into RGB24
- Streams frames to the terminal in an alternate screen buffer
- Supports looping, inversion, grayscale output, and optional hardware acceleration
- Quit playback with `q`, `Q`, or `Ctrl-C`

## Requirements

### Build and run

- Rust toolchain with Cargo

### Video mode only

- [`ffmpeg`](https://ffmpeg.org/)
- [`ffprobe`](https://ffmpeg.org/ffprobe.html)

If `ffmpeg` or `ffprobe` are not available in `PATH`, video playback will fail with a clear error message.

## Installation

Clone the repository and run with Cargo:

```bash
cargo run -- --help
```

Build a debug binary:

```bash
cargo build
./target/debug/terminal-ascii-art --help
```

Build an optimized release binary:

```bash
cargo build --release
./target/release/terminal-ascii-art --help
```

Install the binary into Cargo's bin directory so it can be run as `terminal-ascii-art` from your shell:

```bash
cargo install --path .
terminal-ascii-art --help
```

## Usage

If you have not installed the binary, run commands through Cargo:

```bash
cargo run -- <COMMAND> [ARGS...]
```

If you built locally with `cargo build`, use:

```bash
./target/debug/terminal-ascii-art <COMMAND> [ARGS...]
```

If you installed the crate with `cargo install --path .`, use:

```bash
terminal-ascii-art <COMMAND>
```

Available commands:

- `text` renders text as ASCII art
- `image` renders an image file as ASCII art
- `video` plays a video file as ASCII art in the terminal

## Text Rendering

```bash
terminal-ascii-art text [OPTIONS] <TEXT>
```

Options:

- `--font <FONT>`: built-in font to use, currently `standard`
- `--align <ALIGN>`: `left`, `center`, or `right`
- `--width <WIDTH>`: render width in columns
- `--theme <THEME>`: `plain`, `outline`, or `block`

Examples:

```bash
cargo run -- text "Hello"
./target/debug/terminal-ascii-art text --theme outline "ASCII"
./target/debug/terminal-ascii-art text --align center --width 80 "Centered"
terminal-ascii-art text --theme block "LOUD"
```

## Image Rendering

```bash
terminal-ascii-art image [OPTIONS] <PATH>
```

Options:

- `--width <WIDTH>`: render width in columns
- `--invert`: invert brightness-to-character mapping
- `--color`: emit 24-bit ANSI foreground color

Examples:

```bash
cargo run -- image photos/dogs.jpg
./target/debug/terminal-ascii-art image photos/one_piece_old.jpg --width 100
./target/debug/terminal-ascii-art image photos/dogs.jpg --color
terminal-ascii-art image photos/dogs.jpg --invert
```

## Video Playback

```bash
terminal-ascii-art video [OPTIONS] <PATH>
```

Options:

- `--width <WIDTH>`: render width in columns
- `--fps <FPS>`: target playback frame rate, default `12`
- `--invert`: invert brightness-to-character mapping
- `--grayscale`: disable ANSI color output
- `--loop`: restart playback when the video ends
- `--hwaccel <HWACCEL>`: `auto` or `none`

Examples:

```bash
cargo run -- video videos/earth_spinning.mp4
./target/debug/terminal-ascii-art video videos/earth_spinning.mp4 --width 100 --fps 15
./target/debug/terminal-ascii-art video videos/IMG_2380.MOV --grayscale --hwaccel none
terminal-ascii-art video videos/earth_spinning.mp4 --loop
```

## Sizing Behavior

When no width is supplied, the project tries to detect the current terminal size and fit output automatically.

- Text mode uses terminal width when available
- Image and video modes fit within terminal width and height when available
- Explicit widths larger than the detected terminal width are rejected

Terminal size detection uses:

- Live terminal queries
- `COLUMNS` and `LINES` environment variables
- Fallback terminal-size detection

## Library Usage

The crate can also be used directly from Rust code instead of invoking the CLI.

### Add the dependency

During local development, add it by path:

```toml
[dependencies]
terminal-ascii-art = { path = "../terminal-ascii-art" }
```

If the repository is hosted remotely, depend on it by Git URL:

```toml
[dependencies]
terminal-ascii-art = { git = "https://github.com/your-name/terminal-ascii-art" }
```

### Render text from Rust

The most reusable text API is `render`, along with `RenderOptions`, `Alignment`, `Theme`, and a font implementation such as `StandardFont`.

```rust
use terminal_ascii_art::{Alignment, RenderOptions, StandardFont, Theme, render};

fn main() {
    let options = RenderOptions {
        width: Some(80),
        alignment: Alignment::Center,
        theme: Theme::Plain,
    };

    let output = render("Hello", &StandardFont, &options).unwrap();
    println!("{output}");
}
```

### Other exported APIs

The crate also re-exports APIs for image and video workflows:

- `render_image`
- `render_rgb_frame`
- `play_video`
- `ImageRenderOptions`
- `VideoRenderOptions`
- `HwAccelMode`
- `RenderError`

### What to use downstream

If you are embedding this crate in another Rust application, these are the main entrypoints to use:

- `render` for text-to-ASCII rendering
- `render_image` for loading and converting PNG or JPEG files
- `render_rgb_frame` when you already have decoded RGB24 frame data
- `play_video` when you want terminal playback backed by `ffmpeg`

The crate also exposes CLI types such as `Cli` and `Commands`, plus the convenience `run` function. Those are mainly useful if you want to reuse or extend the command-line interface in another binary. Most library consumers will want the rendering functions listed above.

### Notes

- `play_video` requires `ffmpeg` and `ffprobe` to be available in `PATH`
- Image rendering currently supports PNG and JPEG input
- Video playback is terminal-oriented and uses an alternate screen session for drawing

See [src/lib.rs](/Users/samkuz/Coding/terminal-ascii-art/src/lib.rs) for the exported API surface.

## Development

Format and test the project with:

```bash
cargo fmt --all
cargo test
```

## Project Layout

- [src/main.rs](/Users/samkuz/Coding/terminal-ascii-art/src/main.rs): binary entrypoint
- [src/lib.rs](/Users/samkuz/Coding/terminal-ascii-art/src/lib.rs): library entrypoint and re-exports
- [src/renderer.rs](/Users/samkuz/Coding/terminal-ascii-art/src/renderer.rs): text rendering
- [src/image_renderer.rs](/Users/samkuz/Coding/terminal-ascii-art/src/image_renderer.rs): image and RGB frame rendering
- [src/video.rs](/Users/samkuz/Coding/terminal-ascii-art/src/video.rs): video probing and playback
- [src/terminal.rs](/Users/samkuz/Coding/terminal-ascii-art/src/terminal.rs): terminal sizing and drawing
- [src/font.rs](/Users/samkuz/Coding/terminal-ascii-art/src/font.rs): font definitions
- [src/error.rs](/Users/samkuz/Coding/terminal-ascii-art/src/error.rs): shared error types

## Testing

The test suite covers:

- Text rendering behavior and alignment
- Image scaling and RGB frame conversion
- CLI validation and error handling
- Video argument construction and dependency handling
