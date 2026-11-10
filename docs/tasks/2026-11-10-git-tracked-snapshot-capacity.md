# GIT-SNAPSHOT-001: Restore structured Git operations in ordinary-sized repositories

> Document status: Review / Claimed (emergency local candidate)
> Emergency local candidate authorized by the maintainer; delivery completion not claimed.

## Incident and scope

User reported `git_checkout {branch: "feat/bg-default-continuation", create: true}`
refused with `Git source snapshot limit exceeded; no operation prepared`, and requested
repair as a business availability failure. The inspected repository has about 32.8 MiB
of tracked regular content, beyond a shared 16 MiB body-snapshot limit. Checkout and
Commit both capture every tracked source during preparation and revalidation. This is
not a Git branch-name error, user denial, or ignored target-directory growth.

Required reads: AGENTS.md; docs/sop/AGENT-COLLABORATION.md;
docs/sop/LOCAL-DEV.md; docs/iterations/I300-structured-git-repair.md;
docs/backlog/active/GIT-001-embedded-git-tools.md; git_write/snapshot.rs,
prepared_add.rs and prepared_add_tests/checkout.rs.

Deliverable: structured Checkout create/switch and Commit operate on repositories
with tracked content above 16 MiB without retaining every source body in memory.
Safety acceptance: stale content (including same-length edits with restored mtime),
file type and mode changes remain invalidating; symlink bodies are not dereferenced;
resource exhaustion fails before mutation. No permission facets, admission stages,
Deny policy, helper execution, or final command authorization may be weakened.

## Design and constraints

- Retain SHA-256, byte length and permissions for tracked regular files; use existing
  gix SHA-256 support, no dependency or public API changes. Use a 64 KiB streaming
  buffer, bounded initial-length reads and identity checks around opening/reading.
- Keep recursive Add, configuration, ref and executable byte evidence unchanged.
  Digest evidence is not accepted as reference/configuration evidence.
- Hard: content must still be reread at the existing revalidation points. This is
  TOCTOU reduction, not atomic filesystem confinement or a claim of race elimination.
- Soft: retain the existing 4096-entry bound; separate scan I/O from retained memory
  with a shared 1 GiB per-capture content allowance. This deliberately bounds hashing
  work, not a 1 GiB allocation. Errors identify the file and remaining allowance.
  These limits still constrain larger repositories; do not claim unlimited scale.
- Operation-specific source-set minimization is deferred. This repair retains the
  conservative source set and fixes the body-retention failure without changing
  which source changes invalidate authorization. Add's recursive 16 MiB cap is also
  unchanged and is not represented as solved by this follow-up.

## Ownership and delivery gates

This follows closed I300/GIT-001 scope but does not reopen its completion evidence.
The local patch is an uncommitted experiment under the collaboration SOP. Before
committing implementation, establish a new effective claim on the target branch;
check live owners/iterations and overlapping Git work. Keep other active background
continuation, Server and search/provider work untouched. No implementation branch,
PR, activation or completion SHA is claimed here.

Because retained evidence participates in prepared authorization, independent safety
review of source substitution, link/special-file handling, budget exhaustion and
stale-plan refusal is required before merge. Local self-review is not that gate.
Native Windows execution and exact-head CI also remain required for delivery.

## Verification record

- Red: a 17 MiB tracked-file checkout fixture reproduced the original rejection.
- Green: the repaired fixture creates a branch and commits through existing admitted
  stages. It also covers switching back to an existing branch.
- Streaming tests cover same-length edits after the first chunk with restored mtime,
  shared I/O exhaustion, permissions, symlinks and FIFO rejection.
- A retained-plan test edits a large file after preparation and verifies checkout
  fails without changing HEAD or creating the requested branch.
- Initial `cargo test --locked -p talos-tools --features git --lib`: 393 passed.
- Final `./scripts/release_preflight.sh`: exit 0, including both governance validators,
  locked workspace check/Clippy/tests and the external runtime SDK fixture.
- `cargo xwin check --locked -p talos-tools --all-features --tests --target
  x86_64-pc-windows-msvc`: exit 0. Cross-compilation is not native execution.
- `git diff --check`: passed. Local ephemeral logs are
  `/tmp/talos-git-snapshot-repro.log`, `/tmp/talos-git-snapshot-tests.log`,
  `/tmp/talos-git-snapshot-preflight.log`, `/tmp/talos-git-snapshot-xwin.log`.
- Local boundary review: regular files use no-follow opens and leaf identity checks;
  unexpected source types/changes fail closed; original permission facets and execution
  stages are preserved. This self-review does not replace independent security review.

Remaining work belongs here: effective claim and independent
review before committed delivery; native Windows CI; actual user-tool acceptance.
README describes the resulting bounds. Do not mark Complete based on this document.

## Follow-up: auxiliary evidence capacity (same business failure)

The user's next retry passed tracked-body preparation but failed after approval with
the generic auxiliary identity/Read/limit message. A read-only diagnostic against the
actual repository reproduced this: repeated ancestor records across attribute/ignore
paths exhausted the 4096-entry resolver budget. `/etc/gitattributes` was absent, but
its absence could not be recorded with zero entries remaining. Human approval was not
missing. The original local fixture only covered file size, not repository path depth
and directory count; the earlier repair alone was therefore insufficient.

The candidate now shares immutable Arc component records across one auxiliary
frontier, retaining traversal order for every path. Every reuse rechecks the recorded
identity; new stages retain the same cache and budget. Unique identity records still
consume entry allowance; traversal slots, map records and path strings consume byte
allowance. No global cache, budget reset, permission inheritance or skipped exact leaf
Read is introduced. Other resolver callers use isolated caches. Limits are unchanged.
Errors now include the source path and failure stage for leaf capture, missing Read,
content/path revalidation and identity reservation.

Evidence: actual repository read-only discovery/admitted auxiliary capture passed;
a temporary clone of the actual repository completed structured branch creation via
the existing approved stages (27.98 seconds including clone/test). Original repository
HEAD was never changed by these diagnostics. Disposable diagnostics were removed;
permanent regressions use 160 nested module directories through full checkout and
reject ancestor replacement during extension. Focused suite: 395 passed.
Final combined `./scripts/release_preflight.sh` passed (exit 0): governance,
format/check/Clippy, locked workspace tests and external runtime fixture. Final
`cargo xwin check --locked -p talos-tools --all-features --tests --target
x86_64-pc-windows-msvc` passed without new warnings. Logs:
`/tmp/talos-aux-preflight.log`, `/tmp/talos-aux-xwin.log`,
`/tmp/talos-aux-real-clone.log`, `/tmp/talos-aux-diagnostic.log`.
These are local validation results; effective claim, independent security review,
remote/native Windows execution and user retry acceptance remain uncompleted.

## Follow-up: identify stale sources

The user subsequently reached human approvals and received the generic source-change
error. Concurrent local development in the same working directory is a plausible
trigger, not proof of the exact live mismatch. Revalidation now separates structural
source/submodule/parent changes from content evidence mismatches; content mismatch
names the first changed path without outputting file contents. The large-file stale
plan regression asserts `large.bin` appears and the branch/HEAD remain unchanged.
The conservative tracked-source set and invalidation rules remain intact. Do not
claim unrelated dirty working files are ignored or retry an old plan automatically.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Git tracked-source digest and auxiliary evidence capacity, stale-path diagnostics |
| Claimed At | 2026-11-10 |
| Source Issue | None |
| Governance Claim PR | Direct commit 457e1ac9 |
| Authorization Mode | Emergency override |
| Authorization Evidence | Maintainer explicitly answered “授权本次紧急本地提交” in this session after review of the concrete local repair branch; repeated Git business unavailability and unusable approval/input UI. Local commit only, not merge authority. |
| Implementation PR | #903 |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | Reconcile claim on target branch within two business days; independent security/API review and exact-head CI before merge. |

Incident scope is limited to this owner's Work Slice. Branch:
`fix/git-tui-availability`, base `b1dbb983`. Preserve the user's separate
`feat/bg-default-continuation` checkout. Earlier uncommitted/Unclaimed notes describe
investigation history and are superseded for local commit authority by this record.
Rollback: revert this slice's implementation commit on the repair branch; do not
reset or clean the user's active worktree. Native Windows and real terminal manual
acceptance remain deferred; local MSVC compilation is not native execution.

## Emergency local commit validation

- `RUST_TEST_THREADS=1 ./scripts/release_preflight.sh`: exit 0, including locked
  workspace check/Clippy/tests and external SDK fixtures. Serial test scheduling
  avoids the unrelated shared-host fixture races recorded above; no tests skipped.
- `cargo xwin check --locked -p talos-cli -p talos-tui -p talos-tools --all-features
  --tests --target x86_64-pc-windows-msvc`: exit 0 without new warnings.
- TUI focused suite: 650 passed. Git write focused suite: 304 passed (other tools
  covered by full preflight). Checkout operation presentation regression passed.
- Both repository governance validators passed in the isolated repair worktree;
  scale assessment reports high-risk / release-managed / required worktrees.
- Staged source matches the validated original workspace byte-for-byte. Staged
  diff reviewed; `git diff --cached --check` passed. No unrelated proposals staged.
- Logs: `/tmp/talos-operation-complete-preflight.log`,
  `/tmp/talos-operation-complete-xwin.log`, `/tmp/talos-commit-governance.log`.
- Independent merge review, remote/native Windows CI, and final human terminal
  acceptance are outstanding. No completion claim is made by the local commits.

Local implementation commit: `457e1ac9`. This records local preservation only; status remains Review.

## Main-branch consolidation

The maintainer subsequently requested consolidation into main and cleanup of all
safely removable branches. PR #903 carries this emergency repair and its claim
reconciliation. Independent security/API review and exact-head CI remain required
before merge; device/manual acceptance remains in this owner and status stays Review.
The final combined source matches the full preflight and MSVC cross-check candidate;
both governance validators passed after merging main `f9bfac77`.
