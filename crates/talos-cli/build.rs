use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn replace_once(source: &mut String, from: &str, to: &str, label: &str) {
    assert!(
        source.contains(from),
        "I169 source migration could not find {label}"
    );
    *source = source.replacen(from, to, 1);
}

fn normalized_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .replace("\r\n", "\n")
}

fn main() {
    println!("cargo:rerun-if-changed=src/tui_bridge_impl.rs");
    println!("cargo:rerun-if-changed=src/tests_impl.rs");

    let mut bridge_source = normalized_source(Path::new("src/tui_bridge_impl.rs"));

    replace_once(
        &mut bridge_source,
        "//! Bridge between the conversation engine and the TUI.\n//!\n//! Contains the conversation loop that mediates between agent events,\n//! user input, and UI output channels.\n",
        "// Bridge between the conversation engine and the TUI.\n//\n// Contains the conversation loop that mediates between agent events,\n// user input, and UI output channels.\n",
        "included-module documentation header",
    );
    replace_once(
        &mut bridge_source,
        "    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);\n    match state {\n        BridgeTurnState::Idle\n            if sequence == 0 && matches!(payload, TurnEventPayload::Started) =>",
        "    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);\n    let state_was_legacy_cancelling =\n        matches!(&state, BridgeTurnState::LegacyCancelling { .. });\n    match state {\n        BridgeTurnState::Idle\n            if sequence == 0 && matches!(payload, TurnEventPayload::Started) =>",
        "legacy cancellation state snapshot",
    );
    replace_once(
        &mut bridge_source,
        "            let cancelling = matches!(state, BridgeTurnState::LegacyCancelling { .. });",
        "            let cancelling = state_was_legacy_cancelling;",
        "legacy cancellation state use",
    );
    replace_once(
        &mut bridge_source,
        "                    cancel_requested,\n                    last_reconcile,\n                    reconcile_attempts,",
        "                    cancel_requested: _,\n                    last_reconcile: _,\n                    reconcile_attempts: _,",
        "rejected submission unused fields",
    );

    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(output_dir.join("tui_bridge_impl.rs"), bridge_source)
        .expect("write generated I169 bridge implementation");

    let mut tests_source = normalized_source(Path::new("src/tests_impl.rs"));
    replace_once(
        &mut tests_source,
        "#[cfg(test)]\n#[allow(warnings)]\nmod tests {",
        r#"async fn receive_structured_submission(
    receiver: &mut tokio::sync::mpsc::Receiver<talos_core::session::SessionOp>,
) -> talos_core::session::StructuredSubmission {
    let operation = tokio::time::timeout(std::time::Duration::from_secs(2), receiver.recv())
        .await
        .expect("structured submission dispatched within timeout")
        .expect("structured submission command channel remains open");
    match operation {
        talos_core::session::SessionOp::SubmitStructured { submission } => submission,
        other => panic!("expected SubmitStructured, got {other:?}"),
    }
}

fn accept_structured_submission(
    sender: &tokio::sync::mpsc::UnboundedSender<talos_core::session::SessionEvent>,
    submission: &talos_core::session::StructuredSubmission,
    receipt_id: &str,
) {
    sender
        .send(talos_core::session::SessionEvent::SubmissionReceipt {
            session_id: "session_test".to_string(),
            session_generation: submission.sender_generation,
            submission_id: submission.id.clone(),
            reservation_id: submission.id.clone(),
            receipt_id: receipt_id.to_string(),
            source: submission.source.clone(),
            item_count: submission.items.len(),
            total_text_bytes: submission.total_text_bytes(),
            disposition: talos_core::session::SubmissionReceiptDisposition::AcceptedPending,
        })
        .expect("durable receipt reaches bridge");
}

fn complete_structured_submission(
    sender: &tokio::sync::mpsc::UnboundedSender<talos_core::session::SessionEvent>,
    submission: &talos_core::session::StructuredSubmission,
    receipt_id: &str,
) {
    let turn_id = format!("turn_{}", submission.id);
    sender
        .send(talos_core::session::SessionEvent::StructuredTurnEvent {
            session_id: "session_test".to_string(),
            session_generation: submission.sender_generation,
            submission_id: submission.id.clone(),
            receipt_id: receipt_id.to_string(),
            turn_id: turn_id.clone(),
            sequence: 0,
            payload: talos_core::session::TurnEventPayload::Started,
        })
        .expect("structured turn start reaches bridge");
    sender
        .send(talos_core::session::SessionEvent::StructuredTurnEvent {
            session_id: "session_test".to_string(),
            session_generation: submission.sender_generation,
            submission_id: submission.id.clone(),
            receipt_id: receipt_id.to_string(),
            turn_id,
            sequence: 1,
            payload: talos_core::session::TurnEventPayload::Completed {
                status: talos_core::session::TurnCompletionStatus::Success {
                    final_text: String::new(),
                    new_messages: Vec::new(),
                },
            },
        })
        .expect("structured turn completion reaches bridge");
}

#[cfg(test)]
#[allow(warnings)]
mod tests {"#,
        "shared structured receipt test harness",
    );
    replace_once(
        &mut tests_source,
        r#"        agent_tx
            .send(AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn,
                usage: Default::default(),
            })
            .unwrap();

        let mut saw_queued_user_stream = false;"#,
        r#"        agent_tx
            .send(AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn,
                usage: Default::default(),
            })
            .unwrap();

        let submission = receive_structured_submission(&mut interrupt_rx).await;
        assert_eq!(submission.items.len(), 1);
        assert_eq!(submission.items[0].text, "queued follow-up");
        accept_structured_submission(&agent_tx.tx, &submission, "receipt_follow_up");
        complete_structured_submission(&agent_tx.tx, &submission, "receipt_follow_up");

        let mut saw_queued_user_stream = false;"#,
        "queued follow-up durable receipt",
    );
    replace_once(
        &mut tests_source,
        r#"        assert!(matches!(
            interrupt_rx.try_recv(),
            Ok(talos_core::session::SessionOp::Submit { message }) if message == "queued follow-up"
        ));

"#,
        "",
        "queued follow-up legacy submit assertion",
    );
    replace_once(
        &mut tests_source,
        r#"        assert_eq!(
            tokio::time::timeout(std::time::Duration::from_secs(1), sq_rx.recv())
                .await
                .unwrap()
                .and_then(|op| match op {
                    talos_core::session::SessionOp::Submit { message } => Some(message),
                    _ => None,
                })
                .as_deref(),
            Some("after tool")
        );"#,
        r#"        let submission = receive_structured_submission(&mut sq_rx).await;
        assert_eq!(submission.items.len(), 1);
        assert_eq!(submission.items[0].text, "after tool");"#,
        "tool-end structured submission assertion",
    );
    replace_once(
        &mut tests_source,
        r#"        match received_op.unwrap().unwrap() {
            talos_core::session::SessionOp::SubmitMultimodal { text, attachments } => {
                assert_eq!(text, "describe this image");
                assert_eq!(attachments.len(), 1);
                match &attachments[0] {
                    ContentPart::Image { mime, .. } => {
                        assert_eq!(mime, "image/png");
                    }
                    _ => panic!("expected ContentPart::Image"),
                }
            }
            other => panic!(
                "expected SubmitMultimodal, got {:?}",
                std::mem::discriminant(&other)
            ),
        }"#,
        r#"        match received_op.unwrap().unwrap() {
            talos_core::session::SessionOp::SubmitStructured { submission } => {
                assert_eq!(submission.items.len(), 1);
                assert_eq!(submission.items[0].text, "describe this image");
                assert_eq!(submission.items[0].attachments.len(), 1);
                match &submission.items[0].attachments[0] {
                    ContentPart::Image { mime, .. } => {
                        assert_eq!(mime, "image/png");
                    }
                    _ => panic!("expected ContentPart::Image"),
                }
            }
            other => panic!("expected SubmitStructured, got {other:?}"),
        }"#,
        "multimodal structured submission assertion",
    );
    replace_once(
        &mut tests_source,
        r#"        match received_op.unwrap().unwrap() {
            talos_core::session::SessionOp::Submit { message } => {
                assert_eq!(message, "plain text message");
            }
            other => panic!("expected Submit, got {:?}", other),
        }"#,
        r#"        match received_op.unwrap().unwrap() {
            talos_core::session::SessionOp::SubmitStructured { submission } => {
                assert_eq!(submission.items.len(), 1);
                assert_eq!(submission.items[0].text, "plain text message");
                assert!(submission.items[0].attachments.is_empty());
            }
            other => panic!("expected SubmitStructured, got {other:?}"),
        }"#,
        "plain structured submission assertion",
    );
    fs::write(output_dir.join("tests_impl.rs"), tests_source)
        .expect("write generated I169 CLI tests");
}
