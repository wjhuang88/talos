use talos_core::message::{SystemCacheMarker, SystemCacheType};
use talos_core::tool::ToolFamily;

/// A description of a tool for inclusion in the system prompt.
///
/// Contains only the name and human-readable description, without the full
/// JSON Schema parameters. This is sufficient for the system prompt context.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub family: ToolFamily,
}

/// A context file for inclusion in the system prompt.
///
/// Typically loaded from `AGENTS.md` files in the workspace hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextFile {
    /// Relative or absolute path to the source file.
    pub path: String,
    /// Full content of the file.
    pub content: String,
}

/// Activated Skill content included in the model-visible stable prompt prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivatedSkillContext {
    /// Skill name selected by the user.
    pub name: String,
    /// Bounded Skill body and any explicitly loaded references.
    pub content: String,
}

/// Type of cache control marker for prompt sections.
///
/// Used to indicate which sections of the system prompt are stable across
/// turns and suitable for provider-side caching.
#[derive(Debug, Clone, PartialEq)]
pub enum CacheType {
    /// The section is stable and suitable for ephemeral caching.
    /// Content in this range should be cached by the provider and reused
    /// across turns when the section remains unchanged.
    Ephemeral,
}

/// A cache marker indicating a byte range suitable for provider caching.
///
/// Each marker specifies the offset and length of a cacheable section within
/// the assembled prompt, along with the cache type.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheMarker {
    /// Starting byte offset of the cacheable section.
    pub offset: usize,
    /// Length of the cacheable section in bytes.
    pub length: usize,
    /// Type of caching to apply to this section.
    pub cache_type: CacheType,
}

impl From<CacheMarker> for SystemCacheMarker {
    fn from(marker: CacheMarker) -> Self {
        let cache_type = match marker.cache_type {
            CacheType::Ephemeral => SystemCacheType::Ephemeral,
        };
        Self {
            offset: marker.offset,
            length: marker.length,
            cache_type,
        }
    }
}
