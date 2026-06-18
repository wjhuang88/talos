//! `rmcp` isolation facade for Talos MCP client.
//!
//! Keep all direct `rmcp` model interactions in this file so future version
//! bumps have a single adaptation point.

use serde_json::Value;

use crate::error::{McpError, Result};
use crate::types::McpToolDescriptor;

/// Parses the `tools/list` JSON-RPC result into a vector of Talos-owned tool descriptors.
pub fn decode_tools_list(result: Value, server: &str) -> Result<Vec<McpToolDescriptor>> {
    let raw_tools = if let Some(array) = result.as_array() {
        Value::Array(array.clone())
    } else {
        result.get("tools").cloned().ok_or_else(|| McpError::Rpc {
            server: server.to_string(),
            method: "tools/list".to_string(),
            message: "missing 'tools' field in result".to_string(),
        })?
    };

    let tools_array = raw_tools.as_array().ok_or_else(|| McpError::Rpc {
        server: server.to_string(),
        method: "tools/list".to_string(),
        message: "'tools' field is not an array".to_string(),
    })?;

    let mut descriptors = Vec::with_capacity(tools_array.len());
    for tool_value in tools_array {
        let name = tool_value
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let description = tool_value
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let fallback = serde_json::json!({ "type": "object" });
        let input_schema = tool_value
            .get("inputSchema")
            .cloned()
            .or_else(|| tool_value.get("input_schema").cloned())
            .unwrap_or(fallback);
        let read_only_hint = tool_value
            .get("annotations")
            .and_then(Value::as_object)
            .and_then(|ann| {
                ann.get("readOnlyHint")
                    .or_else(|| ann.get("read_only_hint"))
            })
            .and_then(Value::as_bool)
            .unwrap_or(false);

        descriptors.push(McpToolDescriptor {
            name,
            description,
            input_schema,
            read_only_hint,
        });
    }

    Ok(descriptors)
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
