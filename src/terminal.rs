use terminal_size::{Width, terminal_size};

pub fn detect_terminal_width() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width > 0)
        .or_else(|| terminal_size().map(|(Width(width), _)| usize::from(width)))
}
