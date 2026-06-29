# Architect-Owned High-Risk Work Group

> Status: Active grouping
> Created: 2026-06-28
> Owner boundary: direct architect/senior-agent execution or review required
> Source: user request to separate work that is too risky to delegate blindly

## Purpose

This document groups the backlog items that can materially affect Talos's architecture, security
boundary, runtime dependency surface, public SDK contract, prompt/cache behavior, tool permission
model, or user data integrity.

These items are not ordinary implementation tickets. A different contributor may gather evidence,
write benchmarks, or prepare isolated prototypes, but the design decision, boundary change, and
final integration must be handled through this group before implementation lands.

This group does not replace the owning backlog files, iterations, ADRs, or the existing R27
high-risk governance gate. It is the direct-oversight view of the work that should not be delegated
as routine product development.

## Selection Rules

An item belongs here if at least one condition is true:

- It changes permission, sandbox, approval, tool execution, or remote-control boundaries.
- It introduces or selects a new runtime/native dependency.
- It creates public SDK/API surface that other projects will depend on.
- It changes provider protocol semantics, stream parsing, prompt cache behavior, or context
  injection.
- It introduces background execution, scheduling, ingestion, downloads, or optional asset
  installation.
- It touches persistence or storage replacement decisions for user data.
- It can create hidden architecture coupling between CLI/TUI/web/RPC/runtime layers.

## Direct-Ownership Group

| Group | Items | Why It Is High Risk | Required Handling |
|---|---|---|---|
| H1 Runtime SDK boundary | `RUNTIME-001` | Defines whether Talos can be safely embedded by other Rust projects without importing CLI/TUI/product assumptions. Bad choices become semver-bound architecture debt. | Architect-owned API audit and ADR/proposal before any facade crate/module implementation. |
| H2 Tool family and ingestion architecture | `TOOL-004`, `TOOL-007`, `TOOL-011`, `TOOL-012`, `TOOL-013`, `WEBFETCH-001`, later `TOOL-009` | Search, web/document fetch, progressive tool loading, hybrid permissions, and result bounding all affect permission accuracy, prompt cache stability, context size, and agent behavior. | TOOL-004, TOOL-007, TOOL-012, and TOOL-013 are complete. Next handle WEBFETCH Phase 2+ design or activate TOOL-011 when grep behavior must be stabilized in code. |
| H3 Permission and autonomous execution | `PERM-001`, `SCHED-001`, `TOOL-010` | Guardian approval, exec policy, scheduled task injection, and batch file writes can bypass or dilute the permission model if treated as normal features. | No implementation without deny/ask/allow regression tests and explicit non-bypass proof. |
| H4 Extension and plugin runtime | `PLUGIN-001`, `DIST-001`, `TOOL-008` Phase 3 | WASM/runtime plugin loading and optional asset installation create supply-chain, sandbox, lifecycle, and dependency risks. | Spec and ADR before dependency selection; package installation follows DIST-001 consent and verification rules. |
| H5 Web/remote control surfaces | `WEB-001`, `REMOTE-001`, `REMOTE-002`, `GOV-003` Phase 3 | Local web UI and remote session control can expose logs, approvals, config, and governance actions outside the TUI path. | Loopback/auth/permission/RPC boundaries must be specified before implementation; no auth bypass. |
| H6 Provider and reasoning protocol | `MODEL-003`, provider residual roots in `ARCH-030` | Reasoning fields and stream parsing affect provider contracts, persisted messages, TUI display, and hidden reasoning boundaries. | ADR first; provider-specific request/stream changes require focused tests and public message-boundary review. |
| H7 Shared ecosystem import | `AGENT-002-B`, `AGENT-002-C` | Shared Skill discovery and MCP import can silently inject prompt content or start external processes from `~/.agents`. | Opt-in policy and precedence ADR before runtime loading or server startup changes. |
| H8 Memory/context compression | `MEM-007`, `MEM-005` interaction, `MEM-003` interaction | Active compression can corrupt model-visible context, break prompt cache prefixes, or lose raw tool output. | Deterministic proof, stable-prefix proof, and raw-output preservation before selection. |
| H9 Storage replacement or derived indexes | `STORE-001`, exploration/session SQLite residuals in `ARCH-030` | Storage changes can corrupt user history, claims, memory, or search indexes. | Spike/ADR first; SQLite remains source of truth unless a migration and rollback plan exists. |
| H10 Architecture residual roots when activated | `ARCH-030` roots: session SQLite, Git tools, providers, CLI/TUI roots, exploration store/ingestion | These roots are large because they own behavior-sensitive flows. Blind splits can create more coupling. | Activate one root at a time only when touched by a concrete feature or risk-reducing slice. |

## Excluded From Direct-Ownership Group

These can generally be delegated with normal review unless they touch one of the boundaries above:

- Static site work under `WEB-002`.
- README wording, release-note polish, and non-runtime documentation sync.
- TUI display polish that does not change event/session/permission semantics.
- Pure config import for `AGENT-002-A` after schema stability is proven.
- Research data gathering for TOOL-004, STORE-001, MODEL-002, or REMOTE-002, as long as no
  dependency or implementation decision is made by the delegate.

## Execution Order

1. **H1 Runtime SDK boundary**: start with `RUNTIME-001` API audit and boundary ADR/proposal.
2. **H2 Tool family and ingestion architecture**: `TOOL-004`, `TOOL-007`, `TOOL-012`, and
   `TOOL-013` are complete. Next run WEBFETCH Phase 2+ bounded extraction/save design, or
   activate `TOOL-011` when grep behavior must be proven in code.
3. **H5 Web control MVP design**: define `WEB-001` after the runtime/tool boundaries are clear.
4. **H3 Permission/autonomy packet**: handle `PERM-001`, `SCHED-001`, and `TOOL-010` only after
   the tool family design is stable.
5. **H4/H6/H7 Protocol expansion**: plugin, reasoning, and shared ecosystem import require ADRs
   before implementation.
6. **H8/H9 Storage/context work**: activate only with explicit evidence and rollback boundaries.
7. **H10 Architecture residual roots**: activate opportunistically when a selected item touches the
   root.

## Immediate Implementation Packet

The next direct-owned implementation packet is:

| Packet | Scope | Deliverable | Validation |
|---|---|---|---|
| HP-001 | `RUNTIME-001` Slice 1: embeddable runtime API audit and boundary decision | ADR/proposal classifying SDK public surface and deciding `talos-runtime` crate vs `talos-agent::runtime` module | Complete 2026-06-28 |
| HP-002 | `RUNTIME-001` Slice 2: minimal `talos-runtime` facade and embedder proof | Workspace crate with `RuntimeBuilder`, `RuntimeHandle`, safe-by-default tool wrapping, and mock-provider tests | Complete 2026-06-28 |
| HP-003 | `RUNTIME-001` Slice 3: runtime protocol cleanup and SDK hardening | Command/event serialization tests, product-neutral session policy, diagnostic API extraction, and semver/docs boundary | Complete 2026-06-28 |

## Checkpoints

### HP-001 — Runtime API Boundary Decision (2026-06-28)

Implemented the decision slice without runtime code changes:

- Added ADR-024 and accepted a dedicated `talos-runtime` facade crate for SDK-style embedding.
- Kept `talos-agent` as the turn-loop implementation crate.
- Kept foundational protocol and trait types in `talos-core`.
- Required cleanup of `SessionEvent::AgentEvent` serialization, `SessionConfig::print_mode`, and
  `/mock-request` magic handling before the SDK facade can be considered stable.

Next packet was HP-002: create the minimal `talos-runtime` crate/facade and an embedder proof,
unless the maintainer reprioritized H2 tool-design work first.

### HP-002 — Minimal Runtime Facade (2026-06-28)

Implemented the first code slice:

- Added the `talos-runtime` workspace crate as the SDK-style facade selected by ADR-024.
- Exposed `RuntimeBuilder` for provider, tool, workspace, permission-rule, sandbox, initial
  history, and context-limit construction.
- Exposed `RuntimeHandle` for submit, interrupt, event receiving, and orderly shutdown.
- Wrapped caller-registered tools in runtime-level permission checks; unresolved `Ask` decisions
  are denied in headless embedding instead of silently executing.
- Added mock-provider embedder tests for turn streaming, default write denial, explicit write
  allow-listing, and initial history.
- Updated README, architecture reference, backlog, and Board to mark this as a partial facade, not
  a stable 1.0 SDK surface.

Validation:

- `cargo test -p talos-runtime`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

Next packet is HP-003: protocol cleanup and SDK hardening before any stable embedding claim.

### HP-003 — Runtime Protocol Cleanup And SDK Hardening (2026-06-28)

Completed the protocol hardening slice for the pre-1.0 embedding surface:

- `SessionEvent::AgentEvent` now serializes with a nested `event` payload, so session events
  round-trip through serde without duplicate `type` fields.
- `SessionConfig::print_mode` was replaced with product-neutral `RuntimePolicy` and
  `ApprovalMode`.
- Request preview now uses explicit `SessionOp::PreviewRequest`,
  `Agent::preview_request()`, and `RuntimeHandle::preview_request()` APIs.
- CLI/TUI keep `/mock-request` as caller-owned diagnostic syntax and translate it before calling
  the core runtime; `talos-agent` no longer parses that magic string inside the normal turn loop.
- README and README.zh-CN document the pre-1.0 embedding boundary and distinguish it from stable
  1.0 SDK guarantees.

Validation:

- `cargo test -p talos-core -p talos-agent -p talos-runtime`
- `cargo test -p talos-cli --test skill_runtime_e2e`
- `cargo test -p talos-cli --test memory_prompt_injection`

Next direct-owned packet moved to H2-002 (`TOOL-007`) and is now recorded below.

### H2-001 — TOOL-004 Ripgrep Engine Evaluation (2026-06-28)

Completed the research slice without changing `GrepTool` runtime behavior:

- Confirmed the top-level `ripgrep` crate is the CLI package for `rg`, not the right embedded API.
- Selected ripgrep library crates (`grep-searcher`, `grep-regex`, `grep-matcher`, `ignore`) as the
  preferred Talos grep engine target.
- Rejected host `rg` as a runtime primary path; it remains benchmark/reference only.
- Added ADR-025 for the dependency and architecture decision.
- Added `TOOL-011` as the executable implementation story.
- Updated `TOOL-007` so the holistic tool-set audit can proceed from ADR-025, with
  `WEBFETCH-001` Phase 2+ still included.

Validation:

- `cargo test -p talos-tools grep_tool_tests`
- `cargo info grep-searcher`
- `cargo info grep-regex`
- `cargo info grep-matcher`
- `cargo info ignore`
- `cargo info grep-cli`
- `cargo info ripgrep`
- local host-`rg` reference timings recorded in `TOOL-004` and ADR-025

Next direct-owned packet moved to H2-002 (`TOOL-007`) and is now recorded below.

### H2-002 — TOOL-007 Tool Set Design Audit (2026-06-28)

Completed the holistic tool-set research/design slice without changing runtime behavior:

- Recounted the actual shared native tool surface as 28 tools, plus MCP-only `status`.
- Added `docs/proposals/builtin-tool-family-design.md` with tool-family principles and an
  orthogonality map.
- Confirmed `ToolRegistry` should remain executable capability truth, while progressive loading
  belongs in a presentation policy.
- Confirmed Git tools should remain split for now because structured schemas and permission clarity
  are more valuable than token reduction from a raw `git` subcommand.
- Identified the single-`ToolNature` model as insufficient for hybrid tools such as `save_url`,
  `git_push`, and `git_pull`.
- Added `TOOL-012` for tool-family metadata/progressive loading.
- Added `TOOL-013` for multi-resource permission classification.
- Updated `WEBFETCH-001` so Phase 2+ remains gated by TOOL-013 and TOOL-012.

Validation:

- Source audit of `crates/talos-core/src/tool.rs`
- Source audit of `crates/talos-permission/src/lib.rs`
- Source audit of `crates/talos-cli/src/registry.rs`
- Source audit of `crates/talos-agent/src/prompt/builder.rs`
- Source audit of `crates/talos-agent/src/configuration.rs`
- Source audit of current network/save/Git tool nature implementations

Next direct-owned packet moved to H2-003 (`TOOL-013`) and is now recorded below.

### H2-003 — TOOL-013 Multi-Resource Tool Permissions (2026-06-28)

Completed the permission-boundary implementation slice:

- Added invocation-specific permission profiles in `talos-core` through `ToolPermissionFacet` and
  `ToolResourceKind`.
- Added conservative multi-facet aggregation in `talos-permission`: denied facet wins, otherwise
  ask wins, otherwise allow.
- Updated `save_url` to expose network/domain and write/path facets.
- Updated `git_push` and `git_pull` to expose host command plus remote/network facets; `git_pull`
  also exposes workspace write impact.
- Updated `delete` to include file-vs-directory risk detail in metadata.
- Updated Agent, CLI print, TUI, MCP, and `talos-runtime` paths to share the same profile
  evaluation.
- Added ADR-026 for the multi-resource permission model.

Validation:

- `cargo test -p talos-permission -p talos-tools -p talos-runtime`
- `cargo test -p talos-agent -p talos-mcp -p talos-cli registry`
- `cargo check --workspace`
- `cargo fmt --all -- --check`

### H2-004 — TOOL-012 Tool Family Progressive Loading (2026-06-29)

Status: Complete.

Scope completed:

- Added explicit `ToolFamily` and `ToolPresentationPolicy` metadata in `talos-core`.
- Classified built-in file/search/code-intelligence/Git/network/shell tools without relying on
  name prefixes.
- Kept `ToolRegistry` as execution truth while deriving provider `ToolDefinition`s and prompt
  tool descriptions from the same presentation policy.
- Added a safe always-on baseline: `read`, `write`, `edit`, `ls`, `grep`, and `glob`.
- Added recoverable fallback for registered tools that the model calls from an unloaded family;
  such calls do not execute.
- Split prompt tool descriptions into stable family sections so an added family does not rewrite
  an unchanged family block.

Validation:

- `cargo test -p talos-core tool_presentation_policy`
- `cargo test -p talos-agent prompt::tests`
- `cargo test -p talos-agent tool_presentation`
- `cargo test -p talos-agent unpresented_registered_tool`
- `cargo check --workspace`

Next direct-owned packet is WEBFETCH Phase 2+ bounded extraction/save design unless grep behavior
needs to be stabilized first through `TOOL-011`.

## Relationship To R27

The existing R27 High-Risk Governance Gate remains the active long-running execution gate. This
group updates R27's scope by adding `RUNTIME-001` and by making the direct-ownership set explicit.

R27's rule still applies: this grouping does not authorize tag, push, destructive cleanup, network
spend, new runtime dependency, or permission-boundary changes without the named gates.
