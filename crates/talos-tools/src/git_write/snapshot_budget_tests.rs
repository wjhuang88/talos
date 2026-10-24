use super::read_regular_body;
use std::io::{self, Read};

#[test]
fn failed_and_oversized_reads_consume_observed_bytes() {
    struct PartialFailure(usize);
    impl Read for PartialFailure {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if buffer.is_empty() {
                return Ok(0);
            }
            if self.0 == 3 {
                return Err(io::Error::other("injected I/O failure"));
            }
            let count = buffer.len().min(3 - self.0);
            buffer[..count].copy_from_slice(&b"abc"[self.0..self.0 + count]);
            self.0 += count;
            Ok(count)
        }
    }
    let mut remaining = 10;
    assert!(read_regular_body(PartialFailure(0), &mut remaining).is_err());
    assert_eq!(remaining, 7);
    assert!(read_regular_body(&b"12345678"[..], &mut remaining).is_err());
    assert_eq!(remaining, 0);
}
