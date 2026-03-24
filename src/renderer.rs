use clap::ValueEnum;

use crate::error::RenderError;
use crate::font::{Font, Glyph};
use crate::formatter::align_line;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Theme {
    Plain,
    Outline,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    pub width: Option<usize>,
    pub alignment: Alignment,
    pub theme: Theme,
}

pub fn render(text: &str, font: &dyn Font, options: &RenderOptions) -> Result<String, RenderError> {
    if text.trim().is_empty() {
        return Err(RenderError::EmptyInput);
    }

    let mut rendered_lines = Vec::new();

    for line in text.lines() {
        rendered_lines.extend(render_text_line(line, font, options)?);
    }

    Ok(rendered_lines.join("\n"))
}

fn render_text_line(
    line: &str,
    font: &dyn Font,
    options: &RenderOptions,
) -> Result<Vec<String>, RenderError> {
    let height = font.height();
    let mut rows = vec![String::new(); height];

    for (index, ch) in line.chars().enumerate() {
        let glyph = font.glyph(ch);
        let themed = apply_theme(glyph, options.theme);

        for row_index in 0..height {
            if index > 0 {
                rows[row_index].push(' ');
            }
            rows[row_index].push_str(&themed[row_index]);
        }
    }

    rows.into_iter()
        .map(|line| align_line(&line, options.alignment, options.width))
        .collect()
}

fn apply_theme(glyph: &Glyph, theme: Theme) -> [String; 5] {
    let matrix = glyph.map(|row| row.chars().map(|ch| ch != ' ').collect::<Vec<_>>());

    std::array::from_fn(|row| {
        let mut rendered = String::with_capacity(matrix[row].len());
        for col in 0..matrix[row].len() {
            rendered.push(match theme {
                Theme::Plain => {
                    if matrix[row][col] {
                        '#'
                    } else {
                        ' '
                    }
                }
                Theme::Block => {
                    if matrix[row][col] {
                        '@'
                    } else {
                        ' '
                    }
                }
                Theme::Outline => outline_char(&matrix, row, col),
            });
        }
        rendered
    })
}

fn outline_char(matrix: &[Vec<bool>; 5], row: usize, col: usize) -> char {
    if !matrix[row][col] {
        return ' ';
    }

    let neighbors = [
        row.checked_sub(1).map(|r| matrix[r][col]).unwrap_or(false),
        matrix.get(row + 1).map(|next| next[col]).unwrap_or(false),
        col.checked_sub(1).map(|c| matrix[row][c]).unwrap_or(false),
        matrix[row].get(col + 1).copied().unwrap_or(false),
    ];

    if neighbors.into_iter().all(|filled| filled) {
        ' '
    } else {
        '#'
    }
}

#[cfg(test)]
mod tests {
    use super::{Alignment, RenderOptions, Theme, render};
    use crate::font::StandardFont;

    fn default_options() -> RenderOptions {
        RenderOptions {
            width: None,
            alignment: Alignment::Left,
            theme: Theme::Plain,
        }
    }

    #[test]
    fn renders_single_character_golden_output() {
        let output = render("A", &StandardFont, &default_options()).unwrap();
        assert_eq!(output, " ### \n#   #\n#####\n#   #\n#   #");
    }

    #[test]
    fn renders_multiple_characters_on_joined_rows() {
        let output = render("Hi", &StandardFont, &default_options()).unwrap();
        assert_eq!(output, "#   # #####\n#   #   #  \n#####   #  \n#   #   #  \n#   # #####");
    }

    #[test]
    fn preserves_newlines_between_rendered_blocks() {
        let output = render("Hi\nA", &StandardFont, &default_options()).unwrap();
        let expected = concat!(
            "#   # #####\n",
            "#   #   #  \n",
            "#####   #  \n",
            "#   #   #  \n",
            "#   # #####\n",
            " ### \n",
            "#   #\n",
            "#####\n",
            "#   #\n",
            "#   #"
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn unsupported_unicode_falls_back_to_question_mark() {
        let output = render("é", &StandardFont, &default_options()).unwrap();
        let expected = render("?", &StandardFont, &default_options()).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn block_theme_uses_different_fill_character() {
        let options = RenderOptions {
            theme: Theme::Block,
            ..default_options()
        };
        let output = render("A", &StandardFont, &options).unwrap();
        assert!(output.contains("@@@"));
    }

    #[test]
    fn center_alignment_applies_to_each_rendered_row() {
        let options = RenderOptions {
            width: Some(12),
            alignment: Alignment::Center,
            ..default_options()
        };
        let output = render("A", &StandardFont, &options).unwrap();
        assert_eq!(output.lines().next().unwrap(), "    ### ");
    }
}
