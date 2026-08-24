# Iteration I224: TOOL-024-C Model-Readable Process Job Control

> Document status: Active / Claimed — proposed through governance PR #385; ineffective until merge
> Published plan date: 2026-08-24
> Planned objective: expose bounded session-owned `process` controls over the completed I222
> supervisor without adding Windows, UI, persistence or autonomous-turn behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-24 |
| Work Slice | Model-visible session-scoped `process` read/status/list/cancel over I222's manager, with bounded cursors/wait/output, ownership/redaction, idempotent cancel and runtime fixture/docs; no supervisor redesign, Windows/D1, TUI/D2, Dashboard/I213, I223, persistence, `/auto`, release or Desktop. |
| Claimed At | 2026-08-24 |
| Source Issue | #59 |
| Governance Claim PR | #385 |
| Authorization Mode | Independent review |
| Authorization Evidence | Issue #59 ordered child selection after I222 completion; claim/activation are ineffective until #385 merges. Protected permission/security/API review and exact-head evidence remain mandatory for implementation. |
| Implementation PR | Not started |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | Implementation starts only after #385 reaches `main`; keep I224 Review / Claimed until a real implementation commit and independent exact-head review exist. |

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

Pending claim and implementation.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot self-certify a behavior implementation.
