# SESSION-010: Runtime Resume And Empty Artifact Closure

| Field | Value |
|---|---|
| Story ID | SESSION-010 |
| Type | Session Runtime Reliability Correction |
| Priority | P0 Emergency |
| Status | Complete |
| Source Issue | Maintainer incident report, 2026-08-12 |
| Responsible Actor | @wjhuang88 |
| Selected Iteration | I192 |

## Problem

Resuming a Session whose committed activation target already matches the requested model can be
rejected because startup constructs a different generation-1 activation digest. Startup runtime
initialization also materializes an empty transcript and pending-submission SQLite sidecar before
the first message, so abandoned no-chat launches remain visible in `/resume`.

## Required Outcome

- Resume reuses a committed runtime activation when its target identity exactly matches the
  requested identity and still rejects a genuine identity mismatch.
- `/resume` excludes the active Session and zero-message Sessions. Inside the TUI, both ordinal and
  UUID selection are restricted to that filtered current-workspace list; cross-workspace UUID
  recovery remains available through the explicit CLI `--session <UUID>` path.
- A normal no-chat shutdown removes only the artifacts owned by that empty Session after the
  runtime has stopped; non-empty, pending, forked, live or ambiguous data is preserved.
- Forced-exit residuals remain visible to explicit maintenance rather than being guessed at or
  deleted during startup.

## Exclusions

- No SESSION-008-B activation, durable schema change, broad shutdown protocol or RUNTIME-005 work.
- No automatic deletion of historical empty JSONL files or existing user storage.
- No claim that earlier transient test failures prove concurrency or ENOSPC causes.

## Validation

- Regression tests for matching-target resume and true mismatch rejection.
- Picker tests for active/zero-message filtering.
- Normal no-chat shutdown cleanup plus preservation tests for non-empty, unparseable and pending
  Sessions. Only a transcript whose on-disk length is exactly zero authorizes cleanup.
- Locked format, check, Clippy and workspace tests; independent exact-head review before merge.

## Implementation Evidence

- Implementation commits: `ecd615a0`, dependency-free test correction `c597c0bb`, and review
  data-preservation correction `d98f6d0a`.
- `./scripts/release_preflight.sh` passed on implementation tree `c597c0bb`.
- Read-only storage inventory found 25 sidecars, all paired with transcripts; nine pairs had empty
  TLOG transcripts. This proves premature no-chat materialization, not orphan-sidecar leakage.
- Existing user files were not modified. Forced-kill and historical empty-file cleanup remain
  explicit maintenance concerns outside this completed Story.
- Independent review `5265172266` rejected head `b2376847` because parsed-entry emptiness could
  delete non-empty truncated or future-schema transcripts. Commit `d98f6d0a` changes deletion
  authority to strict zero-byte metadata and proves both ambiguous forms remain byte-identical.
- The independent A2 finding was pre-existing main drift, repaired separately by bounded-maintenance
  PR #207 at main merge `ac8971a3`; the remote Issue/owner validator now passes for 34 open Issues.
- Final exact head `6b2dbdb557f480b623901441025582727e6ba4ff` passed CI run
  `31587076213`, independent natural-person approval `5274917099` and merge-time CAS. The independent
  re-review also preserved five additional non-empty ambiguous byte forms while deleting a true
  zero-byte Session.
- Completion Commit: `512ff32f389167364c02e7058151879b9ce6859a`.
