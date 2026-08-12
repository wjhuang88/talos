//! Clipboard backend for TUI copy and export workflows.
//!
//! Prefers OSC 52 (terminal escape sequence) for cross-platform clipboard
//! writes. On macOS, where Terminal.app can silently ignore OSC 52, it uses
//! `pbcopy` first and retains OSC 52 as the fallback.
//!
//! OSC 52 spec: <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html>.
//! Format: `ESC ] 52 ; c ; <base64> BEL`.

use std::io::{self, Write};
use std::process::{Command, Stdio};

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes bytes using the standard base64 alphabet (RFC 4648 §4).
pub(crate) fn base64_encode(input: &[u8]) -> String {
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        output.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
        output.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(BASE64_ALPHABET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(BASE64_ALPHABET[(b2 & 0x3F) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

/// A clipboard payload encoded for the OSC 52 escape sequence.
pub(crate) struct ClipboardPayload {
    base64: String,
}

impl ClipboardPayload {
    /// Builds a payload from arbitrary text.
    pub(crate) fn new(text: &str) -> Self {
        Self {
            base64: base64_encode(text.as_bytes()),
        }
    }

    /// Formats the payload as an OSC 52 escape sequence (BEL-terminated).
    pub(crate) fn to_osc52(&self) -> String {
        format!("\x1b]52;c;{}\x07", self.base64)
    }
}

/// Which backend actually carried the clipboard write.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ClipboardBackend {
    /// Wrote via the OSC 52 escape sequence.
    Osc52,
    /// Wrote via the host `pbcopy` command.
    Pbcopy,
}

/// Errors a clipboard backend may surface.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ClipboardError {
    /// `pbcopy` is not installed on the host PATH.
    PbcopyNotFound,
    /// `pbcopy` exited with a non-zero status or failed to spawn.
    PbcopyFailed(String),
    /// Writing the OSC 52 escape to stdout failed.
    StdoutFailed(String),
}

/// Writes the OSC 52 escape sequence to stdout, flushing immediately so the
/// terminal emulator sees the sequence before any other terminal I/O.
pub(crate) fn write_osc52(text: &str) -> Result<(), ClipboardError> {
    let payload = ClipboardPayload::new(text);
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(payload.to_osc52().as_bytes())
        .map_err(|e| ClipboardError::StdoutFailed(e.to_string()))?;
    stdout
        .flush()
        .map_err(|e| ClipboardError::StdoutFailed(e.to_string()))?;
    Ok(())
}

/// Pipes `text` into a `pbcopy` child process and waits for it to exit.
pub(crate) fn write_pbcopy(text: &str) -> Result<(), ClipboardError> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                ClipboardError::PbcopyNotFound
            } else {
                ClipboardError::PbcopyFailed(e.to_string())
            }
        })?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| ClipboardError::PbcopyFailed(format!("stdin: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| ClipboardError::PbcopyFailed(e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(ClipboardError::PbcopyFailed(stderr))
    }
}

fn copy_text_with<O, P>(
    text: &str,
    prefer_pbcopy: bool,
    mut osc52: O,
    mut pbcopy: P,
) -> Result<ClipboardBackend, ClipboardError>
where
    O: FnMut(&str) -> Result<(), ClipboardError>,
    P: FnMut(&str) -> Result<(), ClipboardError>,
{
    if prefer_pbcopy {
        if pbcopy(text).is_ok() {
            return Ok(ClipboardBackend::Pbcopy);
        }
        osc52(text)?;
        return Ok(ClipboardBackend::Osc52);
    }

    if osc52(text).is_ok() {
        return Ok(ClipboardBackend::Osc52);
    }
    pbcopy(text)?;
    Ok(ClipboardBackend::Pbcopy)
}

/// Uses the reliable platform clipboard command first on macOS and OSC 52
/// first elsewhere, then falls back to the other existing backend.
pub(crate) fn copy_text(text: &str) -> Result<ClipboardBackend, ClipboardError> {
    copy_text_with(text, cfg!(target_os = "macos"), write_osc52, write_pbcopy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn base64_encode_rfc4648_fixtures() {
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_encode_handles_binary() {
        assert_eq!(base64_encode(&[0x00, 0x01, 0x02]), "AAEC");
        assert_eq!(base64_encode(&[0xFF, 0xFF, 0xFF]), "////");
    }

    #[test]
    fn osc52_format_is_bel_terminated() {
        let payload = ClipboardPayload::new("hello");
        let esc = payload.to_osc52();
        assert!(esc.starts_with("\x1b]52;c;"));
        assert!(esc.ends_with('\x07'));
        // base64("hello") = "aGVsbG8="
        assert_eq!(esc, "\x1b]52;c;aGVsbG8=\x07");
    }

    #[test]
    fn osc52_handles_empty_text() {
        let payload = ClipboardPayload::new("");
        assert_eq!(payload.to_osc52(), "\x1b]52;c;\x07");
    }

    #[test]
    fn osc52_handles_unicode() {
        // base64("中文") = "5Lit5paH"
        let payload = ClipboardPayload::new("中文");
        assert_eq!(payload.to_osc52(), "\x1b]52;c;5Lit5paH\x07");
    }

    #[test]
    fn macos_policy_prefers_pbcopy_without_emitting_osc52() {
        let mut osc52_called = false;
        let backend = copy_text_with(
            "hello",
            true,
            |_| {
                osc52_called = true;
                Ok(())
            },
            |_| Ok(()),
        )
        .expect("pbcopy should succeed");

        assert_eq!(backend, ClipboardBackend::Pbcopy);
        assert!(!osc52_called);
    }

    #[test]
    fn macos_policy_falls_back_to_osc52_when_pbcopy_fails() {
        let backend = copy_text_with(
            "hello",
            true,
            |_| Ok(()),
            |_| Err(ClipboardError::PbcopyNotFound),
        )
        .expect("OSC 52 fallback should succeed");

        assert_eq!(backend, ClipboardBackend::Osc52);
    }

    #[test]
    fn non_macos_policy_keeps_osc52_as_primary() {
        let mut pbcopy_called = false;
        let backend = copy_text_with(
            "hello",
            false,
            |_| Ok(()),
            |_| {
                pbcopy_called = true;
                Ok(())
            },
        )
        .expect("OSC 52 should succeed");

        assert_eq!(backend, ClipboardBackend::Osc52);
        assert!(!pbcopy_called);
    }
}
