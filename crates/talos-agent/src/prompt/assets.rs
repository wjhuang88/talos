/// Default identity text for the Talos agent.
///
/// Prompt assets live under `crates/talos-agent/prompts/` and are embedded at
/// compile time via `include_str!` per ADR-015. This keeps the binary
/// self-contained while making prompt text reviewable as standalone files.
pub const DEFAULT_IDENTITY: &str = include_str!("../../prompts/identity.txt");

pub const TOOL_CALLING_FORMAT: &str = include_str!("../../prompts/tool_calling_format.txt");
pub const TOOL_CALLING_STRICT: &str = include_str!("../../prompts/tool_calling_strict.txt");

/// Memory system prompt section placeholder.
///
/// This asset will be populated as the memory foundation (MEM-001 / I019)
/// matures. It is embedded now so prompt assembly can reference it without
/// runtime file reads.
pub const MEMORY_PROMPT: &str = include_str!("../../prompts/memory.md");
