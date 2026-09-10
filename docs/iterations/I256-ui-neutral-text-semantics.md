# Iteration I256: UI-Neutral Text Semantics

> Document status: Review / Claimed
> Published plan date: 2026-09-10
> Objective: Complete TEXT-001 using the existing talos-text compatibility seam.
> MVP deliverable: The real TUI consumes shared streaming block classification and validated neutral highlight results, preserving current rendering and plain-text fallback.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Shared text classification, language identity and validated semantic fallback; narrow TUI adapters only. |
| Claimed At | 2026-09-10 |
| Source Issue | #511 (parent #466) |
| Governance Claim PR | #530 |
| Authorization Mode | Independent review |
| Authorization Evidence | #530 effective at c8b596980136a577921bf92cdcbe1e517e64d10a; exact head c5f94f31, CI 34474441483, independent API/compatibility APPROVE 5618377071 and CAS 5618385106. Shared account establishes Agent-role separation only. |
| Implementation PR | Not started |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | Claim #530 is effective; converge the complete slice locally, then require fresh exact-head CI, independent API/compatibility review and CAS before implementation merge. |

## Required Reads

- [TEXT-001](../backlog/active/TEXT-001-ui-neutral-text-semantics.md), [CAP-001](../backlog/active/CAP-001-progressive-capability-provider-architecture.md), [LANG-001](../backlog/active/LANG-001-language-provider-contract-migration.md).
- [ADR-072](../decisions/072-capability-provider-bundle-boundary.md), I246 compatibility evidence and [I253](I253-capability-governance-convergence.md) acceptance/migration audit.
- `crates/talos-text/src/lib.rs`, `crates/talos-tui/src/stream_markdown.rs`, `crates/talos-tui/src/app_stream.rs`, `crates/talos-tui/src/highlight.rs`.

## Published Baseline

### Selected Story

| Story | Status at selection | Dependencies | Outcome |
|---|---|---|---|
| TEXT-001 / #511 | Refinement / Unclaimed | CAP-001-A Complete; ADR-072 Accepted; I253 audit Complete; consumer inventory below | Shared renderer-neutral streaming/text semantics with actual TUI integration and deterministic fallback. |

### Scope And Compatibility

- Reuse `talos-text`; do not create a second text crate. Preserve existing public struct/enum
  construction, language alias behavior, serialized spellings and symbol extension policy.
  Add compatible APIs rather than silently tightening existing deserialization or renaming fields.
- Extract UI-independent streaming Markdown block decisions from the current TUI implementation
  into a shared contract. Preserve fences, tables, lists, quotes, flush/reset behavior, hold limits,
  partial input handling and text fidelity for normal inputs. Current code/table limits are
  retained; currently unbounded list/quote and oversized first-line/table-candidate paths gain
  bounded plain-text fallback as an authorized exceptional-input safety correction.
  Shared output carries semantics and source text, not
  ANSI colors, terminal width, Ratatui/GPUI types or layout decisions.
- Keep TUI-specific preview strings and formatting in the TUI adapter. Wire the existing runtime
  path to the shared classifier, not a second unused parallel algorithm. Characterize existing
  behavior before extraction, including code-fence close rules and bounded fallback.
- Complete renderer-neutral highlight validation/fallback: UTF-8 byte boundaries, ordered and
  in-range spans, unsupported/unavailable/malformed results safely return plain source. Preserve
  current canonical-only TUI grammar admission; shared alias normalization does not silently
  enable new TUI languages. Existing optional built-in adapter remains compatible. Whole-result
  fallback instead of the current TUI clamp/skip behavior is an explicit malformed-result safety
  correction; valid span rendering is unchanged. Add a validation interface without changing
  existing HighlightResult/HighlightSpan constructors or serde acceptance.
- The shared classifier consumes valid UTF-8 strings/complete lines, not undecoded byte streams.
  Chunk assembly, CRLF and final partial-line handling stay in the existing TUI adapter and must
  have adapter tests; do not claim the line classifier independently decodes bytes.
- Supply headless public-contract fixtures that require neither TUI nor Desktop imports. This is
  readiness for a Desktop consumer, not a claim of implemented Desktop integration.

### Non-Goals

No LanguageProvider runtime dispatch or symbol-consumer migration (LANG-001), parser trimming,
WASM language provider, dynamic loading, Bundle schema/installation, downloads, new dependency,
default feature/member expansion, normal-input UI layout/color/padding changes, permission/session behavior,
Dashboard/Desktop production binding, release/version/tag/publication. Breaking API or changed
user-visible rendering beyond the two exceptional-input corrections above requires explicit
change control rather than silent expansion.

### Acceptance

1. Given existing language aliases and extension inputs, normalization is deterministic and
   existing public constructors/serialized forms and case-sensitive symbol policy stay compatible.
2. Given streamed fences/tables/lists/quotes/plain text, shared decisions retain all source text
   and existing boundaries; flush/reset and incomplete/oversized input degrade without hangs.
   Chunked UTF-8 input cannot create invalid byte ranges or lose data.
3. Given valid spans, consumers receive renderer-neutral results; invalid order/range/UTF-8
   boundaries or unsupported/unavailable results produce explicit safe plain-text fallback.
4. Given existing TUI rendering fixtures, text, formatting, preview wording and canonical-only
   grammar behavior remain equivalent while non-test TUI code uses the shared contract.
5. A headless downstream consumer compiles/runs without Ratatui, GPUI or parser-native types;
   the default talos-text dependency tree introduces no parser bundle or UI dependency.
6. A real talos binary/PTTY or equivalent binary integration fixture drives streamed Markdown
   through the changed TUI path and asserts user-visible output/fallback. Library tests alone
   cannot certify behavior-facing completion; unavailable human rows are not assumed passed.
7. Public rustdoc, README and architecture docs accurately distinguish shared text semantics
   from the still-unimplemented LanguageProvider, dynamic loading and distribution children.

### Validation And Documentation

Characterization and conformance tests for shared text, streaming chunk boundaries, malformed
span tables and UTF-8/property-style cases; existing TUI highlight/stream/Markdown regressions;
symbol extension regressions; real binary integration evidence. Run default/no-default and
existing code-intelligence configurations with --locked, check dependency trees, then full
`./scripts/release_preflight.sh` using the pinned toolchain before stable push. Run both governance
validators with `COLLABORATION_VALIDATION_BASE=origin/main` and `git diff --check`.
Fresh exact-head CI and independent public API/compatibility review precede CAS merge.
Update `README.md`, `docs/reference/ARCHITECTURE.md`, public rustdoc, TEXT-001/I256 first, then
CAP-001/Board/backlog/iteration index/manifest/Issue matrix and #511.

### Risks And Rollback

Main risks are changing stream buffering/text boundaries during extraction, exporting UI details,
losing Unicode text, or changing fallback/legacy alias behavior. Keep characterization fixtures
and thin compatibility adapters; rollback restores the previous TUI adapter without rewriting
session/config files. No parser or dependency changes are assumed necessary.

## Selection And Consumer Inventory (2026-09-10)

Inspected main `dba3419c88f89ed6aa108de8d638ebe1c8ac3c64`, one clean worktree, no stash and
no open PR. I255/#513 is Complete/Closed via #528 and #529 and #513 is closed.

| Current owner | State | Disposition |
|---|---|---|
| Active / Review / Blocked iterations | None | I255 has reached terminal state; no parallel implementation. |
| I249 | Planned / Unclaimed | Keep unselected dependency pilot; do not activate. |
| I164 | Paused / superseded | Retain history; do not resume. |
| I162 | Historical terminal owner (Complete, with Review outcome) | Not a current Review owner. |
| I256 / TEXT-001 | Planned / Unclaimed preparation | Claim proposal has no effect until main merge. |
| Remaining CAP-001 children and #520 | Unclaimed | No transferred implementation authority. |

Templates, old empty-status iterations and dated historical state words do not activate work.

| Surface | Authority in this slice | Preserved or excluded boundary |
|---|---|---|
| talos-text public semantics | TEXT-001/I256 after effective claim | Existing I246 types and optional built-in adapter retained; LanguageProvider dispatch belongs to LANG-001. |
| TUI stream_markdown/app_stream/highlight, scrollback/scrollback_markdown, app.rs and app/app_tests.rs | Narrow shared-contract consumption only | Normal-input formatting/layout/interaction unchanged; only the explicit exceptional-input corrections above. |
| talos-tools symbol consumers and talos-text symbol/symbol_queries, SourceLocation/SymbolInfo | Compatibility tests/read-only characterization | Constructors/serde and extension policy preserved; no symbol behavior or provider migration. |
| Desktop/Dashboard | None | No files or owner edits; future clients consume neutral types. |
| Cargo workspace | No default changes | Existing members/features/dependency versions preserved. |

This single-developer serial claim proposes the explicit shared-consumer boundary required by
TEXT-001; no separate active TUI/Desktop claimant exists. Independent plan review must validate
this agreement. Any newly overlapping claimant requires a fresh overlap check before editing.

## Completion Evidence

Completion Commit: pending implementation.
This planning/status commit cannot certify completion.

## Resume

Claim/activation #530 is effective at `c8b59698`; implementation is locally converged on
`feat/i256-ui-neutral-text-semantics`. Finish staged-diff review and submit one stable candidate,
then obtain fresh exact-head CI, independent API/compatibility review and merge-time CAS.
Do not mark Complete before merged implementation evidence and owner-first closeout.
Local subtasks and review corrections stay under #511; do not create extra Issues for them.

## Atomic Claim Proposal (2026-09-10)

#530 proposes Active / Claimed for I256 and In Progress / Claimed for TEXT-001 together.
Neither is effective before merge. Published Baseline and the initial selection inventory at
`0f1ae3f9` remain unchanged; that inventory records preparation, not current claim effectiveness.
Main is still `dba3419c`; #530 is the only open PR. The local plan review clarified exceptional
oversized-block and malformed-span fallback before publishing the baseline, and inventoried
direct TUI adapters. No Rust/Cargo/default, Dashboard or Desktop edits are included.
The same governance batch removes the now-closed #513 from the open-Issue snapshot; I255
completion remains recorded in its owners and parent row. No I255 implementation is reopened.

## Effective Activation (2026-09-10)

#530 merged at `c8b596980136a577921bf92cdcbe1e517e64d10a` after exact head
`c5f94f3114d60260528a8ddf40ceadaddadcd7a8` / base
`dba3419c88f89ed6aa108de8d638ebe1c8ac3c64`, CI `34474441483`, independent
API/compatibility approval `5618377071` and CAS `5618385106`. This supersedes
proposal-only current wording; the Published Baseline and dated preparation facts are preserved.
Implementation branch `feat/i256-ui-neutral-text-semantics` starts exactly at the claim merge.
No implementation PR exists yet; all implementation/testing corrections remain local.

## Local Implementation Checkpoint (2026-09-10)

Shared `talos_text::stream` now owns the former TUI line classifier and its characterization
tests; TUI retains preview wording and rendering. Held-state overflow checks cover lists,
quotes and oversized first lines/table candidates. `HighlightResult::validated_spans` rejects
an entire malformed result without changing the old public types or serde forms; TUI uses
that boundary and renders oversized fallback source plainly.

Local checks with pinned toolchain, --locked, debug info disabled and incremental disabled:
`cargo test -p talos-text --no-default-features`: 10 unit + 5 integration tests passed;
`cargo test -p talos-tui --lib`: 567 passed, 0 failed. These are intermediate local results,
not final acceptance. No Cargo/default or dependency changes and no remote candidate.
Next: explicit cross-chunk/CRLF/final-line adapter tests, real binary stream evidence, additional
malformed-result rendering checks, API/user docs, compatibility configurations and full preflight.
Keep Active/Claimed until the complete slice converges; Completion Commit remains pending.

### Chunk Adapter Regression Checkpoint

New adapter tests first failed on two concrete cases: final table-candidate source was replaced
by the transient `rendering table...` preview during finish, and CRLF delimiters retained their
carriage-return byte in display content. Finish now feeds the actual source buffer through the
classifier before flushing and never persists preview wording; line assembly recognizes CRLF
even across chunks. These corrections implement the planned source-fidelity/final-line/CRLF
contract, not a new formatting feature. All 570 TUI library tests then passed, including every
valid UTF-8 split of a mixed Markdown fixture. A further malformed-highlight regression and a
real binary PTY fixture are being added locally; full acceptance remains pending.

### Full Local Preflight And Evidence Boundary (2026-09-10)

`env COLLABORATION_VALIDATION_BASE=origin/main CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 ./scripts/release_preflight.sh`
exited 0 against the implementation tree based on `c8b59698`: governance/claim validators
0 warnings, text-boundary validation 0 errors, format/check/Clippy and locked workspace tests
passed, including 571 TUI library tests and doctests. The optional `code-intelligence`
configuration passed 14 unit + 5 integration tests; no-default passed 10 + 5.

Independent local code pre-review found no blocker and clarified that the classifier's
200-line / 16-KiB budget covers complete held lines, excluding delimiters, not the adapter's
unfinished line. Existing constructors and serde definitions are unchanged; the added validator
does not change deserialization. Existing canonical-only highlight and symbol extension tests
remain the compatibility oracle. The headless default dependency tree contains only serde's
stack, not parsers or UI crates.

Real-terminal evidence is still being converged: cumulative output can include transient previews,
so final-history claims require a stable final-screen assertion. Unterminated fences deliberately
retain the existing inline-Markdown recovery (including emphasis), as tested by
`stream_render_state_recovers_markdown_after_unterminated_code_fence`; this is not the
oversized-block plain-source fallback. No additional production behavior change is authorized.

## Stable Candidate Acceptance (2026-09-10)

Local acceptance is satisfied; delivery is Review / Claimed, not Complete. The implementation
PR number is unknown before the first stable push and will be linked in the PR/Issue and closeout.

| Acceptance | Evidence |
|---|---|
| 1: legacy identity/API compatibility | Existing language alias, serialized-result and symbol extension regressions pass; old public constructors/serde declarations and symbol consumers are unchanged. |
| 2: classification, flush and source fidelity | Seven extracted characterization tests plus headless overflow/source-fidelity fixtures and every-UTF-8-split/CRLF/final-line adapter tests pass. |
| 3: semantic fallback | Headless malformed range/order/UTF-8 cases and TUI `later_bad_span_discards_earlier_coloring` pass; existing unsupported/alias fallback tests pass. |
| 4: actual TUI integration | Non-test app_stream/scrollback use the shared classifier, with preview wording kept in TUI; all 571 TUI library tests pass, including existing rendering fixtures. |
| 5: headless default consumer | talos-text no-default 10+5 tests and default dependency-tree inspection pass without parser/UI dependencies; optional code-intelligence 14+5 tests pass. |
| 6: real binary | `cargo build --locked -p talos-cli` and `python3 scripts/verify_i256_text_terminal.py --timeout 35` pass using an isolated local SSE provider and real interactive TUI. Final no-newline marker remains on the stable completed screen; transcript retains source without hold labels; exit 0. |
| 7: public documentation | Public rustdoc, README and architecture explain neutral types, caller chunk assembly, hold-budget scope and excluded provider/distribution work. |

The Unix-only PTY fixture checks emitted text and stable final-screen content, not colors or
pixel layout. Unknown-language text visibility is paired with existing highlight unit tests;
it alone does not prove absence of coloring. No natural-person or Windows terminal acceptance
is claimed. Normal-input styling is protected by the existing renderer tests, not inferred from
chunk-split consistency alone. Local pre-review is not final exact-head remote approval.

Changed-file inventory: `talos-text/src/lib.rs`, new `talos-text/src/stream.rs` and
`talos-text/tests/semantics_contract.rs`; TUI `app_stream.rs`, `highlight.rs`, `scrollback.rs`,
`stream_markdown.rs`; `scripts/verify_i256_text_terminal.py`; README, architecture, I256,
TEXT-001, CAP-001 child mirror, Board, backlog/index, Issue matrix, manifest and EVOLUTION.
Every change supports TEXT-001 implementation, compatibility evidence or owner-first reporting.
No Cargo/lockfile, dependency/default, permission, Dashboard, Desktop, release or unrelated owner
changes. LANG-001/#510 still owns provider dispatch and symbol migration; other #466 children
remain unclaimed. The unfinished-line memory bound is unchanged and not claimed by this slice.
