//! Explicit host clipboard acquisition, isolated from the conversation process (ADR-100).
use anyhow::{Context, Result, bail};
use std::{collections::HashMap, io::Write, path::PathBuf, process::Stdio, time::Duration};
use talos_core::message::ContentPart;
use tokio::io::AsyncReadExt;

const TIMEOUT: Duration = Duration::from_secs(5);
const FILE_LIST_LIMIT: usize = 64 * 1024;

#[derive(Debug)]
pub(crate) enum Capture {
    Files(Vec<PathBuf>),
    Png(Vec<u8>),
    Authorized(ContentPart),
}

/// Run only in a disposable child. Never read clipboard file paths here.
pub(crate) fn worker() -> Result<()> {
    let result = std::panic::catch_unwind(|| -> Result<()> {
        let mut clipboard = arboard::Clipboard::new().context("clipboard unavailable")?;
        match clipboard.get().file_list() {
            Ok(files) if !files.is_empty() => {
                let bytes = serde_json::to_vec(&files)?;
                anyhow::ensure!(
                    bytes.len() <= FILE_LIST_LIMIT,
                    "clipboard file list too large"
                );
                let mut out = std::io::stdout().lock();
                out.write_all(b"F")?;
                out.write_all(&bytes)?;
                return Ok(());
            }
            Ok(_) | Err(arboard::Error::ContentNotAvailable) => {}
            Err(error) => bail!("clipboard file list unavailable: {error}"),
        }
        let image = clipboard.get_image().map_err(|error| match error {
            arboard::Error::ContentNotAvailable => anyhow::anyhow!("no readable image"),
            arboard::Error::ClipboardOccupied => anyhow::anyhow!("clipboard busy"),
            _ => anyhow::anyhow!("clipboard unavailable"),
        })?;
        let width = u32::try_from(image.width)?;
        let height = u32::try_from(image.height)?;
        let pixels = u64::from(width) * u64::from(height);
        anyhow::ensure!(
            pixels > 0 && pixels <= crate::image_validation::max_pixels(),
            "clipboard image pixel limit exceeded"
        );
        anyhow::ensure!(
            pixels.checked_mul(4) == Some(image.bytes.len() as u64),
            "invalid clipboard RGBA buffer"
        );
        let mut bytes = LimitedWriter(Vec::new());
        use image::ImageEncoder;
        image::codecs::png::PngEncoder::new(&mut bytes).write_image(
            &image.bytes,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )?;
        let mut out = std::io::stdout().lock();
        out.write_all(b"P")?;
        out.write_all(&bytes.0)?;
        Ok(())
    })
    .unwrap_or_else(|_| Err(anyhow::anyhow!("clipboard dependency panicked")));
    if let Err(error) = result {
        let text = error.to_string();
        let kind = if text.contains("no readable image") {
            "no image on clipboard"
        } else if text.contains("busy") {
            "clipboard busy; try again"
        } else if text.contains("unavailable") {
            "clipboard unavailable"
        } else {
            "clipboard image acquisition failed"
        };
        std::io::stdout().write_all(format!("E{kind}").as_bytes())?;
    }
    Ok(())
}

struct LimitedWriter(Vec<u8>);
impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) as u64
            > crate::image_validation::max_single_image_bytes()
        {
            return Err(std::io::Error::other(
                "clipboard PNG exceeds image byte limit",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) async fn acquire(cancel: tokio_util::sync::CancellationToken) -> Result<Capture> {
    let executable = std::env::current_exe().context("locating clipboard worker")?;
    let mut command = tokio::process::Command::new(executable);
    command
        .arg("clipboard-image-worker")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn().context("starting clipboard worker")?;
    let output = child
        .stdout
        .take()
        .context("clipboard worker stdout unavailable")?;
    let operation = tokio::time::timeout(TIMEOUT, async {
        let mut bytes = Vec::new();
        output
            .take(crate::image_validation::max_single_image_bytes() + 2)
            .read_to_end(&mut bytes)
            .await?;
        anyhow::ensure!(
            bytes.len() as u64 <= crate::image_validation::max_single_image_bytes() + 1,
            "clipboard output exceeds limit"
        );
        let status = child.wait().await?;
        anyhow::ensure!(status.success(), "clipboard worker failed");
        parse_capture(bytes)
    });
    let result = tokio::select! {
        result = operation => result,
        _ = cancel.cancelled() => Ok(Err(anyhow::anyhow!("clipboard read cancelled"))),
    };
    match result {
        Ok(Ok(capture)) => Ok(capture),
        failure => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            match failure {
                Ok(Err(error)) => Err(error),
                Err(_) => bail!("clipboard read timed out; try again"),
                Ok(Ok(_)) => unreachable!(),
            }
        }
    }
}

fn parse_capture(mut bytes: Vec<u8>) -> Result<Capture> {
    anyhow::ensure!(!bytes.is_empty(), "empty clipboard response");
    match bytes.remove(0) {
        b'F' => {
            anyhow::ensure!(
                bytes.len() <= FILE_LIST_LIMIT,
                "clipboard file list too large"
            );
            let files: Vec<PathBuf> = serde_json::from_slice(&bytes)?;
            anyhow::ensure!(
                !files.is_empty() && files.len() <= 64,
                "invalid clipboard file count"
            );
            Ok(Capture::Files(files))
        }
        b'P' => Ok(Capture::Png(bytes)),
        b'E' if bytes.len() <= 128 => bail!("{}", String::from_utf8_lossy(&bytes)),
        _ => bail!("invalid clipboard response"),
    }
}

#[derive(Default)]
pub(crate) struct Staging {
    files: HashMap<PathBuf, tempfile::NamedTempFile>,
}
impl Staging {
    pub(crate) fn stage(&mut self, bytes: &[u8], count: usize, total: u64) -> Result<ContentPart> {
        let root = dirs::home_dir()
            .context("home directory unavailable")?
            .join(".talos")
            .join("clipboard");
        self.stage_in(&root, bytes, count, total)
    }
    fn stage_in(
        &mut self,
        root: &std::path::Path,
        bytes: &[u8],
        count: usize,
        total: u64,
    ) -> Result<ContentPart> {
        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(root)?;
        let root = root.canonicalize()?;
        let mut file = tempfile::Builder::new()
            .prefix("image-")
            .suffix(".png")
            .tempfile_in(root)?;
        file.write_all(bytes)?;
        file.as_file().sync_all()?;
        let part = crate::image_validation::create_image_content_part(file.path(), count, total)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        self.files.insert(file.path().to_path_buf(), file);
        Ok(part)
    }
    pub(crate) fn retain_pending(&mut self, parts: &[ContentPart]) {
        self.files.retain(|path, _| {
            parts
                .iter()
                .any(|part| matches!(part, ContentPart::Image {path: p, ..} if p == path))
        });
    }
    pub(crate) fn commit(&mut self, parts: &[ContentPart]) {
        for part in parts {
            if let ContentPart::Image { path, .. } = part
                && let Some(mut file) = self.files.remove(path)
            {
                file.disable_cleanup(true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_rejects_empty_unknown_and_unbounded_file_lists() {
        assert!(parse_capture(vec![]).is_err());
        assert!(parse_capture(b"Xdata".to_vec()).is_err());
        assert!(parse_capture(b"F[]".to_vec()).is_err());
        assert!(matches!(
            parse_capture(b"F[\"/tmp/image.png\"]".to_vec()).unwrap(),
            Capture::Files(_)
        ));
    }
    #[test]
    fn staged_files_are_removed_unless_submission_commits_them() {
        let root = tempfile::tempdir().unwrap();
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(2, 2)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let mut staging = Staging::default();
        let part = staging
            .stage_in(root.path(), bytes.get_ref(), 0, 0)
            .unwrap();
        let ContentPart::Image { path, .. } = &part else {
            panic!("image")
        };
        assert!(path.exists());
        staging.retain_pending(&[]);
        assert!(!path.exists());
        let part = staging
            .stage_in(root.path(), bytes.get_ref(), 0, 0)
            .unwrap();
        let ContentPart::Image { path, .. } = &part else {
            panic!("image")
        };
        staging.commit(std::slice::from_ref(&part));
        drop(staging);
        assert!(path.exists(), "committed image survives composer shutdown");
    }
    #[test]
    fn invalid_stage_is_cleaned_and_existing_limits_apply() {
        let root = tempfile::tempdir().unwrap();
        let mut staging = Staging::default();
        assert!(staging.stage_in(root.path(), b"invalid", 0, 0).is_err());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        assert!(
            staging
                .stage_in(
                    root.path(),
                    b"invalid",
                    crate::image_validation::max_image_count(),
                    0
                )
                .is_err()
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn png_encoder_sink_is_bounded() {
        let mut writer = LimitedWriter(Vec::new());
        assert!(
            writer
                .write_all(&vec![
                    0;
                    crate::image_validation::max_single_image_bytes()
                        as usize
                        + 1
                ])
                .is_err()
        );
        assert!(writer.0.is_empty());
    }
}
