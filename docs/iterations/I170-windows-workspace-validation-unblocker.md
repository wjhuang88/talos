# Iteration I170: Windows Workspace Validation Unblocker

> Document status: Complete (2026-08-01)
> Published plan date: 2026-08-01
> Planned objective: restore the still-missing Windows-native shell and cross-platform workspace validation behavior preserved by recovery PR #121, without mixing I169 steering semantics or restoring obsolete registration code.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Windows Talos presents and executes the `powershell` tool while Unix keeps `bash`/`sh -c`, with one absolute timeout and portable file/test projections.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Completed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / talos recovery session 2026-08-01 |
| Work Slice | I170 only: Windows PowerShell process boundary, absolute shell timeout, child environment scrub, portable path/long-list projections, and cross-platform fixture corrections on current main. |
| Claimed At | 2026-08-01 |
| Source Issue | #119 (dependency recovery context; I170 remained an independent implementation slice) |
| Governance Claim PR | #122 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The repository owner authorized implementation, accepted the final exact-head review outcome and explicitly authorized readiness and merge on 2026-08-01. |
| Implementation PR | #126 — merged |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Satisfied by merged PR #126 and the exact completion evidence recorded below. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TOOL-023-A` | `TOOL-023` | Ready | None | Shell timeout is one absolute deadline and preserves bounded output. |
| `TOOL-023-C` | `TOOL-023` | Ready | `TOOL-023-A` | Windows uses a PowerShell process boundary while Unix behavior remains unchanged. |
| I170 portability slice | None | Recovery audit: still needed | Current tool contribution and permission architecture | Windows-visible paths, long-list output, CRLF and platform fixtures are deterministic without weakening tests. |

### Recovery Provenance

- Historical recovery PR: #121, archival only and never mergeable as-is.
- Historical exact head: `e1da5dd893418a3f6e3737ec900aabe9967b1dda`.
- Historical branch: `recovery/pr-78-i170-20260731`; it remains immutable and was not used for continued development.
- Governance claim merged at `455bfbd5c5316862675aa68c62f1b62bff2e5cc7`.
- Fresh implementation PR #126 was built on current-main architecture and merged independently of recovery PR #121.
- TUI-044/I169 remains a separate implementation slice; I170 completion only satisfies its Windows/current-main prerequisite.

### Scope

- Keep Unix/non-Windows `bash`, `sh -c`, ADR-007 pre-exec hardening, permission nature and shell family unchanged.
- Present `powershell` on Windows and invoke `powershell.exe -NoLogo -NoProfile -NonInteractive -Command`.
- Remove dangerous inherited environment names in child command configuration on every platform without mutating the parent process.
- Enforce one absolute deadline across direct-child execution and stdout/stderr completion; output activity and descendant-held pipe handles may not extend it.
- Integrate the platform shell through the single authoritative `talos-tools` contribution path and preserve outer permission wrapping and presentation filtering.
- Normalize protocol-visible workspace-relative file/search paths to `/`.
- Define deterministic Windows long-list type/permission projection without inventing Unix ownership, link count, or executable bits.
- Repair CRLF comparisons, Unix-only symlink fixtures, and hard-coded Unix temporary-directory test assumptions without deleting, ignoring, or weakening assertions.

### Non-Goals

- No I169 steering, session, scheduler, persistence, provider-budget, or TUI queue behavior.
- No POSIX-to-PowerShell translation, `cmd.exe` fallback, shell-selection setting, PowerShell parser, Job Object claim, descendant process-tree supervision, timeout default change, permission bypass, second shell contribution, or old registry restoration.
- No direct merge, rebase, or modification of recovery PR #121 or its branch.

### Acceptance

- Windows exposes exactly one permission-wrapped shell contribution named `powershell`; Unix exposes exactly one named `bash`.
- Windows executes native PowerShell commands and returns stdout, stderr, exit status and bounded timeout evidence.
- Unix continues to execute through `sh -c` with the existing hardening contract.
- One absolute timeout governs spawn/output/wait and pipe completion; output activity and descendant-held handles cannot extend the deadline.
- Dangerous environment names are absent from the child and remain unchanged in the Talos parent.
- Windows recursive `ls`, glob and grep paths use `/`; long listing begins with one type character and nine conservative permission characters.
- Cross-platform fixtures compile and run without broad skips or reduced assertions.
- Current contribution inventories, permission routing, MCP/CLI presentation and shell output compression remain coherent on both platforms.
- Windows reusable templates fail closed for computed PowerShell expressions and retain only the reviewed inert-token allowlist.

### Validation Matrix

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- focused `talos-tools`, permission, MCP and CLI shell tests on Unix and Windows
- direct-child completion plus descendant-held-pipe timeout regression on Unix and Windows
- `git diff --check`
- project-governance and collaboration-claim validators
- release preflight
- rebuilt Windows direct PowerShell walkthrough and Windows CLI mock smoke
- exact-head Windows and Unix/macOS CI
- process/security and maintainer review before merge

### Risks And Rollback

- Risk: platform naming diverges from permission or contribution identity. The final inventory and wrapper tests passed on both platforms.
- Risk: direct-child exit does not imply inherited pipes are closed. The one absolute deadline remains active through pipe completion and returns without waiting indefinitely for descendant-held EOF.
- Risk: direct-child kill does not supervise descendants. That residual remains explicit and outside I170.
- Risk: restoring historical registry code creates duplicate contributions. PR #126 used only the current contribution factories.
- Rollback: revert merge commit `592254d73a98166df48da0139a02df67e9cd2cd6`; never rewrite a recovery branch.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-01 | Recovery audit | Current main still hardcoded `sh -c`, reset timeout sleeps inside the output loop, lacked Windows child env scrub and portable path/long-list behavior. Recovery PR #121 remained archival. |
| 2026-08-01 | Claim | Governance PR #122 merged after exact-head release preflight, Windows fixture, collaboration and remote owner validation. |
| 2026-08-01 | Activation | Created `fix/i170-windows-shell-portability-20260801` from the exact current-main claim baseline; no recovery branch was modified. |
| 2026-08-01 | Implementation | Added platform shell identity/process construction, child env scrub, portable `ls`/grep/glob paths, conservative Windows long-list projection, and Agent support for PowerShell output. |
| 2026-08-01 | Timeout correction | Kept one deadline across direct-shell completion and pipe closure and added continuous-output plus descendant-held-pipe regressions. |
| 2026-08-01 | Permission review blocker | Review found that parenthesized/computed PowerShell expressions could receive a reusable cwd template under the prior denylist. |
| 2026-08-01 | Permission correction | Replaced the Windows denylist boundary with an inert-token allowlist and added exact-resource regressions for grouping, call, array, variable, member/index and adjacent expression forms. |
| 2026-08-01 | Current-main alignment | Removed the completed one-shot workflow from the branch so current workflow-inventory governance remained authoritative; no product behavior was changed by this alignment. |
| 2026-08-01 | Exact-head validation | Final implementation Head `8cfe8edb2dbda581244f583fb809591391a54298` passed CI run `30705366763` (`CI` run 718) on macOS and Windows. |
| 2026-08-01 | Review acceptance | The exact-head re-review closed the prior permission blocker and recommended approval. The repository owner then explicitly authorized readiness and merge. |
| 2026-08-01 | Completion | PR #126 was squash-merged into `main` as `592254d73a98166df48da0139a02df67e9cd2cd6`. |

## Verification Evidence

- Final implementation Head: `8cfe8edb2dbda581244f583fb809591391a54298`.
- Exact-head CI: run `30705366763` (`CI` run 718), all jobs successful.
- macOS evidence: approved workflow inventory, diff whitespace validation, release preflight, locked workspace format/check/Clippy/tests and governance checks.
- Windows evidence: format, locked workspace check, Clippy, focused native PowerShell/environment/permission/deadline tests, full locked workspace tests, project governance, collaboration claims, rebuilt CLI mock smoke and installer fixture.
- Remote evidence: open-Issue/Owner reconciliation passed.
- Exact-head walkthrough artifact: `8820174164`, named `i170-windows-direct-walkthrough-8cfe8edb2dbda581244f583fb809591391a54298`.
- Artifact digest: `sha256:7bf5936d6b390588b082197f877d5c8c1b8fe6414973b4bb49a6f291c92e42d2`.
- Walkthrough recorded matching exact/checked-out Heads, stdout, stderr, working directory, exit code `7`, retained pre-timeout output, `[timeout]`, bounded elapsed time, no post-timeout completion output, direct-child cleanup, and the direct-child-only residual.
- The reviewed `fetch_url` corrections remained test-only; production proxy behavior was unchanged.

## Completion Evidence

- Completion Commit: `592254d73a98166df48da0139a02df67e9cd2cd6`.
- Implementation PR: #126, merged 2026-08-01.
- Exact implementation Head: `8cfe8edb2dbda581244f583fb809591391a54298`.
- Final exact-head CI: `30705366763`.
- Windows walkthrough artifact: `8820174164`.
- Accepted decision: ADR-057.
- Completed Stories: TOOL-023-A and TOOL-023-C.
- Recovery PRs #120/#121 and their branches remain archival and untouched.

## Variance And Residuals

- The Windows reusable-template boundary is intentionally conservative. Safe but unusual arguments containing spaces, Unicode or unreviewed punctuation may receive exact prompts.
- PowerShell grammar-aware reusable permissions require a separately reviewed lexer/parser decision.
- Windows PowerShell 7 selection remains outside this iteration.
- Process-tree supervision remains outside I170. Talos guarantees timeout cleanup of the direct shell child, not the entire descendant process tree.
- I169/TUI-044 is now eligible for fresh selection from current main, but no I169 implementation was started by this closeout.

## Retrospective

- Recovery branches are useful provenance but cannot substitute for a fresh current-main implementation and exact-head evidence.
- Cross-platform process work needs production-path Windows CI, not only installer fixtures.
- Shell permission reuse must be based on a reviewed allowlist/parser boundary; extending syntax denylists is not a defensible PowerShell security strategy.
- Timeout acceptance must cover process completion and inherited pipe behavior while stating descendant cleanup limits precisely.
- Governance closeout belongs in a separate post-merge change so implementation evidence and completion-state transitions remain auditable.
