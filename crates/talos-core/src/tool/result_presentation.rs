use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::AgentTool;

/// Provenance of a registered tool.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolProvenance {
    /// A native tool registered within the main process.
    #[default]
    Native,
    /// A tool provided by a remote MCP server.
    McpRemote { server: String },
    /// A tool supplied by a plugin package (ADR-028).
    ///
    /// `carrier` is a free-form string (e.g. `"wasm"`) governed by ADR-027 so
    /// future carriers can be introduced without forcing downstream exhaustive
    /// updates. Plugin provenance is descriptive only and does not grant
    /// permissions.
    Plugin {
        name: String,
        version: String,
        carrier: String,
    },
}

/// A structured request for the runtime to disclose a narrower tool backend or
/// a specific tool on a later turn.
///
/// Continuations are advisory presentation updates. They are not permission
/// grants and must not cause a higher-risk backend to execute implicitly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolContinuation {
    /// Tool that should be disclosed or whose backend should be disclosed on a later provider turn.
    pub tool: String,
    /// Backend id to disclose.
    ///
    /// Empty means disclose the tool itself, not a conditional backend. This
    /// preserves the pre-existing field type while supporting tool-level
    /// progressive disclosure.
    pub backend: String,
    /// Machine-readable reason, such as `login_redirect` or `js_rendered_empty`.
    pub reason: String,
    /// Optional human-readable permission preview.
    #[serde(default)]
    pub permission_preview: Option<String>,
}

impl ToolContinuation {
    /// Creates a backend-disclosure continuation.
    #[must_use]
    pub fn disclose_backend(
        tool: impl Into<String>,
        backend: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            tool: tool.into(),
            backend: backend.into(),
            reason: reason.into(),
            permission_preview: None,
        }
    }

    /// Creates a tool-disclosure continuation.
    #[must_use]
    pub fn disclose_tool(tool: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            backend: String::new(),
            reason: reason.into(),
            permission_preview: None,
        }
    }

    /// Returns true when this continuation discloses a whole tool instead of a backend.
    #[must_use]
    pub fn is_tool_disclosure(&self) -> bool {
        self.backend.is_empty()
    }

    /// Adds display-oriented permission preview text.
    #[must_use]
    pub fn with_permission_preview(mut self, preview: impl Into<String>) -> Self {
        self.permission_preview = Some(preview.into());
        self
    }
}

/// The result of executing a tool.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// The output content produced by the tool.
    pub content: String,
    /// Whether the execution resulted in an error.
    pub is_error: bool,
    /// Runtime-only continuation hints for later tool presentation.
    pub continuations: Vec<ToolContinuation>,
}

/// Output of an authorized tool execution that may carry a
/// provider-neutral continuation artifact (ADR-051).
///
/// Most tools produce only the normal [`ToolResult`]. A tool like
/// `read_image` additionally returns `next_provider_parts` — a
/// `Vec<ContentPart>` that the agent delivers to the immediately
/// following provider request exactly once and then discards.
///
/// The continuation artifact is **never** persisted in the session
/// transcript, TLOG, UI, hooks, exports, or compaction. It exists
/// solely for the next `stream_with_tools` call.
#[derive(Debug, Clone)]
pub struct ToolExecutionOutput {
    /// The normal textual tool result (same as `ToolResult`).
    pub result: ToolResult,
    /// Provider-neutral content parts to carry to the next provider
    /// request. Empty for all existing tools.
    pub next_provider_parts: Vec<crate::message::ContentPart>,
}

impl ToolExecutionOutput {
    /// Creates an output with a successful text result and no
    /// continuation parts.
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            result: ToolResult::success(content),
            next_provider_parts: Vec::new(),
        }
    }

    /// Creates an output with an error text result and no continuation
    /// parts.
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            result: ToolResult::error(content),
            next_provider_parts: Vec::new(),
        }
    }

    /// Wraps an existing [`ToolResult`] with no continuation parts.
    pub fn from_result(result: ToolResult) -> Self {
        Self {
            result,
            next_provider_parts: Vec::new(),
        }
    }
}

/// Model, display, and persistence views of one tool result.
///
/// Most tools use the same content for all three views. Tools that return
/// transient model-only coordination data can override
/// [`AgentTool::project_result`] so that UI and durable history receive a
/// sanitized representation without changing the provider-facing result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResultProjection {
    /// Content supplied to the model during the active turn.
    pub model_content: String,
    /// Content emitted to user-facing runtime event projections.
    pub display_content: String,
    /// Content eligible for session persistence and replay.
    pub persistence_content: String,
}

impl ToolResultProjection {
    /// Creates a projection whose three views are identical.
    #[must_use]
    pub fn shared(content: impl Into<String>) -> Self {
        let content = content.into();
        Self {
            model_content: content.clone(),
            display_content: content.clone(),
            persistence_content: content,
        }
    }
}

impl ToolResult {
    /// Creates a successful tool result with the given content.
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
            continuations: Vec::new(),
        }
    }

    /// Creates an error tool result with the given error message.
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
            continuations: Vec::new(),
        }
    }

    /// Adds one runtime continuation hint to this tool result.
    #[must_use]
    pub fn with_continuation(mut self, continuation: ToolContinuation) -> Self {
        self.continuations.push(continuation);
        self
    }
}

/// Categorizes a tool by its operational nature for permission decisions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ToolNature {
    /// Read-only: inspects files/code without side effects.
    #[default]
    Read,
    /// Writes or modifies files.
    Write,
    /// Executes external processes or commands.
    Execute,
    /// Makes network requests (HTTP, API calls).
    Network,
    /// Session-internal plumbing (todo list, scratch state). Always allowed.
    Internal,
}

/// Stable presentation family for a tool.
///
/// Families are model-presentation metadata, not execution registration. The
/// registry remains the source of executable tools; presentation policy decides
/// which registered tools are shown to the provider for a turn/session.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ToolFamily {
    /// File and directory operations.
    #[default]
    File,
    /// Text search and file inspection operations.
    Search,
    /// AST/code-structure tools.
    CodeIntelligence,
    /// Git repository tools.
    Git,
    /// Network, web, and URL tools.
    Network,
    /// Advanced network/API debugging tools that should be disclosed only when needed.
    AdvancedNetwork,
    /// Shell or command execution tools.
    Shell,
    /// Tools supplied by extensions, MCP, or unknown sources.
    Extension,
    /// Plugin tools that must be explicitly disclosed before model presentation.
    Plugin,
}

/// A named conditional backend behind a model-visible tool.
///
/// Backends let one tool expose narrow capabilities only when a presentation
/// policy discloses them. For example, a unified web-reading tool can keep its
/// ordinary HTTP path visible while disclosing an authenticated browser-page
/// backend only after a continuation or strong user intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolBackend {
    /// Stable backend id within the owning tool.
    pub id: String,
    /// Short model-facing description of when this backend is available.
    pub description: String,
}

impl ToolBackend {
    /// Creates a backend descriptor.
    #[must_use]
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
        }
    }
}

/// A policy entry that discloses one backend for one tool.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct ToolBackendDisclosure {
    /// Tool name that owns the backend.
    pub tool: String,
    /// Backend id disclosed for the tool.
    pub backend: String,
}

impl ToolBackendDisclosure {
    /// Creates a backend disclosure entry.
    #[must_use]
    pub fn new(tool: impl Into<String>, backend: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            backend: backend.into(),
        }
    }
}

/// Policy for selecting model-visible tool families.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolPresentationPolicy {
    /// If true, every registered tool is presented.
    pub include_all: bool,
    /// If true, the always-on baseline is presented even when not in `families`.
    pub include_always_on: bool,
    /// Additional families to present.
    #[serde(default)]
    pub families: Vec<ToolFamily>,
    /// Additional individual tools to present.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Conditional backends to present for specific tools.
    #[serde(default)]
    pub backends: Vec<ToolBackendDisclosure>,
}

impl ToolPresentationPolicy {
    /// Presents every registered tool. This preserves pre-TOOL-012 behavior.
    #[must_use]
    pub fn full() -> Self {
        Self {
            include_all: true,
            include_always_on: true,
            families: Vec::new(),
            tools: Vec::new(),
            backends: Vec::new(),
        }
    }

    /// Presents the always-on baseline only.
    #[must_use]
    pub fn always_on() -> Self {
        Self {
            include_all: false,
            include_always_on: true,
            families: Vec::new(),
            tools: Vec::new(),
            backends: Vec::new(),
        }
    }

    /// Presents the default runtime surface while keeping advanced tools hidden
    /// unless explicitly disclosed.
    #[must_use]
    pub fn runtime_default() -> Self {
        Self {
            include_all: false,
            include_always_on: true,
            families: vec![
                ToolFamily::File,
                ToolFamily::Search,
                ToolFamily::CodeIntelligence,
                ToolFamily::Git,
                ToolFamily::Network,
                ToolFamily::Shell,
                ToolFamily::Extension,
            ],
            tools: Vec::new(),
            backends: Vec::new(),
        }
    }

    /// Presents the always-on baseline plus specific families.
    #[must_use]
    pub fn with_families(families: impl IntoIterator<Item = ToolFamily>) -> Self {
        Self {
            include_all: false,
            include_always_on: true,
            families: families.into_iter().collect(),
            tools: Vec::new(),
            backends: Vec::new(),
        }
    }

    /// Presents the always-on baseline plus a specific conditional backend.
    #[must_use]
    pub fn with_backend(tool: impl Into<String>, backend: impl Into<String>) -> Self {
        Self {
            include_all: false,
            include_always_on: true,
            families: Vec::new(),
            tools: Vec::new(),
            backends: vec![ToolBackendDisclosure::new(tool, backend)],
        }
    }

    /// Presents the always-on baseline plus one specific tool.
    #[must_use]
    pub fn with_tool(tool: impl Into<String>) -> Self {
        Self {
            include_all: false,
            include_always_on: true,
            families: Vec::new(),
            tools: vec![tool.into()],
            backends: Vec::new(),
        }
    }

    /// Adds a tool disclosure entry to this policy.
    #[must_use]
    pub fn disclose_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    /// Adds a backend disclosure entry to this policy.
    #[must_use]
    pub fn disclose_backend(mut self, tool: impl Into<String>, backend: impl Into<String>) -> Self {
        self.backends
            .push(ToolBackendDisclosure::new(tool, backend));
        self
    }

    /// Returns true when this policy presents the given tool.
    #[must_use]
    pub fn allows_tool(&self, tool: &dyn AgentTool) -> bool {
        self.include_all
            || (self.include_always_on && tool.is_always_on())
            || self.families.contains(&tool.family())
            || self.tools.iter().any(|name| name == tool.name())
            || self.backends.iter().any(|entry| entry.tool == tool.name())
    }

    /// Returns true when a backend is disclosed for execution.
    #[must_use]
    pub fn allows_backend(&self, tool: &str, backend: &str) -> bool {
        self.include_all
            || self
                .backends
                .iter()
                .any(|entry| entry.tool == tool && entry.backend == backend)
    }

    /// Returns the family set explicitly enabled by this policy.
    #[must_use]
    pub fn family_set(&self) -> HashSet<ToolFamily> {
        self.families.iter().copied().collect()
    }

    /// Returns the disclosed backend ids for one tool.
    #[must_use]
    pub fn backend_set_for(&self, tool: &str) -> HashSet<String> {
        self.backends
            .iter()
            .filter(|entry| entry.tool == tool)
            .map(|entry| entry.backend.clone())
            .collect()
    }
}

impl Default for ToolPresentationPolicy {
    fn default() -> Self {
        Self::full()
    }
}
