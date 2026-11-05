//! Validated per-process memory policy shared by shell and direct exec.

use serde::{Deserialize, Deserializer};

pub(crate) const DEFAULT_MEMORY_MIB: u64 = 2048;
const MIN_MEMORY_MIB: u64 = 64;
const MAX_MEMORY_MIB: u64 = 65536;
const BYTES_PER_MIB: u64 = 1024 * 1024;

/// Deserialize a present field. `serde(default)` handles omission separately,
/// so explicit null cannot silently become the default policy.
pub(crate) fn deserialize_limit<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    validate(value).map_err(serde::de::Error::custom)?;
    Ok(Some(value))
}

fn validate(mib: u64) -> Result<u64, &'static str> {
    if !(MIN_MEMORY_MIB..=MAX_MEMORY_MIB).contains(&mib) {
        return Err("memory_limit_mib must be an integer between 64 and 65536 inclusive");
    }
    mib.checked_mul(BYTES_PER_MIB)
        .ok_or("memory_limit_mib byte conversion overflow")
}

/// Resolved policy retains explicit intent through request-to-step inheritance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProcessMemoryPolicy {
    bytes: u64,
    explicit: bool,
}

impl Default for ProcessMemoryPolicy {
    fn default() -> Self {
        Self {
            bytes: DEFAULT_MEMORY_MIB * BYTES_PER_MIB,
            explicit: false,
        }
    }
}

impl ProcessMemoryPolicy {
    pub(crate) fn resolve(value: Option<u64>, inherited: Self) -> Result<Self, &'static str> {
        match value {
            Some(mib) => Ok(Self {
                bytes: validate(mib)?,
                explicit: true,
            }),
            None => Ok(inherited),
        }
    }

    pub(crate) fn bytes(self) -> u64 {
        self.bytes
    }

    pub(crate) fn is_explicit(self) -> bool {
        self.explicit
    }

    pub(crate) fn describe(self) -> String {
        let mechanism = if cfg!(windows) {
            "Windows Job per-process committed virtual memory; setup required before resume"
        } else if self.is_explicit() {
            "RLIMIT_AS virtual address space; explicit setup required before exec"
        } else {
            "RLIMIT_AS virtual address space; default best-effort, enforcement unconfirmed"
        };
        format!(
            "memory_limit_mib: {} ({mechanism}; not RSS or an aggregate process-tree quota)",
            self.bytes / BYTES_PER_MIB
        )
    }

    pub(crate) fn annotate(
        self,
        mut result: talos_core::tool::ToolResult,
    ) -> talos_core::tool::ToolResult {
        let position = if result.content.starts_with("$ ") {
            result.content.find('\n').map_or(0, |index| index + 1)
        } else {
            0
        };
        result
            .content
            .insert_str(position, &format!("{}\n", self.describe()));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Deserialize)]
    struct Input {
        #[serde(default, deserialize_with = "deserialize_limit")]
        memory_limit_mib: Option<u64>,
    }

    #[test]
    fn omission_and_explicit_default_have_distinct_enforcement_intent() {
        let omitted: Input = serde_json::from_value(json!({})).expect("omitted input");
        let explicit: Input =
            serde_json::from_value(json!({"memory_limit_mib": 2048})).expect("explicit input");
        let default = ProcessMemoryPolicy::default();
        let omitted = ProcessMemoryPolicy::resolve(omitted.memory_limit_mib, default)
            .expect("default policy");
        let explicit = ProcessMemoryPolicy::resolve(explicit.memory_limit_mib, default)
            .expect("explicit policy");
        assert_eq!(omitted.bytes(), 2 * 1024 * 1024 * 1024);
        assert_eq!(omitted.bytes(), explicit.bytes());
        assert!(!omitted.is_explicit());
        assert!(explicit.is_explicit());
        assert_eq!(
            ProcessMemoryPolicy::resolve(None, explicit).expect("inherited policy"),
            explicit
        );
        let step = ProcessMemoryPolicy::resolve(Some(64), explicit).expect("step override");
        assert_eq!(step.bytes(), 64 * BYTES_PER_MIB);
        assert!(step.is_explicit());
    }

    #[test]
    fn invalid_present_values_never_fall_back_to_default() {
        for value in [
            json!(null),
            json!(0),
            json!(63),
            json!(65537),
            json!(-1),
            json!(64.5),
            json!("2048"),
            json!(true),
            json!(u64::MAX),
        ] {
            assert!(serde_json::from_value::<Input>(json!({"memory_limit_mib": value})).is_err());
        }
        for value in [64, 2048, 65536] {
            let input: Input =
                serde_json::from_value(json!({"memory_limit_mib": value})).expect("valid boundary");
            assert_eq!(input.memory_limit_mib, Some(value));
        }
        assert!(
            ProcessMemoryPolicy::resolve(Some(u64::MAX), ProcessMemoryPolicy::default()).is_err()
        );
    }
}
