# 2026-07-01 Month-3 Handoff for Architecture Review

> Status: Submitted for architecture review
> Originator: Sisyphus session 2026-07-01 (Month 3, Weeks 9–12)
> Reviewer: Architecture (Oracle)
> Purpose: Consolidate Month 3 work, document decisions and deviations, and escalate
> unresolved items for judgment.

## 1. Session Summary

Month 3 executed Weeks 9–12 of the four-month self-bootstrap product hardening plan. All
implemented tasks are governance-validated; full workspace test passes; all commits pushed to
`origin/main` and per-task documentation closeouts written into the four-month plan execution
log.

### 1.1 Month-3 task items delivered

| Task | Title | Delivery |
|---|---|---|
| T28 | WEB-001 dashboard prototype | Fulfilled by T42 (revised gate per ADR-031) |
| T40 | ToolProvenance::Plugin variant | `Plugin { name, version, carrier }` added to `talos-core/src/tool.rs`; 3 exhaustive match arms updated; 8 tests across core/conversation/TUI |
| T41 | `/mcp` command + `/plugins` transition | Per ADR-030: `/mcp` shows MCP status; `/plugins` shows transition notice (not alias); 5 tests + README updates |
| T42 | WEB-001 loopback dashboard MVP | **New crate** `talos-dashboard` with `axum 0.8.9`; loopback-only (127.0.0.1:0), per-process bearer token auth, 4 GET-only routes (`/status` `/history` `/governance` `/config`); 10 security tests |
| T43 | Weighted-memory graph storage | `talos-memory/graph.rs`: `GraphNode` / `GraphEdge` / `AssociationResult`, SQLite migration v3, deterministic bounded multi-hop BFS (max 3 hops, min weight 0.3, top-k fanout), exponential decay (7-day half-life); 11 tests |
| T44 | ripgrep engine completion / blocker | Closed by existing T14 ripgrep library engine + 12 parity tests; no work required |
| T45 | Plugin manifest parser | `talos-plugin/src/manifest.rs`: `PluginManifest` / `PluginMetadata` / `PluginSkill` / `PluginTool` with serde+TOML; validation: carrier must be `"wasm"`, non-empty fields, unique tool names; 13 tests |
| T46 | Local WASM plugin fixture | `talos-plugin/src/wasm.rs`: `WasmRuntime` + `WasmModule` behind `wasm` Cargo feature; fuel + epoch interruption timeout; 8 resource/failure tests (ADR-032 categories) |
| T47 | Browser-page record mock backend | `talos-tools/src/browser_page.rs`: `BrowserPageRecord` struct, `BrowserPageLink`, `BrowserPageConnector` trait, `MockBrowserPageConnector` (fixture-based); 8 tests proving no cookies/storage/credentials/DOM leak |
| T48 | talos-runtime publish gate | **Documentation only**: ran `cargo publish --dry-run -p talos-runtime`, captured 4 unpublished workspace dep blockers, updated CRATE-PUBLICATION-MATRIX §A5 |
| T49 | zh-CN site translation | 7 Chinese pages under `site/zh/`, language switchers on all 14 pages (EN/ZH), `scripts/validate_public_site.sh` extended; 0 errors, 0 warnings |
| T50 | Associative recall API default-off | Fulfilled by T43 `graph_recall()`; no additional work |
| T51 | Memory/context compression metrics | `talos-agent/src/compression.rs`: `CompressionMetrics` (events, bytes_saved, estimated_tokens_saved) + `RetrievalMetrics` (calls, results, avg); 5 tests |
| T52 | Second Talos-on-Talos rehearsal | **Real Talos-driven** (not retrospective): Talos autonomously added `TestVariant` to `ToolProvenance` across 3 crates; ~45% self-bootstrap coverage (code edit phase only; tests/validation/commit by external). Evidence: `docs/tasks/2026-07-01-self-bootstrap-rehearsal-t52.md`. Rehearsal code reverted. |
| T53 | WEB-004 site theme/branding | Nord color palette applied to `site/assets/styles.css`; hexagon mark + Nord Frost gradient in `talos-mark.svg` and `favicon.svg`; 0 errors, 0 warnings |
| T54 | Month-3 closeout | This handoff + `cargo test --workspace` = 1347 passed, governance 0 warnings, publish guard PASSED |

### 1.2 Items beyond the plan

| Item | Description |
|---|---|
| ADR-032 wasmtime vs wasmer comparative analysis | Added 6-dimension comparison table + wasmer to Rejected Alternatives; closes documentation gap that ADR-027 named wasmtime without comparison |
| Issue Sync Rule | New governance rule in PRODUCT-BACKLOG.md §Issue Sync Rule + AGENTS.md §Session End Checklist #7: backlog items sourced from GitHub issues must sync status changes back to the issue |
| Issue #7 → TUI-016 | Slash panel smart auto-execute: parameterless commands execute on Enter; parameter commands fill input. Backlog story created. |
| Issue #8 → TODO-001 | Session-level todo list for plan orchestration: user commands view-only, agent tools handle mutations. Backlog story created. |
| Bug fix: UTF-8 panic in approval panel truncation | `crates/talos-tui/src/state.rs:226` used `&arguments[..72]` byte-index slice which panics when byte 72 lands inside a multibyte CJK character. Fix walks back to `is_char_boundary`. + 2 regression tests. **Discovered by user's T52 rehearsal.** |
| TUI panic hook | `mode_runners.rs:415` installs `set_hook` to print panic info to stderr before default handler — future TUI crashes now show their panic message instead of being silently swallowed by raw mode |

## 2. Decisions and documentation corrections

### 2.1 New / corrected ADRs and rules

| ADR / Rule | Status | Reason |
|---|---|---|
| ADR-032 (wasmtime vs wasmer) | Updated 2026-07-01 | Added § Runtime Selection: wasmtime vs wasmer with 6-dimension comparison; wasmer added to Rejected Alternatives (core filesystem sandbox bypass CVEs vs wasmtime's non-default Winch-backend-only escapes) |
| PRODUCT-BACKLOG.md § Issue Sync Rule | Added 2026-07-01 | External GitHub issue reporters and watchers should see status changes via issue comments; closes a transparency gap |
| PLUGIN-001 | Updated 2026-07-01 | Step 2 (add `ToolProvenance::Plugin`) marked complete; next slice (local WASM plugin MVP) recorded under T46 |

### 2.2 Facts corrected from plan or ADRs

| Fact | Correction |
|---|---|
| `ToolProvenance` claimed `#[non_exhaustive]` in ADR-028 | It is not. Three exhaustive match arms were compiler-enforced, which is functionally equivalent for this addition. Documented in T40 checkpoint. |
| `wasmtime` ADR-027 | Named wasmtime without comparison; comparative analysis backfilled in ADR-032 |
| T52 rehearsal | Original T38 record was retrospective (external agent); T52 is the first real Talos-driven rehearsal (~45% coverage) |
| Issue #7 / #8 | Created on GitHub this session; tracked as TUI-016 / TODO-001 in backlog |

## 3. Milestone assessments

| Milestone | Target Week | Status | Evidence |
|---|---|---|---|
| M1 Starting gate complete | 1 | ✅ (Month 1) | Plan checkpoint |
| M2 User-visible tooling hardening | 4 | ✅ (Month 1) | Plan checkpoint |
| M3 Web/governance/memory prototypes | 8 | ✅ (Month 2) | T22, T23, T29, T30, T31 delivered |
| **M4 Extensibility unblocked or deferred** | **12** | **✅ PASSED** | ADR-027/028/029/030/032 accepted; MVP slice started (T40, T45, T46): provenance data model, manifest parser, WASM runtime adapter. No runtime dependency shipped by default (`wasmtime` behind feature flag). |
| M5 Release posture known | 16 | Pending | T55/T56 publish (maintainer-gated), T62–T65 docs/closeout/handoff |

## 4. REL-002 Self-Bootstrap Gap Report

REL-002: "Talos must be able to perform 100% self-bootstrap development as the primary runtime before
v1.0.0 is claimed."

| Criterion | Status | Evidence |
|---|---|---|
| Talos as primary development runtime | ❌ **blocked** | ~45% coverage per T52 rehearsal; agent loop cannot autonomously validate, generate tests, or commit |
| Governance-driven development cycle | ⚠️ **partial** | Owner docs + BOARD + governance validation work; agent cannot yet follow the cycle autonomously |
| Tool reliability for self-development | ⚠️ **partial** | Core tools (read/write/edit/bash/grep) functional; missing multi-file refactor + compiler-error-driven fix loop as agent capability |
| Memory/context for long sessions | ✅ **ready** | Memory graph (T43), compression (T26/T51), associative recall API (T43/T50), metrics (T51) |
| Extensibility for plugin/tooling | ✅ **MVP ready** | Provenance (T40), manifest parser (T45), WASM runtime adapter (T46), dashboard (T42), browser-page mock (T47) |

### 4.1 Self-bootstrap rehearsal progress

| Rehearsal | Coverage | Primary runtime | Key finding |
|---|---|---|---|
| T38 (1st) | ~10% | External (Codex) | Docs-only; Talos provided CLI smoke only |
| **T52 (2nd)** | **~45%** | **Talos** (code edits) | Multi-crate change correct; **compiler-guided fix loop** discovered — Talos followed `cargo check` errors to all exhaustive match sites. Validation/tests/commit still external. |

### 4.2 Critical gap: autonomous validation loop

The biggest blocker to higher self-bootstrap coverage is the missing **autonomous validation
loop**: after editing code, Talos does not run `cargo check` / `cargo test` / `cargo fmt` / `cargo
clippy` and iterate on errors. Without this loop, Talos can produce code edits but cannot confirm
they compile. T62 (third rehearsal) should target this gap.

### 4.3 Stuck items

- T55/T56 real `cargo publish` requires explicit maintainer approval — out of scope.
- T60 automatic associative memory injection — T31 recommends default-off; needs metrics evidence.

## 5. Bug fixes discovered during Month 3

### 5.1 UTF-8 panic in approval panel

**Severity**: high (TUI crash, silent exit, no panic message visible)
**Reproducer**: Run `gh issue create --title "feat: write 和 edit 工具应显示内容输出"` in the
TUI bash tool approval flow. The approval panel's string truncation (`&arguments[..72]`) panics
when byte index 72 lands inside a multibyte CJK character.

**Fix** (`crates/talos-tui/src/state.rs:226`): walk back to nearest `is_char_boundary` before
slicing. Two regression tests added.

**Discovery**: Found via T52 real Talos-driven rehearsal. The panic hook added in the same
commit now makes future crashes visible.

## 6. Items escalated for architecture judgment

### 6.1 Dashboard HTML frontend scope

T42 implemented the dashboard as **API-only** (JSON / masked plain text). The proposal
(`docs/proposals/web-001-loopback-dashboard-design.md`) describes a static-HTML frontend as the
intended MVP. Currently ADR-031 covers the read-only backend boundary but not the HTML rendering
path. Question: is the API-only MVP sufficient as a closed slice, or should HTML rendering ship
in the same iteration? Cost: minimal HTML + CSS via `include_dir`; already in ADR-031 boundary.

### 6.2 plugin lifecycle and runtime sharing

T46 isolated `wasmtime` behind a `wasm` feature in `talos-plugin`. The `tal_memory` graph
(T43) is also a runtime dependency. With TOOL-008 Phase 3 potentially needing wasmtime as a
**permanent** dependency for tree-sitter grammar loading, wasmtime will need to be promoted from
feature-gated to default-on for that crate, with separate compile-time evidence recorded. ADR-032
anticipated this but did not authorize it. Question: should we record the promotion as a separate
ADR when TOOL-008 Phase 3 lands, or treat it as already-implicit in ADR-032?

### 6.3 Self-bootstrap validation loop tooling

T52 showed ~45% coverage. The gap is the missing autonomous validation loop (§4.2). There is no
backlog story or ADR for this. Options: add an explicit "validate" tool that runs cargo check
+ test + fmt + clippy and feeds errors back to the agent loop, OR rely on the bash tool + agent
prompting alone. T62 rehearsal will reveal which path works.

## 7. Files changed this session (commits `f4423b9` → `8a4c3b5`)

16 commits on `origin/main`. Highlights:
- New crate: `crates/talos-dashboard/` (axum server with 4 routes)
- New modules: `talos-agent::wasm.rs`, `talos-memory::graph.rs`, `talos-tools::browser_page.rs`
- Bug fix: `crates/talos-tui/src/state.rs:226` (UTF-8 truncation)
- New panic hook: `crates/talos-cli/src/mode_runners.rs:415`
- Site: 7 Chinese pages under `site/zh/`; Nord theme in CSS and SVG assets
- Docs: 3 stories (TUI-016, TODO-001, T52 rehearsal evidence); ADR-032 updated; Issue Sync Rule

Validation gate at session close:
- `cargo fmt --all -- --check` → clean
- `cargo test --workspace` → **1347 passed**, 0 failed
- `scripts/validate_project_governance.sh .` → 0 warnings
- `scripts/check_publish_guard.sh .` → PASSED
- `talos --version` → `talos 0.2.0`

## 8. Recovery / resume for the architect

- All Month-3 commits are on `origin/main` between `acde17a` (Month-2 closeout) and `8a4c3b5`
  (T54 closeout).
- To start Month-4 work: pick T57 (tool reliability sweep) or T58 (WEB-001/WEB-005 security
  review); both are independent and safe to proceed without maintainer approval.
- To resume T40/T45/T46 plugin work: the manifest parser and WASM runtime adapter are in place;
  integration with `AgentTool` registration (next slice) is the next step, gated on T59
  security review.
- To resume T43 graph work: the storage layer and recall API are complete; further slices
  (T62 ingest, T63 defaults) can proceed without architectural changes.
- To debug any future TUI crashes: re-run with `RUST_BACKTRACE=1 cargo run -p talos-cli`;
  panic now prints location + message to stderr.

## 9. Handoff next-step recommendations

For architecture decision:

1. **Approve T55/T56 publication** (maintainer-only). The 4 dep blockers are documented in
   CRATE-PUBLICATION-MATRIX §A5; pre-publication requires talos-core → talos-permission →
   talos-sandbox → talos-agent gates. No architectural blocker; only a sequencing decision.

2. **Decide on T47 mock backend promotion** (question §6.1 above). If HTML frontend is desired,
   add to Month-4 plan as a new T47.5; otherwise close T47 as-is (API-only, sufficient for
   dashboard MVP boundary).

3. **Record the wasmtime default-on decision path** (question §6.2 above) before TOOL-008
   Phase 3 starts work on tree-sitter WASM grammar loading.

4. **Approve a self-bootstrap validation loop** (question §6.3). May be the single highest-ROI
   improvement for reaching REL-002 compliance, since T52 showed ~45% coverage with code-edit-only
   loops and validation was the dominant external work.

5. **Review T61 (third rehearsal) design** before it starts. T61 should target an autonomous
   code-or-doc slice where Talos validates its own work, not just edits. The T62/T63 gap report
   and T64 readiness report both depend on T61 evidence being representative.

## 10. Open questions for the architect (non-blocking)

1. Should the dashboard mock backend (T47) be promoted to include HTML frontend, or stay API-only?
2. Does TOOL-008 Phase 3 require a new ADR for wasmtime default-on promotion, or does ADR-032 cover it?
3. Is there a preferred pattern for the autonomous validation loop tool, or should T62 explore
   multiple options?
4. Should TODO-001 (session todo list) be scheduled into Month-4 or kept as a backlog-only item?
5. Should the TUI panic hook be moved from `mode_runners.rs` to `talos-tui/src/app.rs` for broader
   applicability (e.g., print mode)?

## Reviewer sign-off

- [ ] Reviewed and acknowledged Month-3 deliverables
- [ ] Decisions captured for §6 escalations
- [ ] Recommendations for §9 approved or deferred
- [ ] Month-4 sequencing approved
