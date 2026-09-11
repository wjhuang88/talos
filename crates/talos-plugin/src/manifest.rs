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

/// Versioned Bundle manifest accepted alongside the legacy Plugin shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub schema_version: u32,
    pub bundle: BundleMetadata,
    #[serde(default)]
    pub skills: Vec<PluginSkill>,
    #[serde(default)]
    pub tools: Vec<PluginTool>,
    #[serde(default)]
    pub hooks: Vec<PluginHook>,
}

/// Stable identity and artifact metadata for a Bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleMetadata {
    pub name: String,
    pub version: String,
    pub carrier: String,
    pub artifact: String,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub talos_protocol: Option<String>,
}

/// A manifest parsed through the compatibility boundary.
#[derive(Debug, Clone)]
pub enum CompatibleManifest {
    Legacy(PluginManifest),
    Bundle(BundleManifest),
}

/// Explicit opt-in required before a legacy manifest can be rewritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationOptions {
    /// Target schema version. Only version 1 is currently supported.
    pub schema_version: u32,
    /// Explicitly authorizes controlled-write migration.
    pub allow_write: bool,
}

/// Convert a legacy manifest to the versioned Bundle representation.
///
/// The caller must opt in explicitly; parsing alone never writes or mutates input.
pub fn migrate_legacy_manifest(
    input: &str,
    options: MigrationOptions,
) -> Result<String, ManifestError> {
    if !options.allow_write {
        return Err(ManifestError::Validation(
            "migration write requires explicit opt-in".into(),
        ));
    }
    if options.schema_version != 1 {
        return Err(ManifestError::Validation(
            "unsupported migration schema version".into(),
        ));
    }
    let value: toml::Value = toml::from_str(input)?;
    let allowed = ["plugin", "skills", "tools", "hooks"];
    if let Some(unknown) = value
        .as_table()
        .and_then(|table| table.keys().find(|key| !allowed.contains(&key.as_str())))
    {
        return Err(ManifestError::Validation(format!(
            "unknown legacy manifest field '{unknown}'"
        )));
    }
    if let Some(plugin) = value.get("plugin").and_then(toml::Value::as_table) {
        let allowed_plugin = [
            "name",
            "version",
            "carrier",
            "artifact",
            "description",
            "talos_protocol",
        ];
        if let Some(unknown) = plugin
            .keys()
            .find(|key| !allowed_plugin.contains(&key.as_str()))
        {
            return Err(ManifestError::Validation(format!(
                "unknown legacy plugin field '{unknown}'"
            )));
        }
    }
    let legacy = parse_manifest(input)?;
    let bundle = BundleManifest {
        schema_version: 1,
        bundle: BundleMetadata {
            name: legacy.plugin.name,
            version: legacy.plugin.version,
            carrier: legacy.plugin.carrier,
            artifact: legacy.plugin.artifact,
            digest: None,
            description: legacy.plugin.description,
            talos_protocol: legacy.plugin.talos_protocol,
        },
        skills: legacy.skills,
        tools: legacy.tools,
        hooks: legacy.hooks,
    };
    bundle.validate()?;
    toml::to_string_pretty(&bundle)
        .map_err(|error| ManifestError::Validation(format!("bundle serialization failed: {error}")))
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
#[serde(deny_unknown_fields)]
pub struct PluginSkill {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginTool {
    pub name: String,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

/// Parse either the legacy Plugin manifest or an explicitly versioned Bundle manifest.
pub fn parse_compatible_manifest(toml_str: &str) -> Result<CompatibleManifest, ManifestError> {
    let value: toml::Value = toml::from_str(toml_str)?;
    let has_plugin = value.get("plugin").is_some();
    let has_bundle = value.get("bundle").is_some();
    if has_plugin == has_bundle {
        return Err(ManifestError::Validation(
            "manifest must contain exactly one of [plugin] or [bundle]".into(),
        ));
    }
    if has_bundle {
        let allowed = ["schema_version", "bundle", "skills", "tools", "hooks"];
        if let Some(unknown) = value
            .as_table()
            .and_then(|table| table.keys().find(|key| !allowed.contains(&key.as_str())))
        {
            return Err(ManifestError::Validation(format!(
                "unknown bundle manifest field '{unknown}'"
            )));
        }
        let manifest: BundleManifest = value
            .try_into()
            .map_err(|error| ManifestError::Validation(format!("bundle manifest: {error}")))?;
        manifest.validate()?;
        Ok(CompatibleManifest::Bundle(manifest))
    } else {
        let allowed = ["plugin", "skills", "tools", "hooks"];
        if let Some(unknown) = value
            .as_table()
            .and_then(|table| table.keys().find(|key| !allowed.contains(&key.as_str())))
        {
            return Err(ManifestError::Validation(format!(
                "unknown legacy manifest field '{unknown}'"
            )));
        }
        if let Some(plugin) = value.get("plugin").and_then(toml::Value::as_table) {
            let allowed_plugin = [
                "name",
                "version",
                "carrier",
                "artifact",
                "description",
                "talos_protocol",
            ];
            if let Some(unknown) = plugin
                .keys()
                .find(|key| !allowed_plugin.contains(&key.as_str()))
            {
                return Err(ManifestError::Validation(format!(
                    "unknown legacy plugin field '{unknown}'"
                )));
            }
        }
        Ok(CompatibleManifest::Legacy(parse_manifest(toml_str)?))
    }
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
        if !safe_relative_path(&p.artifact) {
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
            if !safe_relative_path(&tool.handler) {
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
            if !safe_relative_path(&skill.path) {
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

impl BundleManifest {
    /// Validate the versioned Bundle contract without executing or installing artifacts.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema_version == 0 {
            return Err(ManifestError::Validation(
                "bundle.schema_version must be non-zero".into(),
            ));
        }
        if self.schema_version != 1 {
            return Err(ManifestError::Validation(format!(
                "unsupported bundle schema version {}",
                self.schema_version
            )));
        }
        if self.bundle.name.trim().is_empty() || self.bundle.version.trim().is_empty() {
            return Err(ManifestError::Validation(
                "bundle name and version are required".into(),
            ));
        }
        if self.bundle.artifact.trim().is_empty()
            || self.bundle.artifact.starts_with('/')
            || !safe_relative_path(&self.bundle.artifact)
        {
            return Err(ManifestError::Validation(
                "bundle.artifact must be a safe relative path".into(),
            ));
        }
        if self.bundle.carrier != "wasm" {
            return Err(ManifestError::Validation(
                "bundle.carrier must be 'wasm'".into(),
            ));
        }
        if self
            .bundle
            .digest
            .as_deref()
            .is_some_and(|d| !valid_digest(d))
        {
            return Err(ManifestError::Validation(
                "bundle.digest cannot be empty".into(),
            ));
        }
        validate_components(&self.tools, &self.skills, &self.hooks)
    }
}

fn validate_components(
    tools: &[PluginTool],
    skills: &[PluginSkill],
    hooks: &[PluginHook],
) -> Result<(), ManifestError> {
    let mut names = HashSet::new();
    for tool in tools {
        if tool.name.trim().is_empty()
            || !safe_relative_path(&tool.handler)
            || !names.insert(tool.name.as_str())
        {
            return Err(ManifestError::Validation(
                "invalid or duplicate tool".into(),
            ));
        }
    }
    for skill in skills {
        if skill.name.trim().is_empty() || !safe_relative_path(&skill.path) {
            return Err(ManifestError::Validation("invalid skill".into()));
        }
    }
    for hook in hooks {
        if hook.name.trim().is_empty()
            || !safe_relative_path(&hook.handler)
            || !is_known_hook_event(&hook.event)
        {
            return Err(ManifestError::Validation("invalid hook".into()));
        }
    }
    Ok(())
}

fn safe_relative_path(path: &str) -> bool {
    let path = path.trim();
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains(':')
        && !path
            .split(['/', '\\'])
            .any(|part| part.is_empty() || part == "..")
}

fn valid_digest(digest: &str) -> bool {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit())
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
    fn parse_versioned_bundle_manifest() {
        let toml = r#"
schema_version = 1
[bundle]
name = "my-bundle"
version = "1.0.0"
carrier = "wasm"
artifact = "artifacts/main.wasm"
digest = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
"#;
        let parsed = parse_compatible_manifest(toml).expect("bundle manifest");
        assert!(matches!(
            parsed,
            CompatibleManifest::Bundle(BundleManifest {
                schema_version: 1,
                ..
            })
        ));
    }

    #[test]
    fn reject_mixed_manifest_roots_and_unknown_bundle_fields() {
        let mixed = format!("{}\n[bundle]\nname = \"b\"", VALID_MANIFEST);
        assert!(parse_compatible_manifest(&mixed).is_err());
        let unknown = r#"
schema_version = 1
future = true
[bundle]
name = "b"
version = "1.0.0"
carrier = "wasm"
artifact = "b.wasm"
"#;
        let err = parse_compatible_manifest(unknown).expect_err("unknown field must fail closed");
        assert!(err.to_string().contains("unknown bundle manifest field"));
        let unknown_legacy = format!("unknown = true\n{}", VALID_MANIFEST);
        assert!(parse_compatible_manifest(&unknown_legacy).is_err());
    }

    #[test]
    fn migration_requires_explicit_opt_in_and_preserves_legacy_input() {
        let migrated = migrate_legacy_manifest(
            VALID_MANIFEST,
            MigrationOptions {
                schema_version: 1,
                allow_write: true,
            },
        )
        .expect("migration");
        assert!(migrated.contains("schema_version = 1"));
        assert!(migrated.contains("[bundle]"));
        assert!(
            migrate_legacy_manifest(
                VALID_MANIFEST,
                MigrationOptions {
                    schema_version: 1,
                    allow_write: false
                }
            )
            .is_err()
        );
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
        assert!(
            matches!(err, ManifestError::Validation(ref m) if m.contains("carrier must be 'wasm'"))
        );
    }

    #[test]
    fn reject_malformed_toml() {
        let toml = "this is not valid toml {{{";
        let err = parse_manifest(toml).expect_err("operation should fail");
        assert!(matches!(err, ManifestError::Parse(_)));
    }

    #[test]
    fn reject_missing_plugin_section() {
        let toml = r#"
[other]
key = "value"
"#;
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
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
        let err = parse_manifest(toml).expect_err("operation should fail");
        assert!(
            matches!(err, ManifestError::Validation(ref m) if m.contains("duplicate hook name"))
        );
    }
}
