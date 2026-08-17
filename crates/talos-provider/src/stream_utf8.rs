//! Incremental UTF-8 decoding for provider byte streams.

/// UTF-8 failure observed while decoding a provider stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Utf8StreamError {
    /// The stream contains bytes that cannot form valid UTF-8.
    InvalidSequence,
    /// The stream ended in the middle of a UTF-8 code point.
    IncompleteSequence,
}

/// Retains only an incomplete UTF-8 suffix between transport chunks.
#[derive(Debug, Default)]
pub(crate) struct Utf8StreamDecoder {
    pending: Vec<u8>,
}

impl Utf8StreamDecoder {
    /// Decode the valid prefix of a transport chunk and retain an incomplete suffix.
    pub(crate) fn push(&mut self, bytes: &[u8]) -> Result<String, Utf8StreamError> {
        self.pending.extend_from_slice(bytes);

        match std::str::from_utf8(&self.pending) {
            Ok(text) => {
                let decoded = text.to_owned();
                self.pending.clear();
                Ok(decoded)
            }
            Err(error) if error.error_len().is_none() => {
                let valid_up_to = error.valid_up_to();
                if valid_up_to == 0 {
                    return Ok(String::new());
                }

                let incomplete = self.pending.split_off(valid_up_to);
                let valid = std::mem::replace(&mut self.pending, incomplete);
                String::from_utf8(valid).map_err(|_| Utf8StreamError::InvalidSequence)
            }
            Err(_) => Err(Utf8StreamError::InvalidSequence),
        }
    }

    /// Reject a stream that closes before its final UTF-8 code point is complete.
    pub(crate) fn finish(&self) -> Result<(), Utf8StreamError> {
        if self.pending.is_empty() {
            Ok(())
        } else {
            Err(Utf8StreamError::IncompleteSequence)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reassembles_code_points_split_across_transport_chunks() {
        let text = "A你🙂Z";
        let bytes = text.as_bytes();
        let chunks = [&bytes[..3], &bytes[3..6], &bytes[6..8], &bytes[8..]];
        let mut decoder = Utf8StreamDecoder::default();
        let mut decoded = String::new();

        for chunk in chunks {
            decoded.push_str(&decoder.push(chunk).expect("split UTF-8 should decode"));
        }

        assert_eq!(decoded, text);
        assert_eq!(decoder.finish(), Ok(()));
    }

    #[test]
    fn rejects_an_invalid_byte_sequence() {
        let mut decoder = Utf8StreamDecoder::default();
        assert_eq!(
            decoder.push(&[b'a', 0xff]),
            Err(Utf8StreamError::InvalidSequence)
        );
    }

    #[test]
    fn rejects_an_incomplete_sequence_at_end_of_stream() {
        let mut decoder = Utf8StreamDecoder::default();
        assert_eq!(decoder.push(&[0xe4, 0xbd]), Ok(String::new()));
        assert_eq!(decoder.finish(), Err(Utf8StreamError::IncompleteSequence));
    }
}
