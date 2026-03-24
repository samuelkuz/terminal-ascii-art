use crate::error::RenderError;
use crate::renderer::Alignment;

pub fn align_line(
    line: &str,
    alignment: Alignment,
    width: Option<usize>,
) -> Result<String, RenderError> {
    let Some(width) = width else {
        return Ok(line.to_owned());
    };

    let line_width = line.chars().count();
    if line_width > width {
        return Err(RenderError::ContentWidthExceeded { width, line_width });
    }

    let padding = match alignment {
        Alignment::Left => 0,
        Alignment::Center => (width - line_width) / 2,
        Alignment::Right => width - line_width,
    };

    Ok(format!("{}{line}", " ".repeat(padding)))
}

#[cfg(test)]
mod tests {
    use super::align_line;
    use crate::renderer::Alignment;

    #[test]
    fn left_alignment_without_width_is_identity() {
        let line = align_line("abc", Alignment::Left, None).unwrap();
        assert_eq!(line, "abc");
    }

    #[test]
    fn center_alignment_adds_padding() {
        let line = align_line("abc", Alignment::Center, Some(7)).unwrap();
        assert_eq!(line, "  abc");
    }

    #[test]
    fn right_alignment_adds_padding() {
        let line = align_line("abc", Alignment::Right, Some(7)).unwrap();
        assert_eq!(line, "    abc");
    }

    #[test]
    fn left_alignment_does_not_fail_for_small_width() {
        let error = align_line("abcdef", Alignment::Left, Some(3)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "rendered content width 6 exceeds available width 3"
        );
    }

    #[test]
    fn center_alignment_rejects_small_width() {
        let error = align_line("abcdef", Alignment::Center, Some(3)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "rendered content width 6 exceeds available width 3"
        );
    }
}
