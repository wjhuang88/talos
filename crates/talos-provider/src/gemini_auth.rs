//! Vertex credential admission. Network discovery belongs to explicit Vertex dispatch only.

use serde_json::Value;
use talos_core::provider::{ProviderError, ProviderResult};

const MAX_CREDENTIAL_BYTES: usize = 1024 * 1024;

/// Admit supported credential files before constructing any signer or HTTP request.
/// Error messages intentionally exclude source JSON, paths and upstream errors.
pub(crate) fn parse_credentials(bytes: &[u8]) -> ProviderResult<Value> {
    let invalid = || ProviderError::InvalidResponse("invalid or unsupported Vertex credentials".into());
    if bytes.len() > MAX_CREDENTIAL_BYTES {
        return Err(invalid());
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    if !matches!(value.get("type").and_then(Value::as_str), Some("service_account" | "authorized_user"))
        || value.get("credential_source").is_some()
        || value.get("service_account_impersonation_url").is_some()
        || value.get("universe_domain").is_some_and(|v| v.as_str() != Some("googleapis.com"))
        || value.get("token_uri").is_some_and(|v| !matches!(v.as_str(),
            Some("https://oauth2.googleapis.com/token" | "https://accounts.google.com/o/oauth2/token")))
    {
        return Err(invalid());
    }
    let fields: &[&str] = if value["type"] == "service_account" {
        &["client_email", "private_key_id", "private_key", "project_id"]
    } else {
        &["client_id", "client_secret", "refresh_token"]
    };
    if fields.iter().any(|name| value.get(*name).and_then(Value::as_str).is_none_or(str::is_empty)) {
        return Err(invalid());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn admits_user_refresh_credentials() {
        let v = json!({"type":"authorized_user", "client_id":"fixture-client",
            "client_secret":"fixture-secret", "refresh_token":"fixture-refresh"});
        assert_eq!(parse_credentials(v.to_string().as_bytes()).expect("valid"), v);
    }

    #[test]
    fn rejects_execution_relocation_and_oversized_input_without_disclosure() {
        for v in [
            json!({"type":"external_account", "credential_source":{"executable":{"command":"fixture"}}}),
            json!({"type":"authorized_user", "token_uri":"https://example.invalid/fixture-secret"}),
            json!({"type":"service_account", "universe_domain":"example.invalid"}),
        ] {
            let error = parse_credentials(v.to_string().as_bytes()).expect_err("reject");
            assert!(!error.to_string().contains("fixture-secret"));
        }
        assert!(parse_credentials(&vec![b' '; MAX_CREDENTIAL_BYTES + 1]).is_err());
    }
}
