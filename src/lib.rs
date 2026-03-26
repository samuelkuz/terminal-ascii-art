//! Core library for rendering text, images, and video as terminal-friendly ASCII art.

/// Command-line parsing and validation types.
pub mod cli;
/// Shared error types returned by rendering operations.
pub mod error;
/// Built-in ASCII-art font definitions.
pub mod font;
/// Helpers for layout and line alignment.
pub mod formatter;
/// Image loading, sizing, and frame-to-ASCII conversion.
pub mod image_renderer;
/// Text rendering primitives and theming.
pub mod renderer;
/// Terminal capability detection and alternate-screen playback support.
pub mod terminal;
/// Video probing, decoding, and playback helpers.
pub mod video;

pub use error::RenderError;
pub use font::{Font, StandardFont};
pub use image_renderer::{ImageRenderOptions, render_image, render_rgb_frame};
pub use renderer::{Alignment, RenderOptions, Theme, render};
pub use video::{HwAccelMode, VideoRenderOptions, play_video};

use cli::{Cli, Commands};

/// Executes a parsed CLI command and returns printable output when applicable.
pub fn run(cli: Cli) -> Result<Option<String>, RenderError> {
    match cli.command {
        Commands::Text {
            text,
            font: _,
            align,
            width,
            theme,
        } => {
            let text = Cli::validated_text(&text)?;
            let font = StandardFont;
            let width = resolve_output_width(width, terminal::detect_terminal_width())?;
            let options = RenderOptions {
                width,
                alignment: align,
                theme,
            };

            Ok(Some(render(text, &font, &options)?))
        }
        Commands::Image {
            path,
            width,
            invert,
            color,
        } => {
            let width = resolve_output_width(width, terminal::detect_terminal_width())?;
            let options = ImageRenderOptions {
                width,
                invert,
                color,
            };

            Ok(Some(render_image(&path, &options)?))
        }
        Commands::Video {
            path,
            width,
            fps,
            invert,
            grayscale,
            loop_playback,
            hwaccel,
        } => {
            let width = resolve_output_width(width, terminal::detect_terminal_width())?;
            let options = VideoRenderOptions {
                width,
                fps,
                invert,
                color: !grayscale,
                loop_playback,
                hwaccel,
            };

            play_video(&path, &options)?;
            Ok(None)
        }
    }
}

/// Resolves the effective output width from CLI input and the current terminal size.
///
/// When both values are present, an explicit width that exceeds the terminal width is rejected.
pub fn resolve_output_width(
    requested_width: Option<usize>,
    terminal_width: Option<usize>,
) -> Result<Option<usize>, RenderError> {
    match (requested_width, terminal_width) {
        (Some(requested_width), Some(terminal_width)) if requested_width > terminal_width => {
            Err(RenderError::TerminalWidthExceeded {
                requested_width,
                terminal_width,
            })
        }
        (Some(requested_width), _) => Ok(Some(requested_width)),
        (None, Some(terminal_width)) => Ok(Some(terminal_width)),
        (None, None) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_output_width;

    #[test]
    fn uses_terminal_width_when_cli_width_is_missing() {
        assert_eq!(resolve_output_width(None, Some(80)).unwrap(), Some(80));
    }

    #[test]
    fn rejects_requested_width_larger_than_terminal() {
        let error = resolve_output_width(Some(100), Some(80)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "requested width 100 exceeds terminal width 80"
        );
    }

    #[test]
    fn falls_back_when_terminal_width_is_unavailable() {
        assert_eq!(resolve_output_width(None, None).unwrap(), None);
    }
}
