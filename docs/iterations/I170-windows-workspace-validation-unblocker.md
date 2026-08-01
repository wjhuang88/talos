# Iteration I170: Windows Workspace Validation Unblocker

> Document status: Review
> Published plan date: 2026-08-01
> Planned objective: restore the still-missing Windows-native shell and cross-platform workspace validation behavior preserved by recovery PR #121, without mixing I169 steering semantics or restoring obsolete registration code.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Windows Talos presents and executes the `powershell` tool while Unix keeps `bash`/`sh -c`, with one absolute timeout and portable file/test projections.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Thinking / talos recovery session 2026-08-01 |
| Work Slice | I170 only: Windows PowerShell process boundary, absolute shell timeout, child environment scrub, portable path/long-list projections, and cross-platform fixture corrections on current main. |
| Claimed At | 2026-08-01 |
| Source Issue | #119 (dependency recovery context; I170 remains an independent implementation slice) |
| Governance Claim PR | #122 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Repository owner explicitly instructed implementation to begin after the current-main recovery audit on 2026-08-01; claim merge passed exact-head CI, both governance validators, remote reconciliation, merge-time CAS, and no blocking review feedback. |
| Implementation PR | #126 |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Release only by an explicit maintainer handoff or after PR #126 is merged and completion evidence is recorded. |

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
- Historical branch: `recovery/pr-78-i170-20260731`; it must not be rebased, rewritten, or used for continued development.
- Governance claim merged at `455bfbd5c5316862675aa68c62f1b62bff2e5cc7`; PR #126 starts from that exact current-main baseline.
- The implementation branch merged current `main@5520800be081447663cb690ba78fd17d1374e01e` without rebasing or rewriting its existing commits.
- TUI-044/I169 current owner chain was established by merged governance PR #123 at `ba5a564b4be81366bc3d3e68fd83e7c1d8ce3a4f`; I170 remains an independent prerequisite implementation slice.
- Recovery classification: behavior remains missing; old CLI registration shape is superseded by the current `talos-tools` contribution and outer composition architecture.

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

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- focused `talos-tools`, permission, MCP and CLI shell tests on Unix and Windows
- direct-child completion plus descendant-held-pipe timeout regression on Unix and Windows
- `git diff --check`
- `scripts/validate_project_governance.sh .`
- `scripts/validate_collaboration_claims.sh .`
- `./scripts/release_preflight.sh`
- rebuilt Unix mock smoke and rebuilt Windows PowerShell walkthrough
- exact-head Windows and Unix/macOS CI
- independent process/security review before merge

### Documentation To Update

- `docs/backlog/active/TOOL-023-A-bash-timeout-fix.md`
- `docs/backlog/active/TOOL-023-C-windows-powershell.md`
- `docs/decisions/057-windows-powershell-process-boundary.md`
- `docs/iterations/README.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- README and public capability surfaces only after behavior exists

### Risks And Rollback

- Risk: platform naming diverges from permission or contribution identity. Gate every inventory and wrapper surface.
- Risk: direct-child exit does not imply inherited pipes are closed. Keep the absolute deadline active until direct-child completion and both pipes finish; at expiry return without waiting for descendant-held EOF.
- Risk: direct-child kill does not supervise descendants. Keep that residual explicit; do not claim process-tree termination.
- Risk: restoring historical registry code creates duplicate contributions. Use only current contribution factories.
- Rollback: revert the I170 implementation commits and retain the prior Unix-only shell baseline; never rewrite the recovery branch.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-01 | Recovery audit | Current main still hardcoded `sh -c`, reset timeout sleeps inside the output loop, lacked Windows child env scrub and portable path/long-list behavior. Recovery PR #121 remains archival. |
| 2026-08-01 | Claim | Governance PR #122 merged after exact-head release preflight, Windows fixture, collaboration and remote owner validation. |
| 2026-08-01 | Activation | Created `fix/i170-windows-shell-portability-20260801` from exact `main@455bfbd5c5316862675aa68c62f1b62bff2e5cc7`; no recovery branch was modified. |
| 2026-08-01 | Implementation slice | Added platform shell identity/process construction, child env scrub, portable `ls`/grep/glob paths, conservative Windows long-list projection, and Agent support for PowerShell output. |
| 2026-08-01 | Current-main refresh | Merged `main@5520800be081447663cb690ba78fd17d1374e01e` into the implementation branch as a normal two-parent merge after Issue #124/#125 reconciliation. |
| 2026-08-01 | Validation correction | Earlier Windows full-workspace failures exposed platform-sorted inventory expectations and document fixtures outside their explicit workspace. Expected inventories now sort independently, successful document fixtures execute inside their own workspace, and external-path rejection remains unchanged. |
| 2026-08-01 | Security correction | Audit found that waiting for pipe EOF after direct-shell exit can let descendants extend the operation beyond its deadline. The active correction keeps one deadline over both process completion and pipe closure and adds a descendant-held-pipe regression. |
| 2026-08-01 | Automated review gate | Implementation Head `1ca536159c34437719e4f776db2e02e4afc8510d` passed CI run `30686493121`: macOS release preflight; Windows format/check/Clippy, focused PowerShell/permission/deadline tests, complete locked workspace tests, project governance, collaboration claims and rebuilt CLI mock smoke; remote Issue/Owner reconciliation; and the Windows installer fixture. |
| 2026-08-01 | Review handoff | PR #126 remains Draft until this documentation synchronization Head repeats the same exact-head gates and independent process/security plus maintainer review accepts ADR-057 and the direct-child residual. |

## Verification Evidence

- Implementation Head `1ca536159c34437719e4f776db2e02e4afc8510d` passed CI run `30686493121` with all four jobs green.
- macOS evidence: `git diff --check` plus release preflight, including locked workspace format/check/Clippy/tests and governance validators.
- Windows evidence: format, locked workspace check, Clippy, focused native PowerShell/environment/permission/continuous-output/descendant-held-pipe deadline tests, full locked workspace tests, project governance, collaboration claims, and rebuilt CLI mock smoke.
- Remote evidence: all 26 open Issues reconciled to owner documents and the Windows installer fixture passed.
- Earlier failing Windows inventory and document-boundary fixtures were repaired without deleting, ignoring, weakening, or bypassing external-path checks.
- This Review synchronization commit must repeat the exact-head CI gates before the PR is made ready. Recovery PR #121 evidence is not reused as current proof.

## Completion Evidence

- Completion Commit: pending.

## Variance And Residuals

- ADR-057 and all historical validation claims remain proposed evidence only until re-established on a fresh exact head.
- Process-tree supervision remains outside I170 and must not be inferred from direct-child timeout handling.
- The absolute operation deadline must remain bounded even when an unsupervised descendant inherits the direct shell child's stdout/stderr handles.

## Retrospective

- Pending.
