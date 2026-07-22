//! Shared image validation for safe local image ingestion (ADR-050/051).
//!
//! Used by both `talos-cli` (`/attach`, `--attach`) and `talos-tools`
//! (`read_image` tool). Provides bounded read, MIME detection, pixel
//! limit, decoder panic containment, and SHA-256 digest creation.

use std::path::{Path, PathBuf};

pub const MAX_SINGLE_IMAGE_BYTES: u64 = 20_971_520;
pub const MAX_TOTAL_IMAGE_BYTES: u64 = 52_428_800;
pub const MAX_PIXELS: u64 = 89_478_485;
pub const MAX_IMAGE_COUNT: usize = 4;

const SUPPORTED_MIME_TYPES: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];

#[derive(Debug)]
pub enum ImageValidationError {
    NotRegularFile,
    Directory,
    EmptyFile,
    Oversize { size: u64, limit: u64 },
    AggregateLimitExceeded { total: u64, limit: u64 },
    TooManyAttachments { count: usize, limit: usize },
    UnsupportedFormat,
    MagicByteMismatch { expected: String, found: String },
    PixelLimitExceeded { pixels: u64, limit: u64 },
    IoError(String),
    DecoderError(String),
    DecoderPanic(String),
}

impl std::fmt::Display for ImageValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRegularFile => write!(f, "Path is not a regular file"),
            Self::Directory => write!(f, "Path is a directory"),
            Self::EmptyFile => write!(f, "File is empty"),
            Self::Oversize { size, limit } => {
                write!(f, "File size {size} bytes exceeds limit {limit} bytes")
            }
            Self::AggregateLimitExceeded { total, limit } => {
                write!(
                    f,
                    "Total image size {total} bytes exceeds aggregate limit {limit} bytes"
                )
            }
            Self::TooManyAttachments { count, limit } => {
                write!(f, "Number of attachments {count} exceeds limit {limit}")
            }
            Self::UnsupportedFormat => write!(
                f,
                "Unsupported image format. Supported: PNG, JPEG, GIF, WebP"
            ),
            Self::MagicByteMismatch { expected, found } => {
                write!(f, "Magic byte mismatch: expected {expected}, found {found}")
            }
            Self::PixelLimitExceeded { pixels, limit } => {
                write!(f, "Pixel count {pixels} exceeds limit {limit}")
            }
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
            Self::DecoderError(msg) => write!(f, "Decoder error: {msg}"),
            Self::DecoderPanic(msg) => write!(f, "Decoder panic: {msg}"),
        }
    }
}

pub fn detect_mime_from_magic_bytes(data: &[u8]) -> Option<&'static str> {
    if data.len() >= 8 && data[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some("image/png");
    }
    if data.len() >= 3 && data[..3] == [0xFF, 0xD8, 0xFF] {
        return Some("image/jpeg");
    }
    if data.len() >= 6 && (data[..6] == *b"GIF87a" || data[..6] == *b"GIF89a") {
        return Some("image/gif");
    }
    if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}

pub fn is_supported_mime(mime: &str) -> bool {
    SUPPORTED_MIME_TYPES.contains(&mime)
}

pub fn validate_image_path(
    path: &Path,
    current_count: usize,
    current_total_bytes: u64,
) -> Result<ValidatedImage, ImageValidationError> {
    if current_count >= MAX_IMAGE_COUNT {
        return Err(ImageValidationError::TooManyAttachments {
            count: current_count,
            limit: MAX_IMAGE_COUNT,
        });
    }

    let metadata =
        std::fs::metadata(path).map_err(|e| ImageValidationError::IoError(e.to_string()))?;

    if metadata.is_dir() {
        return Err(ImageValidationError::Directory);
    }
    if !metadata.is_file() {
        return Err(ImageValidationError::NotRegularFile);
    }
    if metadata.len() == 0 {
        return Err(ImageValidationError::EmptyFile);
    }

    let canonical = path.to_path_buf();

    use std::io::Read;
    let file = std::fs::File::open(&canonical)
        .map_err(|e| ImageValidationError::IoError(e.to_string()))?;
    let mut bytes = Vec::new();
    file.take(MAX_SINGLE_IMAGE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ImageValidationError::IoError(e.to_string()))?;

    let file_size = bytes.len() as u64;
    if file_size == 0 {
        return Err(ImageValidationError::EmptyFile);
    }
    if file_size > MAX_SINGLE_IMAGE_BYTES {
        return Err(ImageValidationError::Oversize {
            size: file_size,
            limit: MAX_SINGLE_IMAGE_BYTES,
        });
    }

    let new_total = current_total_bytes + file_size;
    if new_total > MAX_TOTAL_IMAGE_BYTES {
        return Err(ImageValidationError::AggregateLimitExceeded {
            total: new_total,
            limit: MAX_TOTAL_IMAGE_BYTES,
        });
    }

    let header = &bytes[..bytes.len().min(16)];
    let mime =
        detect_mime_from_magic_bytes(header).ok_or(ImageValidationError::UnsupportedFormat)?;

    if !is_supported_mime(mime) {
        return Err(ImageValidationError::UnsupportedFormat);
    }

    let (width, height) = decode_dimensions_from_bytes(&bytes)?;

    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_PIXELS {
        return Err(ImageValidationError::PixelLimitExceeded {
            pixels,
            limit: MAX_PIXELS,
        });
    }

    Ok(ValidatedImage {
        canonical,
        byte_count: file_size,
        mime: mime.to_string(),
        bytes,
    })
}

#[derive(Debug)]
pub struct ValidatedImage {
    pub canonical: PathBuf,
    pub byte_count: u64,
    pub mime: String,
    pub bytes: Vec<u8>,
}

fn decode_dimensions_from_bytes(bytes: &[u8]) -> Result<(u32, u32), ImageValidationError> {
    use std::io::Cursor;
    let result = std::panic::catch_unwind(|| {
        let reader = image::ImageReader::new(std::io::BufReader::new(Cursor::new(bytes)))
            .with_guessed_format()
            .map_err(|e| ImageValidationError::DecoderError(e.to_string()))?;
        reader
            .into_dimensions()
            .map_err(|e| ImageValidationError::DecoderError(e.to_string()))
    });
    match result {
        Ok(Ok(dims)) => Ok(dims),
        Ok(Err(e)) => Err(e),
        Err(payload) => Err(ImageValidationError::DecoderPanic(panic_payload_message(
            &payload,
        ))),
    }
}

fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        return (*s).to_string();
    }
    if let Some(s) = payload.downcast_ref::<String>() {
        return s.clone();
    }
    "unknown panic payload".to_string()
}

pub fn create_image_content_part(
    path: &Path,
    current_count: usize,
    current_total_bytes: u64,
) -> Result<talos_core::message::ContentPart, ImageValidationError> {
    let validated = validate_image_path(path, current_count, current_total_bytes)?;
    let digest = compute_content_digest(&validated.bytes);
    Ok(talos_core::message::ContentPart::Image {
        path: validated.canonical,
        mime: validated.mime,
        byte_count: validated.byte_count,
        content_digest: digest,
    })
}

pub fn compute_content_digest(bytes: &[u8]) -> talos_core::message::ContentDigest {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let raw: [u8; 32] = hasher.finalize().into();
    talos_core::message::ContentDigest::from_raw(raw)
}

pub const fn max_image_count() -> usize {
    MAX_IMAGE_COUNT
}
pub const fn max_single_image_bytes() -> u64 {
    MAX_SINGLE_IMAGE_BYTES
}
pub const fn max_total_image_bytes() -> u64 {
    MAX_TOTAL_IMAGE_BYTES
}
pub const fn max_pixels() -> u64 {
    MAX_PIXELS
}
