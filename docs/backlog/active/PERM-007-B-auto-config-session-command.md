# PERM-007-B: Auto Configuration And Session Command

| Field | Value |
|---|---|
| Story ID | PERM-007-B |
| Type | Permission / Configuration / CLI-TUI Story |
| Priority | P1 |
| Status | Review / Claimed |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | I233 |
| Depends On | PERM-007-A/I218 and ADR-064 Accepted; PERM-006-A/B/C complete |

## Identity / Goal / Value

Provide an explicit, inspectable configuration and session control surface for the accepted
cross-surface `auto` mode without making any model request or changing permission outcomes.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline session |
| Work Slice | Implement only PERM-007-B: `auto.enabled` config default/migration and non-persistent session `/auto` status/on/off control. Exclude model requests/resolver/eligibility/grants/permission results/execution, PERM-007-C/D, PERM-006-D/E, sandbox, Dashboard, Desktop, release, publication and dependency changes. |
| Claimed At | 2026-08-28 |
| Source Issue | #188 |
| Governance Claim PR | #426 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; exact-base validators, CI and independent review required before implementation. |
| Implementation PR | #428 |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Claim #426 merged as `7f47f9c3`; implementation PR #428 requires exact-head CI, independent permission/API review and CAS. |

## Published Baseline

### Scope

- Add the versioned `auto.enabled` configuration field with serde default `true`.
- Add non-persistent per-session override state and `/auto`, `/auto on`, `/auto off` command behavior.
- Report effective state, source, evaluator identity, deadline and circuit state without secrets.
- Reset session override on new/resumed/forked sessions according to ADR-064; `/auto off` restores
  the existing human/headless path.
- Keep configuration parsing and documented migration compatible with old TOML.

### Non-Goals

- No model request, resolver, eligibility predicate, grant, permission result or execution change.
- No PERM-007-C evaluator/circuit implementation, PERM-006-D/E, sandbox/fallback, `/goal` behavior,
  Dashboard, Desktop, release, version, tag, publication or dependency change.

The later PERM-007-C child must establish a separate, independently reviewed model-assessment
contract. Its temporary context is minimal/redacted and digest-bound to the exact permission
request; full conversation, credentials, raw untrusted tool input and provider reasoning are never
forwarded. Model output remains a bounded suggestion subject to deterministic eligibility and
independent validation, never a replacement permission authority.

### Acceptance

- Given an omitted `auto.enabled` field, loading config yields enabled attempted assistance while
  leaving existing permission behavior unchanged.
- Given configured `auto.enabled = false`, `/auto` reports disabled and `/auto on` enables only the
  active session; the override is absent from config and transcript.
- Given `/auto off`, the session reports disabled and subsequent permission flow remains human or
  headless-deny; no model call is made by this slice.
- Given a new, resumed or forked session, no stale prior-session override is inherited.
- Given malformed/unknown command arguments, `/auto` returns bounded deterministic diagnostics and
  leaves the previous state unchanged.

## Planned Validation

- Config serde/default/migration tests and schema snapshot.
- CLI/TUI command and session lifecycle tests, including malformed arguments and redacted status.
- Locked focused workspace tests, format, Clippy, governance validators and `git diff --check`.
- Explicit structural proof that no model/provider/resolver/permission decision path is changed.
- Exact-head CI and independent permission/API review before merge.

## Documentation To Update

- `README.md` configuration and `/auto` command reference.
- PERM-007 parent, Issue #188 status, Board, backlog, iteration index, manifest and migration notes.

## Risks And Rollback

- Risk: a configuration flag is mistaken for unconditional Allow or a session override persists.
- Rollback: omit the field or set `auto.enabled = false`; remove the command while retaining the
  existing human/headless permission path and ADR-064 boundary.

## Residual Destination

Model-assisted eligibility, resolver, audit and circuit behavior belong to PERM-007-C; cross-surface
conformance belongs to PERM-007-D. Neither is authorized by this owner.

Public Rust users that construct `Config` exhaustively must add `auto: AutoConfig::default()` or
use `..Config::default()`. The corresponding compatible workspace version/release is separately
governed; I233 changes no version, tag or publication state.

Session override lifecycle is reset only after a successful runtime replacement publishes an
authoritative command sender; failed or rolled-back transitions do not mutate the active session's
override.
