# I219 / PERM-006-B Scoped Grant Migration

## Status And Version Boundary

I219 replaces Talos's compatibility representation of reusable approvals with first-class scoped
permission grants. The source and report-schema changes described here are approved by ADR-066 for
a future v0.9.0-or-later publication. This implementation does not change the workspace version,
create a tag, or authorize publication.

Configured permission rules remain compatible. No permission configuration or durable session data
migration is required because grants are memory-only and are never serialized.

## Authority Model

Policy and approval authority are separate:

| Authority | Meaning | Lifetime |
|---|---|---|
| `ToolAuthorizationScope::Policy` | Current configured/default/explicit/trusted-workspace policy allowed the request. | Re-evaluated from current policy. |
| `ToolAuthorizationScope::Once` | One exact approved invocation reached the admission fence. | Consumed by one official adapter invocation; not stored. |
| `ToolAuthorizationScope::Session` | A matching first-class grant allowed the request. | Current in-memory permission Session only. |

A Session grant binds the complete tool provenance, tool name, every permission facet, typed
resource kind and normalized exact scope. It can resolve only an otherwise unresolved `Ask`.
Configured or explicit `Deny`, hard boundaries and current restrictions remain authoritative.

## Rust API Changes

| Before | v0.9+ migration |
|---|---|
| `PermissionEngine::add_runtime_allow_rule(rule)` | Build a `PermissionRequest`, call `PermissionSessionState::propose`, obtain explicit approval, then call `approve_once` or `approve_session`. |
| `PermissionRuleSource::RuntimeGrant` | Match `PermissionDecisionSource::Grant { grant_id, grant_source }`; policy rule sources remain `Default`, `Configured` and `Explicit`. |
| `ToolAuthorizationScope::Persisted` | Use `Policy`, `Once` or `Session` according to the authority that produced admission. None of these scopes represents a durable grant. |
| Inspect runtime approvals through `PermissionEngine::rules()` | Keep policy inspection on `rules()`; use redaction-safe `PermissionSessionState` evaluation and grant-count/report APIs for Session authority. |

`PermissionDecisionSource` and `PermissionReason` are exhaustive public enums. Consumers must add
the grant-match variants when moving to the future v0.9+ crate set. Structured report schemas may
emit the grant source/reason and no longer emit `runtime_grant` as a policy source.

## Embedded Runtime Handlers

`ApprovalHandler::request_scoped_approval` receives the compiler-produced `GrantPreview`. Existing
handlers continue to compile because its default implementation delegates to `request_approval`.
New handlers should render the preview directly and return:

- `ApprovalChoice::ApproveOnce` for one invocation; or
- `ApprovalChoice::AlwaysApprove` for reuse inside this `RuntimeHandle` only.

Every `RuntimeBuilder::build()` creates a fresh `PermissionSessionState`. Reusing the same builder,
workspace or durable transcript does not inherit grants. Missing handlers and unresolved headless
approval remain deny-by-default.

## CLI And TUI Behavior

`talos permissions preflight` is read-only. It compiles the same bounded proposal preview used by
interactive approval, without executing a tool or installing authority. Write and external path
scopes are exact-only. Bash reuse consumes the audited tool classifier descriptor: reviewed safe
templates may share a template scope, while mutating or unclassified commands remain exact.

The TUI clears grants only at a successful runtime publication fence. Failed new/resume/fork/model
transitions retain the still-active Session state; successful replacement clears it before the new
Actor starts. `/attach` and print-mode attachments pass the same proposal/admission path before file
ingestion, and changed symlink targets fail exact matching.

## Compatibility And Rollback

- Existing rule JSON/TOML and `PermissionDecision` serialization do not change.
- Disabling Session approval or clearing the in-memory store returns unresolved requests to Ask or
  headless Deny; rollback never writes grants into policy.
- A policy, mode, workspace, registration, restriction or store generation change invalidates a
  pending proposal. Clear before admission invalidates an approved but unstarted invocation.
- Persistent, task, scheduler, inherited and cross-process grants are not represented by this API.

Before a v0.9+ publication, downstream exhaustive Rust and JSON-schema consumers must update for
the enum/schema changes above and rerun their permission integration tests.
