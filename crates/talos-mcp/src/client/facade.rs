//! `rmcp` isolation facade for Talos MCP client.
//!
//! Keep all direct `rmcp` model interactions in this file so future version
//! bumps have a single adaptation point.

use rmcp::model::Tool;
use serde_json::Value;

use crate::error::{McpError, Result};

/// Parses the `tools/list` JSON-RPC result into a vector of rmcp tools.
pub fn decode_tools_list(result: Value, server: &str) -> Result<Vec<Tool>> {
    let raw_tools = if let Some(array) = result.as_array() {
        Value::Array(array.clone())
    } else {
        result.get("tools").cloned().ok_or_else(|| McpError::Rpc {
            server: server.to_string(),
            method: "tools/list".to_string(),
            message: "missing 'tools' field in result".to_string(),
        })?
    };

    serde_json::from_value(raw_tools).map_err(McpError::from)
}

/// Returns the tool name.
pub fn tool_name(tool: &Tool) -> Option<String> {
    serde_json::to_value(tool)
        .ok()?
        .get("name")?
        .as_str()
        .map(ToOwned::to_owned)
}

/// Returns the tool description if present.
pub fn tool_description(tool: &Tool) -> Option<String> {
    serde_json::to_value(tool)
        .ok()?
        .get("description")?
        .as_str()
        .map(ToOwned::to_owned)
}

/// Returns the tool input schema, defaulting to `{ "type": "object" }`.
pub fn tool_input_schema(tool: &Tool) -> Value {
    let fallback = serde_json::json!({ "type": "object" });
    match serde_json::to_value(tool) {
        Ok(value) => value
            .get("inputSchema")
            .cloned()
            .or_else(|| value.get("input_schema").cloned())
            .unwrap_or(fallback),
        Err(_) => fallback,
    }
}

/// Returns whether the tool is hinted as read-only.
pub fn tool_is_read_only(tool: &Tool) -> bool {
    let Ok(value) = serde_json::to_value(tool) else {
        return false;
    };

    value
        .get("annotations")
        .and_then(Value::as_object)
        .and_then(|ann| {
            ann.get("readOnlyHint")
                .or_else(|| ann.get("read_only_hint"))
        })
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Converts `tools/call` result payload into user-visible text.
pub fn call_result_to_text(result: Value) -> String {
    if let Some(content) = result.get("content").and_then(Value::as_array) {
        let mut text_parts = Vec::new();
        for item in content {
            if let Some(text) = item.get("text").and_then(Value::as_str) {
                text_parts.push(text.to_string());
            }
        }
        if !text_parts.is_empty() {
            return text_parts.join("\n");
        }
    }

    if let Some(text) = result.get("text").and_then(Value::as_str) {
        return text.to_string();
    }

    if let Some(output) = result.get("output").and_then(Value::as_str) {
        return output.to_string();
    }

    if let Some(string) = result.as_str() {
        return string.to_string();
    }

    result.to_string()
}
