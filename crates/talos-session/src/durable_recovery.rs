use crate::{DurableSession, SessionError};

impl DurableSession {
    /// Returns transcript evidence associated with one Turn.
    ///
    /// Model-visible entry IDs are returned when present. A hidden terminal
    /// outcome marker contributes one synthetic evidence token so the Actor's
    /// existing startup path invokes the journal classifier even for an Error,
    /// Cancelled, or empty successful Turn. The pending journal then maps the
    /// authoritative outcome to Committed, TerminalError, or TerminalCancelled.
    /// No message or outcome evidence returns an empty vector and remains frozen.
    pub fn committed_turn_entry_ids(&self, turn_id: &str) -> Result<Vec<String>, SessionError> {
        if turn_id.is_empty() {
            return Err(SessionError::DurableTurn(
                "turn_id must not be empty".into(),
            ));
        }

        let mut cursor: Option<String> = None;
        let mut evidence = Vec::new();
        loop {
            let page = self.transcript(cursor.as_deref(), 200)?;
            if page.is_empty() {
                break;
            }
            cursor = page.last().map(|entry| entry.entry_id.clone());
            evidence.extend(
                page.into_iter()
                    .filter(|entry| entry.turn_id.as_deref() == Some(turn_id))
                    .map(|entry| entry.entry_id),
            );
        }
        if self
            .session()
            .read_turn_transcript_outcomes()?
            .into_iter()
            .any(|record| record.turn_id == turn_id)
            && evidence.is_empty()
        {
            evidence.push(format!("terminal-outcome:{turn_id}"));
        }
        Ok(evidence)
    }
}
