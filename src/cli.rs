use clap::{Parser, ValueEnum};

use crate::error::RenderError;
use crate::renderer::{Alignment, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FontName {
    Standard,
}

#[derive(Debug, Parser)]
#[command(name = "terminal-ascii-art", version, about = "Render text as ASCII art")]
pub struct Cli {
    /// Text to render as ASCII art.
    pub text: String,

    /// Built-in font to use.
    #[arg(long, value_enum, default_value_t = FontName::Standard)]
    pub font: FontName,

    /// Horizontal alignment when a width is provided.
    #[arg(long, value_enum, default_value_t = Alignment::Left)]
    pub align: Alignment,

    /// Render width in columns.
    #[arg(long)]
    pub width: Option<usize>,

    /// ASCII art drawing theme.
    #[arg(long, value_enum, default_value_t = Theme::Plain)]
    pub theme: Theme,
}

impl Cli {
    pub fn validated_text(&self) -> Result<&str, RenderError> {
        if self.text.trim().is_empty() {
            return Err(RenderError::EmptyInput);
        }

        Ok(&self.text)
    }
}
