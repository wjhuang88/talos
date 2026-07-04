# PERM-002: Operation-Scoped Permission Rules

| Field | Value |
|---|---|
| ID | PERM-002 |
| Type | Technical Story |
| Priority | P1 |
| Status | Complete; bash always-approve repeat prompt fix implemented 2026-07-04 |
| Depends on | PERM-001 (existing rule engine), ToolNature enum (Read/Write/Execute/Network) |
| Blocks | — |

## Outcome

**核心体验改进：已授权的权限类型+资源不再重复要求授权。**

当用户批准过一次"写 `src/main.rs`"后，后续**所有写操作工具**
（write、edit、delete、save_url）操作同一资源时自动放行。
权限以 `ToolNature`（Read/Write/Execute/Network）为粒度，
而非具体工具名。

2026-07-04 maintainer feedback: the completed behavior was still insufficient for `bash`.
After the user selects `always` for the same command, Talos can still ask again for an identical
command. More broadly, the current shell permission behavior is not usable for normal development:
nearly every bash invocation asks, regardless of command, directory, or previous approval. The fix
must make shell approval deliberate but ergonomic, with stable command/cwd/risk identity and no
broadening of one approval into unrestricted shell access. The same feedback also clarifies that
write approvals should be directory-scoped, not single-file-scoped, so repeated edits in the same
directory do not prompt for every file.

2026-07-04 implementation closeout: Talos now inserts runtime `always` rules ahead of the default
catch-all `Ask` rule without bypassing explicit `Deny` rules. Bash permission profiles use an exact
classification + cwd + normalized-command + environment-shape fingerprint, so identical
command/cwd invocations can reuse `always` while changed commands, changed cwd, or complex shell
syntax ask again. Write `always` rules scope to the target file's parent directory, with root-level
files remaining file-scoped. Terminal and TUI approval prompts include the reusable
`_always_approve_scope` before the user chooses `always`.

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

Fixed residual bug:

- Selecting `always` for a `bash` command did not reliably suppress the next prompt for the same
  command.
- The risk area was mismatch between approval-time rule creation and evaluation-time resource
  extraction for `ToolNature::Execute`, plus runtime allow rules being appended after the default
  `Execute = Ask` rule.
- Default shell behavior was too coarse: unrelated safe validation commands, repeated identical
  commands, different working directories, and potentially dangerous shell expressions all collapse
  into repeated `Ask` prompts.
- Write `always` approval was too fine-grained if it only records one file path; normal development
  requires a directory-level grant for the containing directory.
- This was fixed without turning `bash` into a tool-wide allow rule or relying only on
  directory-based approval.

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
| `write` to `src/main.rs` | `Write` + directory Path `src/**` or `src/` → Allow |
| `edit` to `crates/talos-cli/src/main.rs` | `Write` + directory Path `crates/talos-cli/src/**` → Allow |
| `http_request` to `api.github.com` | `http_request` + Domain `api.github.com` → Allow |
| `bash` running `cargo build` | `bash` + Command `cargo build` + cwd → Allow |

This replaces the current behavior where `a` creates an unscoped
`tool_name = "x" decision = "Allow"` rule.

For write-capable tools, `always` should create a directory-scoped rule:

- Default directory scope is the target file's parent directory.
- The user-visible approval prompt must show the directory pattern being approved.
- The rule applies to write/edit/delete/save_url only through the existing `Write` permission
  facets, not to shell execution.
- The rule must stay workspace-bounded unless the user explicitly approves an external directory.
- Approving `src/**` must not approve `Cargo.toml`, `tests/**`, or sibling directories.
- Approving `crates/talos-cli/src/**` must not approve `crates/talos-cli/Cargo.toml` unless the
  prompt explicitly asks for that broader directory.

For `bash`, `always` must create a scoped identity that can match the same command later:

- command carrier: `bash` or future `sh`;
- exact command string after the same normalization used for approval display;
- working directory;
- permission facets and resource kind;
- environment/risk surface if exposed by the tool input.

The same approval must not match a different command, different working directory, broader shell
expression, different environment exposure, or a different permission facet set.

## Reference Project Findings

2026-07-04 reference check:

- **Codex** separates shell execution policy from generic tool permission. It evaluates parsed shell
  commands against exec-policy rules, uses safe/dangerous command heuristics, tracks whether complex
  parsing was needed, and refuses to suggest over-broad allow prefixes such as `bash`, `sh -c`,
  `python -c`, `node -e`, or `git`. Talos should copy the principle: no whole-shell or interpreter
  prefix approvals; derive only narrow reusable command identities.
- **Claude Code** exposes `permissions.allow`, `permissions.ask`, and `permissions.deny` rules such
  as `Bash(npm run test *)` and evaluates deny/ask/allow separately. It also has sandbox settings
  for bash commands, including auto-allow when sandboxed. Talos should copy the principle:
  command-pattern approval is acceptable only with explicit syntax and sandbox awareness.
- **OpenCode** has `permission.bash` object rules, session `always` approvals, and an explicit
  `doom_loop` guard for identical repeated tool calls. Its `always` action approves future requests
  matching tool-provided patterns for the current session. Talos should copy the principle:
  approval prompts must include the exact patterns that will be reused, and repeated identical calls
  should not keep prompting.
- **Aider** reduces Git friction by treating Git as first-class product behavior rather than
  repeatedly shelling through raw Git prompts. Talos should prefer built-in tools for common
  repository inspection instead of routing every operation through bash.

Talos design conclusion: implement a dedicated shell authorization layer with conservative command
classification, exact identity matching, visible always-approval patterns, and optional sandbox-aware
auto-allow for low-risk commands. Do not rely on a generic `Execute = Ask` rule for all bash calls.

### 6. Bash Authorization Model

Shell execution needs a separate policy layer above generic `Execute = Ask`. The goal is to remove
repeat prompts for safe, intentional development loops while preserving a hard boundary around
arbitrary shell power.

Classify each `bash` invocation before permission evaluation:

| Class | Examples | Default | Reuse Rule |
|---|---|---|---|
| `read_only_inspection` | `pwd`, `git status` only if no built-in tool exists, `cargo metadata --no-deps` | Ask until approved | Exact command + cwd may be reused after `always`. Prefer built-in tools when available. |
| `validation_build` | `cargo check --workspace`, `cargo test -p talos-cli approval`, project-local test scripts | Ask until approved | Exact command + cwd may be reused after `always`; session cache may reuse after `y` only if explicitly implemented. |
| `package_manager_or_network` | dependency install/update, package publish, commands likely to contact network | Ask every distinct command | `always` may only cover exact command + cwd + network facet; no prefix broadening. |
| `write_or_mutating` | commands with `rm`, `mv`, `git checkout`, `git reset`, generated-file writes | Ask | Must expose write/execute facets; exact `always` only when accepted by policy. |
| `complex_shell` | pipes, `&&`, `;`, redirection, command substitution, env assignment, background jobs | Ask | Must not inherit approval from a simpler command. |

The first implementation does not need a perfect shell parser. It does need a deterministic
conservative classifier:

- If parsing is uncertain, classify as `complex_shell` and ask.
- Do not use prefix matching for approvals.
- Do not approve all commands in a directory just because one command in that directory was allowed.
- Prefer direct `exec` or built-in tools when they cover the operation.
- Store the identity used for `always` in the same shape used during later permission evaluation.
- Show the user the exact reusable pattern before accepting `always`; never silently convert a
  single command into `bash = allow`.

Minimum permission identity for `bash`:

```text
tool = bash
command = exact normalized command string
cwd = resolved workspace-relative cwd
classification = read_only_inspection | validation_build | package_manager_or_network | write_or_mutating | complex_shell
facets = sorted permission facets
env_shape = none | names-only hash, never values
```

This identity is what `always` must persist or cache. Any changed field means the next invocation
asks again.

Recommended first implementation:

1. Add `BashCommandIdentity` construction in one shared place used by approval prompts and
   permission evaluation.
2. Add a simple classifier with `validation_build`, `read_only_inspection`, `package_manager_or_network`,
   `write_or_mutating`, and `complex_shell`.
3. Make `always` store the exact identity or a displayed narrow pattern; do not store a cwd-only
   rule.
4. Add an in-session approval cache only for exact identity matches if persistent rules are not yet
   ready.
5. Keep dangerous or complex shell syntax on `Ask` unless there is an explicit exact rule.

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
  Then a rule `Write` + directory Path `src/**` or equivalent is added
  Then subsequent `write` or `edit` calls under `src/` are auto-approved
  Then `write` calls to `Cargo.toml` or `tests/foo.rs` still require approval

- Given user presses `a` on approval for `edit crates/talos-cli/src/main.rs`
  Then subsequent writes under `crates/talos-cli/src/` are auto-approved
  Then writes to `crates/talos-cli/Cargo.toml` still require approval unless the user approved a
  broader directory pattern.

- Given user presses `a` on approval for `bash` command `cargo test -p talos-tools`
  in the workspace root,
  Then Talos stores a scoped allow rule for that exact command identity
  Then the same command in the same cwd does not ask again
  Then `cargo test --workspace` still asks
  Then the same command in a different cwd still asks unless a scoped rule covers that cwd

- Given user presses `a` on approval for a safe validation command,
  When a later `bash` command adds shell control operators, redirection, command substitution, or
  environment assignment,
  Then Talos asks again instead of treating it as the same command.

- Given user presses `a` on approval for `cargo test -p talos-tools`,
  When Talos requests `cargo test -p talos-tools -- --nocapture`,
  Then Talos asks again because the exact command identity changed.

- Given user presses `a` on approval for `cargo test -p talos-tools`,
  When Talos requests `cargo test -p talos-tools` but with a different environment shape,
  Then Talos asks again.

- Given user presses `a` on approval for `cargo test -p talos-tools`,
  When Talos requests `cargo test -p talos-tools && rm -rf target/tmp`,
  Then Talos asks again and classifies it as `complex_shell` or `write_or_mutating`.

- Given a command is available as a built-in tool, such as Git read-only status,
  When Talos wants that operation,
  Then prompt/tool guidance should prefer the built-in tool instead of asking for `bash`.

- Given user presses `y` instead of `a`,
  Then only the current invocation is allowed unless a separate session-scoped cache is explicitly
  implemented and tested.

- Given user presses `n` or deny,
  Then no allow rule or approval-cache entry is created.

### Backward Compatibility

- Given old config with `tool_name = "write" decision = "Ask"` (no resource)
  Then behavior is unchanged (tool-wide Ask, no resource matching)

- Given no `resource_kind` field in config
  Then engine infers kind from tool nature (Network → Domain, Read/Write → Path)

### Validation

- `cargo test -p talos-permission` — new tests for resource extraction and matching
- `cargo test -p talos-cli` — registry tests use scoped rules
- `cargo test -p talos-cli approval` — proves `bash` always-approve suppresses an identical
  follow-up command and does not suppress changed commands
- Tests cover bash command classification, exact identity matching, changed cwd, changed env shape,
  changed command text, complex shell syntax, and no prefix broadening.
- `cargo test --workspace` — no regressions
- Manual: start TUI, choose `always` for one validation command, then verify the identical command
  does not prompt again while a changed command still prompts.

2026-07-04 verification evidence:

- `cargo fmt --all -- --check`
- `cargo test -p talos-permission runtime_allow`
- `cargo test -p talos-tools bash_`
- `cargo test -p talos-cli always_allow`
- `cargo test -p talos-tui head_tail`
- `cargo test --workspace`
- `cargo clippy -p talos-permission -p talos-tools -p talos-cli -- -D warnings`
- `scripts/validate_project_governance.sh .`

Known validation note: `cargo clippy --workspace --all-targets -- -D warnings` remains blocked by
pre-existing `unwrap_used` violations in unrelated test modules such as `talos-exploration`,
`talos-plugin`, and existing `talos-tools` tests. Production clippy for the touched crates passes.

## Non-Goals

- Do not change the `AgentTool` trait or `ToolNature` enum signatures
- Do not add per-tool permission configuration in tool implementations
- Do not add runtime rule editing UI (deferred to TUI-008)
- Do not support regex patterns (glob is sufficient for first iteration)

## Required Reads

- `crates/talos-permission/src/lib.rs` — `PermissionEngine`, `PermissionRule`, `infer_nature()`
- `crates/talos-cli/src/registry.rs` — `PermissionAwareTool`, `TuiPermissionAwareTool`
- `crates/talos-cli/src/approval.rs` — `ApprovalPrompt`, always-approve logic
- `crates/talos-tools/src/bash_tool.rs` — bash command input and permission facets
- `crates/talos-core/src/tool.rs` — `ToolNature` enum, `AgentTool::nature()`
