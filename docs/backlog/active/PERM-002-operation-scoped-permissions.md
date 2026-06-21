# PERM-002: Operation-Scoped Permission Rules

| Field | Value |
|---|---|
| ID | PERM-002 |
| Type | Technical Story |
| Priority | P1 |
| Status | Refinement |
| Depends on | PERM-001 (existing rule engine), ToolNature enum (Read/Write/Execute/Network) |
| Blocks | — |

## Outcome

**核心体验改进：已授权的操作对象不再重复要求授权。**

当用户批准过一次"写 `src/main.rs`"或"访问 `api.github.com`"后，
后续相同工具操作同一资源时自动放行，无需反复确认。权限粒度从
"工具级"升级为"工具+资源级"。

## Problem

Current permission model is tool-level only:

```toml
[[rules]]
tool_name = "write"
decision = "Ask"
```

This means "approve or deny ALL writes." It cannot express:

- "Allow writes to `src/` without asking, ask for everything else"
- "Block network access to internal services, allow public URLs"
- "Allow reads anywhere, ask for writes, deny shell commands outside `scripts/`"

Agents that can browse the web and write files need **operation-scoped**
rules: what you operate on determines the permission, not just which tool
you called.

## Scope

### 1. Generalized Resource Pattern

Extend `PermissionRule` to support a **resource** abstraction instead of
just `path_pattern`:

```rust
pub struct PermissionRule {
    pub tool_name: String,
    /// Path pattern (file operations), domain pattern (network), or empty (any).
    pub resource: Option<String>,
    /// How to interpret the resource field.
    pub resource_kind: ResourceKind,
    pub decision: PermissionDecision,
}

pub enum ResourceKind {
    /// Glob pattern matched against file path (existing behavior).
    Path(String),
    /// Exact or wildcard domain matched against URL host.
    Domain(String),
    /// Combination: path within a workspace directory.
    WorkspacePath(String),
}
```

**Matching rules** (first-match-wins, same as today):

```
Tool + Resource → Decision
```

| Rule | Resource Kind | Example Pattern | Matches |
|---|---|---|---|
| `read` + Allow | Path `src/**` | `src/main.rs` | ✅ Allow |
| `read` + Ask | — (catch-all) | `Cargo.toml` | ⚠ Ask |
| `write` + Deny | Path `Cargo.lock` | `Cargo.lock` | 🚫 Deny |
| `write` + Ask | Path `src/**` | `src/lib.rs` | ⚠ Ask |
| `http_request` + Allow | Domain `api.github.com` | `https://api.github.com/repos/...` | ✅ Allow |
| `http_request` + Ask | — (catch-all) | `https://example.com` | ⚠ Ask |
| `http_request` + Deny | Domain `*.internal.com` | `https://hr.internal.com` | 🚫 Deny |
| `bash` + Ask | Path `scripts/**` | `scripts/deploy.sh` | ⚠ Ask |
| `bash` + Deny | — (no resource, catch-all) | any command | 🚫 Deny |

### 2. Resource Extraction From Tool Input

Each tool exposes **which fields** contain the resource for matching.
The permission engine extracts the resource from the tool's JSON input
based on the tool's `ToolNature`:

| ToolNature | Resource Extracted From | Example Tool Input | Extracted Resource |
|---|---|---|---|
| `Read` | `input["path"]` | `{"path": "src/main.rs"}` | `src/main.rs` |
| `Write` | `input["path"]` | `{"path": "src/lib.rs", "content": "..."}` | `src/lib.rs` |
| `Execute` | `input["command"]` | `{"command": "cargo build"}` | `"cargo build"` (command string, matched against Path pattern for workspace-relative checks) |
| `Network` | `input["url"]` (host only) | `{"url": "https://api.github.com/repos"}` | `api.github.com` |

**Extraction is type-safe**: each tool's `AgentTool::nature()` determines
the extraction logic. Tools that don't match a supported nature fall back
to tool-level matching (resource = None).

### 3. Config Format (user-facing)

```toml
# ~/.talos/config.toml

# Default: ask for all network access, but allow specific domains
[[rules]]
tool_name = "http_request"
resource = "api.github.com"
resource_kind = "domain"
decision = "Allow"

[[rules]]
tool_name = "web_search"
resource = "duckduckgo.com"
resource_kind = "domain"
decision = "Allow"

# Deny internal services
[[rules]]
tool_name = "http_request"
resource = "*.internal.com"
resource_kind = "domain"
decision = "Deny"

# Allow reads anywhere (default behavior, explicit)
[[rules]]
tool_name = "read"
decision = "Allow"

# Ask before writing to src/, deny everything else
[[rules]]
tool_name = "write"
resource = "src/**"
resource_kind = "path"
decision = "Ask"

[[rules]]
tool_name = "write"
decision = "Deny"   # catch-all: deny writes outside src/

# Allow shell commands only in scripts/
[[rules]]
tool_name = "bash"
resource = "scripts/**"
resource_kind = "path"
decision = "Ask"

[[rules]]
tool_name = "bash"
decision = "Deny"   # deny all other shell access
```

### 4. Default Rules

Shipped defaults (can be overridden):

```toml
# Network: ask by default — user must explicitly allow domains
[[rules]]
tool_name = "http_request"
decision = "Ask"

[[rules]]
tool_name = "web_search"
decision = "Ask"

# File read: allow by default
[[rules]]
tool_name = "read"
decision = "Allow"

# File write/edit/delete: ask by default
[[rules]]
tool_name = "write"
decision = "Ask"
```

### 5. Live Approval ("always approve" scoping)

When user presses `a` (always approve) in the approval dialog,
the engine creates a **scoped** rule instead of a tool-wide rule:

| Tool being approved | "Always approve" creates |
|---|---|
| `write` to `src/main.rs` | `write` + Path `src/main.rs` → Allow |
| `http_request` to `api.github.com` | `http_request` + Domain `api.github.com` → Allow |
| `bash` running `cargo build` | `bash` + Path `.` (workspace root) → Allow |

This replaces the current behavior where `a` creates an unscoped
`tool_name = "x" decision = "Allow"` rule.

## Acceptance

### Matching Engine

- Given rules `[read Allow src/**, read Ask]`
  When tool `read` is called for `src/main.rs`
  Then decision is `Allow` (first match wins)

- Given rules `[write Ask src/**, write Deny]`
  When tool `write` is called for `Cargo.toml`
  Then decision is `Deny` (catch-all matches)

- Given rules `[http_request Allow api.github.com, http_request Ask]`
  When tool `http_request` is called for `https://api.github.com/repos`
  Then decision is `Allow`

- Given rules `[http_request Deny *.internal.com]`
  When tool `http_request` is called for `https://hr.internal.com/health`
  Then decision is `Deny`

- Given NO custom rules
  When any tool with `ToolNature::Network` is called
  Then decision is `Ask` (built-in default)

### Always-approve Scoping

- Given user presses `a` on approval for `write src/main.rs`
  Then a rule `write` + Path `src/main.rs` → Allow is added
  Then subsequent `write` calls to `src/main.rs` are auto-approved
  Then `write` calls to `src/lib.rs` still require approval

### Backward Compatibility

- Given old config with `tool_name = "write" decision = "Ask"` (no resource)
  Then behavior is unchanged (tool-wide Ask, no resource matching)

- Given no `resource_kind` field in config
  Then engine infers kind from tool nature (Network → Domain, Read/Write → Path)

### Validation

- `cargo test -p talos-permission` — new tests for resource extraction and matching
- `cargo test -p talos-cli` — registry tests use scoped rules
- `cargo test --workspace` — no regressions
- Manual: start TUI, configure domain allowlist, verify network prompt behavior

## Non-Goals

- Do not change the `AgentTool` trait or `ToolNature` enum signatures
- Do not add per-tool permission configuration in tool implementations
- Do not add runtime rule editing UI (deferred to TUI-008)
- Do not support regex patterns (glob is sufficient for first iteration)

## Required Reads

- `crates/talos-permission/src/lib.rs` — `PermissionEngine`, `PermissionRule`, `infer_nature()`
- `crates/talos-cli/src/registry.rs` — `PermissionAwareTool`, `TuiPermissionAwareTool`
- `crates/talos-cli/src/approval.rs` — `ApprovalPrompt`, always-approve logic
- `crates/talos-core/src/tool.rs` — `ToolNature` enum, `AgentTool::nature()`
