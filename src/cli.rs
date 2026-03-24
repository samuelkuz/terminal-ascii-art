use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::error::RenderError;
use crate::renderer::{Alignment, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FontName {
    Standard,
}

#[derive(Debug, Parser)]
#[command(
    name = "terminal-ascii-art",
    version,
    about = "Render text as ASCII art"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

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
    },
}

impl Cli {
    pub fn validated_text(text: &str) -> Result<&str, RenderError> {
        if text.trim().is_empty() {
            return Err(RenderError::EmptyInput);
        }

        Ok(text)
    }
}
