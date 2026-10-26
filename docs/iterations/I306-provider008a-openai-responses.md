# Iteration I306: Protocol Architecture And OpenAI Responses

> Document status: Active
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
| Implementation PR | #749 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested implementation of parent #724 in the 2026-10-22 talos开发 session; child #748 records the bounded implementation requirement. Governance-only #747 must pass exact-head applicable CI, both governance validators, provider/API technical review and merge-time CAS before this claim is effective. |
| Last Updated | 2026-10-26 |
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
