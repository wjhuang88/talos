use super::*;

impl ConversationEngine {
    pub fn transcript_plain_text(&self) -> String {
        let mut out = String::new();
        for msg in &self.messages {
            Self::append_message_plain(&mut out, msg);
        }
        out
    }

    pub fn transcript_plain_text_with_thinking(&self) -> String {
        let mut out = String::new();
        for msg in &self.messages {
            if msg.role == MessageRole::Reasoning {
                out.push_str("Thinking:\n");
                for line in msg.content.lines() {
                    out.push_str(&format!("| {line}\n"));
                }
                out.push('\n');
                continue;
            }
            Self::append_message_plain(&mut out, msg);
        }
        out
    }

    pub fn transcript_markdown(&self) -> String {
        let mut out = String::new();
        for msg in &self.messages {
            Self::append_message_markdown(&mut out, msg);
        }
        out
    }

    pub fn append_message_plain(out: &mut String, msg: &ChatMessage) {
        if !msg.content.is_empty() {
            out.push_str(&msg.content);
            if !msg.content.ends_with('\n') {
                out.push('\n');
            }
        }
        if let Some(ref tool_call) = msg.tool_call {
            let marker = plugin_observation_key(&tool_call.provenance);
            out.push_str(&format!("▸ {} [{marker}]\n", tool_call.tool_name));
            out.push_str(&format!("  {}\n", tool_call.arguments));
            if let Some(ref result) = tool_call.result {
                let icon = if result.is_error { "✗" } else { "✓" };
                let content = if result.content.is_empty() {
                    "(no output)"
                } else {
                    &result.content
                };
                out.push_str(&format!("  {icon} {content}\n"));
            }
        }
    }

    fn append_message_markdown(out: &mut String, msg: &ChatMessage) {
        if !msg.content.is_empty() {
            out.push_str(&msg.content);
            if !msg.content.ends_with('\n') {
                out.push('\n');
            }
        }
        if let Some(ref tool_call) = msg.tool_call {
            let marker = plugin_observation_key(&tool_call.provenance);
            out.push_str(&format!("### `▸ {} [{marker}]`\n\n", tool_call.tool_name));
            out.push_str("```json\n");
            out.push_str(&tool_call.arguments);
            out.push_str("\n```\n");
            if let Some(ref result) = tool_call.result {
                let label = if result.is_error { "Error" } else { "Result" };
                out.push_str(&format!("\n**{label}:**\n\n"));
                out.push_str("```\n");
                out.push_str(&result.content);
                out.push_str("\n```\n");
            }
        }
    }

    pub fn extension_snapshot(&self) -> ExtensionSnapshot {
        build_extension_snapshot_with_plugins(
            &self.mcp_servers,
            &self.hook_declarations,
            &self.plugin_observations,
            &self.loaded_plugins,
        )
    }
}

pub fn build_extension_snapshot(
    mcp_servers: &[McpServerDiagnostic],
    hook_declarations: &[(String, String, bool)],
    provenance: &[PluginObservation],
) -> ExtensionSnapshot {
    build_extension_snapshot_with_plugins(mcp_servers, hook_declarations, provenance, &[])
}

/// Builds an extension snapshot including typed loaded-plugin state.
pub fn build_extension_snapshot_with_plugins(
    mcp_servers: &[McpServerDiagnostic],
    hook_declarations: &[(String, String, bool)],
    provenance: &[PluginObservation],
    loaded_plugins: &[LoadedPluginDiagnostic],
) -> ExtensionSnapshot {
    let sanitized_mcp: Vec<McpServerDiagnostic> = mcp_servers
        .iter()
        .map(|s| McpServerDiagnostic {
            name: s.name.clone(),
            connected: s.connected,
            tool_count: s.tool_count,
            error: s.error.as_deref().map(categorize_mcp_error),
        })
        .collect();

    let mut seen_mcp = std::collections::HashSet::new();
    let mut collisions = Vec::new();
    for server in &sanitized_mcp {
        if !seen_mcp.insert(&server.name) {
            collisions.push(format!("mcp:{}", server.name));
        }
    }
    let mut seen_hooks = std::collections::HashSet::new();
    for (name, _, _) in hook_declarations {
        if !seen_hooks.insert(name.as_str()) {
            collisions.push(format!("hook:{name}"));
        }
    }

    let declarations = hook_declarations
        .iter()
        .map(|(name, event, enabled)| HookDeclarationDiagnostic {
            name: name.clone(),
            event: event.clone(),
            enabled: *enabled,
        })
        .collect();

    let event_catalog = talos_plugin::ALL_HOOK_EVENT_KINDS
        .iter()
        .map(|s| s.to_string())
        .collect();

    ExtensionSnapshot {
        mcp_servers: sanitized_mcp,
        hooks: HookSnapshot {
            declarations,
            executable_carriers_enabled: false,
            event_catalog,
        },
        loaded_plugins: loaded_plugins.to_vec(),
        provenance: provenance.to_vec(),
        collisions,
    }
}

/// Maps a raw MCP error string to a bounded, fixed category label.
///
/// Never returns any substring of the input. This guarantees no credential,
/// token, or query parameter can leak through diagnostics output regardless
/// of how many times it appears in the raw error text — the raw text is
/// discarded entirely, not scanned-and-patched.
fn categorize_mcp_error(raw: &str) -> String {
    let lower = raw.to_lowercase();
    let category = if lower.contains("timeout") || lower.contains("timed out") {
        "timeout"
    } else if lower.contains("invalid") && lower.contains("config") {
        "invalid_configuration"
    } else if lower.contains("spawn") {
        "spawn_failed"
    } else if lower.contains("disconnect") {
        "disconnected"
    } else if lower.contains("refused")
        || lower.contains("connect")
        || lower.contains("unreachable")
        || lower.contains("dns")
    {
        "connection_failed"
    } else if lower.contains("rpc") || lower.contains("protocol") || lower.contains("json") {
        "protocol_error"
    } else if lower.contains("initializ") {
        "initialization_failed"
    } else if lower.contains("http") {
        "network_error"
    } else {
        "unavailable"
    };
    category.to_string()
}
