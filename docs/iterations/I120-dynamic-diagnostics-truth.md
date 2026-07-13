# Iteration I120: Dynamic Diagnostics Truth

> Document status: Planned — ready for assignment
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

Not started. Activate only after Gate 0 in the execution package passes.
