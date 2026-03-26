use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::error::RenderError;
use crate::renderer::{Alignment, Theme};
use crate::video::HwAccelMode;

/// Built-in font names accepted by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FontName {
    /// The default 5-row block font bundled with the crate.
    Standard,
}

/// Top-level command-line arguments for the application.
#[derive(Debug, Parser)]
#[command(
    name = "terminal-ascii-art",
    version,
    about = "Render text as ASCII art"
)]
pub struct Cli {
    /// The subcommand describing which rendering mode to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// Supported render modes exposed by the binary.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Render text as ASCII art.
    Text {
        /// Text to render as ASCII art.
        text: String,

        /// Built-in font to use.
        #[arg(long, value_enum, default_value_t = FontName::Standard)]
        font: FontName,

        /// Horizontal alignment when a width is provided.
        #[arg(long, value_enum, default_value_t = Alignment::Left)]
        align: Alignment,

        /// Render width in columns.
        #[arg(long)]
        width: Option<usize>,

        /// ASCII art drawing theme.
        #[arg(long, value_enum, default_value_t = Theme::Plain)]
        theme: Theme,
    },

    /// Render an image file as ASCII art.
    Image {
        /// Path to a PNG or JPEG image.
        path: PathBuf,

        /// Render width in columns.
        #[arg(long)]
        width: Option<usize>,

        /// Invert brightness-to-character mapping.
        #[arg(long)]
        invert: bool,

        /// Emit 24-bit ANSI foreground color for each ASCII character.
        #[arg(long)]
        color: bool,
    },

    /// Play a video file as ASCII art in the terminal.
    Video {
        /// Path to a local video file.
        path: PathBuf,

        /// Render width in columns.
        #[arg(long)]
        width: Option<usize>,

        /// Target playback frame rate.
        #[arg(long, default_value_t = 12, value_parser = clap::value_parser!(u8).range(1..=15))]
        fps: u8,

        /// Invert brightness-to-character mapping.
        #[arg(long)]
        invert: bool,

        /// Disable ANSI color output.
        #[arg(long)]
        grayscale: bool,

        /// Loop playback when the video reaches the end.
        #[arg(long = "loop")]
        loop_playback: bool,

        /// Hardware acceleration mode to request from ffmpeg.
        #[arg(long, value_enum, default_value_t = HwAccelMode::Auto)]
        hwaccel: HwAccelMode,
    },
}

impl Cli {
    /// Rejects text inputs that are empty or contain only whitespace.
    pub fn validated_text(text: &str) -> Result<&str, RenderError> {
        if text.trim().is_empty() {
            return Err(RenderError::EmptyInput);
        }

        Ok(text)
    }
}
