# SESSION-UI-001 — Queue visibility and user-history continuity repair

Status: Review. Claim #952 is effective on main; local implementation and standard validation passed, remote review/CI and native acceptance remain open.
Owner: this document. Source: the maintainer's existing session-recovery handoff and
reported missing queued text / missing blank row after user input.
Primary-worktree source: `docs/tasks/2026-11-11-session-recovery-handoff.md`,
section 4; screenshot `/tmp/codex-clipboard-rjjsXe.png` and retained ANSI evidence
`target/recovery-user-separator-pty-result2/result.json`. These local artifacts
are provenance references, not files shipped by this governance-only candidate.

## Outcome and scope

Keep accepted but not yet projected user input visible, preserve a readable queued
body when the terminal has room, and separate a completed shaded user block from
live preview by one unshaded blank row. Verify animation continues through intermediate
tool/error progress until authoritative turn completion. These are existing handoff
items, not a new feature set. This task owns only their CLI/Conversation/TUI repair,
regressions and user documentation; history recovery remains PR #917's own slice.

Exclude approval history/policy, edit/diff line locations, ellipsis/theme redesign,
background enablement, provider retries, new storage formats, releases and deployment.
The completed TUI-026/TUI-044/TUI-049/TUI-060 and ellipsis owners retain their evidence;
this task does not reopen them or borrow their closed claims.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | SESSION-UI-001: accepted queue presentation, queue row allocation, external user-block separator, terminal lifecycle regressions and docs |
| Claimed At | 2026-11-15 |
| Source Issue | None |
| Governance Claim PR | #952 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer requested completion of the existing handoff; effective target-branch claim required before committed implementation |
| Implementation PR | Stable candidate preparation; not yet opened |
| Last Updated | 2026-11-16 |
| Handoff / Release Condition | Effective claim, isolated source validation, exact-head review/CI and native terminal acceptance |

## Hard constraints and implementation boundary

Actor acknowledgement transfers dispatch custody, not permission to erase pending
presentation. Any retained preview is bounded, correlated by submission/receipt and
session generation, and cannot become a second dispatch queue. Stale events cannot
clear current input; resolved terminals and runtime replacement remove stale previews.
No duplicate submission, reordered FIFO, provider-switch bypass or cancellation-policy
change is permitted. Additive public API must remain semver-compatible; any breaking
change requires its own ADR and migration plan before implementation.

Layout remains within terminal bounds. Required credential/approval controls and an
editable composer take precedence. Optional separator cannot displace queued body or
the last history row; extreme short-height fallback must be explicit in tests. Use
actual buffer colors: an empty row inside the user background is not an external gap.
The separator applies only at a FollowTail user-background tail before live preview;
anchored scroll positions and absent previews receive no extra gap. Selection mapping
and same-instance resizing must remain correct. Animation truth remains authoritative
completion, not temporary lack of output.

## Runnable deliverable and acceptance

1. During a long live preview, enqueue CJK/multiline input. Actual frames show queued
   body when space permits and preserve controls through narrow/wide and short/tall resize.
2. After Actor accepts but before projection, pending text stays visible. Test receipt/event
   order, paused/error/cancelled state, stale receipt, generation replacement and later queued
   input. AcceptedPending/PausedPending can remain visible while paused; matching
   terminal cancellation/error and Running user projection clear the corresponding preview.
   Exactly one intended provider submission occurs; preview never requeues work.
3. After direct or queued input enters history, the last shaded row is followed by an
   unshaded empty row before Connecting/live content when space permits. Real ANSI PTY
   evidence includes 40x24, 24x12, 80x24 and 40x12; short-height tests preserve required UI.
4. Feed actual bridge progress/error and terminal events through the production projection.
   Spinner remains active through intermediate states and stops after Success/Cancelled/Error
   completion. Native terminal observation supplements deterministic frame tests.
5. Update README.md and README.zh-CN.md with queue visibility, spacing and tiny-terminal
   fallback behavior, without claiming background continuation or changed execution timing.
6. Pinned/locked focused CLI, Conversation and TUI tests, standard release_preflight,
   independent exact-head review, relevant native CI and terminal acceptance pass before
   delivery. Existing experiments are not selective-candidate completion evidence.

## Baseline, inventory and preserved experiments

Target observed: 5b2fa435 (2026-11-15). Header inventory retains I249 Planned;
I277/I302/I318/I325 Review; I323/I324/I327 Active or proposed Active exactly as their
owners state. No Blocked iteration header was found; I164 remains Paused. No iteration
is selected/activated here and no unrelated owner state is changed. Scale harness
recommends high-risk, release-managed, required worktrees; existing policy is retained.

Nine uncommitted files are preserved separately from recovery commits, split into
geometry (five), accepted queue projection (three), and animation regression (one).
Primary artifact directory: target/recovery-ui-delivery-slices-20261115; its manifest
binds patches to recovery head55ba5752. Disposable-index application/whitespace checks
pass; originals and all mixed work remain retained. Prior 696 TUI tests and isolated
production CLI PTY screenshot evidence establish experiments only, not a new isolated
full gate or installed-host acceptance. Future extraction must recheck target overlap.

## Closure ledger and next step

Requested outcome: close these existing UI handoff defects with runtime evidence.
Artifacts: this owner and derived Board entry; implementation and both READMEs later.
Preserve: existing owners, claims, dirty code, original session data and patch artifacts.
Validate governance with both repository validators, link/whitespace checks and independent
scope review. PR #952 proposes claim/activation atomically with its actual number recorded;
then require target-branch merge before implementation branch/commits. No merge authority
is inferred from this document. All incomplete SESSION-UI-001 work remains owned here;
no Complete or implementation commit is asserted.

Local scope review found no blocker; follow-tail, selection/resize and correlated
terminal-cleanup acceptance were clarified before publication. Initial governance
validation passed with zero warnings; claim validation passed with one pre-existing
SEARCH-001-D3 documentation-intake warning outside this work slice.

## 2026-11-16 effective claim and isolated implementation — Partial

Maintainer explicitly authorized governance merge. Claim #952 became effective at
36eb752844e24a15ed295532d186d3bb6027c458. Implementation branch
fix/session-ui-continuity-20261116 starts from later main e1cbee1e in the isolated
worktree target/session-ui-implementation-20261116. The primary dirty worktree,
original session data, migration backup and retained patches remain preserved.

The candidate reserves queue header/body rows before optional spacing and adds an
unshaded separator only at a FollowTail user-background tail before live preview.
Accepted input remains in a display-only cache until correlated projection or terminal
resolution; runtime replacement clears it. The cache holds at most one prepared
batch (32 items / 256 KiB text, plus the items' existing attachment metadata), in
addition to the pre-existing queue. It is not zero extra storage. Rendered snapshots
remain bounded to 8 entries / 4096 UTF-8 bytes each and never dispatch the cache.
Public API additions retain existing signatures; no persistent format changes.

Locked local library tests: 195 Conversation and 691 TUI tests passed, zero failures
or ignored tests (target/ui-focused-20261116.log). Separate CLI binary tests passed
426 tests, zero failures/ignored (target/ui-cli-tests-20261116.log). The library-only
command did not include CLI bridge tests; the latter is the authoritative bridge
validation. Additional selection/anchored regression validation is in progress.
Both READMEs describe queue custody visibility, external spacing and tiny-terminal
priorities. Independent Agent local-diff review found no blocker; this is not final
exact-commit review or independent human/device acceptance.

Remaining: new geometry regression results, isolated actual CLI PTY (CJK/multiline,
resize, direct and queued-history colors, exactly-once requests), full standard
preflight, stable candidate/commit review and native CI, and native terminal
observation. No product merge, installation or background activation is authorized
by this claim merge. Handoff remains Partial; no Completion Commit asserted.

### Additional runtime and geometry evidence

Additional focused TUI run passed 9 user-related tests (683 filtered), including
same-instance resize/selection mapping and actual ScrollUp removal of the gap in
Anchored mode: target/ui-geometry-tests-20261116.log. Format and whitespace checks
passed. Production CLI build passed; SHA256
397ad5abcb4ce40331cfca058f6f6ec4978cc8788d99f1680e51c14842294cd8.

Actual CLI ANSI PTY acceptance passed at target/session-ui-pty-20261116-r2/result.json:
8 color-checked separator observations (direct and queued-history at 40x24, 24x12,
80x24, 40x12), plus 7 long-reasoning queue frames including 24x6 and 80x8. Bracketed
paste supplied two CJK lines; request capture verified the complete multiline input
appeared exactly once in the second request, with two provider requests total.
Queue body remained visible through same-instance resize. Raw ANSI output is retained
in that artifact directory. Initial harness startup failed before product execution
because system Python lacked pyte; isolated pinned pyte0.8.2/wcwidth0.2.13 wheels
were installed only under target/ui-pty-python-20261116 with PyPI SHA256 provenance.
No user session or system Python package was changed.

These are production-binary/controlled-provider/terminal-emulator observations,
not installed-host acceptance, a real provider quality claim, or native-terminal
visual approval. Full standard preflight and final candidate gates remain open.

### Full-gate locale correction

First full preflight exited101 after873.78s; all2649 manifest files were unchanged.
The Tools suite had1160 passed/1failed: hasconfig_non_path_key_still_triggers_url_prohibition
expected an English native Git diagnostic. The launcher omitted LC_ALL=C/LANG=C,
contrary to LOCAL-DEV.md's documented service-environment requirement. The exact
same compiled test passed under the prescribed locale, without source or assertion
changes (target/ui-git-locale-check-20261116.log). This focused result does not close
the full gate. Repeat full standard preflight with fixed locale and resource limits;
retain the initial failed log/manifest/result at target/ui-full-preflight-20261116*.

### Current candidate standard gate passed

Fixed-locale full standard preflight completed with exit0 in776.13s, including
5353 workspace/doc tests (zero failures/ignored) and both independent Runtime SDK
fixture feature configurations. Manifest2649files remained byte-identical throughout.
Evidence in the primary workspace: target/ui-full-preflight-20261116-r2.log,
-manifest.json and -result.json. Unit talos-ui-full-preflight-20261116-r2 reports
success, CPU482.718s, memory peak2GiB and swap0; prescribed CPU100%/MemoryHigh2G/
MemoryMax3G/TasksMax128/Nice10 and C locale were retained. Memory soft-limit reclaim
occurred without OOM; this is capped validation, not a performance measurement.

Source implementation is ready for staged-diff review and stable candidate creation.
Exact-commit independent review/native CI and native terminal acceptance remain open;
no product merge, installation, release, or whole-handoff completion is claimed.
