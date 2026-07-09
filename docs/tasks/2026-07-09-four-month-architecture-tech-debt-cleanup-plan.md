# 2026-07-09 Four-Month Architecture and Tech Debt Cleanup Plan

**Status**: Planned
**Owner area**: Architecture decomposition, session storage modernization, permission/tool debt cleanup.
**Created**: 2026-07-09
**Timebox**: 16 weeks / roughly 4 months (July – October 2026)
**Primary release marker**: Continue pre-1.0 releases; no v1.0 claim.
**Execution mode**: External Agent primary (Claude/Codex/glm); Talos for validation/testing.
**Direction**: Architecture/tech debt cleanup — prepare the codebase for future feature work by
reducing module size, modernizing session storage, and closing permission/tool residuals.

## Objective

Turn the current post-v0.3.0 state into a disciplined four-month architecture cleanup: decompose the
largest and fastest-growing source modules, modernize session log storage from verbose JSONL to
compact text with archival, close permission and tool debt, and reduce TUI residuals — all without
expanding the permission boundary, adding speculative features, or claiming v1.0.

This plan does not authorize crate publishing, branch pushes, remote dashboard access, browser
automation, write-capable plugin tools, marketplace behavior, or permission-default changes.

## Current State (2026-07-09)

- Version: `v0.3.0`, 1791 workspace tests passing.
- REL-002: NO-GO for v1.0 (0 fully qualifying Talos-primary self-bootstrap sessions).
- I085 paused (MC107 manual TUI residual); I086-I089 planned but not started.
- I106-I109 all in Review (non-qualifying self-bootstrap evidence).
- Key architecture debt (ARCH-030 register, 5 roots grew significantly since last audit):
  - `talos-provider/src/openai.rs`: 2365 lines (+1517 since register)
  - `talos-cli/src/mode_runners.rs`: 2290 lines (+790)
  - `talos-provider/src/lib.rs` (Anthropic): 1677 lines (+844)
  - `talos-permission/src/lib.rs`: 1630 lines
  - `talos-tui/src/state.rs`: 1469 lines
  - `talos-tools/src/git.rs`: 1285 lines (+417)
- SESSION-004 revised: compact text format + archival segment chain (ADR-037).
- ADR-036 approved: zstd C binding for archival compression.

## Non-Authorizations

- No crate publish or `publish = false` removal.
- No branch push unless separately requested; release tasks may push a tag only when explicitly
  authorized.
- No remote dashboard, browser automation, web write route, remote plugin install, marketplace, or
  permission-default change.
- No v1.0 readiness claim until REL-002 evidence is complete.
- No new C/C++ dependency beyond ADR-036 (zstd).

## Four-Month Execution Matrix

| ID | Week | Iteration | Track | Deliverable | Validation | Status |
|---|---:|---|---|---|---|---|
| **Month 1: Provider Architecture + Session Format Foundation** ||||||
| T100 | 1 | I110 | Provider | Decompose `openai.rs` (2365→<800): extract SSE stream parser, retry/error mapping, leaving core request/response in `openai.rs`. | `cargo test -p talos-provider`; workspace tests | Planned |
| T101 | 1-2 | I110 | Provider | Decompose `talos-provider/src/lib.rs` (1677→<800): extract Anthropic request assembly and stream parser. | `cargo test -p talos-provider`; workspace tests | Planned |
| T102 | 2 | I110 | Session | SESSION-004 Slice A: `SessionStore` abstraction, segment chain data structures, `chain.tlog` reader/writer, JSONL legacy reader. | `cargo test -p talos-session`; compatibility tests | Planned |
| T103 | 3 | I110 | Session | SESSION-004 Slice B: compact text (`.tlog`) writer/reader, wire DTOs, corruption tolerance, density benchmark report. | `cargo test -p talos-session`; benchmark evidence | Planned |
| T104 | 4 | I110 | Release | Month-1 closeout: provider decomposition evidence, session format density report, docs sync. | workspace tests; clippy; governance | Planned |
| **Month 2: CLI/Permission Architecture + Session Export** ||||||
| T110 | 5 | I111 | CLI | Decompose `mode_runners.rs` (2290→<800): extract TUI mode, session command handling, MCP/session setup helpers. | `cargo test -p talos-cli`; workspace tests | Planned |
| T111 | 5-6 | I111 | Permission | Decompose `talos-permission/src/lib.rs` (1630→<800): extract profiles/rules, scope management, keep core types in lib.rs. | `cargo test -p talos-permission`; workspace tests | Planned |
| T112 | 6-7 | I111 | Session | SESSION-004 Slice C: transcript/export service (format-neutral, JSON + Markdown export from both JSONL and `.tlog`). | `cargo test -p talos-session`; export tests | Planned |
| T113 | 7 | I111 | TUI | Decompose `talos-tui/src/state.rs` (1469→<800): extract viewport/cursor state, approval state, session lifecycle state. | `cargo test -p talos-tui`; workspace tests | Planned |
| T114 | 8 | I111 | Release | Month-2 closeout: CLI/permission/TUI decomposition evidence, export service tests, docs sync. | workspace tests; clippy; governance | Planned |
| **Month 3: Permission Sandbox + Session Compaction + Tool Cleanup** ||||||
| T120 | 9 | I112 | Permission | PERM-004 ADR: workspace trust sandbox design (Git repo detection, trust boundary, deny precedence). | ADR draft; design review | Planned |
| T121 | 9-10 | I112 | Permission | PERM-004 first implementation: workspace trust detection + opt-in trust approval; non-Git workspaces keep strict mode. | `cargo test -p talos-permission`; permission tests | Planned |
| T122 | 10 | I112 | Session | SESSION-004 Slice D: compaction and archival engine (segment freezing, rule application, zstd compression per ADR-036). | `cargo test -p talos-session`; archival tests | Planned |
| T123 | 11 | I112 | Tool | Decompose `talos-tools/src/git.rs` (1285→<800): split read-only gix tools and host-git write helpers. | `cargo test -p talos-tools`; workspace tests | Planned |
| T124 | 11 | I112 | Tool | TOOL-020: Git diff ref-to-ref comparisons (read-only, path-filtered, `gix` or bounded host-git fallback). | `cargo test -p talos-tools`; diff tests | Planned |
| T125 | 12 | I112 | Release | Month-3 closeout: permission sandbox evidence, compaction engine tests, tool decomposition, docs sync. | workspace tests; clippy; governance | Planned |
| **Month 4: Session Compression + TUI Polish + Final Closeout** ||||||
| T130 | 13 | I113 | Session | SESSION-004 Slice E: tool output compression (Mechanism A: `raw_flag`, inline/external raw) + fork COW semantics. | `cargo test -p talos-session`; fork tests | Planned |
| T131 | 13-14 | I113 | Perf | PERF-001 Phase 1: `models.toml` compile-time materialization via `build.rs` (Phase 2 already done). | `cargo test -p talos-config`; build evidence | Planned |
| T132 | 14 | I113 | TUI | TUI-028 residuals: #25 thinking ripple (two-color three-segment), #28/#39 transient dashboard notification, #24/#31 visual evidence. | `cargo test -p talos-tui`; visual evidence | Planned |
| T133 | 15 | I113 | TUI | TUI-029 decision: thinking history archive policy. Revise ADR-034 if approved, or formally reject with rationale. | ADR revision or rejection doc | Planned |
| T134 | 15 | I113 | Extension | HOOK-001 remaining: `/hooks` lists config-introduced hooks with provenance and ordering. | `cargo test -p talos-plugin`; hook tests | Planned |
| T135 | 16 | I113 | Closeout | Final four-month matrix: ARCH-030 register update, residual owners, release posture, next handoff. | workspace tests; clippy; governance | Planned |

## Milestones

| Milestone | Target Week | Exit Criteria |
|---|---:|---|
| M1 Provider + Session format | 4 | Provider modules under 800 lines; `.tlog` format writes/reads with density evidence; JSONL compatibility verified. |
| M2 CLI + Permission + Export | 8 | CLI/permission/TUI modules under 800 lines; transcript/export service works for both formats. |
| M3 Permission sandbox + Compaction | 12 | PERM-004 workspace trust ADR accepted and first slice landed; session compaction archival works with zstd. |
| M4 Compression + Polish + Closeout | 16 | Tool output compression (Mechanism A) works; TUI residuals closed or formally deferred; ARCH-030 updated. |

## Architecture Decomposition Targets

Files exceeding 800 lines, ordered by current size and growth rate:

| File | Current | Target | Month | Key Extraction |
|---|---:|---:|---:|---|
| `talos-provider/src/openai.rs` | 2365 | <800 | 1 | SSE stream parser, retry/error mapping |
| `talos-cli/src/mode_runners.rs` | 2290 | <800 | 2 | TUI mode, session commands, setup helpers |
| `talos-provider/src/lib.rs` | 1677 | <800 | 1 | Anthropic request assembly, stream parser |
| `talos-permission/src/lib.rs` | 1630 | <800 | 2 | Profiles/rules, scope management |
| `talos-tui/src/state.rs` | 1469 | <800 | 2 | Viewport/cursor, approval, session lifecycle |
| `talos-tools/src/exec_tool.rs` | 1298 | <1000 | 3* | Output capture, timeout, permission gating |
| `talos-tools/src/git.rs` | 1285 | <800 | 3 | Read-only gix tools, host-git write helpers |
| `talos-tools/src/bash_tool.rs` | 1236 | <1000 | 3* | Template policy, permission profiles |
| `talos-tui/src/app.rs` | 1206 | <800 | 2* | Frame/cursor/output queue helpers |

*Months 2-3 items marked with `*` are secondary targets — decompose if primary targets complete
early; otherwise defer to a follow-up plan. All decomposition is behavior-preserving.

## Dependencies

```text
T100/T101 (provider decomposition) → independent
T102 (SessionStore abstraction) → T103 (compact text writer/reader)
T103 → T112 (export service) → T122 (compaction engine) → T130 (tool output compression)
T110 (CLI decomposition) → independent
T111 (permission decomposition) → T120/T121 (PERM-004)
T113 (TUI state decomposition) → T132 (TUI-028 residuals) [soft dependency]
T123 (git.rs decomposition) → T124 (TOOL-020 ref-to-ref diff) [soft dependency]
```

Most tracks are independent and can be parallelized across agents within each month.

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Decomposition introduces subtle behavior change | Medium | High | Behavior-preserving gates; workspace tests must pass before/after each extraction |
| SESSION-004 archival complexity exceeds estimate | Medium | Medium | Slices are independently deliverable; compaction can defer to a later iteration |
| PERM-004 ADR requires security review not available | Low | High | ADR can be accepted without implementation; implementation waits for review |
| zstd C compilation fails on a target | Low | Medium | Feature-gate zstd behind `archive-compress-zstd`; lz4_flex fallback per ADR-036 |
| External agent execution quality varies | Medium | Medium | Each task has explicit acceptance criteria; validation gates per task |

## Recovery Instructions

1. Run `git status --short`.
2. Read this file, `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and the current iteration.
3. Continue from the lowest-numbered planned T-task unless the maintainer explicitly changes
   priority.
4. Update owner docs before derived board/backlog views.
5. Run `scripts/validate_project_governance.sh .` after governance changes.

## Governance Documents Created

| Document | Purpose |
|---|---|
| `docs/decisions/036-zstd-compression-dependency.md` | zstd C binding scoped exception (ADR-008 pattern) |
| `docs/decisions/037-compact-text-session-log-format.md` | Session log format and archival architecture decision |
| `docs/backlog/active/SESSION-004-binary-session-log-format.md` | Revised implementation-ready story (5 slices) |
| `docs/backlog/active/COMP-001-pure-rust-compression-migration.md` | Pure Rust compression candidate tracking |
| This document | Four-month execution plan |

## Related Documents

- `docs/backlog/active/ARCH-030-remaining-production-root-residual-register.md` — architecture debt register
- `docs/backlog/active/ARCH-022-cli-mode-runner-residual-decomposition.md` — CLI decomposition
- `docs/backlog/active/ARCH-023-tui-app-residual-decomposition.md` — TUI decomposition
- `docs/backlog/active/PERM-004-workspace-trust-sandbox.md` — workspace trust sandbox
- `docs/backlog/active/PERF-001-compile-time-embedded-toml.md` — compile-time TOML
- `docs/backlog/active/TUI-028-preview-status-feedback-reliability.md` — TUI feedback
- `docs/backlog/active/TUI-029-thinking-history-archive.md` — thinking history decision
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md` — config hooks
- `docs/backlog/active/TOOL-020-git-diff-ref-comparisons.md` — git diff ref-to-ref
