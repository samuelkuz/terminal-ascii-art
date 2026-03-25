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
    MissingDependency {
        name: &'static str,
    },
    InvalidImageDimensions {
        width: u32,
        height: u32,
    },
    InvalidFrameBuffer {
        expected_len: usize,
        actual_len: usize,
    },
    VideoProbe {
        path: PathBuf,
        message: String,
    },
    VideoDecode {
        path: PathBuf,
        message: String,
    },
    TerminalIo {
        message: String,
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
            Self::MissingDependency { name } => {
                write!(f, "required dependency '{name}' was not found in PATH")
            }
            Self::InvalidImageDimensions { width, height } => {
                write!(f, "image dimensions {width}x{height} cannot be rendered")
            }
            Self::InvalidFrameBuffer {
                expected_len,
                actual_len,
            } => write!(
                f,
                "frame buffer length {actual_len} does not match expected RGB size {expected_len}"
            ),
            Self::VideoProbe { path, message } => {
                write!(f, "failed to inspect video '{}': {message}", path.display())
            }
            Self::VideoDecode { path, message } => {
                write!(f, "failed to decode video '{}': {message}", path.display())
            }
            Self::TerminalIo { message } => write!(f, "terminal I/O error: {message}"),
        }
    }
}

impl Error for RenderError {}
