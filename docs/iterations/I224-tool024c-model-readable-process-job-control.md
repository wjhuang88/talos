# Iteration I224: TOOL-024-C Model-Readable Process Job Control

> Document status: Complete / Closed — implementation merged via PR #386; owner-first closeout pending
> Published plan date: 2026-08-24
> Planned objective: expose bounded session-owned `process` controls over the completed I222
> supervisor without adding Windows, UI, persistence or autonomous-turn behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-24 |
| Work Slice | Model-visible session-scoped `process` read/status/list/cancel over I222's manager, with bounded cursors/wait/output, ownership/redaction, idempotent cancel and runtime fixture/docs; no supervisor redesign, Windows/D1, TUI/D2, Dashboard/I213, I223, persistence, `/auto`, release or Desktop. |
| Claimed At | 2026-08-24 |
| Source Issue | #59 |
| Governance Claim PR | #385 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #385 exact head `12931fef1400f7ce53fe82f3d3453036d2227c56` passed CI `32699927266`, independent review `5391959581`, merge-time CAS, and merged as `ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`. Protected permission/security/API review and exact-head evidence are recorded below. |
| Implementation PR | #386 — merged as `60b0367cf749397bf1167e189e820e82e32baf03` |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | I224 is closed with pre-existing implementation evidence. TOOL-024-D1/D2 and I223 remain separate; Windows remains fail-closed. |

## Published Baseline

Planning target: `main@faf7c0e8719eaaafd4bde0b2820cead8fe2a0e8a` after I222 closeout PR #384.

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-024-C | TOOL-024 / Issue #59 | Planned / Unclaimed | TOOL-024-B/I222 Complete; RUNTIME-005 and PERM-006-C Complete | One model-visible bounded `process` tool over the I222 manager |

## Scope And Execution Order

1. Establish typed process action/result contract and ownership/redaction rules.
2. Register the tool through the existing agent/session composition root.
3. Implement bounded cursor reads, status/list views, cancel and bounded wait.
4. Add focused and real-runtime fixture coverage, then update tool guidance/docs.

Windows Job Object/D1, TUI/D2, I223 and all release/publication work remain separate gates.

## Acceptance And Documentation Targets

- Four actions are reachable through the actual runtime tool registry and preserve foreground behavior.
- Cursor ordering, truncation/eviction, limits, timeout/cancel races and ownership fail closed.
- Permission denial cannot create or control a job; output and summaries are display-safe.
- Update the TOOL-024 owner, model/tool reference and relevant runtime API docs.

## Planned Validation

- Focused `talos-core`/`talos-agent`/`talos-tools` locked tests and an end-to-end registry fixture.
- Full locked workspace check, Clippy, tests and `./scripts/release_preflight.sh`.
- Both governance validators, YAML parse, EOF/whitespace/secret/generated-residual audit.
- Exact-head CI plus independent permission/security/API review and merge-time CAS.

## Risks And Rollback

The principal risks are cross-session job disclosure, unbounded output/wait, duplicate terminal
results and accidental supervisor changes. If any is unresolved, keep C unmerged and leave B intact;
disable only the new process-tool registration.

## Verification Evidence

- Implementation PR #386 exact head `d42c060d618e61218c4c1efe0651e74830807256` was based on
  `ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`, passed exact-head CI `32719779528` (5/5), and
  received independent permission/security/API/process approval `5394777902`.
- It merged into `main` as `60b0367cf749397bf1167e189e820e82e32baf03` after merge-time CAS.
  Local locked Agent/tools/runtime tests, focused check/clippy, full release preflight, both
  governance validators and `git diff --check` passed before push.
- The changed-file inventory remained within the agent/core/tools process-control slice; no
  Dashboard/I213, Windows Job Object, release/publication, persistence or I223 authority was used.

## Completion Evidence

Completion Commit: `60b0367cf749397bf1167e189e820e82e32baf03`. This pre-existing implementation
merge is the evidence; the closeout status commit cannot self-certify the behavior implementation.

## Execution Checkpoint (2026-08-24)

Implementation locally converged from claim merge `ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`.
Focused `cargo check -p talos-agent --locked`, all `talos-agent` tests, the real session registration
fixture, `talos-tools` tests, `talos-runtime` tests, Clippy, governance validators and release
preflight pass. Workspace library validation is blocked only by host Seatbelt permission failures;
the candidate contains no Windows Job Object, Dashboard/I213, TUI, persistence, release or I223
authority. Full exact-head CI and independent review remain pending.
Stable implementation commit: `dcdffc56` plus launch-cancel reap fix `eabb7d3a`; implementation PR #386 is open. Its exact head is resolved by PR API and bound by CI/review evidence rather than self-referential owner text.
Completion remains pending; this status record is not implementation evidence.

The preceding execution checkpoint is a preserved pre-merge record. It is superseded by the
Completion Checkpoint below; current status and evidence are authoritative there.

## Completion Checkpoint (2026-08-24)

PR #386 implementation head `d42c060d618e61218c4c1efe0651e74830807256` merged into `main` as
`60b0367cf749397bf1167e189e820e82e32baf03` after exact-head CI `32719779528`, independent
permission/security/API/process approval `5394777902`, and merge-time CAS. The final corrective
head isolated cancel resources by opaque job ID and preserved `BackgroundJobRequest` compatibility.
I224 is Complete / Closed. TOOL-024-D1/D2 and I223 remain separately governed, and Issue #59
remains open until those children and deferred validation reach their own evidence-bearing terminal
states.

## I223 Deferred Validation Result (2026-08-26)

V59-C1 passed on rebuilt integrated `main@ab750888`. A real CLI session exercised process
list/status/read/cancel; read returned `v59-ready\n` once with `next_cursor=10`. Eleven locked
supervisor/process tests also passed cursor advancement, partial-chunk resume, 32-KiB response
clamping/eviction, cancellation, timeout, shutdown and unknown-job fail-closed behavior.
