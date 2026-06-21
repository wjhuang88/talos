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

**核心体验改进：已授权的权限类型+资源不再重复要求授权。**

当用户批准过一次"写 `src/main.rs`"后，后续**所有写操作工具**
（write、edit、delete、save_url）操作同一资源时自动放行。
权限以 `ToolNature`（Read/Write/Execute/Network）为粒度，
而非具体工具名。

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

### 1. Nature-Based Rule Matching

`PermissionRule` matches on `ToolNature` (Read/Write/Execute/Network),
not on tool name. One "Write to src/" rule applies to ALL write-capable
tools (write, edit, delete, save_url).

```rust
pub struct PermissionRule {
    /// ToolNature this rule applies to.
    pub nature: ToolNature,
    /// Glob (file path) or domain/exact pattern for matching the resource.
    pub resource: Option<String>,
    /// How to interpret the resource field — inferred from nature if absent.
    pub resource_kind: Option<ResourceKind>,
    pub decision: PermissionDecision,
}

pub enum ResourceKind {
    /// Glob matched against file path (Read, Write, Execute tools).
    Path,
    /// Exact or wildcard matched against URL host (Network tools).
    Domain,
}
```

**Matching rules** (first-match-wins via nature + resource):

| Nature | Resource | Example Tool | Resource Extracted | Decision |
|---|---|---|---|---|
| `Write` + Allow | Path `src/**` | `write src/main.rs` | `src/main.rs` | ✅ Allow |
| `Write` + Allow | Path `src/**` | `edit src/main.rs` | `src/main.rs` | ✅ Allow |
| `Write` + Allow | Path `src/**` | `delete src/main.rs` | `src/main.rs` | ✅ Allow |
| `Write` + Ask | — (catch-all) | `write Cargo.toml` | `Cargo.toml` | ⚠ Ask |
| `Network` + Allow | Domain `api.github.com` | `http_request api.github.com` | `api.github.com` | ✅ Allow |
| `Network` + Allow | Domain `api.github.com` | `web_search` | `api.github.com` | ✅ Allow |
| `Network` + Deny | Domain `*.internal.com` | `http_request hr.internal.com` | `hr.internal.com` | 🚫 Deny |
| `Execute` + Ask | Path `scripts/**` | `bash scripts/deploy.sh` | `scripts/deploy.sh` | ⚠ Ask |

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

# Allow Network access to specific domains
[[rules]]
nature = "Network"
resource = "api.github.com"
resource_kind = "domain"
decision = "Allow"

[[rules]]
nature = "Network"
resource = "duckduckgo.com"
resource_kind = "domain"
decision = "Allow"

# Deny internal services (applies to ALL Network-nature tools)
[[rules]]
nature = "Network"
resource = "*.internal.com"
resource_kind = "domain"
decision = "Deny"

# Allow Write to src/ (write, edit, delete, save_url — all Write-nature tools)
[[rules]]
nature = "Write"
resource = "src/**"
resource_kind = "path"
decision = "Ask"

[[rules]]
nature = "Write"
decision = "Deny"   # catch-all: deny writes outside src/

# Allow shell commands only in scripts/
[[rules]]
nature = "Execute"
resource = "scripts/**"
resource_kind = "path"
decision = "Ask"

[[rules]]
nature = "Execute"
decision = "Deny"   # deny other shell access
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
