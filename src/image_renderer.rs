use std::path::Path;

use image::imageops::FilterType;

use crate::error::RenderError;

const ASCII_RAMP: &[u8] = b"@%#*+=-:. ";
const TERMINAL_ASPECT_RATIO: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageRenderOptions {
    pub width: Option<usize>,
    pub invert: bool,
    pub color: bool,
}

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
    let target_width = options.width.unwrap_or_else(|| source_width as usize);
    let target_width =
        u32::try_from(target_width).map_err(|_| RenderError::InvalidImageDimensions {
            width: source_width,
            height: source_height,
        })?;
    let target_height = calculate_target_height(source_width, source_height, target_width)?;

    let resized = image
        .resize_exact(target_width, target_height, FilterType::Triangle)
        .into_rgb8();

    let mut lines = Vec::with_capacity(resized.height() as usize);
    for y in 0..resized.height() {
        let mut line = String::with_capacity(resized.width() as usize);
        for x in 0..resized.width() {
            let pixel = resized.get_pixel(x, y);
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

fn calculate_target_height(
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
        .round() as u32;

    if scaled_height == 0 {
        return Err(RenderError::InvalidImageDimensions {
            width: source_width,
            height: source_height,
        });
    }

    Ok(scaled_height)
}

fn pixel_brightness(red: u8, green: u8, blue: u8) -> u8 {
    (0.299 * red as f32 + 0.587 * green as f32 + 0.114 * blue as f32).round() as u8
}

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
        map_brightness_to_char, pixel_brightness,
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
        assert_eq!(calculate_target_height(200, 100, 80).unwrap(), 20);
        assert_eq!(calculate_target_height(100, 200, 80).unwrap(), 80);
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
    fn small_height_that_rounds_to_zero_is_rejected() {
        let error = calculate_target_height(1000, 1, 1).unwrap_err();
        assert_eq!(
            error,
            RenderError::InvalidImageDimensions {
                width: 1000,
                height: 1,
            }
        );
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
}
