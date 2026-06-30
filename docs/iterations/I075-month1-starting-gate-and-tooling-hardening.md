# Iteration I075: Month 1 — Starting Gate and Tooling Hardening

> Document status: Planned
> Published plan date: 2026-06-30
> Planned objective: Execute the first four weeks (T00–T21) of the four-month self-bootstrap
> product hardening plan: starting-gate inventory, cargo-install gate design, tool-surface audit,
> TUI output summarization, ripgrep grep plan, WEB-005 permission model, and Month-1 closeout.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a validated starting-gate checkpoint, a cargo-install gate checklist, TUI-014
> grep-result summary + TUI-015 head+tail truncation rendering with tests, and a Month-1 closeout
> record with `cargo test --workspace` evidence.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `T00` | Four-month plan | Done | Board/backlog reads | Starting disposition checkpoint published |
| `T01` | Four-month plan | Ready | T00 | This iteration doc created |
| `T02` | ARCH-031 | Done | T00 | talos-cli metadata audit complete |
| `T03` | ARCH-031 | Ready | T02 | Cargo install gate checklist in publication matrix |
| `T04` | TOOL-014 | Done | T00 | Native tool-surface snapshot (30 tools, 1 hidden) |
| `T05` | Four-month plan | Ready | T00 | Docs sync checklist committed |
| `T06` | ARCH-031 | Planned | T03 | talos-cli README content for binary install |
| `T07` | ARCH-031 | Planned | T03 | `cargo install --path` smoke verified |
| `T08` | ARCH-031 | Planned | T03 | `cargo publish --dry-run -p talos-cli` evidence or blocker |
| `T09` | TUI-014 | Planned | TOOL-011/TUI docs | Grep result summary rendering + tests |
| `T10` | TUI-015 | Planned | TUI-014 | Head+tail truncation for long unsuppressed outputs |
| `T11` | REL-002 | Planned | T00 | Self-bootstrap session evidence template |
| `T14` | TOOL-011/ADR-025 | Planned | T04 | Ripgrep-backed grep engine first slice behind engine boundary |
| `T15` | TOOL-011 | Planned | T14 | Search regression tests (hidden-dir, filters, UTF-8, large output) |
| `T17` | TOOL-011 | Planned | T14 | First ripgrep slice complete or precise blocker recorded |
| `T18` | WEB-005 | Planned | BrowserSkill research | Browser-session continuity permission model + page record schema |
| `T19` | WEB-005/TOOL-013 | Planned | T18 | `browser_page_read` permission facet design |
| `T20` | MEM-007 | Planned | T00 | Deterministic pre-entry compression spike for read/grep/git_diff/bash |
| `T21` | Four-month plan | Planned | T00–T20 | Month-1 closeout: validation summary, delivered items, blockers, Month-2 replan |

### Scope

- Governance starting gate: inventory, disposition checkpoint, iteration slices, evidence template.
- Distribution readiness: cargo-install gate design, metadata audit, local install verification,
  dry-run evidence (no real publish), README content for the binary install path.
- Tooling reliability: TUI-014 grep summary, TUI-015 truncation, ripgrep engine first slice,
  search regression coverage.
- WEB-005 permission model design (no browser automation implementation).
- MEM-007 compression spike (default off, no automatic injection).

### Non-Goals

- Real `cargo publish`, tag, release, or `publish = false` removal (T55/T56 — maintainer-only).
- Plugin runtime implementation requiring wasmtime dependency (T46 — ADR-027 review gate).
- Browser automation, vector stores, local-model dependencies.
- Changing permission defaults or enabling automatic memory injection by default.
- Rewriting published iteration baselines (I001–I074).

### Acceptance

- Given the four-month plan is accepted, when a maintainer reads the starting disposition
  checkpoint, then every Active/Review/Planned/Blocked item has an explicit owner doc and gate.
- Given the cargo-install gate checklist exists, when `cargo install --path crates/talos-cli --bin
  talos` is run into a temp `CARGO_HOME`, then the `talos` binary installs and `talos --version`
  succeeds.
- Given TUI-014 is implemented, when a grep tool result with many matches is rendered, then the
  scrollback shows a compact summary (file: count) instead of every raw line, and `/export`
  preserves the full raw output.
- Given TUI-015 is implemented, when an unsuppressed tool output exceeds the line budget, then the
  rendering shows head + tail with an elision marker, and `/export` preserves the full output.
- Given the Month-1 closeout is reached, then `cargo test --workspace` passes and
  `scripts/validate_project_governance.sh .` reports zero warnings.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-tui -p talos-tools` for T09/T10/T14/T15
- `cargo test --workspace` at Month-1 closeout (T21)
- `scripts/validate_project_governance.sh .`
- `scripts/check_publish_guard.sh .` when manifests change
- `cargo install --path crates/talos-cli --bin talos` smoke (T07)

### Documentation To Update

- `docs/reference/CRATE-PUBLICATION-MATRIX.md` (T03 gate checklist)
- `docs/BOARD.md` after owner docs reflect Month-1 status changes
- `docs/backlog/active/TUI-014-grep-result-summary.md` and `TUI-015-head-tail-truncation.md`
- `docs/backlog/active/TOOL-011-ripgrep-backed-grep-engine.md`
- `docs/backlog/active/WEB-005-browser-session-continuity-research.md`
- Root `README.md` / `README.zh-CN.md` if install instructions change

### Risks And Rollback

- Risk: ripgrep library crates add binary size or compile-time regression.
  Rollback: keep the engine boundary internal so the current search implementation remains the
  fallback; feature-gate the ripgrep path if needed.
- Risk: TUI rendering changes break existing scrollback tests.
  Rollback: snapshot-free rendering tests are additive; revert the specific rendering module if
  regressions appear.
- Risk: `cargo install --path` fails due to workspace path dependencies.
  Rollback: record the exact blocker; `--path` install is expected to work today per the T02 audit,
  so failure indicates a manifest regression to fix, not a design gap.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-30 | Activation | Baseline committed (`13e93b9`). T00/T02/T04 completed in initial pass. Governance validation: 0 warnings. Starting this iteration with T01, T03, T05. |

## Verification Evidence

- (to be appended as slices complete)

## Variance And Residuals

- (to be appended)

## Retrospective

- (to be appended at Month-1 closeout)
