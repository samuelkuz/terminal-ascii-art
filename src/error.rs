use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Errors that can occur while validating input or rendering terminal output.
#[derive(Debug, PartialEq, Eq)]
pub enum RenderError {
    /// The supplied text input was empty after trimming whitespace.
    EmptyInput,
    /// The requested render width is wider than the detected terminal.
    TerminalWidthExceeded {
        requested_width: usize,
        terminal_width: usize,
    },
    /// Rendered content could not fit within the requested width.
    ContentWidthExceeded { width: usize, line_width: usize },
    /// Opening or reading an image file failed.
    FileRead { path: PathBuf, message: String },
    /// Decoding a supported image file failed.
    ImageDecode { path: PathBuf, message: String },
    /// An external binary required for video playback was not available.
    MissingDependency { name: &'static str },
    /// Source or target media dimensions were zero or overflowed supported bounds.
    InvalidImageDimensions { width: u32, height: u32 },
    /// A raw RGB frame buffer had an unexpected byte length.
    InvalidFrameBuffer {
        expected_len: usize,
        actual_len: usize,
    },
    /// Probing source video metadata with `ffprobe` failed.
    VideoProbe { path: PathBuf, message: String },
    /// Decoding or streaming video frames with `ffmpeg` failed.
    VideoDecode { path: PathBuf, message: String },
    /// Terminal setup, drawing, or teardown failed.
    TerminalIo { message: String },
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
