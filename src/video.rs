use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use clap::ValueEnum;

use crate::error::RenderError;
use crate::image_renderer::{ImageRenderOptions, render_rgb_frame, resolve_media_dimensions};
use crate::terminal::TerminalSession;

static INTERRUPTED: OnceLock<Arc<AtomicBool>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HwAccelMode {
    Auto,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoRenderOptions {
    pub width: Option<usize>,
    pub fps: u8,
    pub invert: bool,
    pub color: bool,
    pub loop_playback: bool,
    pub hwaccel: HwAccelMode,
}

pub fn play_video(path: &Path, options: &VideoRenderOptions) -> Result<(), RenderError> {
    ensure_dependency("ffmpeg")?;
    ensure_dependency("ffprobe")?;

    let interrupted = interrupted_flag()?;
    interrupted.store(false, Ordering::SeqCst);

    let (source_width, source_height) = probe_video_dimensions(path)?;
    let (target_width, target_height) = resolve_media_dimensions(
        options.width,
        source_width,
        source_height,
        crate::terminal::detect_terminal_width(),
        crate::terminal::detect_terminal_height(),
    )?
    .unwrap_or((source_width, source_height));
    let frame_len = expected_frame_len(target_width, target_height)?;
    let frame_options = ImageRenderOptions {
        width: Some(target_width as usize),
        invert: options.invert,
        color: options.color,
    };

    let mut session = TerminalSession::enter()?;

    loop {
        let mut child = spawn_ffmpeg(path, target_width, target_height, options)?;
        let finished = play_stream(
            path,
            &mut child,
            frame_len,
            target_width,
            target_height,
            &frame_options,
            Duration::from_secs_f32(1.0 / f32::from(options.fps)),
            &interrupted,
            &mut session,
        )?;

        if finished == PlaybackEnd::Quit || !options.loop_playback {
            break;
        }
    }

    Ok(())
}

fn play_stream(
    path: &Path,
    child: &mut Child,
    frame_len: usize,
    frame_width: u32,
    frame_height: u32,
    frame_options: &ImageRenderOptions,
    frame_duration: Duration,
    interrupted: &AtomicBool,
    session: &mut TerminalSession,
) -> Result<PlaybackEnd, RenderError> {
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| RenderError::VideoDecode {
            path: path.to_path_buf(),
            message: String::from("ffmpeg stdout was not available"),
        })?;
    let mut frame_buffer = vec![0; frame_len];
    let mut rendered_frames = 0usize;

    loop {
        if interrupted.load(Ordering::SeqCst) || session.quit_requested()? {
            terminate_child(child);
            return Ok(PlaybackEnd::Quit);
        }

        let frame_start = Instant::now();
        match read_frame(&mut stdout, &mut frame_buffer)? {
            FrameRead::Frame => {
                rendered_frames += 1;
                let frame =
                    render_rgb_frame(&frame_buffer, frame_width, frame_height, frame_options)?;
                session.draw_frame(&frame)?;
                sleep_remaining(frame_duration, frame_start);
            }
            FrameRead::Eof => {
                break;
            }
        }
    }

    let stderr = wait_for_child_stderr(child);
    if rendered_frames == 0 {
        return Err(RenderError::VideoDecode {
            path: path.to_path_buf(),
            message: if stderr.is_empty() {
                String::from("ffmpeg produced no frames")
            } else {
                stderr
            },
        });
    }

    Ok(PlaybackEnd::Completed)
}

fn read_frame(stdout: &mut ChildStdout, buffer: &mut [u8]) -> Result<FrameRead, RenderError> {
    let mut filled = 0usize;
    while filled < buffer.len() {
        match stdout.read(&mut buffer[filled..]) {
            Ok(0) if filled == 0 => return Ok(FrameRead::Eof),
            Ok(0) => {
                return Err(RenderError::VideoDecode {
                    path: PathBuf::new(),
                    message: String::from("ffmpeg ended mid-frame"),
                });
            }
            Ok(read) => filled += read,
            Err(error) => {
                return Err(RenderError::VideoDecode {
                    path: PathBuf::new(),
                    message: error.to_string(),
                });
            }
        }
    }

    Ok(FrameRead::Frame)
}

fn probe_video_dimensions(path: &Path) -> Result<(u32, u32), RenderError> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0:s=x",
        ])
        .arg(path)
        .output()
        .map_err(map_missing_or_probe(path))?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(RenderError::VideoProbe {
            path: path.to_path_buf(),
            message: if message.is_empty() {
                String::from("ffprobe could not inspect the input")
            } else {
                message
            },
        });
    }

    let dims = String::from_utf8_lossy(&output.stdout);
    let trimmed = dims.trim();
    let (width, height) = trimmed
        .split_once('x')
        .ok_or_else(|| RenderError::VideoProbe {
            path: path.to_path_buf(),
            message: format!("unexpected ffprobe output: '{trimmed}'"),
        })?;

    let width = width
        .parse::<u32>()
        .map_err(|error| RenderError::VideoProbe {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let height = height
        .parse::<u32>()
        .map_err(|error| RenderError::VideoProbe {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

    Ok((width, height))
}

fn spawn_ffmpeg(
    path: &Path,
    target_width: u32,
    target_height: u32,
    options: &VideoRenderOptions,
) -> Result<Child, RenderError> {
    let mut command = Command::new("ffmpeg");
    command
        .args(build_ffmpeg_args(
            path,
            target_width,
            target_height,
            options.fps,
            options.hwaccel,
        ))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    command.spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            RenderError::MissingDependency { name: "ffmpeg" }
        } else {
            RenderError::VideoDecode {
                path: path.to_path_buf(),
                message: error.to_string(),
            }
        }
    })
}

fn build_ffmpeg_args(
    path: &Path,
    target_width: u32,
    target_height: u32,
    fps: u8,
    hwaccel: HwAccelMode,
) -> Vec<String> {
    let mut args = vec![String::from("-loglevel"), String::from("error")];
    if hwaccel == HwAccelMode::Auto {
        args.push(String::from("-hwaccel"));
        args.push(String::from("auto"));
    }
    args.extend([
        String::from("-i"),
        path.display().to_string(),
        String::from("-vf"),
        format!("fps={fps},scale={target_width}:{target_height}"),
        String::from("-f"),
        String::from("rawvideo"),
        String::from("-pix_fmt"),
        String::from("rgb24"),
        String::from("-"),
    ]);
    args
}

fn ensure_dependency(name: &'static str) -> Result<(), RenderError> {
    Command::new(name)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                RenderError::MissingDependency { name }
            } else {
                RenderError::VideoDecode {
                    path: PathBuf::new(),
                    message: error.to_string(),
                }
            }
        })?;
    Ok(())
}

fn expected_frame_len(width: u32, height: u32) -> Result<usize, RenderError> {
    let pixels = u64::from(width) * u64::from(height);
    let bytes = pixels.saturating_mul(3);
    usize::try_from(bytes).map_err(|_| RenderError::InvalidImageDimensions { width, height })
}

fn wait_for_child_stderr(child: &mut Child) -> String {
    let _ = child.wait();

    let mut stderr = String::new();
    if let Some(mut handle) = child.stderr.take() {
        let _ = handle.read_to_string(&mut stderr);
    }

    stderr.trim().to_owned()
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn interrupted_flag() -> Result<Arc<AtomicBool>, RenderError> {
    if let Some(flag) = INTERRUPTED.get() {
        return Ok(Arc::clone(flag));
    }

    let flag = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&flag);
    ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::SeqCst);
    })
    .map_err(|error| RenderError::TerminalIo {
        message: error.to_string(),
    })?;

    let _ = INTERRUPTED.set(Arc::clone(&flag));
    Ok(flag)
}

fn sleep_remaining(frame_duration: Duration, started_at: Instant) {
    if let Some(remaining) = frame_duration.checked_sub(started_at.elapsed()) {
        thread::sleep(remaining);
    }
}

fn map_missing_or_probe(path: &Path) -> impl FnOnce(std::io::Error) -> RenderError + '_ {
    move |error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            RenderError::MissingDependency { name: "ffprobe" }
        } else {
            RenderError::VideoProbe {
                path: path.to_path_buf(),
                message: error.to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaybackEnd {
    Completed,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameRead {
    Frame,
    Eof,
}

#[cfg(test)]
mod tests {
    use super::{HwAccelMode, VideoRenderOptions, build_ffmpeg_args};
    use std::path::Path;

    #[test]
    fn ffmpeg_args_include_scaling_and_raw_rgb_output() {
        let args = build_ffmpeg_args(Path::new("clip.mp4"), 80, 24, 12, HwAccelMode::Auto);
        assert!(args.contains(&String::from("-hwaccel")));
        assert!(args.contains(&String::from("auto")));
        assert!(args.contains(&String::from("fps=12,scale=80:24")));
        assert!(args.contains(&String::from("rgb24")));
    }

    #[test]
    fn ffmpeg_args_skip_hwaccel_when_disabled() {
        let args = build_ffmpeg_args(Path::new("clip.mp4"), 80, 24, 12, HwAccelMode::None);
        assert!(!args.contains(&String::from("-hwaccel")));
    }

    #[test]
    fn video_options_can_disable_color() {
        let options = VideoRenderOptions {
            width: Some(80),
            fps: 12,
            invert: false,
            color: false,
            loop_playback: true,
            hwaccel: HwAccelMode::Auto,
        };
        assert_eq!(options.fps, 12);
        assert!(!options.color);
        assert!(options.loop_playback);
    }
}
