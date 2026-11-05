//! Fixed-size child resource-limit evidence; never wait for descriptor EOF.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

const RECORD_LEN: usize = 18;

// Stack-only encoding and bounded nonblocking writes. Rust b940084d7's
// UnixStream::write calls Socket::send_with_flags -> libc::send -> cvt.
// The caller retains the reader throughout synchronous spawn/exec handshake:
// this is required on macOS, where this std pair path uses no MSG_NOSIGNAL.
// Do not detach the writer or move collection before spawn completes.
pub(super) fn send_record(
    mut writer: &UnixStream,
    applied: bool,
    soft: u64,
    hard: u64,
) -> std::io::Result<()> {
    let mut record = [0; RECORD_LEN];
    record[0] = 1;
    record[1] = u8::from(applied);
    record[2..10].copy_from_slice(&soft.to_le_bytes());
    record[10..18].copy_from_slice(&hard.to_le_bytes());
    for _ in 0..4 {
        match writer.write(&record) {
            Ok(RECORD_LEN) => return Ok(()),
            Ok(_) => return Err(std::io::Error::from_raw_os_error(libc::EIO)),
            Err(error) if error.raw_os_error() == Some(libc::EINTR) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::from_raw_os_error(libc::EINTR))
}

pub(crate) struct MemoryEvidence(UnixStream);

impl MemoryEvidence {
    pub(super) fn prepare() -> std::io::Result<(Self, UnixStream)> {
        let (reader, writer) = UnixStream::pair()?;
        reader.set_nonblocking(true)?;
        writer.set_nonblocking(true)?;
        Ok((Self(reader), writer))
    }

    pub(super) fn observe(&mut self) -> Option<(bool, u64, u64)> {
        // One extra byte detects an oversized record instead of accepting a valid prefix.
        let mut record = [0; RECORD_LEN + 1];
        // Spawn's exec handshake precedes this read. Never wait for EOF or infer
        // a missing record from parent limits; a short read is unknown evidence.
        if self.0.read(&mut record).ok()? != RECORD_LEN || record[0] != 1 || record[1] > 1 {
            return None;
        }
        let soft = u64::from_le_bytes(record[2..10].try_into().ok()?);
        let hard = u64::from_le_bytes(record[10..18].try_into().ok()?);
        (soft <= hard).then_some((record[1] == 1, soft, hard))
    }

    pub(crate) fn describe(mut self) -> String {
        match self.observe() {
            Some((applied, soft, hard)) => format!(
                "memory child evidence: setter_applied={applied}; effective_soft_bytes={soft}; effective_hard_bytes={hard}"
            ),
            None => "memory child evidence: unknown (missing or invalid record)".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn incomplete_evidence_is_unknown_without_waiting_for_writer_close() {
        let (mut reader, mut writer) = MemoryEvidence::prepare().expect("channel");
        assert!(reader.observe().is_none());
        writer.write_all(&[1, 1, 0]).expect("partial record");
        assert!(reader.observe().is_none());
    }

    #[test]
    fn evidence_reports_child_bounds_and_setter_status() {
        for applied in [false, true] {
            let (mut reader, writer) = MemoryEvidence::prepare().expect("channel");
            send_record(&writer, applied, 64, 128).expect("record");
            assert_eq!(reader.observe(), Some((applied, 64, 128)));
        }
    }

    #[test]
    fn invalid_record_cannot_be_reported_as_effective_limit() {
        for (version, status, soft, hard) in
            [(2, 1, 64_u64, 128_u64), (1, 2, 64, 128), (1, 1, 129, 128)]
        {
            let (mut reader, mut writer) = MemoryEvidence::prepare().expect("channel");
            let mut record = [0; RECORD_LEN];
            record[0] = version;
            record[1] = status;
            record[2..10].copy_from_slice(&soft.to_le_bytes());
            record[10..18].copy_from_slice(&hard.to_le_bytes());
            writer.write_all(&record).expect("record");
            assert!(reader.observe().is_none());
        }
    }

    #[test]
    fn oversized_record_is_not_accepted_by_valid_prefix() {
        let (mut reader, mut writer) = MemoryEvidence::prepare().expect("channel");
        let mut record = [0; RECORD_LEN + 1];
        record[0] = 1;
        writer.write_all(&record).expect("oversized record");
        assert!(reader.observe().is_none());
    }
}
