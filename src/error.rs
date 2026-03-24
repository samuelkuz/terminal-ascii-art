use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    EmptyInput,
    TerminalWidthExceeded {
        requested_width: usize,
        terminal_width: usize,
    },
    ContentWidthExceeded {
        width: usize,
        line_width: usize,
    },
}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "text input cannot be empty"),
            Self::TerminalWidthExceeded {
                requested_width,
                terminal_width,
            } => write!(
                f,
                "requested width {requested_width} exceeds terminal width {terminal_width}"
            ),
            Self::ContentWidthExceeded {
                width,
                line_width,
            } => write!(
                f,
                "rendered content width {line_width} exceeds available width {width}"
            ),
        }
    }
}

impl Error for RenderError {}
