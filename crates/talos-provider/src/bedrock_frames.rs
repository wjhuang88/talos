//! Incrementally assemble a single AWS event-stream frame across HTTP chunks.
use aws_smithy_eventstream::frame::read_message_from;
use aws_smithy_types::event_stream::Message;
use talos_core::provider::{ProviderError, ProviderResult};

const PRELUDE_SIZE: usize = 12;
const FRAME_OVERHEAD: usize = PRELUDE_SIZE + 4;
const MAX_HEADERS: usize = 128 * 1024;
const MAX_PAYLOAD: usize = 24 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct FrameReader {
    pending: Vec<u8>,
}

impl FrameReader {
    /// Consume at most one frame. A chunk may contain more than one event.
    pub(crate) fn next(&mut self, chunk: &mut &[u8]) -> ProviderResult<Option<Message>> {
        self.fill(chunk, PRELUDE_SIZE);
        if self.pending.len() < PRELUDE_SIZE {
            return Ok(None);
        }
        let total = u32::from_be_bytes(self.pending[..4].try_into().unwrap()) as usize;
        let headers = u32::from_be_bytes(self.pending[4..8].try_into().unwrap()) as usize;
        let payload = total
            .checked_sub(FRAME_OVERHEAD)
            .and_then(|n| n.checked_sub(headers));
        if headers > MAX_HEADERS || !payload.is_some_and(|n| n <= MAX_PAYLOAD) {
            return Err(ProviderError::InvalidResponse(
                "invalid Bedrock frame length".into(),
            ));
        }
        self.fill(chunk, total);
        if self.pending.len() < total {
            return Ok(None);
        }
        let message = read_message_from(self.pending.as_slice()).map_err(|_| {
            ProviderError::InvalidResponse("invalid Bedrock event-stream frame".into())
        })?;
        self.pending.clear();
        Ok(Some(message))
    }

    fn fill(&mut self, chunk: &mut &[u8], target: usize) {
        let count = target.saturating_sub(self.pending.len()).min(chunk.len());
        self.pending.extend_from_slice(&chunk[..count]);
        *chunk = &chunk[count..];
    }

    pub(crate) fn finish(self) -> ProviderResult<()> {
        if self.pending.is_empty() {
            Ok(())
        } else {
            Err(ProviderError::InvalidResponse(
                "truncated Bedrock event-stream frame".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_eventstream::frame::write_message_to;

    fn frame() -> Vec<u8> {
        let mut bytes = Vec::new();
        write_message_to(&Message::new("event"), &mut bytes).unwrap();
        bytes
    }

    #[test]
    fn fragmented_and_adjacent_frames() {
        let bytes = frame();
        for split in 1..bytes.len() {
            let mut reader = FrameReader::default();
            assert!(reader.next(&mut &bytes[..split]).unwrap().is_none());
            let rest = [&bytes[split..], &bytes].concat();
            let mut chunk = rest.as_slice();
            for _ in 0..2 {
                assert_eq!(
                    reader.next(&mut chunk).unwrap().unwrap().payload().as_ref(),
                    b"event"
                );
            }
            assert!(chunk.is_empty());
            reader.finish().unwrap();
        }
    }

    #[test]
    fn rejects_partial_oversized_and_corrupt_frames() {
        let bytes = frame();
        for end in 1..bytes.len() {
            let mut reader = FrameReader::default();
            assert!(reader.next(&mut &bytes[..end]).unwrap().is_none());
            assert!(reader.finish().is_err());
        }
        for offset in [0, 8, 12, bytes.len() - 1] {
            let mut corrupt = bytes.clone();
            corrupt[offset] ^= 0xff;
            assert!(
                FrameReader::default()
                    .next(&mut corrupt.as_slice())
                    .is_err()
            );
        }
    }
}
