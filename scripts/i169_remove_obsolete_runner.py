#!/usr/bin/env python3
"""Remove the superseded structured runner after sealed-plan wiring."""

from pathlib import Path


METHOD = '''    pub(crate) async fn run_for_session_turn_items(
        &self,
        items: Vec<talos_core::session::SubmissionItem>,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        request_context_limit: u32,
    ) -> (AgentResult<String>, Vec<Message>) {
        let prepared = match self
            .prepare_session_turn(&items, history, request_context_limit)
            .await
        {
            Ok(prepared) => prepared,
            Err(error) => return (Err(error), Vec::new()),
        };
        self.run_prepared_session_turn(prepared, event_tx).await
    }

'''


def main() -> None:
    path = Path("crates/talos-agent/src/lib.rs")
    source = path.read_text()
    count = source.count(METHOD)
    if count != 1:
        raise SystemExit(f"expected exactly one obsolete structured runner, found {count}")
    path.write_text(source.replace(METHOD, "", 1))


if __name__ == "__main__":
    main()
