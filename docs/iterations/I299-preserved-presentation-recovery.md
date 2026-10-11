# Iteration I299: Preserved Presentation Recovery

> Document status: Complete / Closed
> Completion Commit: 690926759229818fcec92b1487d0cffef1bf5fcb
> Published plan date: 2026-10-10
> Planned objective: recover effective ownership and deliver the preserved C0 presentation candidate.
> MVP deliverable: truthful process summaries and readable live TUI activity, queued bodies and tool results.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | C0 preserved TUI activity, queue, edit/Todo result presentation and process-tool summaries; trusted native Todo display metadata through core, session, agent, conversation, runtime and TUI carriers under ADR-088; associated regression repairs only. |
| Claimed At | 2026-10-10 |
| Source Issue | None |
| Governance Claim PR | #700 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized serial implementation, stable publication, independent Agent review and gated merge; explicitly permitted non-overlapping concurrency with I293/I294. No separate maintainer reviewer is available; exact-head CI, Agent review, validators and CAS remain required. |
| Implementation PR | #701 |
| Last Updated | 2026-10-11 |
| Handoff / Release Condition | Deliver only after effective claim, local convergence, native acceptance and exact-head gates; retain incomplete work in the handoff ledger. |

PR #698 established Active / Claimed atomically at main commit
`50781eab529e424894d11e7b1c502fb1d0cb50f5`. Implementation branch
`feat/i299-presentation` starts from that merge. Claim CI passed at exact head
`791b521673b579f338d644b1ded5b608920589a5`; independent Agent review approved that
head against base `e58e26783e2f48416b871a394b17ac79b642bb1f`.
PR #700 expanded the effective claim under ADR-088 at
`2f9420f95cd2db3654de463c928debe592a89845`; the implementation branch was fast-forwarded
to that merge before restoring the preserved candidate.

## Published Baseline

### Scope And Historical Recovery

The coordinator is [the session handoff](../tasks/2026-10-10-session-todo-handoff.md).
Historical implementation commits `0a0622c0` and `3af78f04` precede an effective target-branch
claim. Preserve them on `recovery/session-handoff-20261010`; this record does not retroactively
authorize them, invoke an emergency waiver or make local main delivered code. Rebuild the
implementation candidate after the governance merge, as in I289's recovery precedent.

- Preserve live activity animation and queued message body visibility under constrained height.
- Preserve the small body bullet and process action/job_id/cursor/max_bytes/wait_ms summaries.
- Preserve edit soft red/green backgrounds through CJK wrapping and normal/head/tail output.
- Deliver display-only Todo status glyphs and active highlighting without rewriting user titles,
  descriptions, arbitrary checkboxes, error output or durable raw results.
- Repair the independent review finding: multiline user text can spoof Todo rows. Text prefixes,
  list headers or UUID-shaped suffixes alone are not trusted presentation provenance.

### Non-Goals

No tool execution, permission, sandbox, storage or process-hardening change; no new public API
without a separately accepted ADR/migration contract. No MODEL-014 implementation, compaction,
Git repair, validation/agile tools, memory-limit parameter, Search or Auto locale change. The seven
original handoff requirements remain separately scheduled, not completed by C0. No release.

### Acceptance

- Queued bodies remain readable alongside pending tool animation, including narrow/short layouts.
- Process summaries show the supported arguments without changing execution or private evidence.
- Edit background fills cover each projected visual row; original logical text remains unchanged.
- Todo titles/descriptions remain unchanged, including multiline text, forged headers and suffixes;
  status glyph/highlight behavior remains correct for trustworthy item status presentation.
- Normal/head/tail rendering, error and unrelated-tool cases retain their correct behavior.
- Native terminal acceptance confirms animation, constrained-height queue visibility, tinted CJK
  wrapping/selection and Todo appearance. Automated tests alone do not close these observations.

### Planned Validation And Documentation

- Focused locked TUI tests and process-tool summary tests; include multiline spoofing regressions.
- Standard `./scripts/release_preflight.sh` with pinned Rust 1.99.0 and locked workspace checks.
- Both governance validators; exact-head remote CI, independent Agent review and merge-time CAS.
- Update `README.md` and `README.zh-CN.md` for observable TUI presentation; synchronize this owner,
  the coordinator, iterations index and Board owner-first.

### Selection Inventory And Parallel Boundary

| Owner | State at selection | Disposition |
|---|---|---|
| I293 / #696 | Active / Claimed | Other session; Search unchanged; explicit non-overlap authorization. |
| I294 | Active / Claimed | Other session; provider evidence unchanged; explicit non-overlap authorization. |
| I290 / I291 / #682 | Review | Retain Auto locale acceptance and ownership; no takeover. |
| I277 | Review; human checks Deferred | Retain Desktop device/accessibility residuals. |
| I249 | Planned | Keep dependency pilot deferred. |
| I164 | Paused | Keep superseded layout target paused. |
| I296 / I297 / I298 | Terminal owners; claims Closed | Preserve their completion evidence; no reuse of closed claims. |

No other Active, Review, Planned or Blocked iteration header found at selection against
`e58e26783e2f48416b871a394b17ac79b642bb1f`. Story owners remain separately authoritative;
DEPENDENCY-003-A is retained without taking over its scope. Shared derived views use owner-first
union updates. Refresh all owners and open PRs again before merge.

### Risks And Rollback

False Todo recognition can corrupt presentation; broad transcript/API changes exceed this slice.
Resolve provenance conservatively and seek an ADR if required, rather than guessing from text.
Retain historical commits and revert only this bounded implementation if regression occurs.

## Verification Evidence

### Change-control amendment: trusted Todo provenance (2026-10-11)

The maintainer accepted [ADR-088](../decisions/088-trusted-todo-result-presentation.md)
to satisfy the existing complete-list Todo display acceptance. This is a correction to the
means of delivering C0, not a second product objective. The published baseline above remains
intact. The amendment extends only trusted, transient native Todo presentation metadata
across the listed carriers; it does not authorize permission, sandbox, tool execution,
durable-schema, MCP/plugin trust or release changes.

The resulting implementation must decorate every genuine item without parsing user text,
keep model/persistence/export bytes unchanged, reject stale/invalid metadata and document
actual public Rust API migration for the next pre-1.0 minor release. Tests must cover
multiline spoofing, every list item, head/tail projection, concurrent calls, redaction and
hook mutation. Independent API/privacy review and native terminal acceptance remain gates.

PR #700 merged at `2f9420f95cd2db3654de463c928debe592a89845`. Exact head
`be35835ab14f24f9e1b2c4c97393519352b2733c` / base
`45892e7ddf5985aba8fdd5804b364611e1cc7173` passed documentation CI `38110279048`
(four successful jobs; two Rust platform jobs correctly skipped). Independent Agent
APPROVE is recorded in comment `6105271021`; merge-time CAS in `6105289052`.
Both repository validators and the skill validator passed with zero warnings.
This establishes scope only, not implementation completion or native acceptance.

At amendment selection, target `main@45892e7d` has I293 and I294 Active under separate
Search claims, I277 Review with deferred human checks, I249 Planned and I164 Paused.
I290/I291 are now Complete / Closed; #696 is the only open PR and belongs to I293.
The Todo result display carrier work does not change Search routing, provider evidence,
Desktop or dependency work. The maintainer's earlier non-overlap authorization remains
limited to these separate slices; shared derived documents are synchronized owner-first.

### Local implementation checkpoint

Preserved presentation changes have been reapplied after the effective claim. Independent
technical assessment confirms that plain-text Todo results have no structured row provenance.
The conservative implementation converts only the initial producer-owned status position for
create/update/query; subsequent rows stay literal, including genuine later items. It never
re-synchronizes on user-controlled headers, blank lines or UUIDs. Full-list status decoration
remains unimplemented pending trusted provenance; no producer output, storage or public API is
changed. This limitation must remain visible at acceptance rather than being reported complete.

Focused locked TUI validation passed: 603 unit tests, 2 integration tests and 2 doctests,
including 34 tool-display tests. Command:
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p talos-tui --locked -- --quiet`.
The same resource settings with `cargo test -p talos-agent --locked process_tool -- --quiet`
passed all 5 process-tool tests. Both governance validators passed with zero warnings.
Targeted `cargo clippy -p talos-tui -p talos-agent --all-targets --locked -- -D warnings`
also passed with the same resource settings. No validation process remains running at this
checkpoint. These focused checks do not replace full preflight or native acceptance.
Native acceptance and rebuilt full
preflight have not yet passed. The maintainer accepted structured display provenance on
2026-10-11 under ADR-088; #700 now establishes the expanded effective claim. Cross-crate
implementation remains pending.
No scope reduction is assumed.

Historical candidate `0b396c48865b74e4cf3e7d0f711ccb208f264afd` passed full standardized
preflight on 2026-10-10 with debug-free, single-job, non-incremental profiles. Independent Agent
review identified the Todo defect above; it is not approval. Rebuilt candidate requires fresh gates.

## Completion Evidence

### Decision history: complete Todo result decoration

The following proposal was accepted on 2026-10-11; ADR-088 records the binding contract.
Acceptance alone does not expand the effective claim. The existing structured Todo panel already
supports status decoration; the gap is tool-result history containing multiple items.

Source inspection: `ToolResult` carries text/error/continuations; `ToolResultProjection`
separates model/display/persistence strings; `ToolResultDisplay` carries name/error/text.
There is no trusted per-item status channel. Continuations are tool-disclosure requests, so
using them as formatting metadata would violate their contract. Re-querying the Todo database
while rendering would show current state instead of the recorded execution result and is not
an acceptable replacement. Escaping producer text would change model and durable output.

Recommended contract for a separately accepted ADR:

- Native Todo formatting captures typed status spans while assembling the successful result,
  before arbitrary title/description/tag text is appended. Never recover spans by parsing text.
- Only the registered native Todo producer and known semantic metadata type are eligible.
  A tool name supplied by MCP, a plugin or caller text is not authority; wrappers preserve
  source eligibility, and errors discard decorations.
- Carry bounded optional presentation metadata with that exact result and invocation ID through
  permission wrappers, agent events, conversation projection and TUI history. Rendering cannot
  execute tools, query mutable storage or grant permissions.
- Validate UTF-8 boundaries, ordering, non-overlap and exact status text. Bind metadata to its
  source content; if hooks, redaction or other projections change that content without an explicit
  remap, discard metadata and render literally. Never retain stale offsets after sanitization.
- Use checked UTF-8 byte offsets with explicit metadata count/size caps. Invalid metadata
  falls back as a whole, never by guessing missing rows. Bind against final visible content
  using strong digest plus length or equivalently verified identity, without carrying a second
  hidden raw payload into UI metadata.
- Map wrapping and head/tail truncation from validated original positions; omitted spans cannot
  leave orphan decorations. Distinguish the status replacement span from the item-owned row
  highlight span; multiline user text is not itself status provenance.
- Preserve model-facing text, raw durable transcript and exports byte-for-byte. Legacy restored
  results lacking provenance remain literal; persistence of display metadata is a separate
  explicit decision, not an implicit transcript migration.
- An additive optional field still breaks external Rust struct literals, and an added event
  variant affects exhaustive matches. Inventory each public carrier, document constructors and
  migration, and select the next compatible release boundary before implementing or publishing.
- Prove all-item decoration with multiline titles/descriptions/tags, forged headers/UUIDs,
  normal/head/tail display, concurrent calls, error/unrelated tools, hook mutation, private-token
  redaction and raw export equivalence. Independent API/privacy review is required.

Alternative requiring explicit scope acceptance: keep complete glyphs in the structured panel,
but conservatively decorate only the initial unambiguous tool-result status. This alternative
does not fulfill complete tool-result list decoration and cannot silently close that requirement.

No new API implementation begins until the maintainer accepts the boundary and an effective
claim covers the affected cross-crate paths. The seven-task handoff remains active.

Independent Agent design assessment found this direction viable with the trust, bounds,
invalidation and row-span clarifications above. This is not accepted ADR authority or
implementation approval. The maintainer subsequently accepted this boundary on 2026-10-11.

### Accepted implementation-boundary amendment (2026-10-11)

[ADR-088](../decisions/088-trusted-todo-result-presentation.md) authorizes trusted transient
Todo result metadata to fulfill the existing complete-list acceptance, not a new product
objective. Preserve the published baseline above. Prepare an effective claim amendment for
the required core/session/agent/conversation/runtime/TUI carriers before cross-crate code
implementation. The current claim remains limited to its target-branch scope until that
amendment merges. Inventory actual public API changes and publish migration instructions;
no release or durable-schema change is authorized.

Closure ledger: update ADR/index, this owner and handoff checkpoint; preserve all local
presentation code and historical commits; validate both governance harnesses and diff;
record expanded-claim, API migration, full implementation/tests and native acceptance as
remaining I299 work. I299 remains Active / Claimed, not Complete.

### ADR-088 local foundation checkpoint (2026-10-11)

The local candidate adds transient Todo status/item byte ranges and whole-set geometry
validation (UTF-8 boundaries, ordering, exact token, count and content-size limits), plus
a SHA-256/length binding carrier without source-text retention or new dependencies.
Native formatting records item positions while appending output; forged multiline titles,
descriptions and tags do not create extra spans. Exact output preservation has focused tests.

`cargo test -p talos-core --locked todo_presentation` passed 4 tests;
`cargo test -p talos-session --locked todo::` passed 41 tests;
`cargo clippy -p talos-core -p talos-session --all-targets --locked -- -D warnings` passed.
All used single-job, debug-free, non-incremental profiles. The first session test attempt
failed because the new fixture used a nonexistent priority default; it now uses Medium.
This is not end-to-end evidence: production formatter entrypoints still return text only.
Digest computation, producer eligibility, wrapper/event correlation, invalidation and TUI
consumption remain to wire, along with API migration, full preflight and native acceptance.

Pending. No target-branch implementation or native acceptance is claimed.

### ADR-088 composed-chain checkpoint (2026-10-11)

This supersedes the implementation-pending statements in the foundation checkpoint above;
the published baseline and its historical validation remain preserved. Native producers now
emit bound metadata, registered wrappers forward eligibility, execution and observation discard
invalid/error/redacted metadata, and conversation/TUI consume only final-visible-content-bound
spans. Legacy serialized events remain literal. Public API migration is documented in
`docs/reference/I299-TODO-PRESENTATION-MIGRATION.md`.

The independent local review requested a real composed-chain regression and a fix for orphan
highlighting when truncation hides the status line of a multiline item. The orphan fix has a
passing focused `omitted_status` regression. The new CLI integration test
`native_todo_through_permission_runtime_preserves_durable_and_export_text` runs the native
Todo adapter through Runtime's permission wrapper and event loop into ConversationEngine,
verifies transient display metadata reaches the UI output, reads the durable session, and
compares both export modes against serialized/replayed events without presentation authority.
It passed with `cargo test -p talos-cli --test todo_presentation_chain --locked` (1 test).
The test uses an isolated temporary database and mock model; it is not native terminal rendering
or human acceptance evidence. No new dependency was added.

Remaining: incremental independent review, full preflight of the final candidate, native TUI
acceptance, stable implementation commit/PR gates and owner-first closeout. The full handoff's
seven original deliverables remain in scope; this checkpoint does not complete the long task.

### Native terminal acceptance plan

Build `cargo build -p talos-cli --example i299_presentation_acceptance --locked`, then the
maintainer runs `target/debug/examples/i299_presentation_acceptance` in their own terminal.
`--check` exercises only the two real native Todo fixture runs; it does not open or verify a
terminal. The program uses a mock provider, temporary Todo databases and the production
Runtime permission wrapper. Edit/process stages are explicitly display fixtures and execute
no filesystem edit or subprocess tool. It does not load user configuration or sessions.

Advance with `next`, exit with `quit`. Record the candidate SHA with each observation:

- [x] Activity animation and queued message bodies remain visible in a short/narrow window;
      process summary includes supported parameters; this is no process-execution evidence.
- [x] Edit soft red/green backgrounds cover CJK/ASCII wrapped rows; selection text is correct
      and body bullets preserve the intended small appearance.
- [x] All four real Todo statuses are decorated, active item highlighting is correct and
      forged status text in multiline titles remains literal, including after resize.
- [x] Long Todo output preserves head/tail presentation without an orphan highlight;
      scrolling/selection and terminal restoration on exit remain correct.

The maintainer replied “完成了” to the stage 1 animation/queue check for candidate
`b19fc58ed8531122a5f0865beb54a7d356971d6b` (code unchanged from `e96438eb`). This records
stage 1. The maintainer subsequently replied “符合预期” to stage 2's wrapped edit backgrounds
and drag-selection check on the same candidate, then confirmed “符合预期” for stage 3's four
Todo glyphs, active highlight, literal forged title row and narrow-window wrapping.
The maintainer then confirmed “符合预期，已退出” for stage 4, including head/tail truncation,
absence of orphan highlight, scrolling/selection and terminal restoration. All four native
observations passed on this candidate; these are human reports, not inferred automatic results.

### Stable local candidate validation (2026-10-11)

`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 ./scripts/release_preflight.sh`
exited 0 with `release preflight: passed`. This includes workspace check/Clippy/tests and
doctests, external Runtime SDK default/coding fixtures, site/installers, classifier/text-boundary
checks and both governance validators (zero warnings). The TUI suite contains 605 passing
unit tests, including the orphan-highlight regression. No test gate is waived.

The dedicated acceptance example's `--check` run passed for normal and long native Todo
results. It checks all four real spans and confirms the long fixture's active status lies
in the omitted middle while its continuation reaches the tail. Targeted Clippy for the example
and composed-chain integration test passed with `--locked -- -D warnings`.
The example was refined during workspace validation; its final build/self-check and targeted
Clippy are separate evidence, not an assertion that preflight exercised native drawing.

Independent reviewer `/root/i299_amendment_review` approved the incremental dirty-tree
changes without new blocking findings. Stable-commit review remains required. Remote refresh
confirmed `main@2f9420f95cd2db3654de463c928debe592a89845`; the only open PR is other-session
#696 (Search), with no overlap in this candidate. Native acceptance remains pending, so this
owner stays Active; no remote implementation push or completion is claimed.

### Exact-head local review and acceptance handoff (2026-10-11)

Stable local implementation: `e96438eb84066ef178a830ea4b42edb452c78d69`, based on
`2f9420f95cd2db3654de463c928debe592a89845`. Independent Agent reviewer
`/root/i299_amendment_review` returned APPROVE for these exact revisions, with no blocking
API/privacy/scope findings. The reviewer confirmed the prior two defects are fixed and that
Desktop/Dashboard edits are mechanical event API adaptations only. This is an independent
Agent technical review, not natural-person acceptance or a separate GitHub identity.

The four native observations above remain unchecked. The maintainer has been asked to run
`target/debug/examples/i299_presentation_acceptance` and advance to stage 1 with `next`;
no result has been received. Do not repeat prior I279 acceptance as evidence for these changed
surfaces. No implementation push, remote CI or merge has occurred. Subsequent documentation
changes must retain this review's exact revision binding and obtain an incremental rebind
before merge; they do not imply a rerun of unchanged Rust code is necessary.

Next action: collect the four observations sequentially, fix any failure locally, then record
the final candidate and publish only after the stable-stage gate. All seven original handoff
deliverables remain incomplete; C0 acceptance does not terminate the long task.

### Native acceptance complete; stable publication ready

The sequential human reports above complete the four required native observations on
`b19fc58ed8531122a5f0865beb54a7d356971d6b` (implementation `e96438eb`). Independent Agent
review also approved `b19fc58e` against `2f9420f9` after its documentation-only increment.
This checkpoint supersedes earlier pending-human statements. Delivery is now Review;
remote implementation CI, final-head review, merge-time CAS, merge and owner-first closure
remain required. No release or completion is claimed. Subsequent edits here are acceptance
records only; the validated Rust implementation and acceptance executable remain unchanged.

Stable implementation candidate published as PR #701. Final-head CI, incremental independent
review and merge-time CAS remain pending; publication is not completion evidence on main.

### Implementation merged and owner-first closure — 2026-10-11

PR #701 merged into main as `690926759229818fcec92b1487d0cffef1bf5fcb`.
Its final head was `58e7461341908589ee68caf6b51e6b1486eef21a`, with base
`2f9420f95cd2db3654de463c928debe592a89845`. Exact-head CI run `38115866825`
passed all six jobs, including full macOS validation, Windows Rust workspace and
Linux Desktop explicit-feature validation. Independent Agent API/privacy review
approved the same head/base in comment `6105965862`; merge-time CAS was recorded
in comment `6106128866`. This was the authorized single-maintainer path with
independent Agent-role review, not a separate human/GitHub reviewer identity.

Both governance validators passed with zero warnings immediately before merge.
All four native observations passed on the unchanged implementation, as recorded
above. This checkpoint supersedes earlier pending-publication, pending-review,
pending-human and pending-merge statements without rewriting their history.
The existing main implementation SHA above is the completion evidence, not this
status-only closeout commit. Delivery is Complete and the claim is Closed.

No remaining acceptance exists within I299. ADR-088's documented public Rust API
migration must be honored by the next pre-1.0 minor release; no release is made
or authorized by this closure. The seven original handoff requirements remain
incomplete and separately scheduled, starting with T7 Git repair. Other-session
Search/provider work and retained recovery assets are unchanged.
