//! Opaque replay envelopes for OpenAI Responses reasoning items.
//!
//! Talos stores provider-private reasoning in `ReasoningBlock::Redacted`, which is already excluded
//! from display/log projections. Responses needs more than the ciphertext for stateless replay: the
//! item id and summary array are part of the input item contract. This module stores only that
//! minimal replay item behind a Talos-private version marker; optional raw reasoning `content` and
//! response status are deliberately not retained.

use serde_json::{Value, json};

pub(crate) const RESPONSES_REASONING_PREFIX: &str = "talos:openai-responses:reasoning:v1:";
pub(crate) const MAX_REASONING_ITEM_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_REASONING_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

/// Encode one completed Responses reasoning item into an opaque durable envelope.
///
/// `Ok(None)` means the item has no encrypted state and therefore carries nothing Talos can replay
/// statelessly. The returned string never contains optional raw reasoning `content`.
pub(crate) fn encode_completed_reasoning_item(
    item: &Value,
) -> Result<Option<String>, &'static str> {
    if item.get("type").and_then(Value::as_str) != Some("reasoning") {
        return Err("not a Responses reasoning item");
    }
    let Some(encrypted_content) = item
        .get("encrypted_content")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let id = item
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or("Responses reasoning item missing id")?;
    let summary = item
        .get("summary")
        .and_then(Value::as_array)
        .ok_or("Responses reasoning item missing summary")?;

    let replay_item = json!({
        "type": "reasoning",
        "id": id,
        "summary": summary,
        "encrypted_content": encrypted_content,
    });
    let encoded = serde_json::to_string(&replay_item)
        .map_err(|_| "failed to encode Responses reasoning item")?;
    if encoded.len() > MAX_REASONING_ITEM_BYTES {
        return Err("Responses reasoning item exceeds replay size limit");
    }
    Ok(Some(format!("{RESPONSES_REASONING_PREFIX}{encoded}")))
}

/// Decode a Talos-owned Responses reasoning envelope for request replay.
///
/// Unknown/legacy redacted payloads intentionally return `None`, so switching protocols cannot
/// reinterpret another provider's opaque ciphertext as a Responses item.
pub(crate) fn decode_reasoning_item(data: &str) -> Option<Value> {
    let encoded = data.strip_prefix(RESPONSES_REASONING_PREFIX)?;
    if encoded.len() > MAX_REASONING_ITEM_BYTES {
        return None;
    }
    let item: Value = serde_json::from_str(encoded).ok()?;
    if item.get("type").and_then(Value::as_str) != Some("reasoning")
        || item
            .get("id")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || !item.get("summary").is_some_and(Value::is_array)
        || item
            .get("encrypted_content")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
    {
        return None;
    }
    Some(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_roundtrips_minimal_replay_fields_and_drops_raw_content() {
        let source = json!({
            "type": "reasoning",
            "id": "rs_1",
            "summary": [{"type": "summary_text", "text": "safe summary"}],
            "encrypted_content": "ciphertext",
            "content": [{"type": "reasoning_text", "text": "do not persist"}],
            "status": "completed"
        });
        let encoded = encode_completed_reasoning_item(&source)
            .expect("valid item")
            .expect("encrypted replay item");
        assert!(encoded.starts_with(RESPONSES_REASONING_PREFIX));
        assert!(!encoded.contains("do not persist"));
        let decoded = decode_reasoning_item(&encoded).expect("decode envelope");
        assert_eq!(decoded["id"], "rs_1");
        assert_eq!(decoded["encrypted_content"], "ciphertext");
        assert_eq!(decoded["summary"][0]["text"], "safe summary");
        assert!(decoded.get("content").is_none());
        assert!(decoded.get("status").is_none());
    }

    #[test]
    fn missing_encrypted_content_has_no_stateless_replay_payload() {
        let source = json!({
            "type": "reasoning",
            "id": "rs_1",
            "summary": []
        });
        assert_eq!(
            encode_completed_reasoning_item(&source).expect("valid non-encrypted item"),
            None
        );
    }

    #[test]
    fn foreign_redacted_payload_is_never_reinterpreted() {
        assert!(decode_reasoning_item("anthropic-ciphertext").is_none());
        assert!(decode_reasoning_item("{\"type\":\"reasoning\"}").is_none());
    }
}
