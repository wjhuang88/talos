# TOOL-007: Built-in Tool Set Design Audit

**Status**: Complete (research, 2026-06-28)
**Priority**: P2 (Medium-term)
**Source**: User request 2026-06-19
**Depends on**: TOOL-004 complete for grep/search engine direction (ADR-025); TOOL-005/TOOL-006 (bash tool evolution); no implementation dependency

## Problem

Talos currently ships 22 built-in tools across 5 categories.  They were added
incrementally across multiple iterations (I003 through I026) without a holistic
design review.  As the tool set grows, the risk of overlapping functionality,
inconsistent design patterns, and agent confusion increases.

A comprehensive audit is needed to validate that the tool set is:
- **Orthogonal** — no two tools cover the same ground
- **Complete** — the agent can accomplish common coding tasks without
  resorting to `bash` as a crutch
- **Agent-friendly** — tool schemas, descriptions, and result formats help the
  model choose and use tools correctly
- **Safe** — the permission model accurately reflects each tool's real risk

TOOL-004 completed on 2026-06-28 and selected ripgrep library crates as the grep engine direction
through ADR-025. Search semantics and performance shape the baseline tool family, and WEBFETCH-001
Phase 2+ should be planned inside this audit rather than as a separate one-off network/document
tool expansion. `TOOL-011` may implement the selected grep engine before or during this audit, but
the audit can proceed from ADR-025 without waiting for code.

## Current Tool Inventory (22 tools)

### File & Directory (6)
| Tool | Nature | Summary |
|---|---|---|
| `read` | Read | Read file with offset/limit pagination |
| `ls` | Read | List directory (flat/recursive, long format, hidden) |
| `tree` | Read | Print directory tree structure |
| `write` | Write | Create or overwrite a file |
| `edit` | Write | Find-and-replace string in file |
| `delete` | Write | Delete file or directory |

### Search & Inspection (4)
| Tool | Nature | Summary |
|---|---|---|
| `grep` | Read | Regex search with file filter, max results |
| `glob` | Read | Find files by glob pattern |
| `diff` | Read | Unified diff between two files |
| `stat` | Read | File/directory metadata (size, mtime, permissions) |

### Git (10)
| Tool | Nature | Summary |
|---|---|---|
| `git_status` | Read | Working tree status |
| `git_diff` | Read | Staged/unstaged diff |
| `git_log` | Read | Commit history |
| `git_show` | Read | Show commit details |
| `git_branch_list` | Read | List branches |
| `git_add` | Write | Stage files |
| `git_commit` | Write | Create commit |
| `git_push` | Network | Push to remote |
| `git_pull` | Network | Pull from remote |
| `git_checkout` | Write | Switch branch / restore files |

### Code Intelligence (4)
| Tool | Nature | Summary |
|---|---|---|
| `find_symbol` | Read | Find symbol definition by name |
| `find_references` | Read | Find all usages of a symbol |
| `list_symbols` | Read | List symbols in a file |
| `list_imports` | Read | List imports in a file |

### Shell (1)
| Tool | Nature | Summary |
|---|---|---|
| `bash` | Execute | Execute shell command with timeout |

## Research Questions

### 1. Orthogonality & Overlap

- [ ] Do `ls`, `tree`, and `glob` have overlapping use cases?  When should the
  model use one vs. another?  Are the descriptions clear enough?
- [ ] Does `stat` duplicate information available from `ls --long`?
- [ ] Do `grep` and `find_symbol` overlap for code search?  (grep searches
  text; find_symbol uses tree-sitter AST)
- [ ] Are there 10 git tools because git operations are fundamentally
  different, or because the tool set reflects git's CLI surface too literally?
  Could some be consolidated (e.g. `git_show` could be a `git` tool with a
  `subcommand` parameter)?
- [ ] Does `bash` serve as an escape hatch that makes other tools redundant?
  Is there a risk the model defaults to `bash` instead of using safer,
  purpose-built tools?

### 2. Coverage Gaps

- [ ] **Network/document ingestion**: `http_request`, `fetch_url`, and `save_url` exist, but
  WEBFETCH-001 Phase 2+ still needs a cohesive place in the tool family: document extraction,
  result handles, save/download boundaries, and permission classes should be planned as part of this
  audit rather than bolted on separately.
- [ ] **Image reading**: `read` is text-only.  No tool for reading image files
  (screenshots, diagrams).  (Tracked by TOOL-003 residual)
- [ ] **Binary inspection**: No `hexdump` or `file`-type tool.
- [ ] **Environment inspection**: No tool for reading env vars, `$PATH`,
  installed tool versions.  (Model uses `bash which` / `bash --version`)
- [ ] **Config editing**: No structured config read/write tool.  Model edits
  TOML/JSON/YAML as raw text via `edit`.

### 3. Tool Granularity

- [ ] Is 22 tools the right number?  Too many tools increase prompt token
  cost and model decision complexity.  Too few force `bash` fallback.
- [ ] Do the 10 git tools justify their prompt real estate?  How often does
  the model actually use `git_branch_list` vs `git_push`?
- [ ] Should `diff` and `stat` be separate tools, or sub-operations of a
  unified inspection tool?

### 4. Tool Description Progressive Loading

Currently all 22 tool definitions (name, description, JSON Schema, summary
fields) are injected into the system prompt as a flat, static block before
every turn.  This approach has several drawbacks:

- **Token waste**: A session that never touches Git still pays for 10 Git
  tool schemas in every turn's prompt.  For a 30-turn session, this adds up
  to thousands of wasted tokens.
- **Model distraction**: Too many tool choices can degrade the model's ability
  to select the right one, especially for smaller/cheaper models.
- **Cache invalidation**: Adding one new tool invalidates the entire prompt
  cache prefix, even for sessions that never use that tool.

A progressive loading mechanism would inject tool descriptions on demand,
based on context signals:

| Trigger | Tools injected |
|---|---|
| Session start | Always-on tools: `read`, `write`, `edit`, `bash`, `grep`, `glob`, `ls` |
| Agent reads a `.git` directory or mentions "commit" | Git tool family (`git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list`, `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout`) |
| Agent navigates source code or asks "where is X defined" | Code intelligence tools (`find_symbol`, `find_references`, `list_symbols`, `list_imports`) |
| Agent compares files or inspects metadata | `diff`, `stat` |
| Agent explores directory structure | `tree` |

**Research questions**:
- [ ] What is the actual token cost of the full tool schema block vs a
  minimal "always-on" set?  (Measure with real prompt snapshots)
- [ ] How does the model behave when tools appear/disappear mid-session?
  Does it "forget" about previously available tools?  (Test with mock provider)
- [ ] What context signals are reliable triggers for tool injection?
  - File path patterns (`.git/` → git tools)
  - User message keywords ("find", "search" → code intelligence)
  - Agent action history (just ran `grep` on `.rs` → code intelligence likely needed)
- [ ] Should the model be able to explicitly request tool descriptions?
  (e.g. a meta-tool `request_tool_family("git")`)
- [ ] How does progressive loading interact with prompt caching?
  Can we design cache-friendly tool blocks that compose without full
  invalidation?
- [ ] What's the fallback when the model tries to use a tool whose
  description hasn't been loaded yet?  Auto-inject + retry?  Error?
- [ ] Does this align with or conflict with the `Skill` Level 1/2
  progressive activation model (SKILL-002)?  Can they share the same
  mechanism?

**Design constraints**:
- The `ToolRegistry` must remain the single source of truth for tool
  definitions — progressive loading is a *presentation* concern, not a
  *registration* concern.
- The prompt builder (`talos-agent/src/prompt.rs`) owns the injection
  logic; tools themselves are unchanged.
- The mechanism must degrade gracefully for providers that don't support
  mid-session tool list changes.

### 5. Agent Execution Logic

- [ ] **Tool selection patterns**: In real agent runs, which tools are used
  most?  Which are never used?  (Requires telemetry or session analysis)
- [ ] **Chaining**: Do tools compose well?  Can the model `read` → `edit` →
  `diff` → `git_commit` in a natural flow?
- [ ] **Error recovery**: When a tool fails (file not found, permission
  denied), is the error message actionable for the model?
- [ ] **Result size**: Do any tools produce outputs too large for model
  context?  Are truncation/summarization strategies consistent?

### 6. Permission Model Accuracy

- [ ] `git_push` and `git_pull` are classified as `Network` — is this
  correct?  Should they also be `Write` (push mutates remote)?
- [ ] `git_checkout` is classified as `Write` — is branch switching truly a
  write operation or a workspace state change?
- [ ] `delete` is `Write` — but directory deletion is substantially riskier
  than file deletion.  Should they be distinguished?
- [ ] `bash` is `Execute` — but the model can use it to `rm -rf`, `curl |
  sh`, or other dangerous operations.  Is the permission granularity
  sufficient?

### 7. Consistency & Naming

- [ ] Tool names follow no consistent convention: some are verbs (`read`,
  `write`), some are nouns (`diff`, `stat`), some have `git_` prefix.
- [ ] Parameter naming: `path` vs `file_path` vs `pattern` — is it
  consistent across tools?
- [ ] Description quality: Are descriptions specific enough for the model
  to choose correctly?  Compare `"Read file contents"` vs a description
  that mentions offset/limit, binary detection, and line number hint.

## Deliverables

1. **Tool design principles document** — Define what makes a good Talos tool:
   naming convention, description standard, parameter schema rules, result
   format guidelines.
2. **Orthogonality map** — Visual or tabular mapping of tool overlap and gaps.
3. **Recommendations** — Concrete proposals:
   - Tools to consolidate (merge or add subcommand parameter)
   - Tools to add (identified coverage gaps)
   - Tools to rename (consistency fixes)
   - Permission model adjustments
4. **Updated backlog items** — Any recommendations that become new stories.

## Audit Result (2026-06-28)

The audit is complete as a research/design slice. The implementation work is split into focused
follow-up stories rather than folded into this audit.

Deliverables:

- Tool family design proposal: `docs/proposals/builtin-tool-family-design.md`
- Progressive loading story: `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- Multi-resource permission story:
  `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`

Current tool inventory is 28 shared native tools, plus an MCP-only `status` tool. The older
"22 tools" inventory in this file is historical and should not be used as the current count.

Key findings:

- Tool count is not the main issue; flat prompt/provider presentation is. `SystemPromptBuilder`
  renders every tool description and parameter property into a cacheable `# Tools` section, and
  `Agent::with_security` builds provider tool definitions from the full registry once.
- `ToolRegistry` should remain the executable source of truth. Progressive loading should be a
  presentation policy over registered tools, not separate registration.
- Git tools should remain split for now. A single raw `git` tool would reduce prompt rows but lose
  structured schemas and permission clarity.
- `ls`, `tree`, `glob`, and `stat` overlap only superficially. Keep them separate but describe
  their use cases by family.
- `grep` and AST symbol tools are orthogonal. `grep` remains text search; symbol tools remain
  structure search.
- `ToolNature` as a single enum is insufficient for hybrid tools. `save_url` is network + write
  but reports `Write`; `git_push` and `git_pull` use host git and remote/workspace side effects but
  report `Execute`.
- `WEBFETCH-001` Phase 2+ must wait for the permission/result-boundary decisions in this audit
  before adding PDF/Office/document extraction or more save/download tools.

Recommended order:

1. Implement `TOOL-013` before expanding web/document save/extract behavior.
2. Implement `TOOL-012` before trying to reduce prompt/tool-schema cost through progressive
   loading.
3. Implement `TOOL-011` when grep behavior needs to be stabilized in code; otherwise ADR-025 is
   enough for design work.
4. Resume `WEBFETCH-001` Phase 2+ design only after `TOOL-013` is clear.

## Non-goals

- Do not implement any tool changes as part of this research.
- Do not propose removing tools without analyzing agent usage patterns.
- Do not redesign the `AgentTool` trait or `ToolRegistry` — this audit is
  about the tool *set*, not the tool *infrastructure*.

## Relationship To Other Requirements

| Requirement | Relationship |
|---|---|
| TOOL-003 | Residual POSIX tool gaps (image reading, write/edit display) |
| TOOL-004 | Complete research input; ADR-025 fixes the grep engine direction |
| TOOL-011 | Optional implementation follow-up for ripgrep-backed grep before/during audit |
| TOOL-005/TOOL-006 | Bash tool evolution is in scope for consistency analysis |
| WEBFETCH-001 | Network fetch gap is a known coverage issue |
| CODE-001/CODE-002 | Code intelligence tools are a distinct category |
| GIT-001 | Git tool design is inherited from gix CLI surface |
| TOOL-002 | Tool schema validation and dedup infrastructure |

## Required Reads

- `crates/talos-tools/src/` — all tool implementations
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/file_tools/`
- `crates/talos-tools/src/search_tools.rs`
- `crates/talos-tools/src/git.rs`
- `crates/talos-tools/src/symbol.rs`
- `crates/talos-tools/src/diff_stat.rs`
- `crates/talos-tools/src/tree.rs`
- `crates/talos-core/src/tool.rs` — AgentTool trait + ToolNature
- `crates/talos-permission/src/lib.rs` — PermissionEngine + default rules
- `crates/talos-cli/src/registry.rs` — which tools are registered in each mode
- `crates/talos-agent/src/prompt.rs` — how tools are formatted for the model
- `docs/decisions/025-ripgrep-library-search-engine.md` — grep engine direction
- `docs/backlog/active/TOOL-011-ripgrep-backed-grep-engine.md` — optional implementation story
