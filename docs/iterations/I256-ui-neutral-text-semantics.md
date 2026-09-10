# Iteration I256: UI-Neutral Text Semantics

> Document status: Planned / Unclaimed
> Published plan date: 2026-09-10
> Objective: Complete TEXT-001 using the existing talos-text compatibility seam.
> MVP deliverable: The real TUI consumes shared streaming block classification and validated neutral highlight results, preserving current rendering and plain-text fallback.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Shared text classification, language identity and validated semantic fallback; narrow TUI adapters only. |
| Claimed At | Not applicable |
| Source Issue | #511 (parent #466) |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Preparation only; no implementation until finalized atomic claim reaches main. |
| Implementation PR | Not started |
| Last Updated | 2026-09-10 |
| Handoff / Release Condition | Independent API/compatibility plan review, exact-head governance CI and CAS precede implementation from the claim merge or later main. |

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

Finalize one atomic claim/activation PR for I256 and TEXT-001, with actual PR number, current
inventory and independent plan review. After merge, start the implementation branch from that
merge or later main, converge design/code/tests/docs locally, and submit one stable candidate.
Local subtasks and review corrections stay under #511; do not create extra Issues for them.
