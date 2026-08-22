# ADR-066: First-Class Scoped Permission Grants

- Status: Accepted
- Date: 2026-08-22
- Owner: PERM-006-B / Issue #54
- Related: ADR-026, ADR-047, ADR-064, ADR-065; Issues #52, #54, #59 and #188

## Context

Talos currently represents a reusable user approval by inserting an ordinary
`PermissionRule` into `PermissionEngine`. ADR-065 made that insertion source visible for truthful
diagnostics, but deliberately preserved the existing rule-vector matching, precedence and grant
lifetime. It also explicitly withheld authority for PERM-006-B.

That compatibility bridge cannot be the final grant model:

- configured policy and runtime approvals remain values in the same ordered rule vector;
- `add_runtime_allow_rule` receives only a `PermissionRule`, so it cannot bind the complete tool
  provenance or the original multi-facet request;
- CLI/TUI currently broadens a reusable path-write approval to a parent-directory glob, while the
  embedded Runtime constructs exact-resource rules;
- preview text is reconstructed separately from insertion and can drift from installed scope;
- the public `Persisted` execution-authorization label describes a reusable rule even though the
  current approval is memory-only; and
- a future RPC, MCP or plugin surface could create another compiler with different scope.

PERM-006-B must introduce a single grant contract without weakening any effective policy Deny, external-path
authorization, multi-facet coverage or provenance isolation. Because `talos-permission` and
`talos-core` are published SDK crates, this decision also owns the necessary pre-1.0 public API
migration. No release or version change is authorized here.

## Constraint Decomposition

| Constraint | Type | Source | Consequence |
|---|---|---|---|
| Write-capable tools remain permission-gated and no effective policy Deny can be bypassed. | Hard | `AGENTS.md`; PERM-004/PERM-005 | A grant may resolve only an otherwise unresolved `Ask`; it cannot replace Configured, Explicit or future policy Deny, mode restrictions or a hard boundary. |
| Permission and security changes require independent review. | Hard | `AGENTS.md` | This ADR and every implementation head require exact-head permission/security review. |
| Public crate APIs are semver-bound. | Hard | `AGENTS.md`; ADR-065 | Removing the legacy runtime-rule bridge or renaming its lifetime requires an explicit migration and a future v0.9.0-or-later release. |
| Hybrid tools expose every consequential facet. | Hard architecture boundary | ADR-026 | Grant installation and reuse are atomic across the complete request; one approved facet cannot authorize another. |
| External paths require exact reviewed execution authority. | Hard security boundary | ADR-047 | A reusable external-path grant is exact-only and still requires execution-time normalization and authorization. |
| `AlwaysApprove` means only the active session today. | Confirmed current product contract | Issue #54 and current composition roots | The first store is explicit, in-memory and session-owned; no durable or task-scoped promise is introduced. |
| CLI and Runtime grant compilation currently differ. | Confirmed implementation fact | CLI `approval.rs`; Runtime `lib.rs` | One compiler replaces both implementations; convergence may tighten unsafe legacy scope but must not broaden authority. |

## Decision

### 1. Policy and grants are separate authority classes

Configured/default/explicit `PermissionRule` values remain policy. A first-class
`PermissionGrant` is a distinct, non-serialized value with its own identifier, scope, source,
complete tool identity and compiled facet scopes. New grants are never inserted into
`PermissionEngine::rules()` and are never written to permission configuration.

The structured decision report distinguishes policy matches from grant matches. Grant identifiers
are opaque, session-local, non-secret diagnostic values; they are not authorization tokens or
durable foreign keys. Raw request input and secret-bearing resource data remain excluded from safe
`Debug`, logs and serialized diagnostics.

The composition root owns explicit session permission state containing policy evaluation and one
mutable in-memory grant store. The store is not global. That state carries a session identity,
policy revision, store generation and the effective mode/workspace/tool-registration/restriction
generations needed to invalidate stale approvals. The restriction generation includes authoritative
sandbox and fallback policy. Policy mutation, mode/workspace/registration/restriction replacement
and store clear increment the applicable revision or generation.

Initial evaluation and proposal creation capture one session-consistent snapshot, then release the
session lock before waiting for a human or host response. Approval commit reacquires the state and
uses compare-and-swap validation before grant insertion or authorization issuance. Matching and
tool admission each validate a current snapshot. A concurrent caller cannot observe a partially
installed multi-facet grant, and an approval made against an older snapshot cannot cross a newer
Deny or changed context. The exact Rust ownership type is an implementation choice; the public API
must make session lifetime and revision validation explicit.

ADR-065's `PermissionContext.mode` remains diagnostic context, not policy. PERM-006-B consumes the
composition root's existing authoritative mode/sandbox restriction result and generation; it does
not move ownership of those restrictions into the diagnostic context or perform PERM-006-C's later
pipeline convergence.

### 2. Scope is closed to `Once` and `Session`

The initial approval scope has exactly two implemented meanings:

- `Once` binds the exact normalized request identity and never enters the session store. The
  non-authoritative proposal transitions to a non-`Clone` approved-once value; official
  composition roots consume that value to issue the exact authorization set for one invocation.
  Raw `AgentTool` callers remain trusted SDK composition and are not falsely claimed to gain a
  cryptographic one-use capability from this slice.
- `Session` may be inserted only into the explicitly supplied session store. Dropping or clearing
  that store prevents future matching and authorization issuance from those grants. It does not
  claim to cancel an already-started tool call. A new, resumed or forked session starts with an
  empty store unless a later separately accepted decision defines inheritance.

Persistent, task, scheduler, workspace-trust and cross-process grants are not placeholder variants
in the public enum. They require a separate owner and ADR before representation or storage.

### 3. One compiler produces both installation and preview

One compiler in the non-product permission layer consumes the authoritative structured
`PermissionRequest` and selected scope, validates every facet, and returns either one complete
proposed grant or an error. It does not accept a caller-authored `PermissionRule` as a grant.

The compiler returns a non-authoritative `ProposedGrant`, not a storeable authority. It binds full
provenance and facets, session identity, policy revision, store generation, and effective
mode/workspace/tool-registration/restriction generations. It also binds an internal versioned,
domain-separated, collision-resistant fingerprint of the complete authoritative request. That
fingerprint and its potentially secret-bearing preimage are memory-private: neither may enter
`Debug`, logs, serialized reports, previews or other observer surfaces. Audit correlation uses a
separate random opaque session-local request ID that cannot correlate requests across sessions.

The approval preview is derived from that proposal. A UI may not reconstruct scope independently.
Preview data is bounded and safe for the local approval surface: it contains the normalized scope
needed for informed consent but no raw tool input, free-form secret-bearing description or
credential value. Persistent diagnostics use a coarser redacted summary.

Only an explicit approval transition can produce authority. `GrantSource` is an exhaustive closed
enum whose initial reusable sources are interactive human and explicitly configured SDK host
approval handler; safe reports project only that closed class. Model assistance from ADR-064 can
produce only its separately governed invocation-local result and can never produce a Session grant.
The approval transition re-evaluates the exact request and verifies every bound revision/generation
after the approver responds. Any changed request, policy, mode, workspace, tool registration,
restriction, session or store generation invalidates the proposal and returns to Ask/deny.
Revalidation plus Session insertion, or revalidation plus Once authorization issuance, occurs in
one session-state commit boundary. The store accepts only approved Session grants, never a
`ProposedGrant` or approved Once value.

Any missing resource, invalid normalization, unsupported facet, compiler error, store error or
preview-generation failure installs nothing and returns to unresolved `Ask`; a headless caller
therefore denies. There is no partial multi-facet grant.

### 4. Matching binds complete request identity

A proposal and approved Once value bind the private complete-request fingerprint and revision
snapshot. An approved Session grant may keep only the separate opaque request ID for
non-authoritative audit correlation; the private fingerprint is not an observer identifier and
never participates in matching a later request. Session reuse instead binds:

- the complete `ToolProvenance`, including MCP server identity or plugin package identity;
- the tool name;
- facet nature and typed resource kind;
- the normalized compiled resource scope; and
- the Session scope, owning session and truthful approval source.

This separation permits different raw inputs to reuse the same reviewed exact path, Domain or Bash
template scope while preventing the original private fingerprint from becoming a wildcard. Each later
request is first evaluated against current hard boundaries, all-policy Deny and restrictions, then
matched against the compiled scopes. A policy revision change does not by itself erase installed
Session grants; every later match still applies current Deny and restrictions. Session, workspace
or tool-registration replacement clears the affected store, and tool/provenance mismatch never
reuses a grant.

The coarse observer-facing `PermissionToolSource` is insufficient for authorization matching.
Native, MCP and plugin tools with the same displayed name never share grants, and two different MCP
servers or plugin packages never share grants implicitly.

Every consequential request facet must be covered by an authoritative policy `Allow` or a matching
grant for that same request. A matching grant for one facet does not suppress `Ask` or `Deny` on
another facet. Reuse produces an exact execution authorization immediately before tool execution;
the grant itself is not a file-system or process capability.

### 5. Precedence is fail-closed

The authoritative order is:

1. terminal hard security Deny;
2. any matching effective policy `Deny`, including Configured and SDK/Runtime `Explicit` rules;
3. active mode or sandbox restriction;
4. matching scoped grant;
5. remaining configured/default/explicit/trusted-workspace policy using its legacy first-match
   behavior within the non-Deny class; and
6. unresolved interactive `Ask` or headless `Deny`.

A grant can turn only an otherwise unresolved facet into `Allow`. It cannot override Deny, create a
new resource, change a mode, disable sandboxing or grant another provenance. If contradictory
policy rules previously allowed an operation because a matching Allow preceded a matching Deny,
all-policy Deny dominance is an intentional security tightening. Serialized rule syntax and
ordinary non-conflicting rule behavior remain compatible.

An ADR-047 external-path approval boundary that returns `Ask` is resolvable by an exact approved
grant and later exact execution authorization; it is not the terminal hard Deny in step 1.

### 6. Resource-specific scope

#### Workspace paths

The first implementation compiles every reusable path write, including a normalized
workspace-contained write, as exact-only. This intentionally tightens the current CLI/TUI parent
directory glob behavior and preserves the Runtime's narrower authority. A parent-directory or
workspace-recursive grant would broaden at least one current surface and requires a later ADR
amendment with separate security evidence.

For an external path, reusable scope is also exact-only. It never becomes a directory glob, still
passes ADR-047 execution-authorization issuance and is normalized again at execution. An unresolved
path fails grant compilation. Workspace trust cannot broaden it.

#### Bash and process resources

The compiler consumes the permission resource already emitted by the audited Bash/tool classifier.
It does not parse raw shell text or maintain a second allowlist. Only an existing reviewed template
descriptor may yield reusable template scope. Control syntax, unsafe tokens, traversal/absolute
arguments, mutations, package/network operations and unclassified commands remain exact.

#### Network and remote resources

The initial Domain scope is the normalized host resource already emitted by the current typed
facet/extractor; it does not promise scheme, port or path matching. A Remote scope is exact only
when the producing tool supplies a canonical typed identifier. The compiler does not invent an
endpoint grammar or normalize an opaque string ad hoc; unsupported or missing canonical resources
fail compilation. Current tool/provenance/nature/resource-kind distinctions remain authoritative;
operation-level network read versus mutation requires the PERM-006-D typed-effect follow-up and is
not inferred from raw HTTP input here.

### 7. Public API migration

PERM-006-B may make the following pre-1.0 source changes under the release boundary already
established by ADR-065:

| Legacy API | Migration |
|---|---|
| `PermissionEngine::add_runtime_allow_rule(rule)` | Compile a structured request to `ProposedGrant`, complete the approved transition against the current revision snapshot, then consume the approved-once value or insert the approved Session grant. |
| `PermissionRuleSource::RuntimeGrant` | Use the separate grant-match decision source; policy rule sources remain Default, Configured and Explicit. The legacy serialized diagnostic value is no longer emitted. |
| `PermissionDecisionSource` / `PermissionReason` exhaustive matches | Handle the new grant-match source and reason. These public enums are exhaustive today, so this is an explicit v0.9 source/schema migration rather than a silently additive promise. |
| `ToolAuthorizationScope::Persisted` | Map configured/default/explicit/trusted-workspace Allow to `Policy`, current-invocation human/model approval to `Once`, and approved reusable grant authority to `Session`. `Policy` and `Session` are distinct; neither claims durable grant storage. |
| Read runtime approvals through `PermissionEngine::rules()` | Inspect grants only through redaction-safe session-store/report APIs; `rules()` remains policy-only. |

The new public `GrantSource` is a closed exhaustive enum with only `InteractiveHuman` and
`SdkHostApproval` reusable sources in this slice. Safe report projection uses those stable classes;
free-form caller labels and model-produced Session sources are rejected.

The implementation removes in-tree use of the legacy runtime-rule path. It may retain a deprecated
adapter only if the adapter is clearly classified as legacy rule insertion, cannot masquerade as a
first-class grant and does not weaken the invariants above; otherwise it is removed. A future Cargo
publication containing the source break must use v0.9.0 or later and document this table.

`PermissionRule` JSON/TOML, `PermissionDecision` serialization and durable configuration require no
data migration. The structured report JSON Schema gains the documented grant source/reason and no
longer emits `runtime_grant` as a policy-rule source; downstream exhaustive Rust/JSON consumers must
migrate at v0.9. Initial grant/compiler/store types are non-serialized and promise no stable storage
format. The workspace version, tag and publication remain outside PERM-006-B.

## Compatibility And Rollback

Except for the explicitly recorded all-policy-Deny security tightening, callers that do not
configure an approval handler keep current policy behavior. With no grant store, or with an
empty/cleared store, unresolved interactive requests still ask and headless requests deny.
Configured policy is not rewritten, so disabling session grant reuse requires no data rollback.

Operational rollback disables grant insertion and clears the affected session store, returning to
human approval or headless Deny. Clear increments the store generation, prevents future matching or
authorization issuance and invalidates uncommitted proposals.

Authorization issuance is not the admission fence. The official adapter holds a non-`Clone`
pending invocation carrying the validated state generations. Immediately before calling
`execute_authorized`, it reacquires session state, rechecks current hard/policy Deny and restriction
state, and atomically marks that pending invocation started. A clear or relevant generation change
before this fence discards the authorization and returns to Ask/deny; a change after the fence does
not claim to cancel the already-started tool call. Cancellation and full cross-pipeline
evaluate-to-execute coordination remain PERM-006-C work, but B does not leave an issued-yet-unstarted
grant authorization reusable after clear.

Rollback must not reinsert new first-class grants into configured policy. If scope enlargement,
provenance collision, preview/install divergence, Deny bypass or cross-session reuse is observed,
session reuse is disabled until a reviewed correction lands.

## Exclusions

- No persistent, task-scoped, scheduler-inherited or cross-process grant.
- No trusted-workspace broadening or permission-config write.
- No PERM-006-C agent-owned pipeline migration or wrapper removal beyond replacing B's duplicated
  grant compiler calls.
- No PERM-006-D typed-resource migration beyond consuming current typed facets.
- No model-assisted auto decision, `/auto`, sandbox fallback or Issue #188 behavior.
- No TOOL-024 background-process implementation, release, version, tag or publication work.

## Rejected Alternatives

### Keep runtime grants as specially labelled rules

Rejected because a rule lacks complete request/provenance/session identity, remains order-coupled
to policy and cannot make preview and installation one object. ADR-065's `RuntimeGrant` source was a
truthful compatibility bridge, not authorization for this final representation.

### Let each composition root compile its own grant

Rejected because current CLI and Runtime behavior already diverges, and future surfaces would add
more scope and preview drift.

### Store only one grant per facet

Rejected because partial insertion can authorize part of a hybrid request and makes approval
preview differ from installed authority. One approved request installs atomically.

### Persist `AlwaysApprove` immediately

Rejected because current behavior is session-only and durable grants require revocation,
configuration migration and scheduler inheritance decisions outside this Story.

### Match only tool name and coarse provenance class

Rejected because same-named MCP/plugin/native tools and distinct extension providers would share
authority accidentally.

## Validation And Acceptance Gate

Before this ADR becomes Accepted:

- independently verify the current CLI/Runtime scope divergence and ADR-065 non-overlap;
- review all-policy Deny precedence, approval snapshot/restriction revalidation,
  proposal/authority separation, Session scope reuse without input-digest coupling, multi-facet
  atomicity, full provenance identity, exact-only path scope, preview/install identity, session
  lifetime, pre-admission clear fencing, Once non-replay at official adapters and public API/schema
  migration against escape paths;
- run both governance validators with an explicit target-branch base, parse the manifest YAML and
  run `git diff --check`; and
- bind acceptance to the exact decision head, independent Agent-role permission/security/API
  review, CI, merge-time CAS and target-branch merge.

Acceptance authorizes only this decision. PERM-006-B still requires a separate runnable/testable
iteration and effective protected-scope Collaboration Claim before any implementation branch or
Rust/Cargo change.

## Acceptance Evidence

The decision content is fixed at commit
`17088d88ed263cb9a66776182a897e0ca39772e0`. Independent Agent-role permission/security/API review
approved the Proposed candidate at exact head
`33199bd88bd2e8487385b8406514129087170e3d` in PR #358 comment `5376959300`, after exact-head CI
run `32541156457` and the required governance checks passed. Repository-owner acceptance is
recorded in PR #358 comment `5378407820` against the unchanged candidate.

This Accepted status activates only the decision boundary. PERM-006-B remains Ready/Unclaimed with
Selected Iteration None; no implementation, release, version, tag or publication authority follows
from ADR acceptance.

## Implementation Validation Required Later

- Configured and Explicit Deny shadow matching session grants, including conflicting-rule fixtures;
  the same no-handler fixtures prove this intentional tightening is the only policy-only variance;
- policy/mode/workspace/registration/restriction/session/store changes while approval is pending
  invalidate the proposal and install nothing; sandbox/fallback policy is part of restriction state;
- proposals cannot be inserted directly, and only truthful human/host approval can create a
  Session grant;
- approved Once values are non-`Clone`, bind the exact request/revision snapshot, are consumed only
  by the official adapter's current invocation and never appear in the store; concurrent/double-use
  adapter fixtures do not execute twice;
- different request inputs reuse only the same approved compiled Session scope; only the opaque
  request ID is audit correlation, while the private request fingerprint is neither stored in the
  Session grant nor used as a later-match condition;
- store clear/drop and new/resumed/forked session fixtures prove no cross-session reuse; a clear
  between issuance and admission blocks the unstarted invocation, while already-started tool
  cancellation remains outside B;
- CLI, TUI and Runtime adapters compile structurally identical grants and previews;
- workspace and external path grants are exact-only; the legacy CLI/TUI parent glob is absent;
- safe Bash templates share only through existing classifier descriptors; unsafe commands remain
  exact;
- native/MCP/plugin same-name and cross-provider collision fixtures do not match;
- hybrid requests install atomically and require coverage for every consequential facet;
- compiler/store/normalization/preview failures install nothing and fail closed;
- sentinel secrets and low-entropy values cannot be tested through `Debug`, preview, report, log or
  serialization output; opaque request IDs do not correlate across sessions;
- policy Allow, ApproveOnce and Session grant authorizations map to `Policy`, `Once` and `Session`;
- serialized `PermissionRule` and `PermissionDecision` compatibility remains unchanged while
  public exhaustive enum and report-schema migration fixtures cover the documented v0.9 changes;
- locked permission, core, tools, agent, CLI, Runtime, MCP, plugin and workspace validation passes
  at the exact implementation head.

## Reversal Triggers

Revisit or supersede this ADR if implementation cannot preserve session isolation without global
state, existing typed facets cannot express a safe grant without re-parsing raw input, public API
migration has no equivalent downstream path, or adversarial tests show Deny bypass, scope
enlargement, provenance collision, partial multi-facet authority or preview/install divergence.
