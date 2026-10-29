# Iteration I306: Protocol Architecture And OpenAI Responses

> Document status: Review
> Published plan date: 2026-10-22
> Planned objective: establish model-level wire-protocol resolution and deliver a complete OpenAI Responses adapter under ADR-093.
> Baseline rule: preserve this bounded child-A scope; Gemini, Azure or Bedrock use separate owners/iterations.
> MVP deliverable: one executable stateless `/v1/responses` agent round with text, images, native tools, reasoning summary, usage and protocol provenance.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 / talos开发 #724 continuation 2026-10-26 |
| Work Slice | PROVIDER-008-A protocol architecture plus OpenAI Responses only; no Gemini/Azure/Bedrock implementation. |
| Claimed At | 2026-10-22 |
| Source Issue | #748 |
| Governance Claim PR | #747 |
| Implementation PR | #749 (code), #775 (catalog data), #792 (model endpoint runtime), #793 (reviewed endpoint data), #799 (public Copilot runtime) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested implementation of parent #724 in the 2026-10-22 talos开发 session; child #748 records the bounded implementation requirement. Governance-only #747 must pass exact-head applicable CI, both governance validators, provider/API technical review and merge-time CAS before this claim is effective. |
| Last Updated | 2026-10-28 |
| Handoff / Release Condition | Activates only when #747 merges with accepted ADR-093. Complete only with existing implementation commit evidence; siblings B-D remain separate. |

## Published Baseline

Selected [PROVIDER-008-A](../backlog/active/PROVIDER-008-A-openai-responses-protocol.md), bounded
child of PROVIDER-008 / parent #724, with child requirement #748. ADR-093 is the governing
protocol/public-API decision and is accepted atomically in governance PR #747.

### Selection Inventory And Disposition

Selection main: `0caaf83241b8f94cba0cbdca66e3e166068f024d`.

- Parent #724 and all then-current comments re-read; parent PROVIDER-008 remains Refinement / Unclaimed.
- Child requirement #748 was created during claim preparation so the parent epic and child owner do not share one Source Issue.
- Before #747 was opened, no open PR existed for #724 / PROVIDER-008 / OpenAI Responses and no
  matching `724`/`provider` implementation branch was found.
- I293 and I294 retain Search work; I300 retains structured Git scope; I302 retains Windows resource
  attribution Review; I305 is Complete/Closed. None owns provider wire-protocol expansion.
- PROVIDER-007/I297 is Complete/Closed and changes retry defaults only; it is not reopened.
- MODEL-014 is Refinement / Unclaimed and remains separate. I306 may add protocol metadata required
  by #724/#748 but may not absorb general model capability/variant inference.
- Sibling PROVIDER-008-B/C/D are deliberately not claimed.

No different active/review/planned/blocked owner found with the same work slice. If target-branch
truth changes before merge, the merge-time CAS preflight must stop and reconcile rather than silently
steal scope.

### Scope

1. Accept ADR-093 and add `openai-responses` plus the public `#[non_exhaustive]` migration boundary.
2. Centralize protocol-to-adapter construction so future families do not multiply exhaustive factory
   matches across CLI/Desktop/discovery/model browser.
3. Add model-level protocol configuration, catalog hints and provenance using the precedence frozen
   in ADR-093.
4. Implement the dedicated stateless OpenAI Responses request and streaming adapter, including native
   tool calls/results, images, reasoning summaries/encrypted replay handling, usage and terminal/error
   semantics.
5. Expose effective protocol/source on the #724/#748 model/debug surfaces and document the confirmed-
   support default plus explicit `openai-chat` rollback.

### Non-Goals

No Gemini, Azure, Bedrock, Responses built-in tools, Realtime, Assistants, Batch, server-side state,
MODEL-011 probing, MODEL-014 general capability inference, permission/storage redesign, unrelated
retry-policy change or new external dependency unless separately decided and claimed.

### Acceptance

Acceptance is the PROVIDER-008-A owner acceptance in full. In particular:

- confirmed official OpenAI can select Responses only when protocol is unset and support is known;
- explicit chat and unconfirmed compatible gateways stay chat;
- model-level confirmed signals outrank provider defaults and provenance is visible;
- complete `/responses` text/image/tool/reasoning/usage round works without server session state;
- unknown stream events never panic and non-terminal EOF is an error;
- protocol rejection is actionable and never triggers cross-protocol retry;
- old configs round-trip and retain behavior except the documented confirmed-support default change.

### Planned Validation

Focused request/stream/config/catalog tests and proptest during local convergence; mock HTTP full-turn
coverage including tool call and image continuation; then pinned-toolchain locked workspace tests and
`./scripts/release_preflight.sh`. Exact candidate CI and independent API/provider review are required
before merge. Catalog regeneration, if needed, remains a separately reviewable stage/diff rather than
being hidden inside an unrelated code change.

### Documentation / Migration

Update public provider/config documentation, README en/zh examples, model UI/debug provenance and
release migration guidance. The public enum becomes non-exhaustive in the same pre-1.0 minor release;
downstream exhaustive match users must add a catch-all arm.

### Risks / Rollback

The visible risk is default-path change for confirmed-support models. Rollback is explicit
`protocol = "openai-chat"`; never silently retry a second wire protocol after a request begins.
Encrypted reasoning is confidential opaque data and must not enter readable logs/debug surfaces.
Revert the Responses/default-selection stage if protocol correctness cannot be demonstrated rather
than weakening terminal/error or provenance requirements.

## Activation Record

2026-10-22: #747 is the governance-only atomic claim/activation proposal for child #748 under parent
#724. The record in this branch is Active / Claimed and ADR-093 is Accepted, but none of those states
is effective until the finalized exact head merges to main. No production code, implementation tests,
generated catalog, dependency or config behavior changed in the governance candidate. Implementation
must start from the #747 merge or a later main commit.

## Completion / Residuals

No Completion Commit exists. PROVIDER-008-B/C/D and every excluded Responses feature remain residual
under the parent epic. Do not mark I306 or PROVIDER-008-A Complete without an already-existing
implementation/evidence commit SHA.

## Continuation Checkpoint — 2026-10-26

The maintainer requested handoff of the overlong #724 session and continued implementation in this
session. Responsible Actor and Work Slice are unchanged; this is continuation of effective #747,
not a new claim. #747 merged as `4722fee827c71b807bb58458a08d67abe32050e4`. Implementation carrier
is Draft #749, branch `feat/provider-008a-openai-responses`; inherited head was
`4bbae80a38e8b4c916460ba855d64d29b63cf1dc`. The earlier PR description and owner fields were stale.

The inherited code already contained Responses request, semantic/SSE transport, client, reasoning,
invocation bridge, public adapter and CLI/Desktop factory wiring. This checkpoint adds:

- one effective resolver for runtime, public Config queries, model selection and model metadata;
- omission-preserving model selection, credential insertion and credential reload (copying a built-in
  Chat default into user overrides had incorrectly made it explicit);
- Responses plus model-level protocol CLI edits, all three `/connect` choices, rendered cursor and
  clipping coverage, effective protocol/source in picker/browser and model context;
- readable-preview redaction of encrypted reasoning, retaining actual same-identity wire replay;
- real CLI configuration/dispatch regression tests and a controlled binary Agent image/tool round;
- proptest coverage of arbitrary byte partitioning, Unicode, CRLF and missing-terminal EOF;
- [protocol/config migration guidance](../reference/PROVIDER-PROTOCOLS.md) and bilingual README updates.

Test-only dependency decision: reuse the exact existing Cargo.lock proptest source/revision
`3dca198a8fef1b32e3a66f1e1897c955b4dc5b5b` for the already-published mandatory property-test gate.
There is no new runtime dependency or dependency version update. The single lockfile addition is
Talos Provider's direct dev-dependency edge.

Validation is in progress using pinned Rust 1.99.0 and locked dependencies. Initial focused testing
exposed three session-reasoning assertions tied to Chat's JSON location; their shared helper now
checks both adapter shapes without weakening the effort assertion. The new three-choice layout also
required updating the existing rendered/clipped cursor expectation. A first local property-test
compile failed because the pinned `prop_assert!` macro interpreted braces in a stringified
`matches!` expression; explicit diagnostic messages corrected the test macro call.

### Remaining Acceptance / Resume Gate

Keep I306 and PROVIDER-008-A Active/Claimed, #748 open and #749 Draft. No completion is claimed.
Before Ready/merge: finish pinned locked workspace/preflight validation; review and regenerate the
packaged catalog as a distinct generated-data stage (the inherited catalog has no model-level
protocol signals); obtain current-head CI and independent provider/API review; refresh current main,
resolve branch conflicts within this slice and repeat merge-time CAS. Recorded upstream stream
fixtures and the full acceptance matrix must be audited explicitly rather than inferred from
synthetic fixtures. Gemini/Azure/Bedrock, #705 and Search remain outside this work.

### Local Validation Record — Published Source c6db8b88f31e449f1a6e138b672e335113800984

Integrated main: `b21b6146c21894dc42e8fc5bf1d815af71fbc7c8`. Local merge/checkpoints were
`5569e45b`, `28bac9f3` and `15b6529e`; these are local provenance, not remote completion commits.
Published source `c6db8b88f31e449f1a6e138b672e335113800984` has the exact verified local source tree
`119ad3b2aeead9ff7c792ddf34f458be3c2b466a`, with the previous #749 head and integrated main as
parents. GitHub Git-object publication was used because ordinary Git push had no local write
credential; the returned remote tree SHA was checked against the local tree before branch update.
Main's Sandbox owner/iteration won the
unrelated add/add conflicts; both sets of Evolution lessons were retained. No Sandbox, permission,
native Git or MCP implementation change is authored by this slice.

Pinned Rust 1.99.0; all Cargo verification uses locked dependencies. Local build/test debug info is
zero, incremental compilation disabled, four build jobs; these settings do not change test filters.

- Final affected-package run: `cargo test --locked -p talos-config -p talos-provider -p talos-cli
  -p talos-tui` — **1,651 passed across 43 test/doc-test result groups; zero failed, ignored or
  filtered**. Includes the real binary Responses configuration/rollback round, dotted model IDs,
  the two-round Agent image/native-read scenario, Unicode/CRLF/EOF property tests, ciphertext-preview
  regression and forward/reverse rendered wizard/cursor tests.
- `cargo fmt --all -- --check`, workspace check and final
  `cargo clippy --workspace --locked -- -D warnings` passed.
- Both governance validators passed again with zero warnings after owner-evidence synchronization;
  project-scale assessment completed. Subsequent publication mapping changes only evidence prose.
- `python3 scripts/validate_runtime_sdk_fixture.py` passed in both default and coding modes,
  with its independently resolved locked dependency graph and sole direct Talos dependency.

Full-preflight history is retained rather than converted into a green claim:

1. Before main integration, standard `release_preflight.sh` passed its static, governance, formatting,
   workspace check and strict Clippy stages, then failed the unchanged runtime Unix-socket fixture
   `runtime_evidence_rejects_directory_symlinks_and_socket_entries` with OS error 1 / Operation not
   permitted (runtime group 72 passed / 1 failed). Current execution policy disallows Unix socket
   creation. No source test was ignored or weakened.
2. After main integration, standard preflight again passed its non-test gates, then stopped in the
   inherited MCP `git_prepared` group: 26 passed / 21 failed. Two cases fail creating Unix sockets;
   approved native Git cases report `Git maintenance executable differs from the bound Git binary`.
   The local Git 2.51.1 layout has hard-linked executables at different canonical paths, triggering
   the unchanged main guard. This record is not a closure of that separately owned native/security
   behavior, and does not claim a green main baseline.
3. A supplemental workspace run filtering only the single known runtime socket fixture also stopped
   at the same MCP group. It did **not** prove that all remaining workspace tests pass; no broader
   filtering was introduced to manufacture a green full-suite result.

The first controlled Agent fixture inherited the repository process cwd and therefore read the
wrong relative location. Running the child in its isolated workspace corrected the fixture; the
strict read-result assertion and production permissions remain unchanged.

Unfiltered current-head CI and independent provider/API review remain mandatory. Do not mark this
stage Ready, merged, Complete or released from the local affected-package results alone.

### CI3448 Fixture Correction — 2026-10-26

Run `40300248919` / CI3448 on `ee68851dd72ac436eee2b4ade3c4009848b6f865` failed the macOS
`Format + Check + Clippy + Test` job (`121496129738`) in
`openai_responses_http::mock_http_accepts_adr051_multimodal_continuation_without_path_leak`.
The tool-call HTTP case passed; the image case returned `ServerError("")` because the mock did not
match a request containing an image. This failure is inside #748, distinct from the earlier local
Unix-socket/native-Git blockers. Linux Desktop and remote owner reconciliation passed; the Windows
workspace job was still running when this correction began, so no complete CI3448 pass is claimed.

The fixture put its lexical temporary-file path into a durable Image part, although ADR-050/051
requires the grant-time canonical path. The existing image guard re-canonicalizes and omits an image
if the stored path differs. macOS temporary-directory aliases expose that fixture error. A local
Unix file-symlink reproduction confirmed the exact failure without changing production code:
removing only fixture canonicalization gives the same `ServerError`, and restoring it passes.

The correction canonicalizes the existing fixture after file creation and before constructing the
message. Unix runs deliberately use a file alias first so Linux exercises the same precondition.
Filename leakage checks cover both canonical and alias filenames, including JSON-escaped Windows
paths. Assertions still require image bytes, the streamed answer, a semantic terminal event and the
expected HTTP request. No guard, retry, permission, production image or CI policy is relaxed.

Focused validation: both HTTP fixtures passed with the fix; red/green alias reproduction
failed/passed as expected; all nine selected image-guard regressions passed; focused strict Clippy
(`cargo clippy --locked -p talos-provider --test openai_responses_http -- -D warnings`) passed.
Format and both governance validators are checked before the stable correction is published. CI3448's failed first attempt is preserved; a new head requires fresh
unfiltered CI. All remaining I306 acceptance gates and Draft/Active/Claimed state are unchanged.

### Rebase And Catalog Admission Stage — 2026-10-26

The maintainer requested rebase and continued implementation. Old head `5acd2161c184daa1ce9791bc49a2b9223e61bbd4`
is retained on `archive/provider-008a-pre-rebase-20261026`. The final implementation diff was consolidated
and rebased onto `main@2789b97bfe807f486778f3b44c7053e2a0e6e13d` as local `817ec6f6` without conflicts.
This preserves main's I308 scheduler/native-policy work and I309 activation, whose gix implementation
remains a separate owner. The claim/actor/Work Slice are unchanged. CI3451's macOS preflight/Linux
checks passed, but Windows was cancelled during workspace tests; prior validation is historical and
fresh current-head CI is required after this rebase/data stage.

A protocol-only catalog candidate has been prepared and tested with 4,016 optional protocol fields
for the existing 6,474 models, including 119 Responses signals. It preserves all provider roots and
model identity/limits/pricing/capabilities/variants. Per parent #724, generated data is a separate
review PR based on #749; this code PR still carries the older packaged catalog and does not claim
that its new protocol signals are active. The data carrier records the upstream snapshot hash,
counts, withheld rows and Responses signal ledger. Distinct model-endpoint routing remains an
explicit acceptance residual; protocol inference alone does not implement it.

The runtime and generator now share one wire-signal map. The generator admits only implemented
host/model transports using a compatible API root; the runtime suppresses built-in catalog inference
on a replaced endpoint. The centralized factory completes a model-level Anthropic gateway root to
`/messages` exactly once. This does not implement a new protocol family, authorization, SDK transport,
endpoint probing, MODEL-014 capability inference, or a vendor-authentication wrapper.

Verification for this rebased stage is in progress: catalog-build boundary tests, packaged-catalog
and explicit-rollback matrix, affected packages, standard preflight and current-head CI. No Complete,
Ready-for-Review, merge or release state is claimed. Recorded-stream fixture audit, distinct endpoint
routing/residual disposition and independent provider/API review remain open acceptance rows.

#### Rebased Code Validation Checkpoint

Frozen code candidate: local `4187e823232b4cb69f8bb952c71a08727d9ee833`, parented on the rebased
implementation above. The shared `CatalogProviderProtocol` mirror now also supports Responses and
is non-exhaustive; the migration note applies to both public protocol enums in a minor release.

- Pinned/locked core, config, provider, CLI and TUI tests: **1,787 passed in 50 result groups**, zero
  failures, ignores or filters, against the older packaged catalog carried by the code PR.
- The paired data candidate was also exercised during convergence; its isolated final run and four
  snapshot-specific acceptance cases are recorded in its own PR.
- Frozen standard preflight passed formatting, workspace check, strict Clippy and both governance
  validators, then failed the inherited MCP `git_prepared` group at **26 passed / 21 failed** due
  to the previously recorded Unix-socket and native Git maintenance binding conditions. No full
  workspace/preflight success is claimed and no native/security guard was changed.
- Independently resolved external SDK fixture passed default and coding modes with locked dependencies.
- A preliminary preflight began before the catalog-enum edits finished and read inconsistent source
  states, yielding E0599. It is not candidate evidence. Frozen compilation and affected tests above
  supersede that local convergence attempt.
- The first separate-worktree data run reused the code worktree's target directory, causing E0463 /
  colliding StableCrateId metadata. It is not a product failure or acceptance evidence. Data now
  uses its own target directory; code-local package artifacts were cleaned and the SDK rebuilt.

Code and data publication verifies complete Git tree identity, and rewriting #749 uses the old
`5acd2161` head as a force-with-lease precondition. Its old history remains in the archive branch.
Neither code nor data is ready for merge until fresh unfiltered CI and the owner-defined API,
fixture and endpoint residual gates are dispositioned. No sibling or I309 implementation is claimed.

### CI And Stream Lifecycle Review — 2026-10-26

Published code `7fd35dcf1e66317a7523f5cfdf64168079091207` passed CI3457
(run 40326364387); stacked catalog `bb705cc0e53ab07791581f5915a6fd3f3ce75ebf`
passed CI3458 (run 40327420437). Each run has six successful jobs, including macOS
standard release preflight, Windows workspace tests and Linux Desktop validation. This is
current-source machine evidence for those exact heads; earlier local MCP/TUI failures remain
recorded observations with no inferred root cause. A substantive correction invalidates use of
those runs as exact-head acceptance for a new candidate.

Source audit found a stream lifecycle defect: semantic completion/failure did not close the HTTP
body reader. A server retaining the connection could keep the provider channel alive and append
an idle timeout error after a successful terminal; a trailing malformed SSE record could similarly
append a transport error. The correction stops transport processing at the first semantic terminal
or decoder failure, sends that batch in order, and releases the HTTP body immediately. EOF without
a terminal and malformed input before a terminal continue to fail.

Fixture audit: the existing Responses suites use inline synthetic events/mock HTTP responses;
there is no checked-in captured upstream Responses SSE transcript. `openai/openai-python` at
`5d72315876ffd8b72b958623ef4db26da3fb1e61` was inspected: the live-API snapshot in
`tests/lib/responses/test_responses.py::test_output_text` is a non-stream response, and its
WebSocket scenario fixtures do not establish a captured SSE transcript for this acceptance row.
No upstream capture is invented or relabeled. Existing single-call, semantic text/summary/usage,
Unicode/CRLF/chunk/EOF property tests and mock binary rounds remain useful but do not substitute
for recorded upstream parallel-tool/reasoning/error streams. Recorded-stream acceptance and
independent provider/API review remain open. Distinct model-endpoint routing remains withheld,
not implemented by catalog admission. I306/child A stay Active/Claimed and both PRs stay Draft.

#### Terminal Lifecycle Validation

Pinned Rust 1.99.0 and locked dependencies; four build jobs, zero debug info, incremental disabled.
Before the production correction, the HTTP regression failed with 2 passed / 1 failed (semantic
terminal left the channel open); the decoder regression failed with 24 passed / 1 failed (two
outputs instead of one). With the correction, `cargo test --locked -p talos-provider` passed
**347 tests across 15 result groups**, zero failed/ignored/filtered, including both regression
loops' completed and failed outcomes. The strict HTTP-target Clippy check also passed.

Standard `release_preflight.sh` passed static/installer checks, both governance validators, format,
workspace check and strict workspace Clippy. Real binary `responses_agent` (1 test) and
`responses_config` (2 tests) passed in the unfiltered workspace run. Preflight then stopped in the
inherited MCP `git_prepared` group at **26 passed / 21 failed**, again reporting local Unix-socket
creation denial and `Git maintenance executable differs from the bound Git binary`. No full local
workspace/preflight pass is claimed; no native/security implementation, policy or assertion changed.

Official `openai/openai-node` source was also inspected at
`ac0867f7ecfab8afbc249b14c75d9df6f9a1d943`. Its `ResponseStream.test.ts.snap` contains SSE
examples behind an API-snapshot helper, but the checked-in fixtures carry test IDs and do not
establish real-call provenance for all required paths. They are not admitted as captured upstream
acceptance evidence. No upstream code or fixture was copied.

A final target refresh found `main@7ca67d9ecd2aa89ef3c66293ef4adebd01c022d0` after #767. The
post-2789 diff changes only Search's `search_backend.rs` and `web_search.rs`, outside this slice.
A Git merge-tree check is clean. The validation baseline remains 2789 plus this candidate; no
validation of an integrated 7ca67d9e tree is inferred. Repeat merge-time CAS and fresh exact-head CI
before merge.

## Independent Review Corrections — 2026-10-27

Independent read-only Agent review of remote bb31d012 requested changes for missing true
EphemeralImage dispatch, rejected explicit textual tool modes, and missing recorded/parallel streams.
The existing --attach binary test was user-history image replay, not ADR-051 tool image continuation.
The stable local correction stage starts from that head and integrates main 33fad1b8 (including
merged fixture/CI maintenance #778/#777 and independently owned I308/Search work); main work is
retained without taking over those scopes.

- Responses now implements the actual ephemeral trait method, shares image bounds/expiry encoding,
  applies a deadline over dispatch and stalled error-body reads, and disables automatic reposting
  of the image-bearing request under ADR-051. A real CLI read_image → read → final round requires
  exactly one image only in the second request. Separate 503 no-repost and error-body expiry tests
  exercise the request-local carrier rather than durable attachments. Existing Chat/Anthropic
  resource-format branches retain their behavior; their retry behavior is not modified here.
- Compat/TalosStrict use the existing text projection/parser and browser invocation evidence path.
  A previously dropped ProviderConfig.tool_protocol is now retained in the merged runtime view.
  Both explicit modes complete actual CLI read-tool rounds with textual result replay and no native
  tool schema. Native remains the automatic capability choice; unresolved Auto still fails.
- A clearly labeled synthetic fixed SSE stresses two interleaved Unicode calls completed in reverse
  order. Vercel AI SDK persisted reasoning/tool/text/usage and error streams are additionally copied
  byte-for-byte from e59b6e017aece6cc45c4ee7d94af88829621ed8d, with full hashes and retained license
  notice in tests/fixtures/README.md. The harness follows upstream JSONL framing and separates four
  completed turns. This is sourced SDK recording evidence, not a Talos live endpoint call.

Independent WIP re-review approves these implementation corrections and fixture provenance; the
original three blockers are resolved at working-source level. It is not final-head CI/approval or
human authorization. The original single-call/attachment-only gaps, first factory capability
expectation failure and binary text-mode failures before fixing configuration merge remain
historical failures. No assertion or permission guard was removed to conceal them.

Locked verification and final publication mapping are recorded below after source freeze. Keep
Active/Claimed and Draft until fresh complete CI and exact-head review. Distinct model endpoints
remain withheld under the documented residual; Gemini/Azure/Bedrock and MODEL-014 remain separate.

### Corrected Candidate Frozen Validation

Pinned Rust 1.99.0, locked dependencies, four build jobs, zero debug info and incremental disabled.
Final five-package core/config/provider/CLI/TUI run: **1,799 passed across 50 result groups**, zero
failed/ignored/filtered, exit 0. Includes all four actual CLI Agent scenarios, both text modes,
503 no-repost, Responses expiry, interleaved synthetic stream and two sourced SDK replay cases.
The textual stream regression also checks max_output_tokens remains MaxTokens in either text mode.
Strict HTTP-target and CLI-Agent-target Clippy passed after correcting diagnostic unwraps in the
new test code; format/diff checks passed. Complete external runtime SDK validation passed both
independently resolved default and coding modes with locked dependencies.

Frozen standard release_preflight passed static/installer checks, both governance validators,
format, workspace check and strict workspace Clippy, then stopped at inherited MCP git_prepared
with **26 passed / 21 failed**, exit 101, under the recorded socket creation/native Git binding
conditions. This is not a green local workspace/preflight claim. Earlier failed factory capability
expectation and first two explicit-text binary rounds are preserved as convergence failures; the
latter diagnosed the real merged-config field omission rather than an adapter parser failure.

Independent read-only working-source review now APPROVE for all three original blockers, including
upstream fixture hashes/provenance/license. The additional recorded tests assert aggregated summary
and visible text directly without printing encrypted payloads. This working-source conclusion is
not final-head CI/merge approval. The source correction is published as one stable stage after
complete tree verification; new head and relevant data baseline require fresh CI/review. Neither
I306 nor #748 is Complete, and no release is claimed.


### Current Main Integration Checkpoint — 2026-10-27

Code head `17a3e64a` completed CI3479 / run40455281573 with all eight jobs successful,
including full macOS preflight, native Windows workspace/Desktop and Linux Desktop. Its independent
exact-head Agent technical review APPROVE closed the three original blockers. Data head `3aeee187`
also completed CI3480, but a separate data review required archived source input. The source recovery
stage is retained in the data PR ledger and does not relabel prior source evidence.

Merge-time inspection found main advanced to `da280c8a77b25798e59df37f3bdc2bf2fafce1c9`, including
I308 invocation authorization/recovery changes affecting actual Agent tool execution. Integrate that
already-reviewed main before the next stable candidate rather than infer current-base behavior from
CI3479. Preserve main's permission/security implementation without authoring changes in that scope.
The only textual merge conflict was appended EVOLUTION lessons; both sides are retained. Actor,
claim and Work Slice are unchanged. Fresh exact-head CI and independent relevant-base review apply.
No completion, sibling implementation, release or success of interrupted local validation is claimed.


## Merged Implementation Review — 2026-10-28

The [Story owner](../backlog/active/PROVIDER-008-A-openai-responses-protocol.md#merged-implementation--acceptance-review--2026-10-28)
now records Review / Claimed, existing implementation merges and the acceptance residual.
Code #749 merged at `60e27eb6653b3a8fef638a2b9a7294a036b78db0`; catalog #775 merged at
`6af8654c6ee6e422d4d3f4e5dad171c78604cafd`. Completion Commit: Pending.
Earlier Draft/Active and unmerged-data checkpoints are retained as history.

Exact code `1388874d` CI3498/run40510753909 and data `76647990` CI3509/run40537545803 each
passed all eight jobs, including native full workspace/preflight and both governance validators.
Independent read-only Agent technical review APPROVE binds the exact code/data heads, final trees,
source audit and relevant main integration; both carrier PRs record Single-maintainer authorization
and final CAS. This does not claim a distinct natural-person approval or a new local full-test run.
The prior local 1,799/50 and 1,803/51 successes stay attributed to their recorded heads;
inherited MCP 26/21 failure and interrupted/lost attempts are not relabeled.

Real binary runtime evidence is `crates/talos-cli/tests/responses_agent.rs`:
`responses_image_and_native_read_complete_through_real_agent`,
`responses_tool_image_is_sent_once_through_real_agent`,
`responses_compat_tool_round_through_real_agent` and
`responses_strict_tool_round_through_real_agent`. CI runs these unfiltered with the workspace.
They cover attachment/native tools, one-shot read_image continuation and both text tool modes.
The recorded fixture corpus is sourced SDK recording evidence plus synthetic parallel-stream tests,
not a live Talos endpoint capture. The data audit checks all 6,474 records and the frozen source hash;
4,010 optional protocol hints preserve every non-protocol field.

Resume gate: resolve the unimplemented distinct model-endpoint routing / multi-protocol-host coverage
residual (56 withheld catalog records) through implementation or an explicit accepted scope decision.
Keeping it under parent #724 is tracking, not acceptance or a scope waiver. Recheck owner/claim,
non-overlap and applicable validation before implementation; do not start Gemini/Azure/Bedrock or
MODEL-014 under this claim. Final owner-first completion and #748 closure remain pending.
No release or parent #724 completion is claimed.


## HTTP Model Endpoint Correction Stage — 2026-10-28

The maintainer requested continued implementation of the remaining routing residual under the same
effective #747 / #748 / I306 Work Slice and Responsible Actor. This stage starts from main
`e03a2984267eacbd4f65d0085be1d028a8c420ed`, preserving inherited I308/I293 work. It adds explicit
`ModelConfig.base_url`, CLI get/set and effective endpoint/source resolution in the shared adapter
factory. Model connection overrides provider connection. Catalog endpoints require an exact built-in
provider/model, the unchanged catalog provider root and a matching effective wire; custom-provider
unique model matching never authorizes moving its credentials to the catalog vendor endpoint.
An explicit different wire suppresses the catalog wire's connection. Protocol inference is suppressed
when a model supplies an unconfirmed connection, just as for a replaced provider root.

The generator preserves supported HTTP model roots with their protocol. HTTPS or loopback HTTP is
validated by the existing locked `url` 2.5.8 parser (one additional talos-config dependency edge, no
crate/version change). Userinfo, query/fragment, malformed roots and remote cleartext HTTP are denied.
Environment templates remain literal at generation/config persistence and are validated after actual
runtime expansion. Missing environment variables fail before an HTTP request is built. Provider
construction errors propagate through CLI/RPC/TUI rather than panicking; failed TUI preparation must
preserve the old config, session and runtime generation. Desktop already returns construction errors.

The frozen source's 56 endpoint-different records comprise 52 supported HTTP candidates (14 Responses,
34 Messages, 4 Chat), including four Freemodel cross-origin defaults; four Alibaba records are WSS
Realtime transports, an existing explicit non-goal, not HTTP Chat endpoints. Endpoint and credential
destination metadata requires a separately reviewed data projection. This code stage does not change
`models.toml`, activate those rows, or claim runtime SDK authentication wrappers. The subsequent data
stage must prove compressed packaged catalog -> resolver -> actual adapter/binary routing, including
Neon environment roots, Freemodel origin changes, replaced-root suppression, custom-name non-relocation
and explicit wire rollback. Explicit model-URL tests alone do not close that automatic-catalog gate.

Independent WIP review found two blockers (userinfo loopback bypass and CLI construction panic);
source corrections and regression coverage were read-only re-reviewed without a new blocking finding.
Final exact-head object review, unfiltered affected checks, standard preflight and fresh CI remain
separate gates recorded in the carrier PR. One local CLI link attempt failed with an object-file
section-header offset beyond EOF, exit 101; no source root cause is asserted. Its log is retained,
and only generated target artifacts were cleaned. The complete affected-package rerun uses fixed
`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`;
these flags change build artifacts, not test selection or assertions. No unfinished run is a pass.
Status stays Review / Claimed, Completion Commit: Pending; data and final acceptance remain open.
No sibling implementation, scope waiver or release is claimed.


## Reviewed Endpoint Data Candidate — 2026-10-28

Code carrier #792 (`f43cd990`) and the separately reviewed endpoint data stage continue the same
#747 claim. The frozen 2026-10-27 source admits 45 supported, vendor-reviewed HTTP endpoint pairs:
Neon 14 Responses, Ofox 10 Messages, Zenmux 17 Messages and Freemodel 4 Chat. The data candidate adds
only paired model `protocol` / `api_base_url` metadata and validates original non-routing identity,
prices, limits, capabilities, variants and provider records with the original semantic digest.
Total candidate hints are 4,055 (3,857 Chat / 65 Messages / 133 Responses).

The data activation stage also keeps a reviewed model's original vendor root on wire rollback when
no provider URL was persisted; it cannot leak that provider credential to an unrelated adapter default.
This exact-model safeguard does not change custom gateways or older records without API metadata.

Primary vendor root/authentication evidence, exact destination tuples, all 45 rows, hashes and actual
compressed-catalog-to-binary tests are recorded in
[the endpoint ledger](../reference/PROVIDER-008-A-CATALOG-PROTOCOLS.md#reviewed-http-endpoint-projection-candidate--2026-10-28).
Code-first merge and fresh exact-head data validation/review/CAS remain required; candidate data is
not represented as activated before merge. Seven Copilot Messages endpoints remain unreviewed for
vendor authentication/transport compatibility. Four WSS endpoints are the already-excluded Realtime
family, not HTTP Chat. Neither distinction is a scope waiver or implementation claim.
Keep Review / Claimed with Completion Commit: Pending until the remaining acceptance residual is
resolved through implementation/confirmation or an explicit accepted scope decision. #724/#748 stay
open; no sibling implementation or release is claimed.


### Code-first Main Integration — 2026-10-28

Code PR #792 merged as `e53c61c7f0344308a5c5535bb1521fefebbacc70` after exact-head
CI3524 / run40628003993 passed all eight jobs and independent read-only Agent review approved.
The endpoint data branch integrates that actual merged main before first publication. The additional
main delta is #788's two Search backend files; no catalog/source regeneration or claim transfer is
included. Final combined-tree test, standard preflight, independent object review and fresh data-head
CI evidence are recorded in the data PR. Earlier failed local checks remain retained evidence.
Status stays Review / Claimed, Completion Commit: Pending; seven unreviewed Copilot endpoints remain
an acceptance residual. #724/#748 stay open, with no completion or release claim.


### Reviewed Endpoint Data Merged — 2026-10-28

Runtime #792 merged as `e53c61c7f0344308a5c5535bb1521fefebbacc70`; separately reviewed
endpoint data #793 merged as `8e3171534367ec242e02fdc1a90e012d533f8e96`.
The data merge preserves exact implementation head `9bbdc51c84bfb501748d66af3609f47cd6c2e77f`,
tree `dcce9b23073a798d6de33a45f413c88c11f3bf95`, with the actual merged runtime as its other parent.
CI3527 / run40633668603 passed all eight jobs on that data head, including native Windows workspace,
macOS standard release preflight, language plugins, offline dependency audit, all three Desktop
platforms and governance. The independent read-only Agent approved the exact object and independently
checked admission/source/non-routing evidence. Single-maintainer authorization and live CAS passed;
shared-account Agent technical review is not distinct-human approval.

The final combined-tree local config/provider/CLI/TUI run passed 1,694 tests in 46 groups, with zero
failed, ignored or filtered cases. Standard local preflight passed its non-test gates and both
governance validators with zero warnings, then retained the inherited MCP git_prepared result of
26 passed / 21 failed (19 Git-binary binding failures and two OS1 socket failures). An earlier local
link failure and stale diagnostic were also disclosed; no full local preflight pass is claimed.
After independent log verification, an environment restore lost those temporary scratch logs.
Observed results remain recorded in #793; native CI is the durable exact-head platform evidence.
The restored workspace was fetched from the exact head and its offline audit rerun successfully:
6,474 model records, 45 reviewed endpoint pairs, zero mismatches, unchanged source/non-routing hashes.
No new upstream download or broader model regeneration occurred.

The 45 reviewed HTTP pairs are now merged (Neon 14 Responses, Ofox 10 Messages, Zenmux 17 Messages,
Freemodel four Chat); total hints remain 4,055. Seven Copilot Messages endpoints still require vendor
authentication/transport confirmation or implementation. Four WSS rows remain the existing excluded
Realtime family. Neither parent tracking nor withheld data constitutes an acceptance waiver.
Status stays Review / Claimed; Completion Commit: Pending. #724/#748 remain open until the remaining
acceptance residual is resolved by implementation/confirmation or an explicit accepted scope decision.
No sibling implementation, tag or release is claimed.


## Public Copilot Static HTTP Follow-up — 2026-10-28

Continue the same effective #747 Work Slice to resolve the seven public Copilot connection residuals.
New pinned primary evidence confirms a static Bearer/native HTTP path; ADR-093 section 9 bounds the
implementation to the public origin and existing three wire adapters. SDK/OAuth/token exchange or
refresh, enterprise routing and sibling protocol implementations remain excluded. No new claim or
iteration is activated.

The runtime stage must prove actual factory-selected HTTP headers/body for Messages, Chat rollback
and Responses, complete credential masking, origin/path refusal, no redirect or cross-wire replay,
authentication errors and bounded native Agent image/tool round behavior. Model catalog activation is
a separately reviewed data stage using the unchanged archived source. At this checkpoint the seven
rows are still withheld and 4,055 hints / 45 endpoint records remain the baseline. Review / Claimed;
Completion Commit: Pending. No implementation merge, completion, live vendor probe or release is
claimed for this follow-up before its validation and review gates finish.

### Recovery checkpoint — 2026-10-28

A workspace restore reverted local HEAD to 8b8c43e and removed later local Git objects,
the Rust toolchain and temporary logs. Previously observed implementation 67c6a6e passed
unfiltered config/provider/CLI/TUI tests and independent read-only source review; standard
preflight passed check/Clippy/governance then failed MCP git_prepared (26 passed, 21 failed,
Git binding and OS1 socket failures). PATH reordering on the existing test executable reproduced
those counts. These historical results do not certify the restored tree.

Recovery reinstates sensitive Bearer builders, exact-token refusal-body masking and original
endpoint validation with model precedence/template expansion, plus real-startup CLI and factory
regressions. Reconstructed tests require fresh validation and exact-object review. The previous
local commits bcafbdb/a46e273 and their evidence remain unavailable as objects; no remote PR was
published for this follow-up. Seven catalog rows remain withheld; Completion Commit: Pending.
The maintainer's goal is closure of parent #724: finish A, then separately claim/implement/validate
Gemini B, Azure C and Bedrock D before parent acceptance. No sibling claim is yet effective.

### Rebase checkpoint — 2026-10-28

At the maintainer's request, the recovered Copilot work was rebased without conflicts onto
main `5d7d2fcca28379301c05ab712d33518667cbc8fa`, including #791's separately owned routed
search seam. All local changes were preserved through a named stash and restored. Formatting
validation and the frozen 6,474-model projection audit passed (zero mismatches; 45 endpoint
records; seven Copilot rows still withheld). Standard preflight has been started on the combined
tree; no terminal success is claimed. Recovery branch
`recovery/provider008a-copilot-20261028` at `fd104264ccd07aa1d956b896e09c9616113e9b6e`
retains the pre-format reconstruction, not a validated merge candidate.

### Public Copilot native validation — 2026-10-28

Runtime PR #799 head `de30465cd280d8a4c4bbbf973e2f41bdf6064217`, tree
`906b30f81c40fa6d453306241eb716664a5e8c7d`, based on main
`5d7d2fcca28379301c05ab712d33518667cbc8fa`, passed all eight jobs in native
CI3549 / run40705093059, including full Unix workspace validation and native Windows
workspace/Desktop checks. Independent read-only Agent source review APPROVE binds this tree/base;
this is not distinct-human approval. The local standard preflight passed check/Clippy/governance
then failed MCP git_prepared (26 passed / 21 failed, Git binding and denied socket fixtures).
The environment restored an old workspace snapshot and removed the final local four-package log;
its unseen terminal result is not counted as passing. Exact-head native CI provides durable full
workspace evidence. This documentation backfill changes no runtime or catalog data. Final review,
applicable CI and live CAS are still required before merge. Seven catalog rows remain withheld;
Review / Claimed and Completion Commit: Pending remain unchanged. No release is claimed.

### Copilot runtime merged / data convergence — 2026-10-29

Runtime #799 merged as `5ad8cc7ab606f762cb5bb94650e855c9aa456303` after exact-head
CI3552/run40713312471 attempt 2 passed all eight jobs, independent object review APPROVE,
and live CAS under the Single-maintainer path. Attempt 1's Desktop Auto watchdog timeout
remains recorded in the PR. The data work starts from this actual merged main.

The local separate data stage admits exactly seven existing github-copilot Messages records
from the unchanged frozen source. Offline exhaustive audit passes: 6474 models, 52 endpoint
pairs, 4062 protocol hints (3857 Chat / 72 Messages / 133 Responses), zero mismatches and
unchanged non-routing digest acc90a95ff8f87fbf131788548066575d02cd8e1ccb8f3545a9c62e01712947d.
Runtime automatic selection/rollback/isolation tests, catalog ledger synchronization, full
validation, independent data review and native CI remain pending. This local audit alone is
not data delivery or acceptance. Review / Claimed; Completion Commit: Pending. Parent #724
still requires independently claimed Gemini, Azure and Bedrock implementations.

Data validation recovery: the environment retained the source edits but removed the toolchain
and temporary test logs. A fresh standard preflight invocation stopped at cargo-not-found (127);
no local Rust test success is claimed for this data candidate. The offline exhaustive audit was
rerun successfully. Independent review found stale 65/45 snapshot counts and an old Copilot
non-activation assertion; these were updated to 72/52 and catalog-inferred Messages/Bearer,
retaining the separate rollback and gateway-isolation checks. Fresh native CI remains a merge gate.
