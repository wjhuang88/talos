# Iteration I120: Dynamic Diagnostics Truth

> Document status: Complete (2026-07-13) — all stories verified, real binary smoke passed
> Published plan date: 2026-07-13
> Planned objective: Replace hardcoded/stale diagnostics with valid, bounded, dynamically derived
> operator truth.
> Baseline rule: preserve this target after publication; changed targets use a new iteration ID.
> MVP deliverable: a real `talos diagnostics status --json` result parses as JSON and reflects the
> current iteration/gate sources without exposing secrets.

## Published Baseline

### Selected Stories

| Story | Parent | Depends On | Outcome |
|---|---|---|---|
| F100 | I120 | Current governance parsers | Fixture-backed diagnostics contract |
| F101 | I120 | F100 | Serde-generated valid JSON |
| F102 | I120 | F101 | Dynamic iteration/residual state with safe fallback |
| F103 | I120 | F100-F102 | Real-binary closeout and user docs |

### Scope

- Reuse or extract crate-private parsing from `governance.rs`; do not create duplicate mutable state.
- Derive active/open iteration information from the iteration index and current residual gates from
  explicit owner data or a small typed registry with tests.
- Serialize JSON with `serde`/`serde_json`; test quotes, backslashes, control characters, missing
  files, malformed files, non-UTF8 paths where supported, and secret redaction.

### Non-Goals

- No governance writes, dashboard redesign, public API, permission, release, or network behavior.

### Acceptance

- JSON output parses and round-trips through `serde_json::Value`.
- No stale I085 Paused claim or manually escaped user/path data remains.
- Missing/malformed governance sources yield bounded `unavailable` diagnostics, not panic or false
  completion.
- Text and JSON views represent the same typed summary and contain no credential values.

### Validation

- Targeted `talos-cli` unit/integration tests and real binary commands in a clean and fixture workspace.
- Standard validation ladder in the execution package.

### Required Documentation

- README diagnostics section, this iteration, index, Board, and owner docs whose state is corrected.

### Risks And Fallback

- Parser sharing creates broad refactor pressure: keep helpers crate-private and surgical.
- Owner schema is ambiguous: report `unavailable` and record the owner drift; never guess state.

## Execution Record

### Gate 0 — 2026-07-13

- Branch: `feature/i120-dynamic-diagnostics` (from updated `main` at `6a7a0f6`).
- `rustc 1.97.0` (pinned by `rust-toolchain.toml`).
- `cargo metadata --locked --no-deps --format-version 1` — exit 0.
- `scripts/validate_project_governance.sh .` — 0 warnings.
- `./scripts/release_preflight.sh` — passed (fmt, check, clippy `-D warnings`, workspace tests).
- Non-terminal inventory disposition confirmed: no other iteration is Active. I018/I019/I020/I028
  deferred; I081/I082/I083 reconciled to Superseded; I121-I123 blocked on I120; Board-level Review
  items are not iterations.

### F100 — Complete (2026-07-13)

- `DiagnosticsSummary` now derives `serde::Serialize, serde::Deserialize`.
- `active_iterations` dynamically derived from `docs/iterations/README.md` via reused
  `governance::parse_open_iterations()` (no duplicate mutable state).
- Stale `I085 MC107 Paused` claim removed from residual gates (I085 is Complete).
- Fixture tests: clean source, missing index, malformed index, empty table, serde round-trip,
  JSON string escaping, no-secrets invariant, no-stale-I085 assertion (12 tests, all pass).
- `governance::IterationItem` and `parse_open_iterations()` promoted to `pub(crate)`.
- `serde = { version = "1", features = ["derive"] }` added to `talos-cli/Cargo.toml`.
- Validation: `cargo fmt --check`, `cargo check --workspace --locked`, `release_preflight.sh`,
  governance 0 warnings, `git diff --check` — all pass.
- Pre-existing note: `cargo clippy --workspace --all-targets` has pre-existing `unwrap()` violations
  in test code across multiple crates unrelated to this change; `release_preflight.sh` (the
  authoritative workspace gate) does not use `--all-targets` and passes.

### F101 — Complete (2026-07-13)

- `print_json()` replaced with `serde_json::to_string_pretty(&summary)` — all JSON escaping now
  handled by serde, not hand-rolled string formatting.
- 47 lines of manual JSON construction code removed.
- CLI integration test `tests/diagnostics_e2e.rs` created with 7 tests:
  - JSON parses as `serde_json::Value` with all expected fields
  - No secrets in JSON output
  - No stale I085 Paused claim in JSON output
  - Clean iteration source populates `active_iterations`
  - Missing iteration index produces `unavailable` diagnostic
  - Text mode works alongside JSON mode
  - `workspace_root` is a valid JSON string
- Validation: fmt, check, release_preflight, governance 0 warnings, `git diff --check` — all pass.

### F102 — Complete (2026-07-13)

- `collect_diagnostics_summary_at(workspace, root)` added for workspace-aware fixture testing.
- `collect_residual_gates_at(workspace)` wraps the typed registry with workspace awareness.
- 6 new fixture tests:
  - Full summary from clean workspace (iteration README present → I120 found)
  - Full summary from empty workspace (no docs → bounded `unavailable` + typed registry fallback)
  - Full summary from malformed workspace (garbage README → no panic, bounded output)
  - Text and JSON views share the same typed summary (consistent field counts)
  - Residual gates always bounded (non-empty, non-empty strings)
  - Unicode workspace path properly serialized through serde
- Total: 18 unit tests + 7 CLI integration tests, all pass.
- Validation: fmt, check, release_preflight, governance 0 warnings, `git diff --check` — all pass.

### F103 — Complete (2026-07-13)

- Real binary smoke: `talos diagnostics status --json` and `talos diagnostics status` verified in the
  Talos workspace. JSON parses via `serde_json`, shows I120 as active, lists I121-I123 as blocked,
  and contains no stale I085 claim or credential values.
- README diagnostics section updated with `talos diagnostics status --json` command and description.
- Full validation ladder: fmt, `cargo check --workspace --locked`, release_preflight, governance
  0 warnings, `git diff --check` — all pass.

## Retrospective

### Acceptance Verification

| Acceptance | Status | Evidence |
|---|---|---|
| JSON output parses and round-trips through `serde_json::Value` | Pass | `serde_json::to_string_pretty` replaces hand-rolled JSON; 18 unit + 7 CLI integration tests |
| No stale I085 Paused claim or manually escaped user/path data | Pass | Stale claim removed; serde handles all escaping; dedicated test `test_no_stale_i085_paused_claim` |
| Missing/malformed governance sources yield bounded `unavailable` diagnostics | Pass | `collect_active_iterations_at` returns bounded fallback for missing/malformed; 4 fixture tests |
| Text and JSON views represent the same typed summary, no credential values | Pass | Both use `DiagnosticsSummary`; `test_text_and_json_views_share_same_summary`; no-secrets tests |

### What Went Well

- Reusing `governance::parse_open_iterations()` avoided duplicate mutable state.
- serde derive on `DiagnosticsSummary` eliminated the entire class of JSON escaping bugs.
- Workspace-aware `collect_diagnostics_summary_at()` enabled comprehensive fixture tests without
  touching the real workspace.

### Residuals

- Pre-existing `cargo clippy --workspace --all-targets` violations exist in test code across
  multiple crates (unrelated to I120). `release_preflight.sh` does not use `--all-targets`.
- The `collect_residual_gates_at` function currently delegates to the typed registry. Future
  iterations could derive gates from Board/ADR sources if needed.
