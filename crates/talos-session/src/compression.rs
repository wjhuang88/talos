#![allow(dead_code)]
//! Segment compression trait for session archival (ADR-036/037).
//!
//! When session compaction triggers (Slice D), frozen segments are compressed
//! to reduce storage. The `SegmentCompressor` trait abstracts compression
//! behind a swappable interface per ADR-036.

#[cfg(test)]
use std::sync::OnceLock;

pub trait SegmentCompressor: Send + Sync {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn format_tag(&self) -> &'static str;
}

#[derive(Debug)]
pub enum CompressionError {
    CompressFailed(String),
    DecompressFailed(String),
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionError::CompressFailed(msg) => write!(f, "compression failed: {msg}"),
            CompressionError::DecompressFailed(msg) => write!(f, "decompression failed: {msg}"),
        }
    }
}

impl std::error::Error for CompressionError {}

pub struct NoCompressor;

impl SegmentCompressor for NoCompressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(input.to_vec())
    }
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(input.to_vec())
    }
    fn format_tag(&self) -> &'static str {
        "none"
    }
}

#[cfg(feature = "archive-compress-zstd")]
pub struct ZstdCompressor {
    level: i32,
}

#[cfg(feature = "archive-compress-zstd")]
impl Default for ZstdCompressor {
    fn default() -> Self {
        Self { level: 3 }
    }
}

#[cfg(feature = "archive-compress-zstd")]
impl SegmentCompressor for ZstdCompressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        zstd::stream::encode_all(input, self.level)
            .map_err(|e| CompressionError::CompressFailed(e.to_string()))
    }
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        zstd::stream::decode_all(input)
            .map_err(|e| CompressionError::DecompressFailed(e.to_string()))
    }
    fn format_tag(&self) -> &'static str {
        "zstd"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_compressor_round_trip() {
        let c = NoCompressor;
        let data = b"Hello, world! This is test data for compression.";
        let compressed = c.compress(data).unwrap();
        let decompressed = c.decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
        assert_eq!(c.format_tag(), "none");
    }

    #[test]
    fn no_compressor_preserves_empty() {
        let c = NoCompressor;
        let compressed = c.compress(b"").unwrap();
        assert!(compressed.is_empty());
        let decompressed = c.decompress(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }

    #[cfg(feature = "archive-compress-zstd")]
    #[test]
    fn zstd_compressor_round_trip() {
        let c = ZstdCompressor::default();
        let data = b"Hello, world! This is test data for compression.".repeat(100);
        let compressed = c.compress(&data).unwrap();
        assert!(
            compressed.len() < data.len(),
            "zstd should compress repetitive data"
        );
        let decompressed = c.decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
        assert_eq!(c.format_tag(), "zstd");
    }
}
