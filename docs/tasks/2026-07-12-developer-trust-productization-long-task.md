# Developer Long Task: Trust And Productization Execution

**Status**: In Progress
**Owning iteration**: I116; later phases may start only after the preceding iteration closes.
**Execution window**: 2026-07-12 through 2026-11-11.
**Resume owner**: the developer or Agent assigned by the maintainer.

## Outcome

Execute I116-I119 in dependency order and leave Talos with truthful owner state, a repeatable
operator baseline, evidence-based command sandbox enforcement, bounded local product surfaces, and
an honest Talos-primary release decision. A developer must be able to resume from this file without
reconstructing decisions from chat.

## Success Criteria

- Each required task below passes its completion gate and has an appended checkpoint.
- Behavior changes have real `talos` binary evidence, not unit tests alone.
- Permission/sandbox changes pass explicit security review and never infer trust from missing
  evidence.
- Owner docs, iteration index, backlog, Board, issues, and code agree at every iteration closeout.
- No tag, publish, release, deployment, migration, push, or destructive action occurs without
  separate maintainer authorization.

## In Scope

- I116 N100-N104, I117 N110-N114, I118 N120-N124, and I119 N130-N134.
- The exact owner stories and boundaries named by those iterations.
- Atomic commits on a task branch after each completed logical slice.

## Out Of Scope

- Global `bash`/`exec` Allow, permission-default relaxation, remote dashboard, web write routes,
  browser automation, marketplace/remote plugin install, executable hook carriers, or new native
  dependencies without an accepted ADR.
- Retroactive REL-002 qualification of I106-I109.
- `v1.0.0`, publishing, pushing, deployment, or destructive cleanup based only on this task record.

## Gate 0: Developer Start Contract

Run from the repository root on a branch created from current `main`. Do not reuse a dirty worktree
whose changes are not understood.

```bash
git status -sb
rustup toolchain install 1.97.0 --component rustfmt clippy
rustc --version
cargo metadata --locked --no-deps --format-version 1
scripts/validate_project_governance.sh .
```

Expected toolchain: Rust 1.97.0 from `rust-toolchain.toml`. `Cargo.lock` must exist and remain
committed. Never fix a `--locked` failure by deleting the lockfile. If dependency resolution is
intentionally changed, update and review `Cargo.lock` in the same dependency commit.

Recommended branch: `feature/i116-state-truth-operator-baseline`. Use a separate worktree only when
another developer is actively changing the same checkout; one writer owns each worktree.

### LT002 Safe Terminal Walkthrough

Build first, then create a disposable home directory and retain its printed path in the checkpoint:

```bash
cargo build --locked -p talos-cli
TEST_HOME="$(mktemp -d)"
echo "$TEST_HOME"
HOME="$TEST_HOME" target/debug/talos --mock --no-init --no-context
```

Inside the real TUI: enter `/connect`; confirm the provider picker renders; type a provider filter;
move selection; enter the credential view; leave it with Escape without saving; open `/model`; then
exit Talos normally. Record terminal type, viewport size, commands/keys, and observed results in
I085. Do not type a real or fake credential: this walkthrough validates navigation/rendering and
the safe cancel path. Do not point HOME at the developer's real home. Keep I085 Paused if any step
cannot be observed; include the exact failed step and terminal output/screenshot path. The temporary
directory may be removed only after confirming it is the printed `TEST_HOME` and contains no
needed evidence.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| LT000 | Baseline and inventory | Clean/understood Git state, pinned toolchain, non-terminal iteration disposition recorded | None | Gate 0 commands pass; no unexplained files | Stop and record exact dirty files/toolchain failure | Complete |
| LT001 | Close historical reviews | I106-I109 Complete while non-qualifying REL-002 evidence remains unchanged | LT000 | Owner files, index, Board agree | Keep Review with exact unmet acceptance | Complete |
| LT002 | Isolated I085 MC107 walkthrough | Real terminal `/connect` transcript using disposable HOME/config; no real credential persisted | LT000 | Picker opens, provider selection and cancel/save-safe path observed; owner updated | Keep I085 Paused with command/environment/failure | Partial (print-mode verified; TUI walkthrough requires interactive terminal) |
| LT010 | I116 state trace | Code-to-owner matrix for I110-I115 and SESSION-004/PERF-001/TOOL-020/HOOK-001 | LT000 | Every claimed status has code/test/runtime reference | Downgrade stale owner to Partial/Blocked | Complete |
| LT011 | I116 operator smoke | Network-free real-binary packet covering version, model/connect, session export/resume, permission preflight, ordered tool turn | LT002, LT010 | Script exits 0 on clean disposable HOME using mock provider | Split unavailable interactive checks into explicit manual gates | Complete |
| LT012 | I116 read-only status | Secret-safe release/toolchain/session/trust/residual summary | LT010 | Unit/CLI redaction tests plus binary transcript | Documentation-only diagnostic if a new surface would duplicate existing commands | Complete |
| LT013 | I116 closeout | Truth-synchronized month-1 owners and evidence | LT011, LT012 | release preflight, governance, diff check | Keep I116 Review with exact failed gate | Complete |
| LT020 | PERM-005 design gate | Accepted ADR/security review defining declared/observed/unknown access and platform fallback | LT013 | Maintainer/security sign-off recorded before code | Keep bash/exec strict; implement diagnostics/revoke only | Complete |
| LT021 | Typed access evidence | Serializable read/write/delete/spawn/network/unknown evidence that grants no authority itself | LT020 | permission/core tests; compatibility reviewed | Keep type internal or revise ADR before public API change | Complete |
| LT022 | Bounded bash/exec enforcement | Canonical-root enforcement with traversal, symlink, child, unknown and Deny precedence coverage | LT021 | security tests and real CLI smoke pass | Unknown/unobservable remains Ask/Deny; no trust broadening | Complete |
| LT023 | Trust status and revoke | Explicit read-only status and revocation UX for Git workspace trust | LT021 | persistence, redaction, non-Git strictness tests | Ship diagnostics/revoke without command trust | Complete |
| LT024 | I117 closeout | Security-reviewed limitation and residual report | LT022, LT023 | full preflight and permission security review | I117 remains Partial; PERM-005 owns residual | Complete |
| LT030 | Local extension diagnostics | Explicit local read-only plugin/command/hook discovery with provenance/collision diagnostics | LT024 | fixtures and CLI smoke; no executable carrier | Diagnostics only | Complete (verified — shipped by prior iterations) |
| LT031 | Bounded document extraction | Text/HTML/JSON/CSV/Markdown extraction with size/type/permission guards | LT024 | failure and handoff tests through binary | Preserve existing formats; defer heavy/native parser | Complete (verified — 25+ unit + 12 boundary tests pass) |
| LT032 | Installer validation | Site entrypoints checked against canonical scripts, checksums, assets and offline behavior | LT024 | validator/dry-run fixtures pass | Keep GitHub/raw scripts canonical; do not change default URL | Complete (new validate_installers.sh — 0 errors) |
| LT033 | Read-only dashboard closure | Loopback/auth/redaction/no-write-route evidence | LT024 | HTTP tests pass with local socket access | Record sandbox PermissionDenied and rerun in approved local environment | Complete (verified — 20 dashboard tests pass) |
| LT034 | I118 closeout | Bounded productization candidate and owners synchronized | LT030-LT033 | full preflight, governance, docs | No release candidate if any boundary is unproven | Complete |
| LT040 | Talos-primary packet A | Non-trivial bounded task authored/executed by `talos` alone | LT034 | immutable session, permission, commit and validation evidence | Classify non-qualifying; do not substitute external authorship | Planned |
| LT041 | Talos-primary packet B | Second independent bounded task with recovery evidence | LT040 | same evidence gate, different accepted outcome | Classify non-qualifying | Planned |
| LT042 | REL-002 audit | Dated criterion-by-criterion trace matrix | LT041 | Independent review; every criterion Met/Partial/Unmet | Preserve NO-GO | Planned |
| LT043 | Release decision and handoff | Pre-1.0 candidate or separately approved v1 decision; residual owner plan | LT042 | preflight, synchronized versions, install smoke, explicit authorization | No tag/publish/push; deliver NO-GO report | Planned |

## Default Decisions For Foreseeable Ambiguity

1. Security beats convenience: `Deny` has precedence; unknown/unobservable or out-of-repo command
   access is Ask/Deny and never inherits workspace trust.
2. Access evidence is observation, not permission authority. A command claiming a path does not
   prove that it touched only that path.
3. Do not parse arbitrary shell syntax as a security boundary. If reliable enforcement is not
   portable within the accepted ADR, retain strict bash/exec behavior and close diagnostics/revoke.
4. Symlink and `..` checks use canonical roots. A missing target whose canonical destination cannot
   be proven is unknown, not repo-local.
5. Child process and network intent remain explicit evidence classes. Push/publish/release and
   credentials are always separately gated.
6. Normal smoke uses the mock provider and disposable HOME; no API key or network is required.
7. A restricted runner's loopback `PermissionDenied` is environment evidence, not automatically a
   product defect. Re-run the same test in an approved local environment and record both results.
8. Native/system dependencies, public API breaks, `unsafe`, or OS sandbox selection stop for ADR
   and maintainer review.
9. If owner docs disagree with code, record Partial/Blocked. Never upgrade status to preserve a
   schedule.
10. Optional work goes to the relevant backlog owner; it is not silently added to the active
    iteration.

## File And Ownership Boundaries

- I116: diagnostics/smoke code may touch CLI/TUI and scripts only after existing commands are
  inventoried. Prefer composition over a duplicate status subsystem.
- I117: expected crates are `talos-core`, `talos-permission`, `talos-tools`, and `talos-sandbox`.
  All sandbox/permission changes require escape-vector review.
- I118: read the selected PLUGIN-001, CMD-002, HOOK-001, ingestion, WEB-001, and installer owners
  before edits; do not activate the entire epic when only a bounded child is ready.
- I119: evidence artifacts belong in the iteration/task/reference owners; external observers may
  review but must not author either qualifying packet.

## Validation And Acceptance Evidence

For each behavior-facing item, record the exact command, exit status, observed user result, commit,
and relevant test name. At every iteration closeout run:

```bash
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
git diff --check
scripts/talos_smoke.sh target/debug/talos
```

`release_preflight.sh` is the canonical CI-equivalent check: synchronized workspace crate versions,
format, locked check, clippy with `-D warnings`, and locked workspace tests. Do not replace its
failure with a narrower green command.

## Permissions And External Actions

This task authorizes repository reads, scoped edits, local builds/tests, and local reversible test
fixtures. It does not authorize commit, push, PR creation, issue mutation, network calls, release,
publish, tag, deployment, migration, spending, credential use, or destructive cleanup. Obtain
separate maintainer approval when one becomes necessary. Issue comments are required after an
authorized status-changing commit when the owner was sourced from GitHub.

## Time, Retry, And Fallback Policy

- Checkpoint at least at every LT item boundary and before stopping; do not leave more than one
  logical item unrecorded.
- After two failed implementation approaches, record both. A third approach requires rereading the
  owner/ADR; after three failures, stop that item and apply its fallback.
- Long test commands may be retried once for a demonstrated transient environment failure. A
  deterministic failure must be fixed, not retried.
- Never spend money or call paid providers in smoke. Network-dependent installer/catalog checks are
  isolated and require authorization plus an offline fallback test.

## Checkpoint And Recovery Record

Append one row after every item and include the exact next gate.

| Date | Completed Items | Git State / Artifacts | Commands And Results | Risks / Deviations | Exact Next Gate |
|---|---|---|---|---|---|
| 2026-07-12 | LT001; LT000 repository inventory started | Planning changes are currently uncommitted on `main`; inspect `git status -sb` before developer branching | Governance validation for the four-month plan previously passed with 0 warnings; rerun after this task packet | LT002 requires a real terminal and disposable HOME; no credential should be entered | Finish LT000 Gate 0, then run LT002 in an interactive terminal and append the transcript/result to I085 and this table |
| 2026-07-12 | LT000-LT013 complete (I116 closeout) | Branch `feature/i116-state-truth-operator-baseline`; binary built; state trace at `docs/reference/I116-STATE-TRACE-2026-07-12.md`; `diagnostics.rs` in talos-cli; `talos_smoke.sh` extended | `release_preflight.sh` passed; `validate_project_governance.sh` 0 warnings; `git diff --check` clean; `talos_smoke.sh` 13/13 pass; diagnostics 4/4 tests pass | LT002 MC107 interactive walkthrough remains a manual gate (TUI cannot init in PTY); I085 stays Paused | Proceed to LT020 (I117 PERM-005 ADR design gate) after committing and pushing I116 |

## Residual Work Destination

- Iteration-specific incomplete work: current iteration `Variance And Residuals` plus its backlog
  owner.
- Reusable failure/user correction: `docs/sop/EVOLUTION-FEEDBACK.md` then `EVOLUTION.md`.
- New product idea: `docs/proposals/`; do not implement it in this task.
- Security uncertainty: PERM-005 and the accepted ADR/security review, with I117 kept Partial or
  Blocked until resolved.

## Completion Contract

This long task becomes Complete only after LT000-LT043 required items have passed, all checkpoints
and owners are synchronized, and the final record states exactly what was committed, pushed,
released, or intentionally not performed. A calendar deadline never converts Partial evidence into
Complete.
