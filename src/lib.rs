pub mod cli;
pub mod error;
pub mod font;
pub mod formatter;
pub mod image_renderer;
pub mod renderer;
pub mod terminal;

pub use error::RenderError;
pub use font::{Font, StandardFont};
pub use image_renderer::{ImageRenderOptions, render_image};
pub use renderer::{Alignment, RenderOptions, Theme, render};

use cli::{Cli, Commands};

pub fn run(cli: Cli) -> Result<String, RenderError> {
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

            render(text, &font, &options)
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

            render_image(&path, &options)
        }
    }
}

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
