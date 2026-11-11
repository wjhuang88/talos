# Session recovery storage candidate

Status: Partial — isolated storage prerequisite, not full session recovery delivery.
Authorization Mode: Emergency override.
Authorizing maintainer: current user, explicit session recovery incident request.
Incident: ordinary append reused stale parent IDs; legacy automatic archival retained
only a recent head, so restart/history preparation lost older context or refused it.
Branch: fix/session-recovery-storage-20261111.
Base: 55617ad75c84db75718f287d6c05b84cfe55085e.

## Scope and preservation

This candidate extracts only talos-session changes from the preserved dirty worktree.
The original worktree, retained stash and session evidence are untouched. It fixes
parent selection/publication under the shared write lock, cold tip decoding, durable
commit cache updates, fork cache isolation and compaction invalidation. It adds
host-selected history binding without changing identity or transcript bytes, and
redacts legacy text projections without inventing old tool receipts.

Archive inspection reconstructs original record evidence from exact ID/raw-byte
overlap and unique adjacency. It rejects conflicting IDs, missing/forward parents,
cycles, other-live-head overlap and changed source inventories. Input, decoded output,
record count and zstd window limits apply. Original IDs/parents are retained. This
API supplies evidence; it does not itself install a checkpoint or rewrite a transcript.
SQLite binding opens and bounded reopen paths use SQLITE_OPEN_NOFOLLOW. Native archive
and binding operations retain error/unwind boundaries. This is not proof against every
hostile directory replacement or native Windows runtime behavior.

Hard constraints: preserve source bytes and parent validation; grant no model read/write
permission; do not enable background continuation. Host execution leases are still required.
The API additions are additive. This is a storage-only prerequisite: CLI restore/query
wiring, compact admission and removal of automatic archival are separate pending integration
changes. User-facing README recovery instructions must accompany that integration; no CLI
recovery delivery is claimed by this candidate.

## Validation and review

The isolated candidate passes `cargo test --locked -p talos-session --lib`: 331 tests,
zero failures/ignored, log `/tmp/talos-storage-isolated-tests.log`. Regressions exercise
real cloned concurrent writers, multiline cold tails, malformed tails, repeated archive
cuts/tool pairs, unrelated sessions, source mutation, a 128 MiB-window tiny zstd frame,
and SQLite path replacement after metadata validation. Full candidate preflight and
independent safety/API review remain required before merge.

Scale assessment still recommends high-risk/release-managed/required worktrees; log
`/tmp/talos-storage-candidate-scale.log`. No governance policy or iteration baseline is
changed. Rollback is a targeted code revert, never deletion of session/backup data;
reverting parent repair restores the known writer defect.

## Residual ownership

The original full handoff remains active in the preserved primary worktree under
`docs/tasks/2026-11-11-session-recovery-handoff.md` and
`docs/tasks/session-archive-recovery-emergency.md` (not published by this storage slice).
This owner tracks the isolated candidate until integration. Remaining full-goal work:
archive-aware CLI/Actor candidate integration, archived-fork provenance, independent
review, exact-head CI, running-host acceptance including live-provider summary quality,
native Windows acceptance, selective commits and delivery, and the background/UI/approval
handoff tasks. No completion commit or deployment is claimed here.

## Candidate validation follow-up

The first isolated full preflight failed in the Actor checkpoint process-restart
acceptance child at its 20-second turn deadline. Cause remains unestablished; focused
retry log `/tmp/talos-storage-isolated-checkpoint-retry.log`. Do not infer passing
whole-workspace evidence from the 331 Session tests.

Added a failing-then-passing parent-directory-alias regression for SQLite NOFOLLOW:
canonicalize only the parent, never the database filename, preserving directory aliases
while refusing database symlinks. Updated Session suite: 332 passed in
`/tmp/talos-bindings-parent-alias-after.log` (primary worktree; source copied here).
Full preflight must be rerun for this adjustment. No merge or deployment authorization
is inferred from local self-review.
Focused checkpoint retry passed in 0.32 seconds; timeout cause remains unknown.
Final-source full preflight is running in `/tmp/talos-storage-isolated-final-preflight.log`.

MSVC all-feature test cross-check passed using pinned cargo xwin in
`/tmp/talos-storage-final-xwin.log`; native Windows execution remains required.
Second full preflight reached config tests and failed the unchanged
`store_unset_builtin_provider_clears_credentials` assertion. It calls environment-aware
`api_key()` without the ENV_MUTEX used by credential-mutating tests. The primary tree
already contains a separate test correction, intentionally not bundled here.
With ANTHROPIC_API_KEY, OPENAI_API_KEY and OPENAI_COMPAT_API_KEY removed only from the
test process environment and tests serialized, all 273 config tests passed in
`/tmp/talos-storage-config-serial.log`. This does not prove the failed run's precise race.
Standard full preflight is now running with those same environment constraints in
`/tmp/talos-storage-isolated-serial-preflight.log`. This is a local review candidate;
full verification and independent review remain pending, and status stays Partial.
