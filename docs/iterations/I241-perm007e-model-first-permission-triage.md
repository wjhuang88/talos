# Iteration I241: Model-First Permission Triage

> Document status: Complete / Closed
> Planned objective: establish and implement a new, independently reviewed model-first permission
> triage contract that reduces routine approval prompts without blanket shell auto-approval.
> MVP deliverable: a runnable normalized-request matrix proves bounded low-risk shell/read/validation
> requests can receive one-time model assistance while every excluded or uncertain request remains
> human-required or denied.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-E | PERM-007 | Refinement / Unclaimed | PERM-006-C; PERM-007-D; ADR-064 | Model-first triage with constrained shell/exec coverage and fail-closed evidence |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline implementation session 2026-09-01 |
| Work Slice | Implement bounded model-first triage for normalized, redacted low-risk read-only and local-validation shell/tool requests; deterministic classification first, model `AllowOnce`/`HumanRequired` only, exact binding, fail-closed timeout/error/cancellation, audit redaction, and supported-surface conformance. No blanket shell approval, destructive/network/write expansion, sandbox fallback, permanent grants, Desktop, release or publication. |
| Claimed At | 2026-09-01 |
| Source Issue | #456 |
| Governance Claim PR | #459 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-069 accepted through I242 closeout merge `017886886f808dc359f6cb0b77e9bef53b822e6f`; implementation claim is ineffective until this finalized record reaches `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | After claim merge, converge locally and submit one stable implementation candidate for independent permission/security/API review. |

## Current Nonterminal Inventory And Disposition

| Iteration | Current state | I241 disposition |
|---|---|---|
| I241 | Complete / Closed | Implemented and merged as PR #460; closeout records exact-head evidence below. |
| I242 | Closed | ADR-069 accepted; use as normative decision contract. |
| I207, I208 | Planned / Unclaimed | Preserve steering order; no overlap. |
| I164 | Paused / superseded | Do not restore. |

Other Active, Review, Planned and Blocked iterations remain under their existing owners; this claim
does not modify Dashboard, Desktop, TOOL-024 or unrelated permission authorities.

## Governance Gate

Before activation, confirm accepted ADR-069, define the normalized shell schema and threat matrix,
then establish an effective Collaboration Claim. The proposed claim is ineffective until merged to
`main`.

Required decision read: `docs/decisions/069-model-first-permission-triage.md`.

## Scope And Exclusions

Use the PERM-007-E owner as the normative scope. Exclude blanket shell approval, destructive/network
operations, secrets, script interpreters, pipes/redirection/substitution, background execution,
sandbox expansion, Desktop, release and publication.

## Acceptance And Validation

- deterministic classifier runs before model assessment;
- model input is redacted and structurally normalized;
- valid low-risk results admit only one `AllowOnce`;
- stale, ambiguous, malformed, failed or timed-out results fail closed;
- CLI/TUI/Runtime/MCP semantics remain equivalent where applicable;
- focused adversarial tests, workspace locked checks, governance validators and independent security
  review pass at exact head.

## Next Step

ADR-069 was accepted before implementation. Implementation started from the claim merge and is now
complete.

## Execution Evidence

Implementation is locally converged on branch `feat/i241-model-first-permission-triage`.
The first candidate adds bounded native `exec` triage for `pwd`, constrained `ls`/`rg`, `git
status`, and exact local validation forms (`cargo fmt --check`, `cargo check`, `cargo test`,
`cargo clippy`). Shell syntax, pipelines, redirects, command substitution, background mode,
absolute/parent paths, unsupported arguments and symlink escapes remain ineligible and fall back to
the existing human-required path. Model input contains only closed risk/operation classes and a
one-way argument digest; raw arguments, paths and environment contents are not sent. Auto-approved
exec requests must not provide caller environment overrides (including `PATH`, toolchain, or
wrapper variables); any non-empty or malformed `env` field falls back to the existing human-required
path so the assessed command inherits the process environment unchanged.

Local validation: `cargo fmt --all -- --check`; `cargo check --workspace --locked`; `cargo test
-p talos-agent --locked` (all tests passed); focused auto-resolver tests include exact argument
binding, shell/path escape rejection, and caller-environment rejection. The initial independent
security review found environment overrides could bypass the bounded-effects claim; this candidate
records the local fix before fresh exact-head CI and review; both CI and follow-up review passed.

## Completion Evidence

- Completion Commit: `b7735ef8` (merge of implementation PR #460; source implementation commit `96d655d6`)
- Exact implementation head: `96d655d628bb3d463d6fa263270693da9bb14ccb`
- Base: `81ad333cbfdc2f4f1f6219e9a65e522c9adbadb0`
- Exact-head CI: `33475638936`, all five jobs successful
- Independent permission/security/API review: APPROVE, comment `5489739524`, bound to exact head; shared-account Agent-role limitation disclosed
- Merge-time CAS: head, base, CI, review and clean merge state verified before merge
