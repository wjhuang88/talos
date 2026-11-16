# EDIT-LOCATIONS-001 — Edit and diff source line locations

Status: Active / Claimed proposed via #921; ineffective until this governance record merges to main.
Owner: this document; originated in user-requested session handoff (preserved in the original worktree).

## Outcome and scope

Show real old/new file line locations in diff results and ordinary, prepared and anchored
edit previews so users can locate changes without counting output rows. This owns only
line-location presentation and its tests/docs. It excludes tool argument summaries,
permission policy, mutation semantics, background continuation, general TUI styling and
session recovery. Existing implementation owners retain their scopes.

Hard constraints: preserve exact before/after evidence and permission checks; never assign
file locations to omission notices or absent sides. Disjoint anchored operations retain
individual offsets, including shifts caused by preceding insertions/deletions. Unicode,
CRLF and missing final newlines must not merge or misnumber output. Output remains bounded.
No public API or persisted data format change is intended.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | EDIT-LOCATIONS-001: old/new diff and edit preview line locations, regressions and user documentation |
| Claimed At | 2026-11-12 |
| Source Issue | None |
| Governance Claim PR | #921 |
| Authorization Mode | Independent review |
| Authorization Evidence | User requested completion of the session handoff; standard effective claim remains required before committed implementation |
| Implementation PR | Not started |
| Last Updated | 2026-11-12 |
| Handoff / Release Condition | Effective claim, isolated implementation, validation, independent review and native display acceptance |

## Acceptance

1. A real diff with inserted lines shows distinct old/new coordinates, blank absent-side
   columns and unchanged input files. Missing final newline cannot concatenate changes.
2. Actual ordinary and prepared string edits locate a partial-line replacement in the original
   file. Anchored disjoint edits show old coordinates from the original and new coordinates
   from the result after earlier mutations. Admission/freshness refusal behavior is preserved.
3. Empty sides and truncated/omitted regions receive no fabricated line number. Long Unicode
   inputs remain within existing preview limits; no hidden full-file preview is added.
4. Actual TUI rendering retains numbers, +/- semantics, colors, clipping and unchanged raw
   tool results across narrow/wide resize. Native display acceptance remains separate.
5. Both READMEs explain the old/new columns and bounded edit locations. Selective diff review
   excludes unrelated argument-summary changes in the same TUI file.
6. Pinned locked focused file-write/git and TUI tests, full release_preflight, applicable native
   Windows tests and exact-head independent review pass before delivery. Completion needs an
   existing pushed implementation SHA; no completion commit currently exists.

## Existing candidate and verification boundary

Preserved primary base: `55617ad7`. Local source changes remain in the primary worktree.
The selective patch at `/tmp/talos-edit-locations-selective-20261112.patch` contains seven
source/test files, excludes unrelated tool argument summaries, and was checked using a
throwaway Git index against that base; the real index was untouched. SHA256:
`296aa27c4cabacf141527cc707d1edd0d005b921350624062e1b8259af61c1f7`.
README changes still need selective extraction. This is preparation, not a delivered commit.

Historical validation of the mixed primary tree is recorded in
the preserved session backlog continuation record in the original worktree: 101 file-write tests,
numbered TUI regressions and full `/tmp/talos-edit-line-preflight.log` passed. These results
do not prove that a newly isolated patch or future target head passes. Revalidate that exact
candidate after effective claim and extraction; do not relabel historical evidence.

## Next steps and residual ownership

Verify current target-branch owners and overlap before proposing a governance-only claim.
The claim becomes effective only after target merge under AGENT-COLLABORATION. Then apply
and validate the isolated candidate, selectively include both README updates, obtain review
and native acceptance, and record implementation/delivery evidence here. No iteration is
selected or activated by this task-level proposal. All remaining EDIT-LOCATIONS-001 work stays
owned here; the handoff remains Partial.

Preparation base: `6c81390a`. Open PR inventory #861/#907/#912 (Search/storage),
#916 (approval history), #917 (recovery), #918 (Azure) and #920 (Git compatibility)
showed no overlapping line-location owner. Existing iterations and claims remain unchanged.
Both governance validators and scale assessment passed before PR creation; finalized
claim validation and independent review precede merge. No implementation branch is created.
