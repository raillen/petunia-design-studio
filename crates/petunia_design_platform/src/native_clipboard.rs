//! Linux MIME clipboard adapter using the established compositor/X11 clients.
//! Call on a worker: transfers are bounded, cancellable, and time out. No shell
//! evaluation, display-server pointer or toolkit type crosses this boundary.
use crate::PlatformError;
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

pub const FRAGMENT_MIME: &str = "application/vnd.petunia-design-studio.fragment+zip";
pub const MAX_CLIPBOARD_BYTES: usize = 16 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinuxClipboardBackend {
    Wayland,
    X11,
}
impl LinuxClipboardBackend {
    pub fn detect() -> Result<Self, PlatformError> {
        if !cfg!(target_os = "linux") {
            return Err(PlatformError::Unsupported(
                "native object clipboard currently requires Linux".into(),
            ));
        }
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            Ok(Self::Wayland)
        } else if std::env::var_os("DISPLAY").is_some() {
            Ok(Self::X11)
        } else {
            Err(PlatformError::Unsupported(
                "no Wayland/X11 display is available".into(),
            ))
        }
    }
    pub fn text_mime(self) -> &'static str {
        match self {
            Self::Wayland => "text/plain;charset=utf-8",
            Self::X11 => "UTF8_STRING",
        }
    }
    fn command(self, mime: &str, write: bool) -> Command {
        match self {
            Self::Wayland => {
                let mut cmd = Command::new(if write { "wl-copy" } else { "wl-paste" });
                cmd.args(["--type", mime]);
                if !write {
                    cmd.arg("--no-newline");
                }
                cmd
            }
            Self::X11 => {
                let mut cmd = Command::new("xclip");
                cmd.args([
                    "-selection",
                    "clipboard",
                    "-target",
                    mime,
                    if write { "-in" } else { "-out" },
                ]);
                cmd
            }
        }
    }
    fn spawn_error(self, error: std::io::Error) -> PlatformError {
        PlatformError::ClipboardError(format!(
            "native clipboard client unavailable ({}, install {}): {error}",
            match self {
                Self::Wayland => "Wayland",
                Self::X11 => "X11",
            },
            match self {
                Self::Wayland => "wl-clipboard",
                Self::X11 => "xclip",
            }
        ))
    }
    pub fn write(
        self,
        mime: &str,
        bytes: Vec<u8>,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<(), PlatformError> {
        if bytes.len() > MAX_CLIPBOARD_BYTES {
            return Err(PlatformError::ClipboardError(
                "clipboard transfer byte budget exceeded".into(),
            ));
        }
        let mut child = self
            .command(mime, true)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| self.spawn_error(e))?;
        let mut input = child
            .stdin
            .take()
            .ok_or_else(|| PlatformError::ClipboardError("clipboard input unavailable".into()))?;
        let (send, receive) = mpsc::sync_channel(1);
        let writer = std::thread::spawn(move || {
            let result = input.write_all(&bytes);
            drop(input);
            let _ = send.send(result);
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        let result = loop {
            if cancelled() || Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                break Err(PlatformError::ClipboardError(
                    "clipboard transfer cancelled or timed out".into(),
                ));
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    break if status.success() {
                        Ok(())
                    } else {
                        Err(PlatformError::ClipboardError(
                            "clipboard owner rejected MIME transfer".into(),
                        ))
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(PlatformError::ClipboardError(e.to_string()));
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        // Clients read stdin before daemonizing, so killing the client unblocks
        // a pending pipe write. Persistent clipboard ownership is their job.
        writer
            .join()
            .map_err(|_| PlatformError::ClipboardError("clipboard writer failed".into()))?;
        receive
            .recv()
            .map_err(|_| PlatformError::ClipboardError("clipboard writer disconnected".into()))?
            .map_err(|e| PlatformError::ClipboardError(e.to_string()))?;
        result
    }
    pub fn available_types(
        self,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Vec<String>, PlatformError> {
        let command = match self {
            Self::Wayland => {
                let mut command = Command::new("wl-paste");
                command.arg("--list-types");
                command
            }
            Self::X11 => self.command("TARGETS", false),
        };
        let bytes = self.read_command(command, cancelled)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| PlatformError::ClipboardError("invalid clipboard type list".into()))?;
        if text.len() > 64 * 1024 {
            return Err(PlatformError::ClipboardError(
                "clipboard type list budget exceeded".into(),
            ));
        }
        Ok(text
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .take(1024)
            .map(str::to_owned)
            .collect())
    }
    pub fn read(self, mime: &str, cancelled: &dyn Fn() -> bool) -> Result<Vec<u8>, PlatformError> {
        self.read_command(self.command(mime, false), cancelled)
    }
    fn read_command(
        self,
        mut command: Command,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<Vec<u8>, PlatformError> {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| self.spawn_error(e))?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| PlatformError::ClipboardError("clipboard output unavailable".into()))?;
        let (send, receive) = mpsc::sync_channel(1);
        let reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = output
                .take((MAX_CLIPBOARD_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .map(|_| bytes);
            let _ = send.send(result);
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut received = None;
        let result = loop {
            if let Ok(value) = receive.try_recv() {
                received = Some(value);
            }
            if received
                .as_ref()
                .is_some_and(|r| r.as_ref().is_ok_and(|b| b.len() > MAX_CLIPBOARD_BYTES))
            {
                let _ = child.kill();
                let _ = child.wait();
                break Err(PlatformError::ClipboardError(
                    "clipboard byte budget exceeded".into(),
                ));
            }
            if cancelled() || Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                break Err(PlatformError::ClipboardError(
                    "clipboard transfer cancelled or timed out".into(),
                ));
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    break if status.success() {
                        Ok(())
                    } else {
                        Err(PlatformError::ClipboardError(
                            "clipboard MIME is unavailable".into(),
                        ))
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(PlatformError::ClipboardError(e.to_string()));
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        reader
            .join()
            .map_err(|_| PlatformError::ClipboardError("clipboard reader failed".into()))?;
        result?;
        let bytes = received
            .unwrap_or_else(|| {
                receive
                    .recv()
                    .unwrap_or_else(|_| Err(std::io::Error::other("clipboard reader disconnected")))
            })
            .map_err(|e| PlatformError::ClipboardError(e.to_string()))?;
        if bytes.len() > MAX_CLIPBOARD_BYTES {
            return Err(PlatformError::ClipboardError(
                "clipboard byte budget exceeded".into(),
            ));
        }
        Ok(bytes)
    }
}
