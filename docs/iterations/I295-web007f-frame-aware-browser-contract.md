# Iteration I295: Frame-Aware Browser Host Contract

> Document status: Complete / Closed; implementation merged by PR #683. Native browser/CDP remains an explicit downstream residual.
> Published plan date: 2026-10-06
> Planned objective: deliver an opt-in host-executed frame-aware browser v2 contract with exact admission, origin-bound permission and shared conformance evidence.
> Baseline rule: preserve this objective and acceptance; different outcomes need a new iteration.
> MVP deliverable: an embedded host can register a bound test executor, discover a child frame, authorize one exact action, and receive a bounded result; stale or unauthorized calls never mutate.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | WEB-007-F / #618 only; frame-aware v2 public contract, prepared invocation, browser-specific permission integration and host conformance |
| Claimed At | 2026-10-06 |
| Source Issue | #618 |
| Governance Claim PR | #669 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #683 exact head CI passed; Agent-role security/API review approved the exact candidate; merge-time CAS completed before merge. |
| Implementation PR | #683 (merged) |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Implementation complete; native browser/CDP/process-carrier remains a downstream residual outside this slice. |

## Published Baseline

### Selected Story And Dependencies

WEB-007-F / #618 is the sole selected child of WEB-007 / #452. ADR-086 and the linked
frame-aware contract govern this slice. The parent V1 proposal remains unimplemented and is not
claimed Complete by I295. This slice may publish separately negotiated v2 without pretending
that a V1 executor or native plugin exists. Process carrier and native browser are separate
unclaimed delivery stories; their real-browser evidence is required before #520 Request B can
claim production-native fulfillment.

### Scope

- Exact 21-operation v2 request and bounded typed output contract, opaque frame/snapshot/element
  identity, canonical origins and session/epoch invalidation.
- Opt-in host `BrowserExecutor` and `ManagedBrowserTool`; prepared one-shot invocation path with
  browser-specific, invocation-only permission across Agent and embedded Runtime.
- Provider raw-argument integrity for v2; standalone MCP must reject v2 until equivalent raw
  ingress proof exists. Other tools keep their existing paths.
- Shared deterministic host conformance fixtures, including approval/navigation and
  post-dispatch races, child-origin isolation, projection secrecy and no mutation replay.
- External host example and user/API migration documentation.

### Non-Goals

No default registration, Chromium/CDP/native plugin, process carrier, account or profile policy,
credential handling, selectors/script/evaluate, WEB-005/TOOL-014 change, V1 delivery, release or
publication. I293/I294 search work and their owner files remain outside this Work Slice.

### Acceptance

- A host can explicitly compose v2, discover a frame, obtain an exact authorization and execute
  one current-document action with a bounded typed projection.
- Cross-origin child action cannot inherit a parent or generic Network grant. Invalid schema,
  duplicate keys, stale/foreign refs, expired/reused/swapped tickets and lost lifecycle state
  cause zero unauthorized executor calls and zero mutation.
- Approval-time navigation fails before dispatch; post-dispatch navigation fails before wrong-
  document mutation; transport ambiguity never triggers replay.
- Same-origin, cross-origin, nested, delayed, duplicate, rebuilt and detached frame fixtures plus
  all mandatory cases in the contract pass against the host executor path.
- V2 remains absent by default and unavailable on standalone MCP until raw-ingress proof exists;
  V1 definition and legacy tool behavior remain unchanged.

### Planned Validation

- Focused protocol, provider, permission, Agent, Runtime, MCP and host fixture tests with `--locked`.
- `./scripts/release_preflight.sh` (includes locked workspace checks/tests).
- Both governance validators and `git diff --check`.
- Exact-head CI and independent permission/security/API review; merge-time CAS.
- External host example exercised with a deterministic bound executor. Native browser evidence
  is explicitly not claimed.

### Documentation To Update

- Embedded Runtime SDK and tool composition usage/API migration guide; browser host example.
- WEB-007-F owner, parent WEB-007 dependency note, Product Backlog, iterations README and Board.

### Risks And Rollback

The principal risks are pre-permission parser bypass, broad legacy grant reuse, stale-document
mutation and leaking child content into parent projections. Fail closed per route/operation.
Rollback is removal of explicit v2 composition; legacy/default inventories remain unchanged.

## Iteration Inventory And Parallel Disposition

At selection on 2026-10-06: I293 and I294 are Active under separate SEARCH-001-C/D claims and
continue unchanged; the maintainer explicitly authorized non-overlapping I295 parallel work and
owner-first union updates to shared derived views. I277, I290 and I291 stay Review with their
existing acceptance gates. I249 stays Planned and deferred. I164 remains Paused. No owner,
acceptance or implementation authority from those items transfers to I295.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-10-06 | Proposed | PR #669 proposes atomic claim and activation; both are ineffective until its finalized exact head merges to main. |
| 2026-10-08 | Activation verified | GitHub confirms #669 head c9bfe6e746282e9390d0ff52513057af005dff99, base 1a397cfa1cbbee4cdc435cb450980b0fa6a25011, merged at 2026-10-06T06:28:43Z as 10f3715a77c4b1215455e89d26252d6b8bfa48e3. This establishes claim, not implementation completion. |
| 2026-10-08 | Local convergence | Maintainer requests branch closure into main before handoff. Six unpublished local implementation commits were rebased onto origin/main 3eee0caa. Protocol vocabulary, scalar raw-request validation and ticket bookkeeping exist; no executor, permission or Runtime composition is delivered yet. |

### Closure Ledger

- Requested outcome: finish the published I295 host contract, merge verified implementation and synchronize owners before branch cleanup/handoff.
- Assets preserved: Published Baseline, ADR-086 boundary, legacy tools, other sessions' work and all local implementation changes.
- Required implementation: typed request/schema parity and normalization; trusted per-frame identity/origin state; exact invocation permission; prepared dispatch and host executor; bounded output; provider ingress and wrapper routing; deterministic conformance and external host example.
- Required evidence: focused and workspace locked checks, preflight, governance validators, exact-head CI and independent permission/security/API review, followed by merge-time CAS.
- State synchronization: I295 owner first, WEB-007-F second, derived views and Issue #618 after authoritative evidence. No Complete state until acceptance and existing implementation SHA are available.
- Residual destination: this owner until its published acceptance is fulfilled. Native browser/process carrier remain the published non-goals, not substitutes for incomplete host integration.

## Verification Evidence

Local pre-rebase head `2bbe082f` passed 15 focused `browser_executor` tests with
`cargo test -p talos-tools --features network --locked browser_executor` and focused Clippy with
`cargo clippy -p talos-tools --features network --locked -- -D warnings`. These test raw request
validation and reservation bookkeeping only; they do not establish permission, executor,
provider ingress, schema parity or runtime acceptance. Final candidate evidence remains pending.

Design-only Agent-role review approved the corrected proposal on
an uncommitted diff over `9712d458`; it is not exact-head code review or runtime evidence.

## Completion Evidence

### Local Protocol Checkpoint (2026-10-08, not delivery evidence)

The local candidate now includes typed command branches and a generated closed wire schema,
default normalization, and actual schema-validator comparisons for 21 branches plus abusive
scalar values. `jsonschema 0.58.6` was verified with `cargo search`/`cargo info` and added only as
a dev dependency with default features disabled (no HTTP/file reference retrieval). The lockfile
update records its transitive test dependencies; no browser/native transport is added.

Eighteen focused browser tests passed. Earlier in the same local cycle all 128 network-feature
tool tests passed; final full-candidate checks remain pending. Independent incremental pre-review
identified schema/URL mismatch, lexical integer handling and public structural-schema ambiguity.
URL credential rejection and integer normalization were corrected and covered by real schema
evaluation. Full URL semantic equivalence and common-layer API placement remain review items;
this checkpoint is not an implementation APPROVE or complete host conformance.

The broader `--all-targets` Clippy run also exposed test unwraps in the new browser modules;
these were corrected. An existing `useless_vec` finding in `search_backend.rs` belongs to the
search surface and must be reconciled before final workspace validation, without silently
overwriting concurrent search work.

Pending. A future owner-first closeout must cite an already-existing implementation SHA.

## Variance And Residuals

Local context follow-up: `BrowserContext` now owns bounded tab/frame state and its observation
ticket registry together. Parent/child origins are distinct, Pending/opaque observations reject,
and navigation/tab close/lost synchronization invalidate reservations. Checked reference counters
prevent within-domain reuse after eviction; UUID generation has panic containment. Twenty-five
focused browser tests and library Clippy passed. These are local, uncommitted checks, not final
candidate evidence. Observation preparation is not authorization or executor dispatch.

Independent Agent-role pre-review identified context/ticket synchronization, generation scope,
opaque-versus-unknown origin distinction, random-source containment and URL normalization as
integration risks. Unified ownership and invalidation are implemented; complete origin-state
variants, snapshot/element state, permission capabilities, executor and composition-root wiring
still require implementation and fresh review. No change to the Published Baseline is implied.

Native iframe operation, V1 base contract and standalone MCP raw-ingress support remain separate
gates. The owner cannot claim them delivered from host fake-executor tests.

### Local Admission Checkpoint (2026-10-08, uncommitted)

All 21 command branches now prepare an invocation with an operation-family typed target and
revalidate it against current host state. Session bootstrap needs no frame reference; unknown
origin evidence rejects navigation, while an explicitly host-verified opaque document can be
navigated but not observed. Element-targeted requests validate the exact snapshot/element tuple.
Session replacement revokes old references and outstanding tickets, including same-numbered
nonces in a new domain. Authorization challenges bind the canonical JSON digest and actual
ticket identity/epochs/expiry, rather than a caller-supplied nonce.

`cargo test -p talos-tools --features network --locked` passed 143 tests, including 32 browser
tests. `cargo clippy -p talos-tools --features network --locked -- -D warnings` failed on the
unused authorization-consumption field/methods: final dispatch is still absent. No lint
suppression was added. Formatting and `git diff --check` passed. A provisional executor API
was removed before publication because dispatch-time executor substitution and its output
types did not satisfy the contract. Required next work is fixed host/executor ownership,
complete output validation/projection and authorized one-shot dispatch, then the published
composition/ingress/conformance scope. This checkpoint is not stable-candidate or completion
evidence; Active/Claimed remains unchanged.

### Local Host And Raw-Ingress Checkpoint (2026-10-08, uncommitted)

The fixed host now owns its executor and serializes one-shot authorized dispatch. Backend
readiness precedes permission; final context validation, cancellation, deadline and panic
containment guard dispatch. Observation release revalidates document identity and rejects
foreign snapshot, URL-origin and inventory metadata. These host fixtures do not establish
native-browser conformance or completion of the mandatory isolation matrix.

Core now provides a bounded original-argument carrier that rejects duplicate decoded keys,
non-scalar fields and trailing JSON without exposing arguments through Debug. Host admission
accepts this carrier directly and repeats semantic validation; provider transport integration
is still absent. Reserializing a parsed Value remains explicitly insufficient ingress proof.

Ticket and registry ownership now share revocation state: dropping an invocation, discarding
it, clearing lifecycle reservations or destroying the host invalidates its old authorization
challenge immediately. Abandoned reservations are reclaimed on subsequent admission. Tests
exercise 512 abandoned requests without capacity exhaustion or executor execution.

Observed local checks: 50 browser tests passed using
`cargo test -p talos-tools --features network --locked --lib browser_executor`; focused library
Clippy passed with `-D warnings`; `git diff --check` passed. Core raw-argument test and four
I179 public-path/source-layout tests passed before the host-only follow-ups. Fetch confirmed
the branch contains origin/main (six local commits ahead, zero behind at inspection).

Remaining delivery work: common-layer prepared API and dedicated permission composition,
ManagedBrowserTool, Agent/Runtime/provider routing, standalone MCP refusal, transient artifact
ownership and confidentiality fixtures, complete conformance matrix, runnable external example
and migration documentation. Full workspace/preflight, exact-head CI, independent security/API
review, merge and owner-first closeout have not occurred. State remains Active / Claimed.

### Local Integration Review Checkpoint (2026-10-08, uncommitted)

Provider original-argument carriers, prepared Agent execution, explicit ManagedBrowserTool
composition, actual Runtime integration and standalone MCP refusal now exist locally. The
deterministic `i295_browser_host` example runs child discovery, one exact authorized action and
stale-reference rejection. Tools 166 unit tests, Agent 440 unit tests and two Runtime integration
tests passed during this local cycle. These supersede the earlier implementation-gap inventory
only where named; they do not establish full conformance.

Independent Agent-role inspection of the uncommitted candidate returned REQUEST CHANGES:
compat/strict original arguments, screenshot capability ownership/delivery, an externally usable
trusted lifecycle update handle, complete isolation/barrier fixtures, and typed failure outputs.
The reviewer shares the workspace/account and is not an independent human or final exact-head
approval. None of these gates is waived.

The compat/strict follow-up now extracts protected arguments from the original text block using
RawValue, rejecting duplicate routing fields and duplicate decoded argument keys. Both provider
adapters use the transient event route, preserving ordinary calls. Provider 137 unit tests and
library Clippy with `-D warnings` passed; actual SSE coverage for these new text paths and fresh
review remain necessary. Formatting and `git diff --check` passed. The independent Runtime SDK
fixture boundary check passed; it was check-only, not external browser execution evidence.

[Browser migration](../reference/I295-BROWSER-HOST-MIGRATION.md) records the explicit tools
dependency, default denial, cancellation and trusted-executor responsibilities. The screenshot,
lifecycle, typed failure and full conformance findings remain local implementation work, followed
by external host consumption, full preflight, stable candidate commit/push, exact-head CI/review,
merge and owner-first closeout. No implementation PR or completion evidence is claimed here.

### Local Lifecycle And Screenshot Checkpoint (2026-10-09, uncommitted)

The host lifecycle handle is lock-independent and wakes parked dispatches on invalidation;
per-poll and final release generation fences prevent stale results from being released. Trusted
context rebuilds replace the session and fail closed on concurrent invalidation or panic. The
artifact store validates actual PNG decoding and IHDR dimensions under bounded allocation,
enforces 120-second expiry, entry/byte limits, invocation-resource and generation ownership,
one-shot consumption, and failure/session reclamation. Host fixtures cover successful capture,
metadata mismatch, navigation during capture, replay, wrong generation, expiry, corruption and
capacity.

This remains a local checkpoint, not delivery evidence: the provider-neutral message model
currently carries path-backed images, while browser screenshots are transient in-memory bytes.
A new ephemeral image carrier must be added across Agent and both provider adapters without
allowing bytes, references or paths into durable messages, display, persistence or replay.
Native descendant pixel masking remains host evidence, not a fake-executor claim. Full
conformance, external consumer validation, preflight, exact-head CI/review, merge and
owner-first closeout remain pending.

### Local Candidate Verification (2026-10-09)

Candidate `4aa0f8779f933eb373b4416ee0d50490acfb0fdf` incorporates the ephemeral-image
carrier, request-budget and visual-capability admission, bounded provider delivery deadline,
child-document isolation fixtures and external host composition. It is rebased onto
`9a7b4b8cf014058c04d6b83fa5d9a35e7236ec03`; all nine I295 patches were preserved.
The Runtime SDK remains independently consumable with one direct Talos dependency; the
separate browser-host fixture explicitly imports Runtime and Tools. Native-browser evidence
remains excluded and must be supplied by a downstream real host implementation.

Local locked verification passed: 71 browser tests, two Runtime host integration tests,
the Provider suite, the Agent suite (447 unit tests plus integration and documentation tests),
the standalone MCP original-only refusal test, both external consumer executions and focused
six-crate all-target Clippy. The Agent source-contract test was corrected to check the actual
sealed ephemeral dispatch and carried protocol; runtime behavior was unchanged. The Runtime
SDK validation script also passed. Both governance validators reported zero warnings.

Independent Agent-role security/API review approved the exact candidate and base above with
no blocking findings. This reviewer shares the workspace/account and is not an independent
human. Its incremental review verified patch identity after rebase and the test-only correction.

The first full preflight failed during workspace test linking with ENOSPC; it is not passing
evidence. Generated build artifacts were cleaned. The complete rerun then passed with incremental
compilation disabled, debug symbols disabled and two build jobs; no tests or gates were removed.
Remote CI, merge-time CAS, implementation merge and owner-first closeout remain outstanding. No
completion is claimed.

### Owner-First Closeout (2026-10-09)

PR #683 merged to `main` as `06a5f6e6244f1b3dc9719033090f87ef485b1860` after all six exact-head
CI jobs passed and the merge-time CAS confirmed unchanged head `1385a7e72c1261596f6fd8b49a9535fc71fbb3b8`,
base `9a7b4b8cf014058c04d6b83fa5d9a35e7236ec03`, effective claim #669, no overlapping blocker,
and the required review evidence. The implementation acceptance matrix and local/remote validation
are complete for the published host contract.

Completion Commit: `06a5f6e6244f1b3dc9719033090f87ef485b1860`

This completion applies only to the opt-in host contract and deterministic conformance boundary.
No native Chromium/CDP browser or process carrier is claimed; that downstream work remains
explicitly residual.
