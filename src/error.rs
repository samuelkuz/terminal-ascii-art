use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
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
    FileRead {
        path: PathBuf,
        message: String,
    },
    ImageDecode {
        path: PathBuf,
        message: String,
    },
    InvalidImageDimensions {
        width: u32,
        height: u32,
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
            Self::ContentWidthExceeded { width, line_width } => write!(
                f,
                "rendered content width {line_width} exceeds available width {width}"
            ),
            Self::FileRead { path, message } => {
                write!(f, "failed to read image '{}': {message}", path.display())
            }
            Self::ImageDecode { path, message } => {
                write!(f, "failed to decode image '{}': {message}", path.display())
            }
            Self::InvalidImageDimensions { width, height } => {
                write!(f, "image dimensions {width}x{height} cannot be rendered")
            }
        }
    }
}

impl Error for RenderError {}
