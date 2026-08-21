# ADR-064: Bounded Model-Assisted Auto Permission Decisions

> Status: Proposed
> Date: 2026-08-22
> Owner: PERM-007-A / I218
> Supersession: on acceptance, supersedes ADR-011; until then ADR-011 remains authoritative.

## Context

Issue #188 requests a configurable cross-surface `auto` mode, default enabled, that asks a model to
resolve low-risk permission prompts and asks a human only when safety cannot be established. The
current accepted ADR-011 disables Guardian assistance by default and prohibits first-version
write-capable auto-approval. Current code has multiple Ask resolvers and no structured model seam.

Treating the originating model as an implicit second permission engine would violate the
non-bypass boundary. Refusing every write-capable Ask, however, would make the requested unattended
development mode ineffective because ordinary workspace writes default to Ask. The decision must
therefore define a narrow, code-enforced maximum authority before implementation.

The current-path and threat evidence is
[`I218-AUTO-PERMISSION-THREAT-MATRIX.md`](../reference/I218-AUTO-PERMISSION-THREAT-MATRIX.md).

## Constraint Decomposition

| Constraint | Type | Source | Consequence |
|---|---|---|---|
| All writes pass the permission pipeline; Deny cannot be bypassed. | Hard | `AGENTS.md` #4; ADR-011 | Model assistance occurs only after authoritative Ask and cannot replace policy. |
| Permission/sandbox changes require independent security review. | Hard | `AGENTS.md` #5 | Exact-head threat-matrix review gates acceptance and every protected implementation child. |
| Public crate APIs are semver-bound. | Hard | `AGENTS.md` #6 | Adding a public config field or Runtime seam needs a migration/version plan. |
| `auto` defaults enabled and supports `/auto`. | Soft product target | Issue #188 / maintainer | “Enabled” means attempt bounded assistance, never unconditional Allow. |
| Model judgment may be wrong, injected, unavailable or expensive. | Assumption | ADR-011 and threat model | Deterministic eligibility, closed schema, deadline, no retry and circuit breaker bound the model. |
| Third-party Runtime hosts own composition. | Hard architecture boundary | ADR-024/052 | SDK/MCP do not implicitly read CLI global config; hosts explicitly inject a resolver. |

## Decision

### 1. Authority order and eligible result

The authoritative order is:

1. normalize and validate the structured request;
2. evaluate all permission facets and hard boundaries;
3. return policy `Allow` or terminal `Deny` unchanged;
4. for `Ask`, run deterministic auto-eligibility checks;
5. if eligible and effective mode is on, request one isolated model assessment;
6. validate and bind its output to the exact request; otherwise request human approval;
7. construct/revalidate exact execution authorization immediately before execution.

The model output has only `AllowOnce` and `HumanRequired`. It cannot emit a reusable grant,
replacement resource, altered operation, policy rule, sandbox fallback or persistent approval.
`AllowOnce` is valid only for the exact request digest, policy revision, mode generation and
session, and is consumed once. Any Deny or hard-boundary result is terminal before the model seam.

### 2. First implementation maximum authority

The first implementation may assess only an Ask composed entirely of workspace-local `Write`
facets for creation of one new structured text file by an explicitly allowlisted native Talos
tool. Modification of any existing file remains ineligible.
The operation must carry a typed managed-workspace lease proving an isolated non-`main` worktree
and must not overwrite a pre-existing user or parallel-session change. Talos CLI derives this lease
from the effective claimed Work Slice; SDK hosts may supply an equivalent narrowly scoped lease
without depending on Talos repository governance documents.

The following are always ineligible and therefore require a human or headless Deny:

- Execute or Network facets, mixed profiles containing either, direct shell/process execution;
- external/unresolved paths, VCS internals, credentials, secret stores or sensitive config;
- modification, delete, rename, chmod, binary mutation, sandbox fallback or persistent grants;
- plugin/MCP-originated tools, Talos CLI work without an effective claim, `main`, and workspaces
  without a typed managed-workspace lease; and
- any request whose safe assessment would require unredacted secrets or large raw content.

Eligibility is a code-owned allowlist and predicate. The evaluator cannot declare itself eligible.
Expanding any excluded class requires a new ADR or explicit ADR-064 amendment with fresh security
review and adversarial evidence.

The request and one-shot authorization bind `TargetState::Absent`, the normalized target path and a
trusted open parent-directory capability/identity held through the mutation. Creation must be
relative to that capability and use atomic no-clobber semantics. A path-string parent recheck
followed by `create_new` is not sufficient against parent-directory swap. If another actor creates
the target after assessment, execution fails without opening or replacing it; if the platform
cannot provide the capability-bound primitive without separately approved dependency/`unsafe`
work, that platform remains ineligible. The current `WriteTool` check-then-`tokio::fs::write`
sequence is explicitly insufficient and must be replaced before the tool enters the allowlist.

Automatic modification is deferred because a content digest check followed by rename is not a
cross-process atomic compare-and-write. A later ADR amendment must bind a trusted target-state
version to the one-shot authorization, define an implementable atomic mutation primitive on every
supported platform, and pass same-path parallel-modification fixtures before any existing file is
eligible.

### 3. Mode, configuration and session precedence

The CLI configuration shape is `auto.enabled`, with a serde default of `true`. Absence in an old
configuration therefore enables *attempted bounded assistance*, not permission Allow. A user can
persist `false` to restore the existing human/headless path.

`/auto`, `/auto on` and `/auto off` operate on the active runtime session. The no-argument form
reports effective state, source, evaluator identity, deadline and circuit state. A session override
wins over `auto.enabled` but is not written to config or transcript. New, resumed and forked
sessions initialize from current config rather than inheriting a stale override.

Precedence is terminal policy Deny/hard boundary, then composition support, then session override,
then config/default. Goal is only an operation-mode input; it receives no additional authority.

CLI/TUI/Goal/print use the same contract when supplied equivalent typed context. Print/headless
maps `HumanRequired` or resolver failure to Deny. `talos-runtime` and MCP do not read CLI config;
their existing no-handler default remains `HeadlessDeny`, and an embedder must explicitly inject
the resolver and effective policy.

### 4. Evaluator isolation, input and output

The assessment is a separate tool-free, non-recursive provider request. It may use the selected
configured model in the initial implementation, but its identity and policy version are visible in
status/audit. A later utility-model route is compatible but not required by this decision.

Input is a closed, versioned structure containing safe tool/provenance/risk identifiers,
workspace-relative resource labels, operation subtype, bounded redacted intent/change preview,
operation mode, policy/mode generation and request digest. Conversation and preview fields are
explicitly untrusted data. Raw environment, credentials, full arguments, full file contents,
provider reasoning and external-path contents are forbidden.

Output is a closed schema with `schema_version`, `request_digest`, `decision`, `reason_code` and
`confidence`. `AllowOnce` requires `confidence=high` and the sole initial allow reason
`bounded_workspace_text_create`; human-required reason codes are closed and non-authoritative.
Free-form explanation is display-only and cannot affect execution. Unknown/extra/conflicting
fields, any lower confidence, injection indicators or request-digest mismatch become
`HumanRequired`.

### 5. Deadline, cost and circuit breaker

Each Ask permits at most one evaluator request and no automatic retry. The default deadline is
eight seconds and configuration may not exceed thirty seconds. Timeout, provider error, malformed
output or validator failure becomes human confirmation when available and Deny otherwise.

The session circuit opens after two consecutive technical/validation failures or three consecutive
`HumanRequired` outcomes. While open, all Ask outcomes skip the evaluator. Only explicit
`/auto on` resets the circuit; ordinary successful turns and session resume do not silently reset
it. Disabling the mode cancels an in-flight assessment and invalidates its mode generation.

### 6. Audit and privacy

The resolver constructs a redacted structured decision report before any model `AllowOnce` becomes
an execution authorization. Failure to construct that report becomes human confirmation or
headless Deny. Reports distinguish policy Allow, policy Deny, model AllowOnce, human approval and
headless Denial. They contain safe request/audit IDs and digests, tool/provenance/risk class,
effective mode source, evaluator/policy versions, outcome/reason code, latency and circuit state.
They never store raw prompts, reasoning, secrets, full arguments/content or credential-bearing
paths. The UI may show a redacted explanation but persistence uses stable reason codes; delivery to
an optional durable sink remains host-owned and sink failure cannot add authority.

### 7. Compatibility, migration and rollback

PERM-007-B must treat the new public `Config` field as a Rust source-compatibility change for
exhaustive struct literals. It must ship only under an explicitly selected compatible workspace
version step, document `..Config::default()`/builder migration, preserve old TOML parsing and avoid
changing unrelated Cargo default-feature composition. This ADR itself changes no public API.

PERM-007-C integrates only after PERM-006-A/B/C provide the structured decision and single
authoritative evaluate-to-execute seam. Existing custom `ApprovalHandler` behavior remains valid;
no resolver means the existing human or headless-deny behavior.

Operational rollback is `auto.enabled = false`, `/auto off`, or omission of the Runtime resolver.
Security rollback disables the resolver while leaving the permission engine and human approval
path intact. A source rollback restores the previous default without migrating grants because
model decisions never persist grants.

## Child Delivery Boundaries

| Child | Runnable deliverable | Acceptance focus |
|---|---|---|
| PERM-007-B | Versioned `auto.enabled` configuration and typed `/auto` session state/status only | precedence, defaults, session transitions, public-config migration; no model call |
| PERM-007-C | Deterministic create-only eligibility plus isolated evaluator, closed schemas, audit and circuit breaker at the PERM-006-C seam | Deny precedence, absent-target/parent binding, atomic no-clobber create, privacy, injection, timeout, replay and no grants |
| PERM-007-D | Shared cross-surface conformance and rollout/rollback evidence | CLI/TUI/Goal/print/Runtime/MCP parity and human/headless fallback |

Each child needs its own runnable iteration, effective Collaboration Claim, protected-scope review
and pre-existing implementation evidence before completion. Acceptance of this ADR authorizes none
of them.

## Rejected Alternatives

- **Let the originating model approve its own tool call directly**: no independent request,
  deterministic eligibility or auditable boundary.
- **Default-on for every Ask**: makes Execute/Network/external-path prompts implicit model
  authority and violates the requested human fallback.
- **Keep all writes human-only**: safe but prevents even new bounded artifacts; create-only
  no-clobber is a smaller independently provable first step.
- **Allow model-assisted modification using hash-check then rename**: still has a cross-process
  check-to-replace race; deferred until a portable atomic compare-and-write contract is reviewed.
- **Persist model approvals as grants**: compounds model error across later operations and policy
  changes.
- **Make SDKs read `~/.talos/config.toml`**: violates host-owned Runtime composition and creates
  ambient authority.

## Validation And Acceptance Gate

Before this ADR becomes Accepted:

- independently review every threat-matrix row at the exact decision head;
- prove the current-path statements against the named source seams;
- confirm default-on means attempted assistance, not default Allow;
- confirm first-version write eligibility is create-only, binds absent target/path/open parent
  capability, uses capability-relative atomic no-clobber at the mutation point and cannot override
  Deny, modify existing content, create grants, approve external/execute/network/sandbox paths or
  reuse results;
- verify privacy, injection, race, timeout, circuit, headless, semver and rollback contracts; and
- run both governance validators with an explicit target-branch base, manifest YAML parsing and
  `git diff --check`.

No Rust test can establish behavior at this decision-only stage. Implementation children must add
the adversarial and conformance fixtures specified by the threat matrix.

## Consequences

- The product default can be “on” while actual authority remains narrow, typed and fail-closed.
- Governed unattended creation of new text files under a typed managed-workspace lease can avoid
  repeated human prompts without coupling third-party SDK hosts to Talos governance documents.
- Editing existing files remains human-mediated until a portable atomic compare-and-write boundary
  is separately accepted.
- General shell, network, external-path and arbitrary-workspace Ask outcomes still require a human.
- The separate evaluator adds latency and provider cost; bounded timeout/circuit behavior makes that
  visible and reversible.
- PERM-006-A/B/C remains on the critical path; this decision removes policy ambiguity but does not
  accelerate implementation authority.

## Reversal Triggers

Disable or supersede this decision if adversarial testing shows scope enlargement, secret leakage,
result replay, policy bypass, cross-surface divergence, unacceptable false approval, or inability
to distinguish user/parallel-session changes. Any proposal to admit modification, Execute,
Network, external paths, sandbox fallback, persistent grants or unmanaged workspaces requires a
fresh decision and independent security review.
