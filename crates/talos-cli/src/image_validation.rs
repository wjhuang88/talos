//! Image attachment validation — delegates to `talos-tools::image_validation`.
//!
//! This module re-exports the shared implementation so existing callers
//! (`tui_bridge.rs`, `mode_print.rs`) continue to work without changes.

#[allow(unused_imports)]
pub use talos_tools::image_validation::{
    ImageValidationError, ValidatedImage, compute_content_digest, create_image_content_part,
    detect_mime_from_magic_bytes, is_supported_mime, max_image_count, max_pixels,
    max_single_image_bytes, max_total_image_bytes, validate_image_path,
};
