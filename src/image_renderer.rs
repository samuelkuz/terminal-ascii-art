use std::path::Path;

use image::imageops::FilterType;

use crate::error::RenderError;

const ASCII_RAMP: &[u8] = b"@%#*+=-:. ";
const TERMINAL_ASPECT_RATIO: f32 = 0.45;

/// Options controlling image and RGB-frame rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageRenderOptions {
    /// Preferred output width in columns.
    pub width: Option<usize>,
    /// Whether bright pixels should map to dense characters instead of sparse ones.
    pub invert: bool,
    /// Whether to emit 24-bit ANSI foreground color escape sequences.
    pub color: bool,
}

/// Loads an image from disk, resizes it for the current terminal, and renders ASCII output.
pub fn render_image(path: &Path, options: &ImageRenderOptions) -> Result<String, RenderError> {
    let reader = image::ImageReader::open(path).map_err(|error| RenderError::FileRead {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let image = reader.decode().map_err(|error| RenderError::ImageDecode {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    let (source_width, source_height) = (image.width(), image.height());
    let (target_width, target_height) = resolve_media_dimensions(
        options.width,
        source_width,
        source_height,
        crate::terminal::detect_terminal_width(),
        crate::terminal::detect_terminal_height(),
    )?
    .unwrap_or((source_width, source_height));

    let resized = image
        .resize_exact(target_width, target_height, FilterType::Triangle)
        .into_rgb8();
    render_rgb_frame(resized.as_raw(), resized.width(), resized.height(), options)
}

/// Resolves the target media dimensions using explicit width and terminal bounds when available.
pub fn resolve_media_dimensions(
    requested_width: Option<usize>,
    source_width: u32,
    source_height: u32,
    terminal_width: Option<usize>,
    terminal_height: Option<usize>,
) -> Result<Option<(u32, u32)>, RenderError> {
    match requested_width {
        Some(requested_width) => {
            if let Some(terminal_width) = terminal_width {
                if requested_width > terminal_width {
                    return Err(RenderError::TerminalWidthExceeded {
                        requested_width,
                        terminal_width,
                    });
                }
            }
            let max_width = u32::try_from(requested_width).map_err(|_| {
                RenderError::InvalidImageDimensions {
                    width: source_width,
                    height: source_height,
                }
            })?;

            if let Some(terminal_height) = terminal_height {
                return fit_media_within_bounds(
                    source_width,
                    source_height,
                    max_width,
                    usable_terminal_height(terminal_height) as u32,
                )
                .map(Some);
            }

            let target_height = calculate_target_height(source_width, source_height, max_width)?;
            Ok(Some((max_width, target_height)))
        }
        None => match (terminal_width, terminal_height) {
            (Some(terminal_width), Some(terminal_height)) => {
                let available_width = u32::try_from(terminal_width).map_err(|_| {
                    RenderError::InvalidImageDimensions {
                        width: source_width,
                        height: source_height,
                    }
                })?;
                let available_height = usable_terminal_height(terminal_height) as u32;
                fit_media_within_bounds(
                    source_width,
                    source_height,
                    available_width,
                    available_height,
                )
                .map(Some)
            }
            (Some(terminal_width), None) => {
                let target_width = u32::try_from(terminal_width).map_err(|_| {
                    RenderError::InvalidImageDimensions {
                        width: source_width,
                        height: source_height,
                    }
                })?;
                let target_height =
                    calculate_target_height(source_width, source_height, target_width)?;
                Ok(Some((target_width, target_height)))
            }
            (None, Some(terminal_height)) => {
                let target_height = usable_terminal_height(terminal_height) as u32;
                let target_width =
                    calculate_target_width(source_width, source_height, target_height)?;
                Ok(Some((target_width, target_height)))
            }
            (None, None) => Ok(None),
        },
    }
}

/// Resolves only the target media width using the same sizing rules as [`resolve_media_dimensions`].
pub fn resolve_media_width(
    requested_width: Option<usize>,
    source_width: u32,
    source_height: u32,
    terminal_width: Option<usize>,
    terminal_height: Option<usize>,
) -> Result<Option<usize>, RenderError> {
    Ok(resolve_media_dimensions(
        requested_width,
        source_width,
        source_height,
        terminal_width,
        terminal_height,
    )?
    .map(|(width, _)| width as usize))
}

/// Calculates the output height for a target width while compensating for terminal cell aspect ratio.
pub fn calculate_target_height(
    source_width: u32,
    source_height: u32,
    target_width: u32,
) -> Result<u32, RenderError> {
    if source_width == 0 || source_height == 0 || target_width == 0 {
        return Err(RenderError::InvalidImageDimensions {
            width: source_width,
            height: source_height,
        });
    }

    let scaled_height = ((source_height as f32 / source_width as f32)
        * target_width as f32
        * TERMINAL_ASPECT_RATIO)
        .floor()
        .max(1.0) as u32;

    Ok(scaled_height)
}

/// Scales media to fit within both width and height bounds while preserving aspect ratio.
fn fit_media_within_bounds(
    source_width: u32,
    source_height: u32,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32), RenderError> {
    if source_width == 0 || source_height == 0 || max_width == 0 || max_height == 0 {
        return Err(RenderError::InvalidImageDimensions {
            width: source_width,
            height: source_height,
        });
    }

    let width_scale = max_width as f32 / source_width as f32;
    let height_scale = max_height as f32 / (source_height as f32 * TERMINAL_ASPECT_RATIO);
    let scale = width_scale.min(height_scale);

    let target_width = (source_width as f32 * scale).floor() as u32;
    let target_height = (source_height as f32 * scale * TERMINAL_ASPECT_RATIO)
        .floor()
        .max(1.0) as u32;
    let target_width = target_width.max(1);

    Ok((target_width, target_height))
}

/// Calculates the output width for a target height while compensating for terminal cell aspect ratio.
fn calculate_target_width(
    source_width: u32,
    source_height: u32,
    target_height: u32,
) -> Result<u32, RenderError> {
    if source_width == 0 || source_height == 0 || target_height == 0 {
        return Err(RenderError::InvalidImageDimensions {
            width: source_width,
            height: source_height,
        });
    }

    let scaled_width = ((target_height as f32)
        / ((source_height as f32 / source_width as f32) * TERMINAL_ASPECT_RATIO))
        .floor()
        .max(1.0) as u32;

    Ok(scaled_width)
}

/// Renders a raw RGB24 frame buffer into ASCII art, optionally preserving pixel color.
pub fn render_rgb_frame(
    frame: &[u8],
    width: u32,
    height: u32,
    options: &ImageRenderOptions,
) -> Result<String, RenderError> {
    let expected_len = expected_rgb_buffer_len(width, height)?;
    if frame.len() != expected_len {
        return Err(RenderError::InvalidFrameBuffer {
            expected_len,
            actual_len: frame.len(),
        });
    }

    let mut lines = Vec::with_capacity(height as usize);
    for row in frame.chunks_exact((width as usize) * 3) {
        let mut line = String::with_capacity(width as usize);
        for pixel in row.chunks_exact(3) {
            let brightness = pixel_brightness(pixel[0], pixel[1], pixel[2]);
            let character = map_brightness_to_char(brightness, options.invert);

            if options.color {
                line.push_str(&format_colored_char(
                    character, pixel[0], pixel[1], pixel[2],
                ));
            } else {
                line.push(character);
            }
        }
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

/// Computes the expected byte length for an RGB24 frame buffer.
fn expected_rgb_buffer_len(width: u32, height: u32) -> Result<usize, RenderError> {
    let pixels = u64::from(width) * u64::from(height);
    let bytes = pixels.saturating_mul(3);
    usize::try_from(bytes).map_err(|_| RenderError::InvalidImageDimensions { width, height })
}

/// Reserves one terminal row for the prompt when auto-fitting media height.
fn usable_terminal_height(terminal_height: usize) -> usize {
    terminal_height.saturating_sub(1).max(1)
}

/// Converts an RGB pixel to perceived luma for ASCII ramp mapping.
fn pixel_brightness(red: u8, green: u8, blue: u8) -> u8 {
    (0.299 * red as f32 + 0.587 * green as f32 + 0.114 * blue as f32).round() as u8
}

/// Maps a brightness value to a character in the configured ASCII ramp.
fn map_brightness_to_char(brightness: u8, invert: bool) -> char {
    let normalized = brightness as f32 / u8::MAX as f32;
    let index = (normalized * (ASCII_RAMP.len() - 1) as f32).round() as usize;
    let mapped_index = if invert {
        ASCII_RAMP.len() - 1 - index
    } else {
        index
    };

    ASCII_RAMP[mapped_index] as char
}

/// Wraps a non-space ASCII character in a 24-bit ANSI foreground color sequence.
fn format_colored_char(character: char, red: u8, green: u8, blue: u8) -> String {
    if character == ' ' {
        return String::from(" ");
    }

    format!("\x1b[38;2;{red};{green};{blue}m{character}\x1b[0m")
}

#[cfg(test)]
mod tests {
    use super::{
        ASCII_RAMP, ImageRenderOptions, calculate_target_height, format_colored_char,
        map_brightness_to_char, pixel_brightness, render_rgb_frame, resolve_media_dimensions,
        resolve_media_width,
    };
    use crate::error::RenderError;

    #[test]
    fn brightness_mapping_uses_expected_ramp() {
        assert_eq!(map_brightness_to_char(0, false), ASCII_RAMP[0] as char);
        assert_eq!(
            map_brightness_to_char(u8::MAX, false),
            ASCII_RAMP[ASCII_RAMP.len() - 1] as char
        );
    }

    #[test]
    fn invert_flips_brightness_mapping() {
        assert_eq!(map_brightness_to_char(0, true), ' ');
        assert_eq!(map_brightness_to_char(u8::MAX, true), '@');
    }

    #[test]
    fn aspect_ratio_height_is_stable() {
        assert_eq!(calculate_target_height(200, 100, 80).unwrap(), 18);
        assert_eq!(calculate_target_height(100, 200, 80).unwrap(), 72);
    }

    #[test]
    fn auto_width_respects_terminal_height() {
        assert_eq!(
            resolve_media_width(None, 640, 360, Some(120), Some(24)).unwrap(),
            Some(90)
        );
    }

    #[test]
    fn portrait_video_fits_full_terminal_height() {
        assert_eq!(
            resolve_media_dimensions(None, 1080, 1920, Some(80), Some(24)).unwrap(),
            Some((28, 23))
        );
    }

    #[test]
    fn explicit_width_shrinks_when_portrait_media_would_overflow_height() {
        assert_eq!(
            resolve_media_dimensions(Some(80), 1080, 1920, Some(80), Some(24)).unwrap(),
            Some((28, 23))
        );
    }

    #[test]
    fn explicit_width_still_validates_terminal_width() {
        let error = resolve_media_width(Some(100), 640, 360, Some(80), Some(24)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "requested width 100 exceeds terminal width 80"
        );
    }

    #[test]
    fn zero_dimension_resize_is_rejected() {
        let error = calculate_target_height(10, 10, 0).unwrap_err();
        assert_eq!(
            error,
            RenderError::InvalidImageDimensions {
                width: 10,
                height: 10,
            }
        );
    }

    #[test]
    fn small_height_clamps_to_single_row() {
        assert_eq!(calculate_target_height(1000, 1, 1).unwrap(), 1);
    }

    #[test]
    fn options_can_disable_inversion() {
        let options = ImageRenderOptions {
            width: Some(80),
            invert: false,
            color: false,
        };
        assert_eq!(options.width, Some(80));
        assert!(!options.invert);
        assert!(!options.color);
    }

    #[test]
    fn brightness_uses_weighted_rgb_luma() {
        assert_eq!(pixel_brightness(255, 0, 0), 76);
        assert_eq!(pixel_brightness(0, 255, 0), 150);
        assert_eq!(pixel_brightness(0, 0, 255), 29);
    }

    #[test]
    fn colored_char_wraps_character_with_truecolor_escape() {
        assert_eq!(
            format_colored_char('#', 1, 2, 3),
            "\u{1b}[38;2;1;2;3m#\u{1b}[0m"
        );
    }

    #[test]
    fn colored_spaces_stay_plain_spaces() {
        assert_eq!(format_colored_char(' ', 1, 2, 3), " ");
    }

    #[test]
    fn rgb_frame_renderer_supports_grayscale_output() {
        let options = ImageRenderOptions {
            width: Some(2),
            invert: false,
            color: false,
        };
        let frame = [0, 0, 0, 255, 255, 255];

        assert_eq!(render_rgb_frame(&frame, 2, 1, &options).unwrap(), "@ ");
    }

    #[test]
    fn rgb_frame_renderer_supports_color_output() {
        let options = ImageRenderOptions {
            width: Some(1),
            invert: false,
            color: true,
        };
        let frame = [255, 0, 0];

        assert_eq!(
            render_rgb_frame(&frame, 1, 1, &options).unwrap(),
            "\u{1b}[38;2;255;0;0m*\u{1b}[0m"
        );
    }

    #[test]
    fn rgb_frame_renderer_rejects_incomplete_buffers() {
        let options = ImageRenderOptions {
            width: Some(1),
            invert: false,
            color: false,
        };
        let error = render_rgb_frame(&[255, 0], 1, 1, &options).unwrap_err();
        assert_eq!(
            error,
            RenderError::InvalidFrameBuffer {
                expected_len: 3,
                actual_len: 2,
            }
        );
    }
}
