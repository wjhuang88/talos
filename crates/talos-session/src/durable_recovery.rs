use crate::{DurableSession, SessionError};

impl DurableSession {
    /// Returns the durable transcript entry IDs committed for one Turn.
    ///
    /// The lookup is read-only and cursor-paginated so Actor startup can
    /// reconcile a journal row left `Running` after a crash between transcript
    /// commit and journal finalization. An empty result means the transcript
    /// does not prove a commit and must never trigger automatic re-execution.
    pub fn committed_turn_entry_ids(&self, turn_id: &str) -> Result<Vec<String>, SessionError> {
        if turn_id.is_empty() {
            return Err(SessionError::DurableTurn(
                "turn_id must not be empty".into(),
            ));
        }

        let mut cursor: Option<String> = None;
        let mut committed = Vec::new();
        loop {
            let page = self.transcript(cursor.as_deref(), 200)?;
            if page.is_empty() {
                break;
            }
            cursor = page.last().map(|entry| entry.entry_id.clone());
            committed.extend(
                page.into_iter()
                    .filter(|entry| entry.turn_id.as_deref() == Some(turn_id))
                    .map(|entry| entry.entry_id),
            );
        }
        Ok(committed)
    }
}
