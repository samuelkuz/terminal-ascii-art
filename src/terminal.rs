use std::io::{Stdout, Write, stdout};
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
    disable_raw_mode, enable_raw_mode, size,
};
use crossterm::{ExecutableCommand, QueueableCommand};
use terminal_size::{Height, Width, terminal_size};

use crate::error::RenderError;

pub fn detect_terminal_width() -> Option<usize> {
    detect_live_terminal_size()
        .map(|(width, _)| width)
        .or_else(detect_terminal_width_from_env)
        .or_else(|| terminal_size().map(|(Width(width), _)| usize::from(width)))
}

pub fn detect_terminal_height() -> Option<usize> {
    detect_live_terminal_size()
        .map(|(_, height)| height)
        .or_else(detect_terminal_height_from_env)
        .or_else(|| terminal_size().map(|(_, Height(height))| usize::from(height)))
}

fn detect_live_terminal_size() -> Option<(usize, usize)> {
    size()
        .ok()
        .map(|(width, height)| (usize::from(width), usize::from(height)))
        .filter(|(width, height)| *width > 0 && *height > 0)
}

fn detect_terminal_width_from_env() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width > 0)
}

fn detect_terminal_height_from_env() -> Option<usize> {
    std::env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|height| *height > 0)
}

pub struct TerminalSession {
    stdout: Stdout,
}

impl TerminalSession {
    pub fn enter() -> Result<Self, RenderError> {
        enable_raw_mode().map_err(|error| RenderError::TerminalIo {
            message: error.to_string(),
        })?;

        let mut stdout = stdout();
        stdout
            .execute(EnterAlternateScreen)
            .and_then(|writer| writer.execute(DisableLineWrap))
            .and_then(|writer| writer.execute(Hide))
            .and_then(|writer| writer.execute(Clear(ClearType::All)))
            .and_then(|writer| writer.flush())
            .map_err(|error| RenderError::TerminalIo {
                message: error.to_string(),
            })?;

        Ok(Self { stdout })
    }

    pub fn draw_frame(&mut self, frame: &str) -> Result<(), RenderError> {
        self.stdout
            .queue(MoveTo(0, 0))
            .and_then(|writer| writer.queue(Clear(ClearType::All)))
            .map_err(|error| RenderError::TerminalIo {
                message: error.to_string(),
            })?;

        for (row_index, line) in frame.lines().enumerate() {
            let row = u16::try_from(row_index).map_err(|_| RenderError::TerminalIo {
                message: String::from("frame exceeds terminal row addressing limits"),
            })?;

            self.stdout
                .queue(MoveTo(0, row))
                .and_then(|writer| writer.queue(Clear(ClearType::CurrentLine)))
                .map_err(|error| RenderError::TerminalIo {
                    message: error.to_string(),
                })?;

            self.stdout
                .write_all(line.as_bytes())
                .map_err(|error| RenderError::TerminalIo {
                    message: error.to_string(),
                })?;
        }

        self.stdout
            .flush()
            .map_err(|error| RenderError::TerminalIo {
                message: error.to_string(),
            })
    }

    pub fn quit_requested(&mut self) -> Result<bool, RenderError> {
        if !event::poll(Duration::from_millis(0)).map_err(|error| RenderError::TerminalIo {
            message: error.to_string(),
        })? {
            return Ok(false);
        }

        match event::read().map_err(|error| RenderError::TerminalIo {
            message: error.to_string(),
        })? {
            Event::Key(key_event)
                if key_event.kind == KeyEventKind::Press
                    && matches!(key_event.code, KeyCode::Char('q') | KeyCode::Char('Q')) =>
            {
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn restore(&mut self) -> Result<(), RenderError> {
        self.stdout
            .execute(Show)
            .and_then(|writer| writer.execute(EnableLineWrap))
            .and_then(|writer| writer.execute(LeaveAlternateScreen))
            .and_then(|writer| writer.flush())
            .map_err(|error| RenderError::TerminalIo {
                message: error.to_string(),
            })?;

        disable_raw_mode().map_err(|error| RenderError::TerminalIo {
            message: error.to_string(),
        })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
