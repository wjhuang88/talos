//! Identity-scoped, opaque Gemini model-part replay envelopes.
//! Signatures remain attached to their original parts; generated IDs are stored separately.

use serde_json::{Value, json};
use std::collections::HashSet;
use talos_core::message::ReasoningBlock;

const PREFIX: &str = "talos:google-gemini:parts:v1:";
const MAX_BYTES: usize = 8 * 1024 * 1024;

/// Intentionally lacks Debug: parts may include confidential thought signatures.
pub(crate) struct GeminiReplay {
    pub(crate) parts: Vec<Value>,
    pub(crate) bindings: Vec<(String, usize)>,
}

fn validate(parts: &[Value], bindings: &[(String, usize)]) -> Result<(), &'static str> {
    let mut ids = HashSet::new();
    let mut indexes = HashSet::new();
    for (id, index) in bindings {
        if id.is_empty() || !ids.insert(id) || !indexes.insert(*index) {
            return Err("invalid Gemini replay tool association");
        }
        let call = parts
            .get(*index)
            .and_then(|part| part.get("functionCall"))
            .filter(|call| call.is_object())
            .ok_or("invalid Gemini replay tool association")?;
        if call
            .get("id")
            .is_some_and(|native| native.as_str() != Some(id.as_str()))
        {
            return Err("Gemini replay tool identity mismatch");
        }
        if call
            .get("name")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
            || call.get("args").is_some_and(|args| !args.is_object())
        {
            return Err("invalid Gemini replay function call");
        }
    }
    for (index, part) in parts.iter().enumerate() {
        if !part.is_object()
            || part.get("thoughtSignature").is_some_and(|s| !s.is_string())
            || part.get("text").is_some_and(|s| !s.is_string())
            || part.get("thought").is_some_and(|s| !s.is_boolean())
        {
            return Err("invalid Gemini replay part");
        }
        if part.as_object().is_some_and(|fields| {
            fields.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "text" | "thought" | "thoughtSignature" | "functionCall"
                )
            })
        }) {
            return Err("unsupported Gemini replay part");
        }
        let call = part.get("functionCall").is_some();
        if call != indexes.contains(&index) || (call && part.get("text").is_some()) {
            return Err("invalid Gemini replay tool association");
        }
        if !call && part.get("text").is_none() && part.get("thoughtSignature").is_none() {
            return Err("unsupported Gemini replay part");
        }
    }
    Ok(())
}

pub(crate) fn encode(
    provider: &str,
    model: &str,
    parts: &[Value],
    bindings: &[(String, usize)],
) -> Result<ReasoningBlock, &'static str> {
    if provider.is_empty() || model.is_empty() {
        return Err("missing Gemini replay identity");
    }
    validate(parts, bindings)?;
    let payload = json!({"provider":provider,"model":model,"parts":parts,"bindings":bindings});
    let payload = serde_json::to_string(&payload).map_err(|_| "cannot encode Gemini replay")?;
    if payload.len() > MAX_BYTES {
        return Err("Gemini replay exceeds limit");
    }
    Ok(ReasoningBlock::Redacted {
        data: format!("{PREFIX}{payload}"),
    })
}

pub(crate) fn decode(
    provider: &str,
    model: &str,
    block: &ReasoningBlock,
) -> Result<GeminiReplay, &'static str> {
    let ReasoningBlock::Redacted { data } = block else {
        return Err("unsupported Gemini replay block");
    };
    let payload = data
        .strip_prefix(PREFIX)
        .ok_or("foreign Gemini replay format")?;
    if payload.len() > MAX_BYTES {
        return Err("Gemini replay exceeds limit");
    }
    let value: Value =
        serde_json::from_str(payload).map_err(|_| "invalid Gemini replay envelope")?;
    if provider.is_empty()
        || model.is_empty()
        || value["provider"].as_str() != Some(provider)
        || value["model"].as_str() != Some(model)
    {
        return Err("Gemini replay provider or model mismatch");
    }
    let parts = value["parts"]
        .as_array()
        .cloned()
        .ok_or("invalid Gemini replay parts")?;
    let bindings: Vec<(String, usize)> = serde_json::from_value(value["bindings"].clone())
        .map_err(|_| "invalid Gemini replay bindings")?;
    validate(&parts, &bindings)?;
    Ok(GeminiReplay { parts, bindings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_stay_in_place_without_inserting_generated_ids_into_signed_parts() {
        let parts = vec![
            json!({"text":"summary","thought":true,"thoughtSignature":"signature-a"}),
            json!({"functionCall":{"name":"lookup","args":{"q":1}},"thoughtSignature":"signature-b"}),
            json!({"text":"tail"}),
        ];
        let bindings = vec![("local-generated-id".into(), 1)];
        let encoded = encode("google", "model", &parts, &bindings).expect("encode");
        let replay = decode("google", "model", &encoded).expect("decode");
        assert_eq!(replay.parts, parts);
        assert_eq!(replay.bindings, bindings);
        assert!(replay.parts[1]["functionCall"].get("id").is_none());
        assert!(decode("another-provider", "model", &encoded).is_err());
        assert!(decode("google", "another-model", &encoded).is_err());
    }

    #[test]
    fn forged_associations_and_foreign_payloads_are_rejected() {
        let parts = vec![json!({"functionCall":{"id":"native","name":"ping"}})];
        assert!(encode("google", "model", &parts, &[]).is_err());
        assert!(encode("google", "model", &parts, &[("other".into(), 0)]).is_err());
        assert!(encode("google", "model", &parts, &[("native".into(), 1)]).is_err());
        assert!(
            encode(
                "google",
                "model",
                &parts,
                &[("native".into(), 0), ("native".into(), 0)]
            )
            .is_err()
        );
        let encoded = encode("google", "model", &parts, &[("native".into(), 0)]).expect("valid");
        assert_eq!(
            decode("google", "model", &encoded).expect("decode").parts,
            parts
        );
        assert!(
            decode(
                "google",
                "model",
                &ReasoningBlock::Redacted {
                    data: "foreign-private".into()
                }
            )
            .is_err()
        );
    }
}
