//! Request-owned payloads, separate from durable conversation types.

use std::time::Instant;

/// An encoded PNG available only until its producer's monotonic deadline.
///
/// This type deliberately implements neither serialization nor cloning. It must be moved
/// into a single provider request, never inserted into conversation history or previews.
pub struct EphemeralImage {
    bytes: Vec<u8>,
    expires_at: Instant,
}

impl EphemeralImage {
    /// Encoded payload size for request admission, without exposing image content.
    pub fn byte_count(&self) -> u64 {
        self.bytes.len() as u64
    }

    /// Takes ownership of producer-validated PNG bytes and their original expiry.
    /// Invalid size, signature, or an elapsed deadline is refused without retaining bytes.
    /// Producers remain responsible for decoding and validating image dimensions.
    pub fn png(bytes: Vec<u8>, expires_at: Instant) -> Option<Self> {
        if bytes.len() > 2 * 1024 * 1024
            || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
            || Instant::now() >= expires_at
        {
            return None;
        }
        Some(Self { bytes, expires_at })
    }

    /// Consumes the payload at the provider encoding boundary, refusing expired data.
    /// The deadline is returned so transport dispatch can recheck after encoding or waits.
    pub fn into_live_png(self) -> Option<(Vec<u8>, Instant)> {
        (Instant::now() < self.expires_at).then_some((self.bytes, self.expires_at))
    }
}

impl std::fmt::Debug for EphemeralImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EphemeralImage([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn refuses_invalid_and_expired_payloads() {
        let future = Instant::now() + Duration::from_secs(120);
        assert!(EphemeralImage::png(b"private-data".to_vec(), future).is_none());
        assert!(EphemeralImage::png(b"\x89PNG\r\n\x1a\n".to_vec(), Instant::now()).is_none());
        assert!(EphemeralImage::png(vec![0; 2 * 1024 * 1024 + 1], future).is_none());
    }

    #[test]
    fn redacts_payload_and_retains_original_deadline() {
        let deadline = Instant::now() + Duration::from_secs(120);
        let bytes = b"\x89PNG\r\n\x1a\nprivate-image".to_vec();
        let image = EphemeralImage::png(bytes.clone(), deadline).expect("bounded PNG carrier");
        assert_eq!(format!("{image:?}"), "EphemeralImage([redacted])");
        assert_eq!(image.into_live_png(), Some((bytes, deadline)));
    }

    #[test]
    fn consumption_rechecks_expiry_without_sleep() {
        let image = EphemeralImage {
            bytes: b"\x89PNG\r\n\x1a\nprivate-image".to_vec(),
            expires_at: Instant::now(),
        };
        assert!(image.into_live_png().is_none());
    }
}
