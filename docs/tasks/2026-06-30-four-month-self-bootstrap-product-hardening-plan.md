# 2026-06-30 Four-Month Self-Bootstrap Product Hardening Plan

**Status**: Ready for delegated execution
**Owner area**: Product hardening, self-bootstrap readiness, distribution, tool quality, web/context
ingestion, memory/context reliability, and extensibility gates
**Primary release marker**: `REL-002` v1.0 self-bootstrap gate
**Created**: 2026-06-30
**Timebox**: 16 weeks / roughly 4 months

## Objective

Move Talos from a useful pre-1.0 agent runtime toward a credible self-bootstrap product: Talos
should be installable through normal user channels, usable as an embeddable runtime, able to ingest
web/document context safely, able to govern its own work, and able to record enough evidence for
future Talos-on-Talos development sessions.

This is not a 1.0 release authorization. It is a concrete four-month execution plan that produces
the prerequisites and evidence needed before `REL-002` can become a real release gate.

## Operating Constraints

- Do not tag, release, publish a new crate, or remove `publish = false` without explicit maintainer
  approval for that exact action.
- Plugin architecture ADRs are accepted, but runtime implementation still requires the focused
  dependency/security review named by ADR-027.
- Do not add browser automation, plugin runtime dependencies, local-model dependencies, vector
  stores, or remote-control transports without the owning ADR/spike gate.
- Do not claim `v1.0.0` readiness until `REL-002` evidence exists.
- Preserve existing published iteration baselines. Append execution facts instead of rewriting
  older plans.
- Every implementation slice must update owner docs before `docs/BOARD.md`.

## Success Criteria

- CLI installation has a validated `cargo install talos-cli --bin talos` path and clear README
  documentation, even if real crates.io publication is separately approved later.
- `talos-runtime` has a usable SDK support contract, examples, and publish-gate evidence.
- Tooling is reliable enough for self-development: search, fetch, file, Git, shell, and TUI result
  display have deterministic, bounded behavior.
- Web/context ingestion has a safe `fetch_url` facade, bounded local document extraction, and a
  gated browser-session continuity design with at least one prototype path behind permissions.
- Governance and self-bootstrap flows can select work, preserve board/backlog/iteration integrity,
  and record validation evidence without prompt-only discipline.
- Memory/context systems can support long-running development sessions without hidden-output leaks
  or unbounded prompt growth.
- At least two dry-run Talos-on-Talos development sessions are recorded as rehearsal evidence. They
  need not satisfy `REL-002` completely, but they must expose remaining gaps.

## Required Reads

- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/active/ARCH-031-crate-publication-boundary.md`
- `docs/reference/CRATE-PUBLICATION-MATRIX.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/WEB-005-browser-session-continuity-research.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md`
- `docs/backlog/active/MEM-007-active-context-compression.md`
- `docs/backlog/active/MEM-008-weighted-associative-memory-graph.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/proposals/plugin-encapsulation-format.md`
- `docs/backlog/active/CMD-002-command-taxonomy-realignment.md`
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md`
- `docs/backlog/active/TOOL-011-ripgrep-backed-grep-engine.md`
- `docs/backlog/active/TUI-014-grep-result-summary.md`
- `docs/backlog/active/TUI-015-head-tail-truncation.md`
- `docs/backlog/active/WEB-003-site-internationalization.md`
- `docs/backlog/active/WEB-004-site-theme-branding.md`

## Track Overview

| Track | Theme | Outcome |
|---|---|---|
| A | Governance and self-bootstrap evidence | Talos can plan, execute, validate, and record work with less external scaffolding. |
| B | Distribution and SDK readiness | Cargo install path, runtime SDK docs, publish gates, and package guardrails are executable. |
| C | Tooling and context ingestion | Search/fetch/document/Git/shell/TUI output paths are bounded and dependable. |
| D | Web control and browser continuity | WEB-001/WEB-005 move from research to gated MVP designs and safe prototypes. |
| E | Memory and context reliability | Long-session memory/context behavior is measurable, bounded, and safe. |
| F | Extensibility unblockers | Plugin/hook/command work receives ADRs and narrow implementation slices only after gates. |
| G | Product docs and release readiness | Public site, README, release notes, and user docs match actual shipped behavior. |

## Four-Month Execution Matrix

| ID | Week | Track | Deliverable | Dependencies | Validation | Status |
|---|---:|---|---|---|---|---|
| T00 | 1 | A | Inventory Active/Review/Planned/Blocked work and publish the starting disposition checkpoint. | Board/backlog reads | Governance validator; no owner-doc drift | Complete |
| T01 | 1 | A | Create iteration slices for this plan without rewriting older baselines. | T00 | New iteration docs pass governance | Complete |
| T02 | 1 | B | Audit `talos-cli` package metadata for Cargo install readiness. | ARCH-031 | `cargo metadata`; package manifest review | Complete |
| T03 | 1 | B | Design `cargo install talos-cli --bin talos` gate: package name, binary target, README, dry-run, uninstall/upgrade notes. | T02 | Gate checklist in ARCH-031/matrix | Complete |
| T04 | 1 | C | Audit current native tool surface after `fetch_url`/`http_request` split. | TOOL-014 | Tool list snapshot; prompt-surface diff | Complete |
| T05 | 1 | G | Define docs sync checklist for README, README.zh-CN, site, release notes, and crates.io docs. | T00 | Checklist committed | Complete |
| T06 | 2 | B | Add or update crate/package README content for `talos-cli` binary install without promising library API. | T03 | README link checks; package list | Complete |
| T07 | 2 | B | Verify local install path: `cargo install --path crates/talos-cli --bin talos` into a temp `CARGO_HOME`. | T03 | Install smoke; `talos --version` | Complete |
| T08 | 2 | B | Run `cargo publish --dry-run -p talos-cli` without removing `publish = false`; record blockers. | T03 | Dry-run evidence or blocker list | Complete |
| T09 | 2 | C | Implement TUI-014 grep result summary rendering. | TOOL-011/TUI docs | TUI tests; snapshot-free rendering tests | Complete |
| T10 | 2 | C | Implement TUI-015 head+tail truncation for long unsuppressed tool outputs. | TUI-014 | TUI tests; `/export` raw-output proof | Complete |
| T11 | 2 | A | Add self-bootstrap session evidence template under docs/tasks or docs/reference. | REL-002 | Governance validation | Complete |
| T12 | 3 | B | Add runtime SDK quickstart examples for provider/tool injection, approvals, custom/append prompt, preview, shutdown. | RUNTIME-001 | `cargo test -p talos-runtime`; docs compile where applicable | In Progress |
| T13 | 3 | B | Define `talos-runtime` SDK support contract and direct-use caveats for `talos-agent`. | ARCH-031/RUNTIME-001 | Docs updated; matrix updated | Complete |
| T14 | 3 | C | Start TOOL-011 ripgrep-backed grep engine implementation behind a feature or internal engine boundary. | ADR-025 | Unit tests compare current grep behavior | Planned |
| T15 | 3 | C | Add regression tests for search hidden-dir behavior, include filters, large output summary, and UTF-8 snippets. | T14 | `cargo test -p talos-tools` | Planned |
| T16 | 3 | G | Update public site roadmap to reflect cargo install, SDK, and self-bootstrap positioning accurately. | T05 | Site validator | Complete |
| T17 | 4 | C | Finish first ripgrep-backed grep slice or record a precise blocker. | T14 | Parity tests; performance note | Planned |
| T18 | 4 | D | WEB-005 design: browser-session continuity permission model, page record schema, and no-cookie-leak boundary. | WEB-005/BrowserSkill research | ADR/proposal update | Planned |
| T19 | 4 | D | Define `browser_page_read` permission facet and how it composes with `fetch_url` continuation disclosure. | T18/TOOL-013 | Permission tests planned | Planned |
| T20 | 4 | E | MEM-007 spike: deterministic pre-entry compression strategies for read/grep/git_diff/bash outputs. | MEM-007 | Prototype notes; cache-stability risks | Planned |
| T21 | 4 | A | Month-1 closeout: validation summary, delivered items, blockers, next-month replan. | T00-T20 | `cargo test --workspace`; governance | Planned |
| T22 | 5 | D | WEB-001 MVP design: loopback-only dashboard for status/history/governance/config read surfaces. | WEB-001 | Proposal update; threat model | Planned |
| T23 | 5 | D | Define local web auth boundary: loopback constraints, startup token, no secret echo, no permission bypass. | T22 | Security checklist | Planned |
| T24 | 5 | C | Harden `fetch_url`: redirect diagnostics, sparse HTML hints, content-type summary, continuation tests. | TOOL-014/WEBFETCH | `cargo test -p talos-tools -p talos-agent` | Planned |
| T25 | 5 | C | Document `fetch_url` vs `http_request` vs `save_url` in README and site capability pages. | T24 | Docs/site validators | Planned |
| T26 | 5 | E | Implement MEM-007 minimal compression packet for one low-risk tool family, default off. | T20 | Raw output preserved; stable-prefix test | Planned |
| T27 | 5 | A | Add governance status command enhancement or report mode for active/blocked/planned disposition. | GOV-003 | CLI tests; governance fixture tests | Planned |
| T28 | 6 | D | Prototype WEB-001 read-only dashboard if ADR/design gate passes; otherwise leave implementation blocked with exact reasons. | T22/T23 | Localhost smoke; no secret leakage | Planned |
| T29 | 6 | D | WEB-005 prototype design for browser page record ingestion without automation. | T18/T19 | Mock tests; no real browser dependency unless approved | Planned |
| T30 | 6 | E | MEM-008 schema spike for weighted association graph: nodes, edge weights, decay, multi-hop bounds. | MEM-008 | SQLite migration prototype tests | Planned |
| T31 | 6 | E | Research automatic associative memory injection: budget, triggers, default-off policy, evaluation metrics. | T30 | Decision note; no default enable | Planned |
| T32 | 6 | F | Plugin runtime boundary ADR: WASM v1, `wasmtime` preferred pending dependency review, dylib rejected, Lua deferred. | plugin proposal | ADR-027 accepted | Complete |
| T33 | 6 | F | Plugin provenance ADR for future `ToolProvenance::Plugin`. | T32 | ADR-028 accepted | Complete |
| T34 | 7 | F | Atomic component model ADR for skill/mcp/hook and plugin package declarations. | plugin proposal | ADR-029 accepted | Complete |
| T35 | 7 | F | Command taxonomy ADR: `/mcp`, `/plugins`, `/hooks`, and transition notice policy. | CMD-002 | ADR-030 accepted | Complete |
| T36 | 7 | C | Add permission/profile audit tests for `fetch_url`, `http_request`, `save_url`, and future browser-page facet. | T19/T24 | Permission boundary tests | Planned |
| T37 | 7 | B | Update CRATE-PUBLICATION-MATRIX with cargo install dry-run evidence and SDK publish blockers. | T07/T08/T13 | Matrix reviewed | Planned |
| T38 | 7 | A | First Talos-on-Talos rehearsal session: documentation-only change with full evidence template. | T11/GOV-003 | Evidence record; external assistance labeled | Planned |
| T39 | 8 | A | Month-2 closeout and replan: decide whether WEB-001/WEB-005 are ready and whether ADR-027 dependency/security review clears plugin runtime work. | T22-T38 | Workspace tests; governance | Planned |
| T40 | 9 | F | Implement minimal `ToolProvenance::Plugin` data model and rendering paths. | T33 | Core/conversation/TUI tests | Planned |
| T41 | 9 | F | Implement `/mcp` command and `/plugins` transition notice; `/plugins` no longer silently means MCP. | T35/CMD-002 | Conversation/TUI command tests | Planned |
| T42 | 9 | D | Implement WEB-001 read-only status/history/governance page subset if Month-2 gate passed. | T28 | Browser/local smoke; no secret echo | Planned |
| T43 | 9 | E | Implement weighted-memory graph storage behind a feature/config flag if spike accepted. | T30/T31 | SQLite tests; retrieval deterministic | Planned |
| T44 | 9 | C | Complete ripgrep-backed grep engine or keep current engine with recorded rejection/blocker. | T17 | Parity/performance evidence | Planned |
| T45 | 10 | F | Implement plugin manifest parser only; no executable artifact instantiation during discovery. | T32/T34 | Parser tests; schema validation | Planned |
| T46 | 10 | F | After ADR-027 dependency/security review, implement one local WASM plugin package fixture with a read-only tool behind permission gate. | T45 | Trap/timeout/error tests | Planned |
| T47 | 10 | D | Implement safe browser-page record mock backend for `fetch_url` if WEB-005 gate passed. | T29/T36 | No cookie/storage exposure; continuation tests | Planned |
| T48 | 10 | B | Prepare `talos-runtime` publish gate: dry-run dependency closure, SDK docs, examples, support caveats. | T13/T37 | `cargo publish --dry-run -p talos-runtime` or blocker | Planned |
| T49 | 10 | G | WEB-003 zh-CN site translation slice. | WEB-003 | Site validator; link checks | Planned |
| T50 | 11 | E | Implement associative recall API default-off; no automatic prompt injection yet. | T43 | Unit tests; bounded multi-hop tests | Planned |
| T51 | 11 | E | Add metrics for memory/context compression: tokens saved, retrieval hits, hidden-output drops. | T26/T50 | Metrics tests; docs | Planned |
| T52 | 11 | A | Second Talos-on-Talos rehearsal: small code change through Talos runtime if feasible. | T38/tooling readiness | Evidence record; validation | Planned |
| T53 | 11 | G | WEB-004 site theme/branding optimization using Talos visual identity. | WEB-004 | Site validator; no external assets | Planned |
| T54 | 12 | A | Month-3 closeout: self-bootstrap gap report against every REL-002 acceptance criterion. | T00-T53 | REL-002 checklist update | Planned |
| T55 | 13 | B | If explicitly approved, remove `publish = false` for `talos-cli` and publish/install smoke; otherwise keep as ready gate. | T07/T08/maintainer approval | Real publish or documented non-action | Planned |
| T56 | 13 | B | If explicitly approved, publish or reserve `talos-runtime`; otherwise record exact blockers. | T48/maintainer approval | Real publish or documented non-action | Planned |
| T57 | 13 | C | Tool reliability sweep: flaky tests, Windows/Unix command assumptions, shell naming residuals. | TOOL-006/tool docs | Targeted tests; issue list | Planned |
| T58 | 13 | D | WEB-001/WEB-005 security review: auth, permissions, provenance, logs, local-only guarantees. | T42/T47 | Review document; threat model | Planned |
| T59 | 14 | F | Plugin MVP security review and decision: continue implementation, defer, or split smaller. | T46 | Review document; no hidden runtime dep | Planned |
| T60 | 14 | E | Decide automatic associative memory injection: reject, keep default-off, or enable under config with metrics. | T50/T51 | ADR/proposal update | Planned |
| T61 | 14 | A | Third Talos-on-Talos rehearsal: architecture-sensitive doc/code slice if feasible. | T52/T54 | Evidence record | Planned |
| T62 | 15 | G | Consolidate release/user docs: README, site, crate docs, install docs, SDK examples, changelog draft. | All tracks | Link/site validators | Planned |
| T63 | 15 | A | Produce v1.0 readiness report: pass/fail for REL-002, remaining blockers, and next quarter plan. | T61/T62 | Governance validation | Planned |
| T64 | 16 | A | Final four-month closeout: validation matrix, commits, unreleased changes, follow-up backlog updates. | T00-T63 | `cargo test --workspace`; governance; publish guard | Planned |
| T65 | 16 | G | Handoff final artifacts to maintainer: release posture, install posture, SDK posture, self-bootstrap posture. | T64 | Final handoff doc | Planned |

## Milestones

| Milestone | Target Week | Exit Criteria |
|---|---:|---|
| M1 Starting gate complete | 1 | Inventory, iteration slices, and cargo-install gate design exist. |
| M2 User-visible tooling hardening | 4 | TUI output summarization, grep/ripgrep plan, and WEB-005 permission model are ready or blocked precisely. |
| M3 Web/governance/memory prototypes | 8 | WEB-001/WEB-005/MEM-007/MEM-008 have accepted prototype paths or explicit blockers. |
| M4 Extensibility unblocked or deferred | 12 | Plugin ADRs are accepted and MVP started, or implementation remains blocked with exact decisions. |
| M5 Release posture known | 16 | Cargo install, SDK publish, self-bootstrap readiness, and docs posture are all explicitly recorded. |

## Validation Policy

Minimum validation before closing any implementation slice:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- Targeted `cargo test -p <crate>` for touched crates
- `scripts/validate_project_governance.sh .`
- `scripts/check_publish_guard.sh .` when any Cargo manifest or publish plan changes
- Site validator when `site/` or public-site docs change
- Permission/security review note when touching browser, web control, plugin runtime, sandbox, or
  tool execution boundaries

Run `cargo test --workspace` at each monthly closeout and before the final closeout.

## Explicit Non-Authorizations

This plan does not authorize:

- real `cargo publish`;
- release tags;
- GitHub Releases;
- new runtime dependencies for browser/plugin/vector/local-model work;
- enabling browser automation;
- enabling automatic memory injection by default;
- changing permission defaults;
- making `talos-cli` a stable library API.

Those require separate maintainer approval or accepted ADRs as named above.

## Team Handoff Prompt

Use this prompt to hand the work to a development team:

```text
You are taking over Talos's four-month self-bootstrap product hardening plan.

Start by reading:
- docs/tasks/2026-06-30-four-month-self-bootstrap-product-hardening-plan.md
- docs/BOARD.md
- docs/backlog/PRODUCT-BACKLOG.md
- docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md
- docs/backlog/active/ARCH-031-crate-publication-boundary.md
- docs/reference/CRATE-PUBLICATION-MATRIX.md

Your job is to execute the plan in order, preserving Talos governance:
1. Owner docs are the source of truth. Update owner docs before docs/BOARD.md.
2. Do not rewrite published iteration baselines. Append execution facts.
3. Do not publish crates, remove publish=false, create tags/releases, add browser/plugin/vector/local-model runtime dependencies, or alter permission defaults without explicit maintainer approval or an accepted ADR.
4. Plugin architecture is unblocked by ADR-027/028/029/030. Runtime implementation still requires ADR-027's focused `wasmtime` dependency/security review; do not add Lua, dylib, marketplace, remote install, auto-discovery, or write-capable plugin tools in the first slice.
5. Keep cargo install work focused on the binary package path: cargo install talos-cli --bin talos. Do not promise a stable talos-cli library API.
6. Every slice must have runnable validation: cargo fmt/check, targeted tests, governance validation, and broader workspace tests at monthly closeouts.
7. Record self-bootstrap rehearsal evidence using the plan's evidence template. Label any external agent help explicitly.

Begin with T00-T05. At the end of each week, update this task with completed items, blockers,
validation evidence, and the next ready item. At the end of each month, run cargo test --workspace
and scripts/validate_project_governance.sh ., then write a closeout checkpoint.
```

## Recovery Instructions

If work is interrupted:

1. Run `git status --short`.
2. Read this task's latest checkpoint.
3. Read the owner docs for the next planned item.
4. Run `scripts/validate_project_governance.sh .` before editing governance files.
5. Continue from the lowest-numbered planned item that is not complete, unless the maintainer
   explicitly changes priority.

---

## Execution Log

### Approved Startup Contract (2026-06-30)

Recorded per `docs/sop/LONG-RUNNING-TASK.md` before status became `In Progress`.

- **Outcome**: Execute the four-month matrix in order, pushing until a natural block, with
  segmented commits and pushes to `main`.
- **Scope this run**: Push from T00 forward until a task requires unapproved action (real
  `cargo publish`, tag/release, `publish = false` removal, new plugin/browser/vector/local-model
  runtime dependency without a cleared ADR, or maintainer-only approval).
- **Out of scope this run**: T46 (wasmtime runtime dep — needs ADR-027 focused review),
  T55/T56 (real publish — needs maintainer approval). These are recorded as blockers, not
  executed.
- **Branch**: `main` (consistent with repository history).
- **Commits/pushes**: Segmented per task cluster; baseline commit `13e93b9` already pushed.
  Subsequent commits use `[model:glm-5.2]`.
- **Validation per slice**: `cargo fmt --all -- --check`, `cargo check --workspace`, targeted
  `cargo test -p <crate>`, `scripts/validate_project_governance.sh .`,
  `scripts/check_publish_guard.sh .` when manifests change.
- **Default for ambiguity**: Follow confirmed defaults (record-and-skip blocked items; owner
  docs before BOARD; append, never rewrite published baselines).
- **Interrupt condition**: Stop and checkpoint when an unconfirmed irreversible action is
  required or when three consecutive validation failures occur on one slice.

### Checkpoint T00 — Starting Disposition (2026-06-30)

**Task**: T00 — Inventory Active/Review/Planned/Blocked work and publish the starting
disposition checkpoint.

**Starting inventory** (sourced from `docs/BOARD.md`, verified against owner docs 2026-06-30):

| Bucket | Item | Owner Doc | Starting State |
|---|---|---|---|
| Active | R27 High-Risk Governance Gate | [task](2026-06-27-personal-oversight-high-risk-roadmap.md) | In Progress; T2/I058 moved to Review. Gate grants no personal approval authority. |
| Active | Two-Month Architecture Optimization | [task](2026-06-27-two-month-architecture-optimization-plan.md) | Complete (fulfilled); production roots under ARCH-030. |
| Paused | Architect-Owned High-Risk Work Group | [task](2026-06-28-architect-owned-high-risk-work-group.md) | Paused by maintainer 2026-06-29. |
| Paused | I011 S2 Provider Plugin Architecture | [PROV-001](../backlog/active/PROV-001-provider-schema.md) | Superseded by I015; schema-only under ADR-013. |
| Planned (this plan) | REL-002 v1.0 Self-Bootstrap Gate | [REL-002](../backlog/active/REL-002-v1-self-bootstrap-release-gate.md) | Target release marker for this plan. |
| Planned | Plugin Encapsulation Architecture | ADR-027/028/029/030 | Accepted baseline 2026-06-30. Implementation gated by ADR-027 wasmtime review. |
| Planned | PLUGIN-001 Plugin System | [PLUGIN-001](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md) | Local WASM MVP after ADR-027 review. |
| Planned | CMD-002 Command Taxonomy | [CMD-002](../backlog/active/CMD-002-command-taxonomy-realignment.md) | ADR-030 accepted; `/mcp` + `/plugins` transition notice. |
| Planned | HOOK-001 Config Hooks | [HOOK-001](../backlog/active/HOOK-001-config-introduced-hooks.md) | ADR-029 accepted; config schema + diagnostics first. |
| Planned | RUNTIME-001 Embeddable Runtime | [RUNTIME-001](../backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md) | Pre-1.0 facade complete; SDK contract + examples remain. |
| Planned | ARCH-031 Crate Publication Boundary | [ARCH-031](../backlog/active/ARCH-031-crate-publication-boundary.md) | 11 crates at 0.2.0; product crates `publish = false`. |
| Planned | TOOL-011 Ripgrep Grep Engine | [TOOL-011](../backlog/active/TOOL-011-ripgrep-backed-grep-engine.md) | ADR-025 selected library crates; implementation not started. |
| Planned | WEB-005 Browser Session Continuity | [WEB-005](../backlog/active/WEB-005-browser-session-continuity-research.md) | Research; `fetch_url` backend design + permission facet. |
| Planned | WEB-001 Embedded Web Control | [WEB-001](../backlog/active/WEB-001-embedded-web-control-surface.md) | Research; loopback-only dashboard MVP. |
| Research | MEM-007 Active Context Compression | (referenced in plan) | Spike target for Month 1. |
| Research | MEM-008 Weighted Memory Graph | [MEM-008](../backlog/active/MEM-008-weighted-associative-memory-graph.md) | Schema spike target for Month 2. |
| Refinement | SESSION-001 Interactive Session Lifecycle | [SESSION-001](../backlog/active/SESSION-001-interactive-session-lifecycle.md) | Children delivered; refinement continues. |
| Tracking | ARCH-011 Architecture Watchlist | [ARCH-011](../backlog/active/ARCH-011-architecture-watchlist.md) | Promote only on concrete evidence. |

**No items in Review** at start (all resolved 2026-06-29 per BOARD).

**Dispositions for this plan's dependencies**:
- Architecture block on PLUGIN-001/CMD-002/HOOK-001 is **cleared** by ADR-027/028/029/030.
  These items are Planned, not Blocked, for this plan's purposes.
- Runtime plugin implementation (T40, T45, T46) remains gated by ADR-027's focused wasmtime
  dependency/security review — out of scope until that review is recorded.
- Real publication (T55, T56) remains gated by explicit maintainer approval — out of scope.

**Validation**: `scripts/validate_project_governance.sh .` to be run after this checkpoint is
committed; must report no owner-doc drift introduced by this plan's status updates.

**Next item**: T01 — Create iteration slices. Then T02–T05 (parallel-safe audits and checklists).

### Checkpoint Week 1 (T00–T05) — Complete (2026-06-30)

**Completed task items**: T00, T01, T02, T03, T04, T05.

**Current state and artifacts**:
- `docs/iterations/I075-month1-starting-gate-and-tooling-hardening.md` — Month 1 iteration
  covering T00–T21, following the iteration TEMPLATE.
- `docs/reference/CRATE-PUBLICATION-MATRIX.md` §A7 — cargo install gate checklist (T03):
  package identity table, local install path (works today), crates.io blockers (3 independent),
  install-doc requirements, exit criteria.
- `docs/reference/DOCS-SYNC-CHECKLIST.md` (T05): tool/command/install/SDK sync surfaces with
  baseline counts (30 native tools, 8 families, 1 hidden).
- `docs/iterations/README.md` updated with I075 row.
- `docs/reference/README.md` updated with new reference docs.
- T00 disposition checkpoint above.

**Key findings from audits**:
- T02: `cargo install --path crates/talos-cli --bin talos` works today. crates.io publish blocked
  by 3 independent factors (`publish = false` + 4 unpublished workspace deps + 2 transitive
  `publish = false` deps). No gap to fix for the `--path` path.
- T04: 30 native tools across 8 families; 29 presented by default; `http_request` is the only
  hidden tool (AdvancedNetwork family, disclosed via continuation). Current baseline is clean.

**Commands/checks and actual results**:
- `scripts/validate_project_governance.sh .` → 0 warnings (after T00 checkpoint).
- Governance re-validation after Week 1 artifacts pending commit.

**Open risks or deviations**: None. All Week 1 items were audit/design/doc deliverables with zero
code changes.

**Next task item**: T06 (talos-cli README for binary install), T07 (local install smoke),
T08 (dry-run evidence), T09 (TUI-014 grep summary), T10 (TUI-015 truncation), T11 (evidence
template). T06–T08 are Track B distribution work; T09–T10 are TUI implementation; T11 is
governance. All are independent and can be parallelized.

**Recovery or resume instruction**: Baseline commit `13e93b9` is on `origin/main`. Week 1
artifacts are staged for the next commit. To resume: read this checkpoint, then start T06 or T09.

### Checkpoint Week 2 (T06–T11) — Complete (2026-06-30)

**Completed task items**: T06, T07, T08, T09, T10, T11.

**Current state and artifacts**:
- `crates/talos-cli/README.md` (T06): crate-level binary install README with cargo install/upgrade/
  uninstall instructions and binary-only support boundary.
- T07 install smoke: `cargo install --path crates/talos-cli --bin talos --root
  /tmp/talos-install-smoke` succeeded (4m 48s release build); `talos --version` → `talos 0.2.0`.
- T08 dry-run: `cargo publish --dry-run -p talos-cli` fails immediately — `publish = false`
  blocks even the packaging step. Evidence recorded in CRATE-PUBLICATION-MATRIX §A7.
- `docs/reference/SELF-BOOTSTRAP-EVIDENCE-TEMPLATE.md` (T11): rehearsal evidence template for
  Talos-on-Talos sessions (T38/T52/T61).
- TUI-014 (T09): `grep` added to `THRESHOLD_SUMMARIZE` set; `summarize_grep_result()` parses
  file/match counts from grep output format with fallback. Summary format:
  `grep matched {N} lines in {M} files, {B} bytes`.
- TUI-015 (T10): `build_head_tail_scrollback_lines()` — first 10 lines, dim `⋯ {N} lines omitted`
  separator, last 10 lines. Uses shared `SUMMARIZE_OUTPUT_THRESHOLD_LINES = 30` gate. `/export`
  path confirmed unaffected (writes raw content, never enters scrollback builder).

**Commands/checks and actual results**:
- `cargo install --path crates/talos-cli --bin talos` → success, `talos 0.2.0` runs.
- `cargo publish --dry-run -p talos-cli` → blocked by `publish = false` (expected).
- `cargo test -p talos-tui` → 171 passed, 0 failed (+2 doctests).
- `cargo clippy -p talos-tui -- -D warnings` → no warnings.
- `cargo fmt --all -- --check` → clean.
- `scripts/validate_project_governance.sh .` → 0 warnings.

**Open risks or deviations**: None. T09/T10 are display-layer-only changes; no tool data or
export path affected. 8 new tests pin the behavior including the `/export` invariant.

**Next task item**: Week 3 — T12 (runtime SDK quickstart examples), T13 (SDK support contract),
T14 (ripgrep grep engine first slice), T15 (search regression tests), T16 (site roadmap update).

**Recovery or resume instruction**: Week 1+2 commits on `origin/main`. To resume: read this
checkpoint, then start T12 or T14 (both independent).
