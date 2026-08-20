# TEST-001: Process HOME Isolation In Parallel Tests

| Field | Value |
|---|---|
| Story ID | TEST-001 |
| Type | Test Infrastructure Story |
| Priority | P1 |
| Status | Ready / Unclaimed |
| Source | [GitHub Issue #316](https://github.com/wjhuang88/talos/issues/316) |
| Selected Iteration | None |
| Depends On | Existing config-path injection and test temporary-directory helpers |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #316 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Select a runnable iteration and establish an effective claim before changing test environment ownership or helpers. |

## Identity / Goal / Value

Keep parallel configuration and CLI tests deterministic by preventing one test's process-wide home
directory mutation from leaking into another test.

## Scope

- Inventory tests that mutate `HOME`, `USERPROFILE`, `HOMEDRIVE` or `HOMEPATH`.
- Prefer explicit per-test config and credential roots; serialize unavoidable process-environment
  mutation behind one shared test-only guard.
- Give spawned integration binaries an isolated writable home without changing the parent test
  process for unrelated cases.
- Add deterministic parallel regression coverage for the observed configuration and hook paths.

## Exclusions

- No product config-path migration or user-visible credential behavior change.
- No weakening, skipping or reclassification of macOS sandbox/seatbelt tests.
- No ad hoc release-preflight environment requirement.

## Acceptance

- Repeated parallel `talos-config` and relevant CLI integration tests cannot observe another test's
  temporary or removed home directory.
- Standard release preflight passes without a caller-supplied HOME workaround.
- Credential redaction, atomic storage and sandbox tests retain their current coverage.

## State / Status Owners

- Story scope and status: this file.
- Remote report and reproduction evidence: GitHub Issue #316.
- Derived views: `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md`.

## User-Facing Documentation

None. This is test-infrastructure-only and cannot claim a user behavior change.

## Required Reads

- `docs/sop/TESTING.md`
- `crates/talos-config/src/tests.rs`
- `crates/talos-cli/tests/`
- `scripts/release_preflight.sh`

## Residual Destination

Any product config-path or credential-storage defect discovered while isolating tests requires a
separate product owner and iteration.
