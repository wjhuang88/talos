//! Plugin manifest parser (T45, ADR-027/029).
//!
//! Parses and validates plugin package manifests without instantiating any
//! executable artifact. A manifest declares the plugin identity, carrier,
//! artifact path, and optional atomic components (skills, tools).

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::event::ALL_HOOK_EVENT_KINDS;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("manifest validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    #[serde(default)]
    pub skills: Vec<PluginSkill>,
    #[serde(default)]
    pub tools: Vec<PluginTool>,
    #[serde(default)]
    pub hooks: Vec<PluginHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub carrier: String,
    pub artifact: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub talos_protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSkill {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTool {
    pub name: String,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    pub name: String,
    pub event: String,
    pub handler: String,
    #[serde(default)]
    pub priority: Option<i32>,
}

pub fn parse_manifest(toml_str: &str) -> Result<PluginManifest, ManifestError> {
    let manifest: PluginManifest = toml::from_str(toml_str)?;
    manifest.validate()?;
    Ok(manifest)
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        let p = &self.plugin;
        if p.name.trim().is_empty() {
            return Err(ManifestError::Validation("plugin.name is empty".into()));
        }
        if p.version.trim().is_empty() {
            return Err(ManifestError::Validation("plugin.version is empty".into()));
        }
        if p.artifact.trim().is_empty() {
            return Err(ManifestError::Validation("plugin.artifact is empty".into()));
        }
        if p.carrier != "wasm" {
            return Err(ManifestError::Validation(format!(
                "plugin.carrier must be 'wasm' (got '{}'); other carriers are not yet supported",
                p.carrier
            )));
        }
        let mut seen_tools: HashSet<&str> = HashSet::new();
        for tool in &self.tools {
            if tool.name.trim().is_empty() {
                return Err(ManifestError::Validation(
                    "tool name is empty in [[tools]]".into(),
                ));
            }
            if tool.handler.trim().is_empty() {
                return Err(ManifestError::Validation(format!(
                    "tool '{}' has empty handler",
                    tool.name
                )));
            }
            if !seen_tools.insert(&tool.name) {
                return Err(ManifestError::Validation(format!(
                    "duplicate tool name '{}'",
                    tool.name
                )));
            }
        }
        for skill in &self.skills {
            if skill.name.trim().is_empty() {
                return Err(ManifestError::Validation(
                    "skill name is empty in [[skills]]".into(),
                ));
            }
            if skill.path.trim().is_empty() {
                return Err(ManifestError::Validation(format!(
                    "skill '{}' has empty path",
                    skill.name
                )));
            }
        }
        let mut seen_hooks: HashSet<&str> = HashSet::new();
        for hook in &self.hooks {
            if hook.name.trim().is_empty() {
                return Err(ManifestError::Validation(
                    "hook name is empty in [[hooks]]".into(),
                ));
            }
            if hook.handler.trim().is_empty() {
                return Err(ManifestError::Validation(format!(
                    "hook '{}' has empty handler",
                    hook.name
                )));
            }
            if !is_known_hook_event(&hook.event) {
                return Err(ManifestError::Validation(format!(
                    "hook '{}' references unknown event '{}'",
                    hook.name, hook.event
                )));
            }
            if !seen_hooks.insert(&hook.name) {
                return Err(ManifestError::Validation(format!(
                    "duplicate hook name '{}'",
                    hook.name
                )));
            }
        }
        Ok(())
    }
}

fn is_known_hook_event(event: &str) -> bool {
    ALL_HOOK_EVENT_KINDS
        .iter()
        .any(|kind| kind.to_string() == event)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = r#"
[plugin]
name = "my-plugin"
version = "0.1.0"
carrier = "wasm"
artifact = "artifacts/my-plugin.wasm"
description = "A test plugin"

[[tools]]
name = "greet"
handler = "tools/greet.wasm"

[[skills]]
name = "my-skill"
path = "skills/my-skill/SKILL.md"

[[hooks]]
name = "pre-call"
event = "BeforeProviderCall"
handler = "hooks/pre-call.wasm"
priority = 10
"#;

    #[test]
    fn parse_valid_manifest() {
        let manifest = parse_manifest(VALID_MANIFEST).expect("valid manifest");
        assert_eq!(manifest.plugin.name, "my-plugin");
        assert_eq!(manifest.plugin.version, "0.1.0");
        assert_eq!(manifest.plugin.carrier, "wasm");
        assert_eq!(manifest.plugin.artifact, "artifacts/my-plugin.wasm");
        assert_eq!(manifest.tools.len(), 1);
        assert_eq!(manifest.tools[0].name, "greet");
        assert_eq!(manifest.skills.len(), 1);
        assert_eq!(manifest.skills[0].name, "my-skill");
        assert_eq!(manifest.hooks.len(), 1);
        assert_eq!(manifest.hooks[0].name, "pre-call");
        assert_eq!(manifest.hooks[0].event, "BeforeProviderCall");
        assert_eq!(manifest.hooks[0].handler, "hooks/pre-call.wasm");
        assert_eq!(manifest.hooks[0].priority, Some(10));
    }

    #[test]
    fn parse_minimal_manifest_no_components() {
        let toml = r#"
[plugin]
name = "bare"
version = "0.1.0"
carrier = "wasm"
artifact = "bare.wasm"
"#;
        let manifest = parse_manifest(toml).expect("minimal manifest");
        assert!(manifest.tools.is_empty());
        assert!(manifest.skills.is_empty());
        assert!(manifest.hooks.is_empty());
    }

    #[test]
    fn reject_empty_name() {
        let toml = r#"
[plugin]
name = ""
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(matches!(err, ManifestError::Validation(ref m) if m.contains("name is empty")));
    }

    #[test]
    fn reject_empty_version() {
        let toml = r#"
[plugin]
name = "p"
version = ""
carrier = "wasm"
artifact = "x.wasm"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(matches!(err, ManifestError::Validation(ref m) if m.contains("version is empty")));
    }

    #[test]
    fn reject_empty_artifact() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = ""
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(matches!(err, ManifestError::Validation(ref m) if m.contains("artifact is empty")));
    }

    #[test]
    fn reject_non_wasm_carrier() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "lua"
artifact = "x.lua"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(
            matches!(err, ManifestError::Validation(ref m) if m.contains("carrier must be 'wasm'"))
        );
    }

    #[test]
    fn reject_dylib_carrier() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "dylib"
artifact = "x.so"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(
            matches!(err, ManifestError::Validation(ref m) if m.contains("carrier must be 'wasm'"))
        );
    }

    #[test]
    fn reject_malformed_toml() {
        let toml = "this is not valid toml {{{";
        let err = parse_manifest(toml).unwrap_err();
        assert!(matches!(err, ManifestError::Parse(_)));
    }

    #[test]
    fn reject_missing_plugin_section() {
        let toml = r#"
[other]
key = "value"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(matches!(err, ManifestError::Parse(_)));
    }

    #[test]
    fn reject_duplicate_tool_names() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"

[[tools]]
name = "dup"
handler = "a.wasm"

[[tools]]
name = "dup"
handler = "b.wasm"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(
            matches!(err, ManifestError::Validation(ref m) if m.contains("duplicate tool name"))
        );
    }

    #[test]
    fn reject_empty_tool_name() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"

[[tools]]
name = ""
handler = "a.wasm"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(
            matches!(err, ManifestError::Validation(ref m) if m.contains("tool name is empty"))
        );
    }

    #[test]
    fn reject_empty_tool_handler() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"

[[tools]]
name = "t"
handler = ""
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(matches!(err, ManifestError::Validation(ref m) if m.contains("empty handler")));
    }

    #[test]
    fn manifest_describes_permissions_without_granting() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"

[plugin.permissions]
fs = ["read"]
network = false
"#;
        let manifest = parse_manifest(toml).expect("manifest with permissions section");
        assert_eq!(manifest.plugin.name, "p");
    }

    #[test]
    fn parse_hook_declaration() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"

[[hooks]]
name = "turn-start"
event = "TurnStart"
handler = "hooks/turn-start.wasm"
"#;
        let manifest = parse_manifest(toml).expect("valid manifest");
        assert_eq!(manifest.hooks.len(), 1);
        assert_eq!(manifest.hooks[0].name, "turn-start");
        assert_eq!(manifest.hooks[0].event, "TurnStart");
    }

    #[test]
    fn reject_unknown_hook_event() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"

[[hooks]]
name = "bad"
event = "MadeUpEvent"
handler = "hooks/bad.wasm"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(matches!(err, ManifestError::Validation(ref m) if m.contains("unknown event")));
    }

    #[test]
    fn reject_duplicate_hook_names() {
        let toml = r#"
[plugin]
name = "p"
version = "0.1.0"
carrier = "wasm"
artifact = "x.wasm"

[[hooks]]
name = "dup"
event = "TurnStart"
handler = "hooks/a.wasm"

[[hooks]]
name = "dup"
event = "TurnComplete"
handler = "hooks/b.wasm"
"#;
        let err = parse_manifest(toml).unwrap_err();
        assert!(
            matches!(err, ManifestError::Validation(ref m) if m.contains("duplicate hook name"))
        );
    }
}
