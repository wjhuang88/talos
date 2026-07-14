//! Permission planning and inspection commands.

use anyhow::{Context, Result};
use clap::Subcommand;
use serde_json::{Value, json};
use talos_permission::{PermissionDecision, PermissionEngine};

use crate::approval::{always_allow_rule_descriptions, always_allow_scope_entries};
use crate::registry::build_print_tool_registry;

/// Subcommands for `talos permissions`.
#[derive(Subcommand, Clone)]
pub(crate) enum PermissionsCommand {
    /// Build a read-only permission preflight packet for expected tool operations.
    Preflight {
        /// Expected operation in the form `tool={"json":"input"}`. Repeat for multiple operations.
        #[arg(long = "operation", value_name = "TOOL=JSON", required = true)]
        operations: Vec<String>,
        /// Render machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Inspect or manage workspace trust (ADR-038/040).
    Trust {
        #[command(subcommand)]
        command: TrustCommand,
    },
}

/// Subcommands for `talos permissions trust`.
#[derive(Subcommand, Clone)]
pub(crate) enum TrustCommand {
    /// Show workspace trust status for the current directory.
    Status,
    /// Revoke workspace trust for the current directory.
    Revoke,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionPreflightPacket {
    summary: String,
    operations: Vec<PermissionPreflightOperation>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionPreflightOperation {
    tool: String,
    current_decision: String,
    always_scopes: Vec<PermissionPreflightScope>,
    descriptions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionPreflightScope {
    nature: String,
    resource_kind: String,
    resource: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedOperation {
    tool: String,
    input: Value,
}

/// Runs a `talos permissions` subcommand.
pub(crate) fn run_permissions_command(command: PermissionsCommand) -> Result<()> {
    match command {
        PermissionsCommand::Preflight { operations, json } => {
            let parsed = operations
                .iter()
                .map(|operation| parse_operation(operation))
                .collect::<Result<Vec<_>>>()?;
            let packet = build_preflight_packet(&parsed)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&packet_to_json(&packet))?
                );
            } else {
                println!("{}", render_preflight_packet(&packet));
            }
            Ok(())
        }
        PermissionsCommand::Trust { command } => run_trust_command(command),
    }
}

fn run_trust_command(command: TrustCommand) -> Result<()> {
    let talos_root = crate::storage::resolve_talos_root();
    let trust_store = talos_permission::WorkspaceTrustStore::new(&talos_root);
    let workspace = std::env::current_dir().context("failed to get current directory")?;

    match command {
        TrustCommand::Status => {
            let is_git = talos_permission::is_git_workspace(&workspace);
            let trusted = trust_store.is_trusted(&workspace);
            let trusted_count = trust_store.trusted_count();

            println!("=== Workspace Trust Status (ADR-038/040) ===");
            println!();
            println!("  Workspace:      {}", workspace.display());
            println!("  Git repository: {}", if is_git { "yes" } else { "no" });
            println!("  Trusted:        {}", if trusted { "yes" } else { "no" });
            println!("  Trusted total:  {} workspace(s)", trusted_count);
            println!();
            if !is_git {
                println!("  Note: non-Git workspaces cannot be trusted (ADR-038).");
            } else if trusted {
                println!(
                    "  Trust effect: repo-contained file writes auto-allow (Deny rules still win)."
                );
                println!(
                    "  Bash/exec: still per-command unless access evidence proves repo-local read (ADR-040)."
                );
            } else {
                println!("  Trust is not active. Use --trust during session start to grant it.");
            }
            println!();
            println!("  Use 'talos permissions trust revoke' to remove trust.");
        }
        TrustCommand::Revoke => {
            let was_trusted = trust_store.is_trusted(&workspace);
            trust_store.revoke_trust(&workspace)?;
            if was_trusted {
                println!("Revoked workspace trust for: {}", workspace.display());
                println!(
                    "Subsequent writes and commands in this workspace will use strict permission behavior."
                );
            } else {
                println!("Workspace was not trusted: {}", workspace.display());
            }
        }
    }
    Ok(())
}

fn parse_operation(raw: &str) -> Result<ParsedOperation> {
    let (tool, json) = raw
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("invalid operation '{raw}': expected TOOL=JSON"))?;
    let tool = tool.trim();
    if tool.is_empty() {
        anyhow::bail!("invalid operation '{raw}': tool name is empty");
    }
    let input = serde_json::from_str::<Value>(json.trim())
        .with_context(|| format!("invalid JSON for operation '{tool}'"))?;
    if !input.is_object() {
        anyhow::bail!("invalid operation '{tool}': input JSON must be an object");
    }
    Ok(ParsedOperation {
        tool: tool.to_string(),
        input,
    })
}

fn build_preflight_packet(operations: &[ParsedOperation]) -> Result<PermissionPreflightPacket> {
    let registry = build_print_tool_registry(None);
    let engine = PermissionEngine::with_workspace_root(std::env::current_dir()?);

    let mut entries = Vec::new();
    for operation in operations {
        let tool = registry
            .get(&operation.tool)
            .ok_or_else(|| anyhow::anyhow!("tool not found: {}", operation.tool))?;
        registry.validate_input(&operation.tool, &operation.input)?;
        let profile = tool.permission_profile(&operation.input);
        let current_decision = permission_decision_label(engine.evaluate_profile(
            &operation.tool,
            &profile,
            &operation.input,
        ));
        let always_scopes = always_allow_scope_entries(&operation.tool, &profile, &operation.input)
            .into_iter()
            .map(|entry| PermissionPreflightScope {
                nature: entry.nature,
                resource_kind: entry.resource_kind,
                resource: entry.resource,
            })
            .collect::<Vec<_>>();
        let descriptions =
            always_allow_rule_descriptions(&operation.tool, &profile, &operation.input);

        entries.push(PermissionPreflightOperation {
            tool: operation.tool.clone(),
            current_decision,
            always_scopes,
            descriptions,
        });
    }

    Ok(PermissionPreflightPacket {
        summary: format!(
            "{} expected operation(s), {} reusable scope(s)",
            entries.len(),
            entries
                .iter()
                .map(|entry| entry.always_scopes.len())
                .sum::<usize>()
        ),
        operations: entries,
        notes: vec![
            "Preflight is read-only: it does not execute tools or install allow rules.".to_string(),
            "Choosing always later installs session-scoped allow rules; configured deny rules still win.".to_string(),
            "High-risk bash commands stay exact unless their audited template policy says otherwise.".to_string(),
        ],
    })
}

fn permission_decision_label(decision: PermissionDecision) -> String {
    match decision {
        PermissionDecision::Allow => "allow".to_string(),
        PermissionDecision::Ask => "ask".to_string(),
        PermissionDecision::Deny(reason) if reason.is_empty() => "deny".to_string(),
        PermissionDecision::Deny(reason) => format!("deny: {reason}"),
    }
}

fn render_preflight_packet(packet: &PermissionPreflightPacket) -> String {
    let mut lines = Vec::new();
    lines.push("Permission preflight".to_string());
    lines.push(format!("Summary: {}", packet.summary));
    lines.push(String::new());

    for (index, operation) in packet.operations.iter().enumerate() {
        lines.push(format!(
            "{}. {} — current decision: {}",
            index + 1,
            operation.tool,
            operation.current_decision
        ));
        if operation.descriptions.is_empty() {
            lines.push("   No reusable always scope is available.".to_string());
        } else {
            lines.push("   Always approve scope:".to_string());
            for description in &operation.descriptions {
                lines.push(format!("   - {description}"));
            }
        }
    }

    lines.push(String::new());
    lines.push("Notes:".to_string());
    for note in &packet.notes {
        lines.push(format!("- {note}"));
    }
    lines.join("\n")
}

fn packet_to_json(packet: &PermissionPreflightPacket) -> Value {
    json!({
        "summary": packet.summary,
        "operations": packet.operations.iter().map(|operation| {
            json!({
                "tool": operation.tool,
                "current_decision": operation.current_decision,
                "always_scopes": operation.always_scopes.iter().map(|scope| {
                    json!({
                        "nature": scope.nature,
                        "resource_kind": scope.resource_kind,
                        "resource": scope.resource,
                    })
                }).collect::<Vec<_>>(),
                "descriptions": operation.descriptions,
            })
        }).collect::<Vec<_>>(),
        "notes": packet.notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_operation_accepts_tool_json() {
        let parsed = parse_operation(r#"bash={"command":"cat Cargo.toml"}"#).unwrap();

        assert_eq!(parsed.tool, "bash");
        assert_eq!(parsed.input["command"], "cat Cargo.toml");
    }

    #[test]
    fn preflight_packet_reports_reusable_bash_template() {
        let operations = vec![
            parse_operation(r#"bash={"command":"cat src/main.rs"}"#).unwrap(),
            parse_operation(r#"bash={"command":"cat Cargo.toml"}"#).unwrap(),
        ];

        let packet = build_preflight_packet(&operations).unwrap();

        assert_eq!(packet.operations.len(), 2);
        assert_eq!(packet.operations[0].current_decision, "ask");
        assert_eq!(
            packet.operations[0].always_scopes[0].resource,
            packet.operations[1].always_scopes[0].resource
        );
        assert!(
            packet.operations[0].always_scopes[0]
                .resource
                .starts_with("bash:read_only_inspection:template:")
        );
    }

    #[test]
    fn preflight_packet_keeps_high_risk_bash_exact() {
        let operations = vec![parse_operation(r#"bash={"command":"rm generated.txt"}"#).unwrap()];

        let packet = build_preflight_packet(&operations).unwrap();

        assert_eq!(packet.operations[0].current_decision, "ask");
        assert!(
            packet.operations[0].always_scopes[0]
                .resource
                .starts_with("bash:write_or_mutating:exact:")
        );
    }

    #[test]
    fn render_preflight_packet_explains_no_execution_or_rule_install() {
        let operations =
            vec![parse_operation(r#"bash={"command":"cargo test approval"}"#).unwrap()];
        let packet = build_preflight_packet(&operations).unwrap();

        let rendered = render_preflight_packet(&packet);

        assert!(rendered.contains("Permission preflight"));
        assert!(rendered.contains("current decision: ask"));
        assert!(rendered.contains("does not execute tools or install allow rules"));
        assert!(rendered.contains("configured deny rules still win"));
    }
}
