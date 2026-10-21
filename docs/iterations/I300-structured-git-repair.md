## Current convergence plan — 2026-10-21

Latest evidence: published `3b00e98e` passed complete local release preflight;
exact-head CI 39587933025 is terminal failure on Windows (20-minute workspace
test timeout), with macOS full validation passing. The native full-Push baseline
times out before Talos dispatch, so Job supervision is not established as the
cause. Initialized-child Checkout also hits the stage limit. Test-only follow-up
preserves sender logs on timeout and reports Checkout mode and admission profiles.
Host selections pass; native confirmation remains required. See the terminal
comparison checkpoint below. This is diagnosis, not Windows/Pull activation or
completion of any package.

This is the current execution entrypoint. Earlier checkpoints are preserved evidence,
not additional acceptance requirements. The published Scope And Acceptance remains
binding. I300/T7 is **Active / Claimed**; Windows production Git and production Pull
remain disabled. This plan does not grant new permissions or activate later tasks.

Planning closure ledger: outcome = a finite delivery path for the existing repair;
artifacts = this owner and the handoff coordinator; preserve = published baseline,
accepted ADRs, historical evidence and uncommitted code; synchronization = owner first,
coordinator second (Board state unchanged); checks = governance, collaboration and diff;
remaining implementation/evidence = the five work packages below. No new iteration,
branch mode or worktree policy is selected by this planning update.

Planning validation (2026-10-21): repository and skill governance validators pass
with zero warnings; collaboration claim validation passes with zero warnings;
git diff --check passes. A table verb initially triggered the completion-status
heuristic; clarified the pending action and reran successfully. No Rust changes
were made or product tests rerun for this planning slice. I300 remains Active.

### Diagnosis and constraints

Confirmed: local HEAD b37026e3 contains three commits after remote candidate 7036ddce;
CI 39444015640 at that earlier candidate failed three Windows tools tests, with further
production-gate failures. Local auxiliary-capture changes are not ready: compilation
and 31 host tests do not establish cancelled-ancestor/leaf identity binding. The daemon
Push timeout root cause is unknown. Ordinary configured Pull behavior remains incomplete.
Inference: the current loop optimizes individual defensive checks without a finite
activation path; accumulated code/test counts therefore overstate delivered progress.

Hard: authorization before effects, retained identity, bounded/cancellable process
ownership, original configured-behavior acceptance, real production entrypoints and
platform evidence. Soft: work-package order and candidate size may change on evidence.
Assumption: existing authorization/evidence/process machinery can serve the remaining
paths without a new general framework; verify this in P1 rather than designing one now.

### P1 caller and authorization map — 2026-10-21

- Sender discovery publishes auxiliary path Read facets in prepared_add.rs, but
  those normalize spelling; they do not cover the full filesystem-order chain.
- config_state auxiliary overlap helpers perform metadata/canonicalize/read_link;
  capture_auxiliary then captures Entry bytes without a token parameter or retained
  leaf identity. These are separate resolution/admission and evidence gaps.
- execute_admitted captures before attribute probes and compares afterwards, but
  further Checkout/receiver probes precede mutation. Final auxiliary evidence must
  survive and be checked after those probes, immediately before spawn.
- target_config::Sources checks its complete frontier before capture and retains
  directory/regular evidence, but only for its own Checkout source set. It rejects
  links and is not a general authorization for the auxiliary helpers.
- Existing pull_resolution provides component/link/metadata-only handle machinery;
  its current caller authority is Pull-pointer-only and its directory-target contract
  must not be silently reused for file leaves or other operations.

Independent reviewer git_plan_review confirmed the authority gap and the minimal
separate capability on 2026-10-21. ADR-089 contains the concrete auxiliary resolution contract, accepted by the
maintainer on 2026-10-21 through the explicit full-scope confirmation. Target configuration
link support remains a distinct unaccepted proposal. No compiler/runtime activation
has been made. Implement the accepted scope through actual callers; do not publish the incomplete
capture patch as P1 closure. Independent Windows timeout diagnosis may continue.

### P1 accepted capability compiler checkpoint — 2026-10-21

The maintainer accepted the full auxiliary-resolution proposal. The permission
compiler now recognizes only its fixed metadata Read capability for the five
structured mutation tools with an absolute executable and source Read anchor.
An actual negative test exposed generic Network/Remote acceptance of that name;
the compiler now reserves it and rejects other natures rather than relying on
the generic pair table. Independent review also caught trim-normalized Network/Internal spellings;
reserved-name checking now matches compiler trimming and rejects padded variants.
No prepared runtime publishes or consumes it yet.

Tests cover missing/relative anchors, unrelated tools, wrong nature, unknown
capability and changed tool/command/source set/input fingerprints. The existing
and new discovery tests are in grant_discovery_tests.rs, keeping test code out
of the production module. `cargo test -p talos-permission --locked` passes all
148 unit, 11 integration and one doc test after the split. Existing Pull grants
remain unchanged. Normalized fingerprints alone cannot distinguish original
traversal spellings: runtime must bind the frozen source set to PreparedPlanIdentity.

Remaining P1 work: reuse component resolver for file endpoints under the accepted
capability, enforce at every real traversal call site, publish actual target/directory
Read frontiers, retain shared budgets and identities across continuation/probes,
and revalidate immediately before spawn. Preserve the eight-stage bound: fold
resolution into an existing admitted discovery phase where possible; do not silently
raise the limit or break ordinary local Push's existing eight-stage path. Native
Windows and production-chain review/activation still follow the package exit gates.

Fetch completed with no upstream-only main commits (322 local-only at e67a654d),
so no rebase was needed. The auxiliary capture patch remains uncommitted and is
not evidence of P1 closure. Both governance validators pass with zero warnings, formatting/diff checks pass,
and final independent static review found no remaining compiler-stage blocker.
The full permission suite was rerun successfully after the whitespace fix.

### P1 runtime integration checkpoint — 2026-10-21

This checkpoint supersedes the earlier statements that the accepted auxiliary
capability has no runtime consumer. Changes remain local and uncommitted; P1
and I300 remain Active, with Windows production Git and production Pull disabled.

Sender mutation now captures actual-target Read evidence through AuxiliaryPaths
and revalidates it after probes immediately before mutation. Checkout target
configuration/hooks and newly discovered child sources extend the retained resolver
without resetting its budget. Checkout deletion and target overlap checks consume
retained traversal paths instead of independently traversing the filesystem.
Receiver auxiliary files, configuration origins and declared includes now join the
same invocation resolver. Receiver hook/alternate-store checks consume its snapshots;
configuration comparison uses retained discovery bytes rather than reopening raw
spellings. Receiver storage checks retain access to that same evidence through final
execution. These changes do not extend the separate Pull resolution authority.

Validation: locked git-feature git_write tests pass 416/416 on the host after the
receiver configuration migration (log /private/tmp/i300-p1-receiver-config.log).
The preceding Checkout selection passed 50/50. One sandbox run denied process
observation; the full suite was rerun outside the sandbox. A receiver hook diagnostic
regression was corrected before the passing run. These results are not Windows
runtime, full-workspace preflight, independent security review or merge evidence.
The earlier cargo xwin check failed with disk exhaustion; authorized incremental
cache cleanup freed space, but Windows checking has not yet been rerun.

Follow-up validation: a direct prepared receiver skiplist regression passes
(1/1, /private/tmp/i300-p1-receiver-regression.log), covering missing exact target
Read and same-content inode replacement after capture. Authorized cleanup of
rebuildable target/debug artifacts restored about 14 GiB free space. Pinned
cargo xwin check -p talos-tools --features git --tests --target
x86_64-pc-windows-msvc --locked now passes with incremental/debug information
disabled (/private/tmp/i300-p1-shared-xwin.log). It reports existing unused-code
warnings; it does not provide native Windows runtime evidence. Both governance
validators passed with zero warnings at the preceding runtime checkpoint.

Remaining P1: audit dynamic child attribute sources and all source-set/token
frontiers; expand receiver traversal/continuation coverage beyond the direct
skiplist regression; remove superseded capture code and dead methods; run permission and
Windows checks plus independent boundary review. P2's native timeout cause and
P3–P5 acceptance remain open. Do not infer package completion from test counts.

P1 follow-up (2026-10-21): the completed host git_write run passes 417/417
(/private/tmp/i300-p1-child-external-final.log). Dynamic child worktree
`.gitattributes` still uses snapshot capture inside inspect_child_filters; it
must migrate after child metadata/index admission, not by scanning the index
early. The earlier early-scan attempt was withdrawn after its admission-order
regression failed. No dynamic-source closure is claimed.
Removed the unused resolver source-set helper and restricted legacy auxiliary
leaf-capture characterization helpers to tests. Resolver regressions pass 6/6
with the locked git-only feature selection
(/private/tmp/i300-p1-resolver-cleanup.log), including a link redirected between
Path approval and resolution: the old canonical-target grant is rejected and a
fresh target grant succeeds. An initial assertion incorrectly assumed Path
approval did not canonicalize links; inspection of authorization.rs corrected
the test premise. This is uncommitted local evidence, not independent review,
Windows runtime validation or P1 completion.

P1 dynamic child attribute candidate (2026-10-21): mutation dispatch now checks
all child worktree/gitdir/common Read grants before inspecting child indexes.
Index-selected worktree attribute paths extend the invocation resolver and its
shared budget; new sources return a fresh plan and exact resolved-target Read
frontier before body capture or check-attr. Already-bound source sets reuse the
current admission without adding a stage. Child inspection no longer captures
or reopens dynamic attributes through raw snapshot paths; resolver evidence
owns their identity and body revalidation. Protected Checkout paths retain only
the admitted nonmissing attribute names. The eight-stage ceiling is unchanged;
deep transition combinations still need P3 usability validation.

The first full git-only host run passes 418/418
(/private/tmp/i300-p1-child-continuation-host.log). Its sandbox predecessor had
417 passes and one process-observation PermissionDenied, superseded by the host
run. A separate real-child regression passes 1/1 and checks external target
Read denial, fresh approval and same-content inode replacement refusal
(/private/tmp/i300-p1-child-attribute-target.log). Permission tests pass all 148
unit, 11 integration and one doc test (/private/tmp/i300-p1-child-permission.log).
MSVC xwin checking passes with seven existing reduced-feature warnings
(/private/tmp/i300-p1-child-continuation-xwin.log); the initial sandbox attempt
could not update the cargo-xwin clang-cl cache link and was rerun with approved
host permissions. A final full run after the no-new-source stage optimization
passes 419/419 (/private/tmp/i300-p1-child-continuation-final.log). Final MSVC
xwin checking also passes (/private/tmp/i300-p1-child-continuation-xwin-final.log),
with seven existing warnings. Both governance validators pass with zero warnings
(/private/tmp/i300-p1-child-governance.log and /private/tmp/i300-p1-child-claims.log).
Independent boundary review, remaining receiver coverage, native
Windows evidence and P1–P5 delivery gates remain open. Changes are uncommitted.

Independent current-source static boundary review found no blocking defect in
the dynamic-source admission, same-plan optimization, replacement checks or
Checkout protection. It did not run tests or establish atomic TOCTOU/native
Windows guarantees. All-target/all-feature talos-tools Clippy with locked
dependencies and -D warnings passes (/private/tmp/i300-p1-child-clippy.log),
after correcting seven int-plus-one/needless-borrow diagnostics in this local
batch. Current-head workspace preflight and native Windows remain required.

Stage checkpoint: admitted capability-token plumbing is committed separately as
91155adf. Runtime integration is now committed as 710582ec. Standard
release_preflight.sh session 46729 stopped at formatting differences before workspace tests
(/private/tmp/i300-p1-runtime-preflight.log). After cargo fmt --all, a new
standard run in session 9773 terminated with exit 101 during workspace test
compilation because the disk was full
(/private/tmp/i300-p1-runtime-preflight-final.log). This is not a passing
workspace gate. After confirming no Cargo/rustc/preflight process was running,
the previously authorized rebuildable target/debug artifacts were removed;
available space recovered from 115 MiB to approximately 14 GiB. A fresh standard
run in session 97575 used CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0
and CARGO_PROFILE_TEST_DEBUG=0. The preceding execution recorded terminal exit 0
and `release preflight: passed`, including workspace tests and both runtime SDK
fixtures. The resumed session handle is unavailable; this result is inherited
terminal evidence, not the disk-failure log above. Staged review completed before
710582ec. After rebase, rerun the full gate with a durable log for the new source.
Native Windows and P1–P5 are still open.

### Finite delivery packages

P2 diagnostic checkpoint (2026-10-21): the isolated Git-daemon Push fixture now
also captures packet tracing, distinguishing receive-pack dispatch from protocol
progress. Only the fixed test receiver is traced; logs are surfaced on failure.
The locked git-feature host regression passed 1/1 in 1.30 seconds. This does not
establish the Windows timeout cause or validate a native fix; a native Windows
run with this diagnostic is still required. Production policy, deadlines and
Windows/Pull activation gates are unchanged. The incomplete auxiliary capture
patch remains outside this test-only stage.

These are execution packages inside I300, not five new Stories or revised acceptance.
Each stays open until its exit evidence exists; none is complete at this checkpoint.

| Package / dependency | Runnable outcome and remaining work | Exit evidence / fallback |
|---|---|---|
| P1 common authorization boundary / first | Finish auxiliary dependency admission, filesystem-order resolution, retained directory/link/leaf or absence evidence, and shared resource limits. Map existing Sources, PathIdentity and RegularEvidence to actual callers before adding code. | Denial before reads, cancelled-ancestor and same-content replacement, missing-to-present and link-redirection regressions; independent authorization review; Windows MSVC check plus native tests. Keep affected execution disabled if authority is insufficient; present the concrete contract gap together. |
| P2 Windows execution / P1 | Make the existing supervised structured Git path work on Windows, including actual receiver Push and cancellation. Resolve the daemon timeout from a minimal native reproduction; verify receiver shell/install baseline and auxiliary capture fixes together. | Real transfer writes the approved receiver ref; deny/replay/identity/cancel tests and terminal cleanup receipts pass; production Agent/Runtime/MCP tests pass after reviewed activation. Keep gate closed on unexplained failure; never substitute a longer timeout or compile success. |
| P3 Checkout transitions / P1, P2 for Windows evidence | Finish initialized-child Checkout transitions and configured effects through the existing admission chain, including target hooks/configuration and worktree/metadata boundaries. | Native success, denial before effects, identity changes and partial/failure behavior for the full transition matrix on supported platforms; protected review. Record an explicit unsupported case rather than claiming the whole transition complete. |
| P4 complete Pull / P1, P3; P2 for Windows activation | Close selected current/target/intermediate attributes and executable filter/merge-driver authority, default multihead helper effects, parent and initialized-child fetch, then merge/rebase integration. Reuse accepted per-repository and worktree-scope grants. | Ordinary configurations including unused global LFS remain usable; selected programs require their own grants; recursive fetch, conflicts, cancellation and partial effects are truthful through real Agent/Runtime/MCP entrypoints. Review and enable the complete three-gate chain only when safe. Conservative broad refusal is containment, not the final exit. |
| P5 delivery / P1–P4 | Reconcile every published acceptance row, original two-file staging fixture preserving unrelated index data, diff fidelity and configured mutation behavior; update user docs and publish the repair. | Current-head full release_preflight, Unix/Windows CI, protected review and merge-time CAS; README languages/GIT-001/owner/Board synchronization; PR #735 or explicitly recorded successor merged; already-pushed implementation SHAs before Complete. |

P2's minimal timeout diagnosis may be investigated while P1 is under review, but no
second implementation stream or new authorization is implied. The immediate sequence
is P1 caller/authority map and contract decision if needed, P1 implementation and tests,
then one coherent Windows candidate for P2. Do not resume isolated capture patches
before that map explains how admission and evidence reach the mutation boundary.

### Keep implementation and decisions bounded

- Every new finding must cite an existing acceptance row or protected boundary and be
  assigned to P1–P5. A reproducible escape, lost data, unmanaged process or supported
  behavior regression blocks its package. Optional capabilities and gix replacement
  research belong to their existing owners, not the T7 critical path. Unclear findings
  receive one bounded reproduction/call-chain investigation before classification.
- Implement shared boundary behavior once where actual callers need it. Keep operation
  sequencing in operation modules and tests in test modules. Avoid a new policy engine,
  generic plugin mechanism or speculative refactor. Remove only superseded code/tests.
- Existing human decisions persist. Pull-only pointer resolution does not authorize
  general auxiliary traversal; the proposed target-link capability is still unaccepted.
  If exact Path Read cannot express P1's reads, present one concrete grouped decision:
  exact operations, affected paths, bounds, denied cases, reuse, compatibility impact
  and alternative. No silent widening or asking the same decision again.
- Human judgment may select a documented operation scope or explicitly approve a
  compatibility tradeoff. It cannot replace programmatic token enforcement, identity
  validation, supervision or final evidence. Changes to promised supported behavior
  require an explicit baseline amendment; leaving a gate closed does not finish T7.
- Stop speculative patching when two successive attempts at the same blocker add no
  new causal evidence. Summarize the failing trace, falsified hypotheses and smallest
  next experiment; ask the maintainer only if a real contract/scope/environment choice
  remains. This is a scheduling checkpoint, not a waiver or automatic task deferral.

### Candidate and reporting cadence

Use one logical commit per reviewed change, preserving current local work. Run focused
checks during implementation; batch independent review around a coherent boundary.
At a stable package candidate, fetch/reconcile main, run required local preflight and
MSVC cargo xwin checks, then publish once for native exact-head CI. Native-only diagnosis
may use a clearly labeled diagnostic candidate; it is not a merge-ready delivery stage.
Inspect failures before retrying; substantive corrections require fresh head evidence.

A stage may merge only if its advertised behavior is independently complete and all
applicable gates pass. PR #735 is accumulated work: do not call it a narrow Windows-only
stage or merge it on the strength of one passing sub-suite. If a safe subset is worth
separating, first inventory its dependencies and review scope; no blind cherry-picking.
An intermediate merge never closes remaining I300 acceptance rows.

Report: production capability enabled, package exits passed, named blockers, next
experiment/decision and evidence head. Do not estimate completion from commits, lines
or test totals. No date estimate is justified until P1's authority and P2's timeout
root cause are resolved. All later handoff tasks retain their serial dependencies.

---

Current Windows supervision checkpoint (2026-10-18): maintainer accepted
ADR-068 foreground extension to reuse the existing Job implementation for
structured Git. Independent review found direct-child-only cleanup on non-Unix
could leave Commit/Push descendants writing after cancellation. Preparation now
refuses on non-Unix before discovery, and the runner independently refuses its
old fallback. This is containment only; Job integration, lossless environment,
stdin/output bounds, terminal cleanup receipts, Windows runtime evidence and
fresh independent review remain required. T7 stays Active; production Pull stays
disabled. No stage merge is claimed. Details and resume steps are in the handoff.

Pull child remote discovery checkpoint (2026-10-16): admitted child
configuration now exposes only the last `remote.<name>.url` value through a
bounded parser. The Pull configuration stage adds a Network facet for the
selected child `origin` URL without contacting it; invalid/non-UTF-8/empty
values refuse. Pull tests pass 76/76 and cargo check passes
(`/private/tmp/i300-child-remote-discovery.log`). Remote helper validation,
child object admission, recursive fetch and integration remain open.

Pull child configuration checkpoint (2026-10-16): `PullChildIdentity` now
transitions active children into `PullChildConfiguration`. Each child requests
exact Read admission for its identity-bound common-dir `config`, captures the
configuration through the existing bounded host inspection, and only then
returns to integration selection. Pull tests pass 76/76 and cargo check passes
(`/private/tmp/i300-child-config.log`). Include-chain expansion, remote-specific
admission, child object checks and recursive execution remain open.

Pull child identity checkpoint (2026-10-16): after admitted activity
resolution, selected active children now enter a separate `PullChildIdentity`
stage. It requests exact Read admission for each child worktree and `.git`
marker, then binds `ChildLocation`; missing/stale identity or absent admission
refuses before child effects. Locked Pull tests pass 76/76, cargo check passes,
and diff check passes (`/private/tmp/i300-child-identity.log`). Configuration
chain, child remote, object verification, recursive fetch and integration remain
open; T7 stays Active.

Correction (2026-10-16): the missing-attribute conclusions in the two checkpoints
below are incorrect and superseded. A native Git regression now stages root and
nested `.gitattributes`, removes both worktree files, and confirms ordinary
`check-attr` still returns the nested index attribute value. Missing worktree
rules therefore require index fallback; until implemented, activity resolution
refuses them. Missing system/global/info sources remain empty layers. The prior
76/76 and final-source Clippy claims were not supported by a full run at that
head; only the earlier 75-test run and a separate focused test were observed.
T7 remains Active; this correction is not completion evidence.

Index attribute fallback checkpoint (2026-10-16): the dedicated child-attribute
stage now probes only the already admitted index through bounded `ls-files --stage`
and `cat-file blob :path` calls. Missing index entries remain empty layers;
nonzero or noisy probes refuse. Locked Pull tests pass 76/76 and all-target
all-feature Clippy passes (logs `/private/tmp/i300-index-attribute-{fallback,clippy}.log`).
Child identity/config/remote authorization and recursive execution remain open.

Pull activity evaluation checkpoint (2026-10-16, `5251f731`): admitted
attribute snapshots now build a gix attribute search in the dedicated
`PullChildAttributes` stage and resolve `submodule.active` attribute pathspecs
to Enabled/Disabled. Missing `.gitattributes` files remain empty rule layers;
non-regular entries and parser failures refuse safely. Tests cover `attr`,
`-attr`, `!attr`, `attr=value`, parent/child/info precedence, exclusions and a
native Git `check-attr` comparison. Locked all-feature Pull tests pass 76/76;
all-target all-feature Clippy passes. Pull child identity/config/remote
admission, index-backed child source semantics, recursive child fetch and
integration execution remain open. Production Pull remains disabled; T7 stays
Active.

Attribute missing-file correction (2026-10-16): `f464b8df` keeps absent
`.gitattributes` evidence as an empty rule layer instead of demanding an index
fallback. Native `git check-attr` reads worktree attribute sources for ordinary
(non-`--cached`) inspection; index fallback remains limited to current
`.gitmodules` declarations. Pull tests pass 76/76 after the correction and
Clippy passes. This does not close T7: child identity/config/remote admission,
object checks/retries, nested fetch, integration execution, lifecycle budgets,
security review and delivery gates remain open.

Post-rebase verification (2026-10-16): local I300 history now descends from
origin/main `3bc38d78`; the attribute ancestor inventory is `1c0a386b` after
rebase. The staged child-attribute admission implementation passes locked
all-feature Pull tests (72/72), all-target all-feature Clippy with `-D warnings`,
workspace formatting, project governance (zero warnings), and collaboration
claim validation. Logs: `/private/tmp/i300-post-rebase-{pull-tests,clippy,fmt,governance,claims}.log`.
The Board conflict retained remote I301 completion and effective local I300
claim status. These are local checkpoint checks, not full preflight, independent
security review, remote CI or completion evidence. Attribute evaluation and all
previously listed T7 residuals remain open; Production Pull remains disabled.

Child attribute admission checkpoint (2026-10-16): after rebasing the local
implementation onto origin/main `7b33bc9f`, `3a484f47` retains lexical ancestor
attribute paths. Post-fetch discovery now requests those exact worktree paths
plus Git-reported system/global and common-dir info attributes in a fresh
PullChildAttributes stage. It checks the complete frontier before capturing
bounded regular-file/absence and parent evidence, then revalidates before and
after capture. Old tokens cannot cover newly discovered paths; a focused fixture
also exercises refusal with those tokens and detects later content tampering.
Locked all-feature Pull tests pass 72/72
(/private/tmp/i300-child-attribute-admission.log). This is admission and evidence
capture only: attribute matching, index fallback, child discovery/execution,
integration, full lifecycle budget/security/runtime evidence and publication
remain open. Production Pull remains disabled; no completion status changes.
Earlier local checkpoint SHAs below describe pre-rebase history and are not
pushed completion evidence.

Child selection policy checkpoint (2026-10-16, `e9321876`, `03da666e`):
current-index declared candidates now retain On/OnDemand eligibility by module
name, preserving unmapped paths for separately admitted legacy discovery. This
selection does not mark a module seen before an initialized child yields a task.
Historical candidates retain activity precedence: last per-name active boolean,
then submodule.active pathspecs, then valued URL presence. Ordinary pathspecs use
panic-guarded gix plumbing without filesystem discovery; glob, exclusion,
icase, literal, directory and absolute-root cases pass. Eight matching cases also
match isolated native Git 2.54 submodule-init outcomes (no network access).
Attribute pathspecs stay
explicitly unresolved, and invalid activity stays dormant until scheduling;
neither is silently treated as inactive. Locked all-feature Pull tests pass
70/70 and all-target all-feature tools Clippy passes (logs:
/private/tmp/i300-child-activity-tests.log and
/private/tmp/i300-child-activity-clippy.log). These are local policy tests, not
native recursive-fetch equivalence or delivery evidence. Remaining: admitted
attribute evaluation, legacy populated-path discovery, initialized/absorbed child
identity/config/remote admission, required-object checks and retries, nested
fetch, integration execution and original delivery gates. Production Pull stays
disabled; I300/T7 remains Active; later handoff tasks remain unactivated.

Current module source checkpoint (2026-10-16, `4d751699`): Pull now admits
worktree .gitmodules as an exact Read path before capturing regular-file or
absence evidence. Revalidation binds that evidence across parent fetch and
post-fetch discovery. Current declarations use overwrite semantics and reject
invalid recursion values immediately; historical first-value semantics remain
unchanged. Missing worktree declarations fall back to admitted index objects,
then HEAD objects. Includes are not followed. Real self-fetch coverage exercises
worktree/index/HEAD/absent sources and independent module/FETCH_HEAD tampering;
a missing exact Read token refuses before source capture. SHA-1/SHA-256 index
coverage retains the module blob and rejects a symlink declaration. Locked
all-feature Pull tests pass 67/67; all-target all-feature tools Clippy passes
(logs: /private/tmp/i300-current-modules-tests.log and
/private/tmp/i300-current-modules-clippy.log). Candidate active/pathspec selection,
legacy populated-path fallback, child identity/config/remote admissions, nested
fetch, integration and final delivery gates remain open. Production Pull stays
disabled; I300/T7 remains Active. This is local stage evidence, not delivery.

Current-index child checkpoint (2026-10-16, `34650804`): post-fetch
continuation now retains current index gitlinks separately from historical
changed children. Sparse index directories expand through bounded admitted
object probes; index evidence is checked before retaining candidates. Parsing
is panic-guarded, validates SHA-1/SHA-256 IDs and paths, and rejects unresolved
index stages and duplicate candidates without opening child paths. The real
self-fetch fixture retains an index-only child whose worktree directory is
absent. Locked all-feature Pull tests pass 65/65; all-target all-feature tools
Clippy passes (logs: /private/tmp/i300-index-children-tests.log and
/private/tmp/i300-index-children-clippy.log). Broader real sparse-expansion
runtime evidence, current .gitmodules mapping/selection, child admissions and
recursive fetch, integration, independent review and final delivery gates
remain open. Production Pull stays disabled; T7/I300 remains Active.

Child policy precedence checkpoint (2026-10-16, `55521958`, corrected by `f96af155`): retained
pre-fetch configuration now resolves changed named candidates using explicit
global mode first, last current per-module value second, historical declaration
third, then on-demand default. Module names remain case sensitive. Explicit
global modes bypass invalid per-module values; invalid current values remain
dormant pending candidate selection, whereas historical invalid values follow
native demand fallback. Selected invalid current values must refuse.
Post-fetch discovery retains each candidate's effective policy without opening
child paths. The real self-fetch fixture verifies the retained default; policy
regression covers overrides, duplicate values, casing, implicit/empty values
and invalid-value handling. Locked all-feature Pull tests pass 64/64 and
all-target all-feature tools Clippy passes (logs:
/private/tmp/i300-child-precedence-tests.log and
/private/tmp/i300-child-precedence-clippy.log). Current-index/active-policy
candidates, child identity/config/remote admissions, nested fetch, integration,
independent review and full delivery gates remain open. Production Pull remains
disabled; T7/I300 remains Active.

Historical recursion checkpoint (2026-10-16, `c1d47878`): admitted
.gitmodules blobs now retain first-per-module fetchRecurseSubmodules policy
independently of path declarations and repeated sections. Implicit true,
explicit empty false, on-demand, case-sensitive names and invalid historical
values are preserved; invalid historical values remain distinct rather than
aborting discovery. Named changed-child aggregation retains the first-seen
historical policy alongside commit/path and required objects. Source basis:
Git 2.54 submodule-config.c parse_fetch_recurse/config callback and submodule.c
get_fetch_recurse_config/fetch_task_create. Locked all-feature Pull tests pass
63/63; all-target all-feature tools Clippy passes. Logs:
/private/tmp/i300-historical-recursion-tests.log and
/private/tmp/i300-historical-recursion-clippy.log. Current-config/global
precedence, current-index candidates, child admissions/fetch, integration and
final delivery gates remain open. Production Pull stays disabled; T7 Active.

Historical declaration-order checkpoint (2026-10-16, `dadfa1de`): Pull
module parsing now consumes ordered gix config events, preserving the first
path declaration even when followed by an implicit duplicate. A first implicit
path still refuses atomically. Name-only declarations are retained for later
legacy-name collision checks. Parsing remains bounded, panic-guarded and does
not follow includes or open child paths. Cargo.lock uses gix-config 0.60.0;
older session notes referring to 0.58.0 are not current dependency evidence.
Locked all-feature Pull tests pass 62/62 and all-target all-feature tools
Clippy passes (logs: /private/tmp/i300-module-order-tests.log and
/private/tmp/i300-module-order-clippy.log). Child recursion policy, child
admission/fetch, integration, independent review and full current-head delivery
gates remain open; production Pull stays disabled and T7 remains Active.

Named child aggregation checkpoint (2026-10-16, `ad62ef9e`): post-fetch
discovery now aggregates declared changed modules by case-sensitive name,
retaining the first revision-walk commit/path and the deduplicated required
child object IDs across historical path changes. Missing tree evidence refuses.
The real self-fetch fixture verifies the retained module requirement; the pure
regression covers rename history, first-seen order and distinct name casing.
Undeclared gitlinks remain retained in tree evidence pending separately admitted
populated-path discovery; they are not silently declared ineligible. Locked
all-feature Pull tests pass 61/61; all-target all-feature tools Clippy passes.
Child recursion policy, current-index candidates and legacy name fallback,
identity/config/remote admissions, nested fetch, integration and final delivery
gates remain open. Production fetch remains disabled; I300/T7 stays Active.

Changed gitlink checkpoint (2026-10-16, `7ba1fbdd`): post-fetch discovery
now runs bounded recursive combined diffs for each newly reachable commit and
retains only new-side gitlinks at changed paths. Real two-parent merge fixtures
exclude links inherited from either parent and include a novel merge result;
the real self-fetch fixture covers a nested gitlink path. NUL framing, duplicate
and traversal failures preserve retained state. Configured diff.orderFile now
requires exact Read admission and retained path/content evidence even without
pre-fetch rebase cleanliness checks. Initial regressions exposed a missing -r
and a macOS canonical-path assertion; both were corrected before final validation.
Locked all-feature Pull tests pass 60/60 and all-target all-feature tools Clippy
passes. Child policy/name aggregation, initialized child identity/config/remote
admission, nested fetch, integration, runtime evidence, independent review and
full current-head preflight remain open. Production fetch remains disabled;
I300/T7 remains Active. This is a local implementation stage, not publication.

Historical module checkpoint (2026-10-16, `a1580b8d`): post-fetch tree
discovery now reads each .gitmodules blob by its validated object ID through
the owned runner, then parses it with gix without includes and with a panic
boundary. It retains case-sensitive path-to-name mappings, first path per name,
and refuses malformed/oversize declarations, traversal and ambiguous duplicate
paths atomically. It does not authorize child paths or use module URLs as remote
authority. The real self-fetch fixture verifies mapping without a worktree
.gitmodules or child directory. Locked all-feature Pull tests pass 59/59 and
all-target all-feature tools Clippy passes. Native changed-gitlink/policy
selection, child identity/config/remotes and fresh admission, nested fetch,
integration, runtime evidence, independent review and full preflight remain
open. Production fetch stays disabled; I300/T7 remains Active.

Child tree checkpoint (2026-10-16, `db4d9414`): admitted post-fetch discovery
now lists each newly reachable commit tree through the owned bounded runner,
retaining gitlink paths/OIDs and the root .gitmodules blob OID. The pure parser
validates SHA-1/SHA-256 IDs, mode/type pairs, framing, duplicate paths and path
traversal; symlink/gitlink .gitmodules is refused. It performs no child filesystem
access. A real self-fetch fixture retains a gitlink and module blob even though
neither the child path nor .gitmodules exists in the worktree. Locked all-feature
Pull tests pass 58/58; the subsequent graph-order regression and all-target
all-feature tools Clippy pass. Graph IDs now deduplicate without sorting, retaining
revision walk order needed for later first-seen child name/path association.
These are tree inventories, not native changed-gitlink selection or child fetch
plans. Module blob parsing, native change/policy selection, per-child identity and
approval, nested execution, integration, independent review, Agent/MCP and final
preflight remain open. Production fetch remains disabled; I300/T7 stays Active.

Fetched graph checkpoint (2026-10-16, `14e34ef4`): prefetch now retains validated
all-ref object tips; admitted post-fetch discovery traverses every FETCH_HEAD
tip, including not-for-merge entries, excluding the retained prefetch tips.
The resulting new commit set is retained for pending child tree discovery.
A real self-fetch of a dangling commit proves that commit is included while its
reachable parent is excluded. Graph listings accept only complete nonzero
SHA-1/SHA-256 IDs, bounded to 1 MiB; revision argv is bounded to 32 KiB and
refuses without dropping exclusions. Probes use the existing owned runner and
unchanged cumulative command/output budget. Locked all-feature Pull tests pass
57/57 and all-target all-feature tools Clippy passes (after fixing a test-only
cloned_ref_to_slice_refs warning). `d0a8d2ed` adds six native child-policy cases:
explicit global recursion overrides per-child values; only default global mode
uses child values. The initial opposite assumption failed the native fixture
and was corrected. Child tree/config discovery, per-child authorization/fetch,
integration writes, runtime evidence, independent review and final preflight
remain open. Production fetch remains disabled and I300/T7 stays Active.

Post-fetch Pull evidence checkpoint (2026-10-16, `6f710376`): fetched metadata
can now be rebound to the retained gitdir/common-dir/worktree identities after
the parent fetch changes refs and objects. The fresh capture checks the old
location guard before rediscovery, binds FETCH_HEAD and fixed evidence again,
and refuses changed location pointers or replacement roots. A real local-fetch
regression proves intended ref/object drift is accepted while changed
FETCH_HEAD, commondir redirection and gitdir replacement are rejected. The
Pull suite passes 53/53 and all-target all-feature tools Clippy passes. Approved
standard preflight passed against source `6f710376`: tools 754/754, workspace
checks/tests/Clippy, governance/claims, doctests and both external Runtime SDK
fixtures; log `/private/tmp/i300-policy-postfetch-preflight-approved.log`.
The sandboxed run reached 753/754 and hit only the known process-observation
`PermissionDenied`; log `/private/tmp/i300-policy-postfetch-preflight.log`.
Production fetch, child discovery/approval, integration, independent review and
Agent/MCP Pull evidence remain open. I300/T7 stays Active.

Recursive Pull policy checkpoint (2026-10-16, `e41d7ec1`): retained prefetch
evidence now distinguishes default, off, on and on-demand modes and respects
the declaration order of `submodule.recurse` and `fetch.recurseSubmodules`.
The PullFetch permission description states the retained mode and that the
parent command defers child effects. Real Git 2.54 fixtures confirm native
child-object imports for the four ordered on/off/on-demand cases, including
the test-fixture correction needed to preserve configuration declaration
order. Locked all-feature Pull tests pass 52/52; tools all-target all-feature
Clippy, formatting and diff checks pass. This does not select children, execute
fetch, or complete recursive Pull. Fresh post-fetch discovery, per-child plans,
integration and full current-head validation remain open. I300/T7 stays Active.

Pull recursive-child checkpoint (2026-10-16, `18beb0bc`): maintainer accepted
staged equivalent recursion. The fixed parent fetch now includes
`--no-recurse-submodules`; a native Git 2.54 fixture proves native parent fetch
imports a changed child object while the prepared parent fetch updates the same
parent remote-tracking ref and leaves the child object untouched. Locked Pull
tests pass 51/51 and tools all-target all-feature Clippy passes. This commits
only the execution boundary and evidence; production fetch remains disabled.
Post-fetch parent discovery, per-child identity/config/remote plans, fresh child
Read/Write/Network approvals, nested iteration, partial/cancel/budget handling,
independent review and Agent/MCP gates remain unimplemented. ADR-089 now records
the accepted timing difference and authorization contract. Standard full
preflight against source commit `01a03688` passed outside the sandbox: tools
752/752, workspace tests/checks/Clippy, governance/claims zero warnings and
external runtime SDK fixture; log
`/private/tmp/i300-recursive-preflight-approved.log`. The first sandboxed run
reached 751/752 tools tests and hit `PermissionDenied` only in the existing
process-observation case; log `/private/tmp/i300-recursive-preflight.log`.
I300/T7 remains Active.

Pull bundle-source checkpoint (2026-10-16, `205458e5`): configuration
preparation now refuses a nonempty or valueless final `fetch.bundleURI` before
fetch planning. A native Git regression demonstrates that a bundle imports
objects even when the selected remote subsequently fails; the prepared stage
refuses without importing the object or creating FETCH_HEAD. Empty final values
retain Git's bundle-disable semantics, including duplicate-value precedence.
Locked all-feature Pull tests pass 50/50; tools all-target all-feature Clippy and
formatting pass. This is a bounded authorization prerequisite, not completed
Pull execution. Full preflight for this source, recursive-submodule effect
analysis, production fetch/integration, independent review and Agent/MCP gates
remain open. The prior full preflight applies to `4fee02d5`, not this source.
Remote refresh confirms origin/main has no commits missing from this branch.
I300/T7 remains Active; no merge, push or completion is claimed.

Pull legacy-remote checkpoint (2026-10-16, `4fee02d5`): Pull now rejects
legacy `.git/remotes` and `.git/branches` destination files both before
configuration discovery and after fetch planning. Four prepared-stage cases
refuse before FETCH_HEAD/index mutation. Locked all-feature Pull tests pass
49/49; all-target all-feature tools Clippy and both governance validators pass.
Current-source sandboxed full preflight failed with 749 tools tests passing
and one process-observation `PermissionDenied` in
`prepared_local_push_runs_all_eight_admitted_stages`; log
`/private/tmp/i300-rebased-legacy-preflight.log`. An approved outside-sandbox
standard `release_preflight.sh` rerun passed against source commit `4fee02d5`
(750 tools tests passed and the external runtime fixture passed); log
`/private/tmp/i300-rebased-legacy-preflight-approved.log`. The sandbox-only
process-observation failure is environmental. This gate does not close Pull
fetch/integration, independent review, Agent/MCP evidence or publication.
Separate gix tracking commit `87340529` records release availability only.
Production Pull fetch/integration, independent review, Agent/MCP evidence and
publication remain open. I300/T7 remains Active.

Rebase checkpoint (2026-10-16): integrated `origin/main` at `c869a9af`
(PR #726) into `feat/i300-git-repair`. The append-position conflict in
session tests preserves both the upstream absent-journal regression and the
I300 prepared-process cleanup tests. Locked Agent session tests pass after
rebase; branch diff whitespace checks pass. Prior FETCH_HEAD implementation
`595d6f64` is now `cfe17104`; its owner checkpoint `497a6356` is now
`c5616b2f`. Earlier SHA references remain historical checkpoints, not new
exact-head validation. Pull production fetch/integration and fresh full
preflight remain open; no completion or publication is claimed.

Pull fetch destination checkpoint (2026-10-16, `595d6f64`): prefetch
evidence now binds FETCH_HEAD as well as post-fetch evidence. Previously a
stable symlink at that fetch output was outside the prefetch evidence set.
The new regression checks absent-file creation, regular-content drift and
external symlink refusal with an unchanged external sentinel. Locked
all-feature Pull tests pass 48/48; formatting and staged-diff checks pass.
The prior full-preflight pass predates this change. Production fetch and
integration remain disabled; fresh full validation and delivery gates remain
open, and I300/T7 remains Active.

Pull metadata redirection and full-gate checkpoint (2026-10-16,
`e7ef0081`): added direct regressions for a discovered ref directory
replaced by a symlink and for a redirected metadata root before its first
enumeration. Locked all-feature inventory selection passed 7/7; the staged
diff and formatting checks passed. The standard `release_preflight.sh` rerun
outside the sandbox completed successfully with the current source tree,
including workspace tests and external runtime fixtures
(`/private/tmp/i300-low-disk-preflight-approved.log`). Its sandboxed attempt
failed only at process observation with `PermissionDenied` (745 tools tests
passed, one environmental failure). Pull production fetch/integration is still
disabled; independent review, Agent/MCP execution evidence and publication
remain open. I300/T7 stays Active and the rest of the handoff queue is intact.

Pull continuation capacity checkpoint (2026-10-16, `14ddb555`): removed
remaining per-path approval continuations for internal absent tracking evidence
and redundant graph namespace facets. The admitted closed metadata scope covers
those paths; worktree/attribute admission remains separate. The shared runner
regression now requires reaching fetch within three discovery stages, with both
a ten-level ref namespace and an absent tracking reflog; creating that reflog
after planning invalidates retained evidence. Locked all-feature Pull tests pass
45/45 and all-target all-feature tools Clippy passes (before the final test-only
assertion). Fetch/integration remains disabled and all delivery gates remain
open. Fetched origin/main remains `0ef5a532`; rebase reports up to date. Existing
cache-cleanup authorization was used for target/debug/incremental after available
space fell to about 2 GiB. Standard preflight then failed with ENOSPC during
workspace compilation; both governance validators had passed with zero warnings.
Under the existing broader authorization, target/debug rebuildable artifacts
were cleared for a retry with incremental compilation and debug symbols disabled.
No workspace-preflight pass is claimed; no source or Git data was removed.

Pull scoped graph discovery checkpoint (2026-10-16, `2c867bb3`): production
prefetch discovery now requires the closed metadata Read facet and exact Read
tokens for both identity-bound repository roots, then completes the bounded
metadata inventory within one admitted stage. Descendant depth no longer causes
one approval continuation per directory. Worktree sources and attribute files,
including common-dir info/attributes, retain separate exact Read admission.
The real shared-stage-budget regression includes a ten-level reference namespace;
a missing-capability regression refuses after configuration discovery. Locked
all-feature Pull library tests pass 45/45, all-target all-feature tools Clippy
passes, and diff checks pass. Independent security/API review, current-head
workspace preflight and Agent/MCP evidence remain open. Fetch and integration
remain disabled; the next implementation slice must bind fresh post-fetch Read
and integration Write plans within the unchanged eight-stage bound. I300/T7 and
the complete handoff queue remain unfinished; this commit is local/unpublished.

Pull current-branch fetch checkpoint (2026-10-16): implementation commit
`cd94435e` adds native Pull's `--update-head-ok` to the fixed fetch command;
regression commit `d819eb8c` verifies with real Git that an explicit current-branch
refspec updates HEAD while leaving worktree reconciliation for the next stage.
Both focused locked, all-feature library tests passed, as did formatting and
diff checks. These tests cover command construction/native behavior only.
Production fetch/integration remains disabled; the incomplete continuation draft
was withdrawn. Fresh post-fetch admission, integration execution, stage-budget
closure, runtime evidence and current-head preflight/review remain required.
I300/T7 remains Active. The branch is rebased on fetched `origin/main` at
`0ef5a532`; these two stage commits are local and not yet published.

Pull helper-admission checkpoint (2026-10-15, commit `093edd1a`): Pull
configuration now refuses selected-remote upload-pack and VCS helpers before
fetch planning. Active automatic maintenance refuses a configured
`gc.recentObjectsHook` or nonempty `gc.repackFilterTo`; disabling
`maintenance.auto` keeps those settings inactive. The locked all-feature
`talos-tools` Pull library selection passes 43/43, including the new regression,
and formatting/diff checks pass. `origin/main` was fetched at `0ef5a532`;
`git rebase origin/main` reported up to date. This is a bounded pre-fetch
correction, not production fetch or integration. The prior full preflight
predates this commit, so current-head preflight, independent review, platform
evidence and publication remain open. I300/T7 stays Active.

Final-source gate checkpoint (2026-10-15; unpublished): approved standard
`release_preflight.sh` completed outside the sandbox with exit 0 (session 8083).
It covered governance/claims, the workspace tests including the 699-test
`talos-tools` batch, MCP Git transports, doctests, and both external runtime
SDK fixtures. The process-observation stage test also passed outside the
sandbox with all features (session 52128). The prior sandbox `PermissionDenied`
was an environment restriction. T7 remains Active because compatibility,
Pull/Push residuals, platform evidence, independent review, and publication
gates are not closed.

Pull boundary checkpoint (2026-10-15; unpublished): the complete feature-enabled
Pull preparation test selection passed 17/17 outside the sandbox (session 62796).
It covers configuration admission, graph/ref and sparse-index evidence, filters,
promisor and rebase refusal paths, source drift and shared budgets. This confirms
the bounded discovery/refusal contract only; `Phase::PullFetch` now validates
the accepted metadata Write admission but still refuses before spawning fetch
because post-fetch and integration execution are not yet enabled.

Pull contract acceptance checkpoint (2026-10-15; unpublished): the maintainer
accepted the dedicated `git:pull-repository-metadata` Read/Write contract.
Acceptance authorizes implementation of that closed capability; it does not
authorize generic metadata access or enable fetch/integration before fresh
effect binding, review, and Agent/MCP denial/success evidence are complete.

Pull capability compiler checkpoint (2026-10-15; unpublished): the accepted
`git:pull-repository-metadata` facet now compiles only for `git_pull` requests
with a bound absolute Git command and same-nature absolute repository roots;
the focused permission test passes. The prepared executor consumes the facet
only to validate the PullFetch proof; fetch/integration remains disabled pending
effect binding and runtime evidence.

Pull permission regression checkpoint (2026-10-15; unpublished): the complete
`talos-permission` discovery test group passes 4/4 and all-target all-feature
Clippy passes after enabling the accepted Pull capability compiler scope.

Pull request-facet checkpoint (2026-10-15; unpublished): prepared `git_pull`
requests now carry the accepted metadata Read facet alongside the existing
repository-root Reads. The 17-test Pull preparation selection still passes;
the facet is not yet consumed by fetch or integration execution.

Pull fetch-write profile checkpoint (2026-10-15; unpublished): the
`PullFetch` continuation now carries a dedicated metadata Write facet plus
identity-bound gitdir/common-dir Write facets, while explicitly excluding
worktree Write. The sparse-index stage-budget regression passes; execution
still refuses before spawning fetch until the admitted fetch command and
post-fetch evidence/revalidation are implemented.

PullFetch profile regression checkpoint (2026-10-15; unpublished): complete
Pull tests pass 17/17 after updating the former no-Write assertion to the new
stage contract. The test now permits only identity-bound metadata Write facets,
rejects Network and all unrelated Write facets, and Clippy passes for both
`talos-permission` and `talos-tools` with all targets/features.

PullFetch admission checkpoint (2026-10-15; unpublished): the stage now
consumes and validates its one-use proof, requiring Write admission for both
identity-bound repository roots and the dedicated Pull metadata capability.
Missing or stale proof fails closed; a valid proof still returns the explicit
unimplemented fetch/integration error, so no host fetch is spawned yet.

Historical-state note (2026-10-15): older entries below that call the Pull
metadata contract proposed or unanswered record the state at their timestamps.
They are superseded by the maintainer acceptance above and ADR-089's accepted
contract heading; they remain useful historical evidence and do not imply that
production fetch/integration is delivered.

Pull fetch command checkpoint (2026-10-15; unpublished): a production-used
fixed command builder now binds the absolute Git executable, admitted worktree,
resolved remote, foreground maintenance overrides and Trace2-disabled frozen
environment. Newline remotes and relative executable paths refuse. Its pure
argv/environment test passes. The command is constructed only after fetch
admission, but is not spawned until fetch effects and post-fetch evidence are
fully bound. A full disk condition required clearing the previously approved
rebuildable `target/debug/incremental` cache (1.3 GiB); the focused test then
passed with `CARGO_INCREMENTAL=0`.

Pull post-fetch evidence checkpoint (2026-10-15; unpublished): existing bounded
`PullEvidence::capture(..., fetched=true)` tests bind FETCH_HEAD together with
HEAD/tracking reflogs, index and merge-state metadata, and reject changed
FETCH_HEAD during later integration probes. This evidence layer is ready for a
production continuation, while the integration selector remains test-only and
no fetch has been spawned.

Pull fetch-builder compile checkpoint (2026-10-15; unpublished): `cargo check
--locked -p talos-tools --all-features` passes with `CARGO_INCREMENTAL=0` after
adding the fixed fetch command builder and PullFetch admission checks. No
network or host fetch was invoked.

Pull transport-admission checkpoint (2026-10-15; unpublished): Pull preparation
now resolves the configured fetch destination from retained, revalidated Git
configuration. A credential-free built-in `git://` destination appears as its
own Network(Remote) facet in the PullFetch profile; other transports remain
unsupported and fail closed at dispatch. The 19-test Pull selection passes and
`cargo check --locked -p talos-tools --all-features` passes. This closes the
previous gap where the prepared fetch profile had metadata authority but no
separate network approval target. It does not execute fetch, authorize helpers,
local source reads, credentials or redirects, or implement integration. Fetch
and integration, their failure/cancellation evidence, MCP coverage, review and
publication remain open; T7 remains Active.

Pull hook-boundary checkpoint (2026-10-15; unpublished): Pull configuration
discovery now derives the identity-bound `reference-transaction` and
`pre-auto-gc` hook paths from the retained `core.hooksPath` (or gitdir hooks
default). PullFetch requires exact Read admission for those paths and captures
their regular-file evidence immediately before any future fetch; executable
hooks and configured fetch helpers remain fail-closed. The existing Pull
selection remains green and no hook or fetch process is started. Production
fetch/integration, MCP coverage and final review/publication remain open.

Pull transport refusal regression checkpoint (2026-10-15; unpublished):
unsupported HTTPS destinations remain visible as exact Network(Remote) facets
for review, while PullFetch refuses before command construction or FETCH_HEAD
mutation because helper/credential/redirect effects are not bound. Executable
hook and unsupported-transport cases are included in the 21-test Pull selection,
which passes; production fetch remains disabled.

Initialized-child integration checkpoint (2026-10-15; unpublished): MCP Checkout
passes 40 Unix scenarios (7972, `/private/tmp/i300-initialized-mcp.log`). Combined
initialized new child, target configuration and old-child deletion refuses before
mutation at the unchanged eight-stage bound; other initialized allow/deny/replay
and target-configuration cases pass. Real Git production-command ancestor-prefix
regression passes (37219, `/private/tmp/i300-child-filter-prefix.log`), and focused
independent re-review finds no new blocker. Tools all-target Clippy passes 59178;
governance and claims validators pass zero-warning 26115/44453. Sandboxed full
tools run has 343 pass / 1 process-observation PermissionDenied. Approved full
standard preflight is live as 28347, log
`/private/tmp/i300-initialized-current-preflight.log`; resume, do not restart.
Linked/ancestor production behavior and other original acceptance/publication
residuals remain open. I300/T7 stays Active, with no completion SHA.

Startup-selection and external-worktree checkpoint (2026-10-15; unpublished):
raw startup selection matches native Git in stale-private, commondir-presence,
private-override, bare and missing-version fixtures (two tests, log
`/private/tmp/i300-child-layout-native-rules.log`). Discovery and effective
worktree roots now retain separate source evidence. The initialized-child real
Checkout fixture preserves tracked content and sentinels for external worktrees,
refuses external clean filters without execution, and retains raw-target denial
coverage; it passes as session 69259, log
`/private/tmp/i300-child-external-parity.log`. Earlier broader child 25/25 and
Checkout 47/47 results predate the final check-attr fix. Independent review found
discovery-cwd prefix confusion; check-attr now explicitly binds gitdir/worktree
and cwd to the effective root. Fresh broad validation/review, linked/ancestor
layouts, MCP denial/replay and full current-source preflight remain required.
This supersedes the conservative inactive-target residual below, not the other
I300 acceptance or publication gates. T7 stays Active; no completion SHA exists.

Raw startup layout checkpoint (2026-10-15; unpublished): independently
admitted common/config and gitdir/config.worktree now receive bounded Rust
no-include parsing. Raw core.worktree targets enter the next Read frontier and
retain canonical/path identity evidence before ordinary Git setup. Sources and
targets remain in continuation profiles and are revalidated. Independent targeted
review reports no new security blocker for this chain. cargo check passes
(88345); raw include-masking evidence test passes (85382); real initialized-child
fixture now additionally removes the external raw-target Read while include
masks it, and confirms refusal before ordinary setup with branch/sentinel intact
(5316, /private/tmp/i300-child-raw-denial.log). This supersedes the preceding raw
startup security finding, not other open delivery gates. Compatibility residual:
both raw targets are conservatively requested even when Git suppresses a stale
config.worktree or linked common hint; implement actual startup selection rules
and verify them before claiming broad layout parity. External worktree status,
MCP/denial/replay combinations and fresh full standard gate remain open.

Initialized-child continuation (2026-10-15; unpublished, not delivery evidence):
added a separate CheckoutChildWorktree authorization frontier, retained bootstrap
config origins, and bound admitted worktree pathname identities for revalidation.
Discovery-only Git config now explicitly passes both --git-dir and --work-tree
pointing at the previously admitted child directory. Source review of Git v2.54.0
setup_explicit_git_dir confirms GIT_WORK_TREE wins before core.worktree resolution;
a real fixture with a nonexistent configured worktree passes without accessing
that target (21164, /private/tmp/i300-child-bound-config-test.log). Real native and
prepared initialized-child preservation passes; focused new_gitlink tests pass
3/3 (70780, /private/tmp/i300-child-bound-bootstrap-tests.log).
OPEN SECURITY FINDING: merged --list --includes core.worktree does not establish
the startup layout value. Git startup reads raw common/config and optional
config.worktree without includes; an include can mask an unapproved raw target.
Before ordinary setup, independently inventory/admit/bind those raw layout target
spellings and retain their evidence. External effective-worktree status parity,
denial/replay tests, stage-budget combinations, MCP coverage and full fresh gate
remain open. This draft is not security approved and must not be published.

Current-source checkpoint (2026-10-15): standard preflight session 52429
finished successfully; `/private/tmp/i300-new-directory-preflight-approved.log`
ends `release preflight: passed`. This validates the nonempty uninitialized
new-gitlink directory checkpoint only. Subsequent initialized-child discovery
changes compile but are not final-gate or security-review approved. Independent
review found (1) ordinary child Git setup can resolve core.worktree before its
Read scope is bound, and (2) child config origins must retain explicit Read in
intermediate target-config/deletion stages. The second finding is patched by
retaining each discovered config-file Read in both continuation and mutation
profiles; regression validation remains pending. The first finding remains open:
bootstrap configuration without ordinary worktree setup, discover and admit the
effective worktree, then bind subsequent child status to that admitted location.
Do not treat the prior full gate as validation of these newer production edits.
T7 remains Active; later serial tasks and publication gates remain incomplete.

# Iteration I300: Structured Git Permission And Diff Repair

2026-10-20 Windows shell environment preparation: receiver validation refuses
MSYS/Cygwin runtime and argument/environment conversion overrides without
changing the frozen environment. ADR-089 records the pinned research source and
its limits. Host shell-environment regression passes 1/1; locked MSVC test-target
xwin checking passes with seven existing warnings. Independent static review
found no blocker; Windows-specific runtime tests and complete shell support
remain unverified. No production Windows/Pull activation or T7 completion.

2026-10-20 verified auxiliary-parent stage: standard `release_preflight.sh`
completed with exit 0 on c9167926 (session 49895), including workspace tests and
both external Runtime SDK consumer modes. Log:
`/private/tmp/i300-aux-parent-preflight.log`. The sandbox early-exit test and all
48 sandbox tests passed in this full run; this does not establish the cause of
the earlier intermittent failure. Fresh fetch has zero main-only commits.
Native Windows run 39401797353 is still executing its workspace tests and does
not contain c9167926. Publish the stable stage for native confirmation without
claiming Windows activation, Pull delivery or completion of T7.

2026-10-20 Windows auxiliary-parent candidate: verbatim include paths now
resolve each `..` only after checking that its cancelled ancestor is an existing
directory. Original ancestor spellings remain in the auxiliary inventory;
missing and regular-file ancestors have an additional Windows-only refusal
regression. Host config-state regressions pass 31/31, and the Windows MSVC test
target passes `cargo xwin check --locked` (seven existing reduced-feature
warnings). These are host and compilation evidence, not native Windows success.
Independent static security review found no blocking issue; the existing
check-to-use race remains. The Checkout hook exact-read regression also passes
1/1. A Windows volume-root parent regression was added on review advice;
final locked MSVC test-target compilation passes, while native confirmation
remains required. Both governance validators pass with zero warnings.

The latest full preflight on 67112a30 failed the sandbox early-exit stdin test
with an unconfirmed cleanup receipt. A focused retry and all 48 sandbox library
tests pass; the cause remains unknown, so the earlier preflight failure is not
closed. The test now reports the execution error and internal receipt failure
on recurrence. Run 39401797353 remains live at this checkpoint. Production
Windows structured Git and Pull remain disabled; T7 and the full handoff queue
remain incomplete.

Current gate handle (2026-10-15): full approved standard preflight is live
as session 52429, `/private/tmp/i300-new-directory-preflight-approved.log`;
resume this handle rather than restart. Governance and claims pass with zero
warnings (76783). Strengthened native/prepared preservation fixture additionally
passes (19900): bare-shaped HEAD/objects/refs and inert fsmonitor config, deep
children, external broken symlink, FIFO and socket remain intact; socket receives
no connection and command sentinel stays absent. Log:
`/private/tmp/i300-new-directory-preservation.log`. Only tests changed after
starting this production-source preflight. Full gate outcome is not yet inferred.

New-directory continuation checkpoint (2026-10-15; unpublished): nonrecursive
Checkout now admits existing nonempty ordinary directories when their bounded
top-level name inventory has no Git metadata name or protected HFS/NTFS alias.
A fresh exact Read precedes enumeration; retained directory identity and names
are revalidated before mutation without opening/statting descendants. Limits
are 4,096 entries and 1 MiB of names per directory per capture. Windows nonempty
directories remain refused pending runtime evidence; initialized children remain
an actual compatibility gap. Removed-gitlink empty-directory constraints remain
separate and unchanged. Independent production review has no blocking finding.
Checkout 46/46 passes (18419), focused evidence tests pass (80416), and Git
all-target locked Clippy passes (89249). Real MCP Checkout matrix now has 33
cases on Unix and passes (61664), including nonempty direct mutation, target
configuration, old-child deletion, exact Read denial and Write denial/replay.
Logs: `/private/tmp/i300-new-directory-checkout.log`,
`/private/tmp/i300-new-directory-mcp.log`, `/private/tmp/i300-new-directory-clippy.log`.
Full preflight 33389 predates this production edit and is no longer final-source
evidence. T7 remains Active; all other functionality/publication residuals and
later serial tasks remain open.

Next capability parity evidence (2026-10-15): isolated native Git fixtures with
`submodule.recurse=false` introduce a new gitlink at an existing nonempty
directory and at an existing initialized child repository; both checkout calls
exit 0, select the target branch and preserve sentinel bytes (and child `.git`).
The current empty-directory shortcut still refuses both, so these remain actual
T7 compatibility gaps, not out-of-scope cases. Next implementation must discover
and bind child metadata/configuration/status effects with fresh admission;
do not merely remove emptiness refusal or infer the empty-child authorization
covers initialized repositories.

Empty-directory public-entry verification (2026-10-15): real MCP Checkout
matrix now covers 26 cases, including new empty gitlink allow/Write denial,
exact directory Read denial with every Write allowed, target onbranch config
continuation, and simultaneous nonempty old-gitlink deletion. Every case checks
request replay refusal and success/preservation state. Final matrix passes
(89457), `/private/tmp/i300-empty-directory-mcp-final.log`; MCP all-target locked
Clippy passes (73082). Unix late directory-to-symlink substitution additionally
refuses with referent bytes retained (71549). Independent review confirms the
fixtures exercise the intended three continuation paths, with no blocking
finding. MCP denial text intentionally reports generic `approval denied`, so
test rules isolate the denied Read without assuming private reason exposure.
These are test-only additions after full preflight 33389; production code is
unchanged. Remaining T7 scope, Windows runtime evidence and delivery gates stay
open.

Current empty-directory standard gate passed (2026-10-15): approved session
33389 exited 0, log `/private/tmp/i300-empty-directory-preflight-approved.log`
ends `release preflight: passed`. This covers the new exact-Read continuation,
all locked workspace checks/Clippy/tests, doctests and both external runtime SDK
modes. Final independent review has no blocking finding; separate governance
and claim validators pass with zero warnings (26966, 35612). This supersedes the
running checkpoint below, not T7's remaining functionality or publication gates.

New target gitlink directory checkpoint (2026-10-15; unpublished): Checkout
now admits an existing ordinary empty directory through a separate exact-Read
continuation before enumerating it. The final plan retains that Read, refreshes
its identity, and rechecks directory identity/emptiness before execution.
Independent review identified the initial missing Read gate and continuation
retention/identity omissions; final independent review finds no remaining
blocking finding. Native parity, late child/inode replacement, nonempty/initialized
refusal, missing Read and old-proof tests accompany the change. Checkout 46/46
passes (71610), and locked Git all-target Clippy passes (82271); strengthened
assertions pass the focused rerun (29074). Full standard preflight is running
outside the sandbox as approved session 33389, log
`/private/tmp/i300-empty-directory-preflight-approved.log`; no outcome is inferred.
The full socket preflight below
predates this production change and is not current-final evidence. T7 and all
original residuals/publication gates remain open.

Current socket-version standard gate passed (2026-10-15): approved session 1885
exited 0; `/private/tmp/i300-socket-preflight-approved.log` ends `release
preflight: passed`. This includes locked workspace check/Clippy/tests, doctests,
standard boundary/governance validators and both external runtime SDK modes.
It covers the socket production changes and type-replacement regression above
the preceding FIFO checkpoint. Only documentation changed during this run.
This is local validation, not publication or full T7 acceptance; configuration
link authorization, Pull, remaining transports/recursive/platform coverage and
exact-head publication gates remain open.

Socket final-boundary checkpoint (2026-10-15): dedicated test additionally
rejects socket-to-FIFO and socket-to-symlink replacements (29377), preserving
the distinction between deletion entry types without contacting either endpoint.
MCP all-target locked Clippy passes (94769); governance and Collaboration Claim
validators pass with zero warnings (84557). Current-source standard preflight
is running outside the sandbox as approved session 1885, log
`/private/tmp/i300-socket-preflight-approved.log`. No result is inferred yet.

Unix socket deletion checkpoint (2026-10-15; unpublished): an isolated native
Git fixture confirms nonrecursive gitlink-to-file Checkout deletes a socket
entry and restores target file bytes. Deletion inventory now distinguishes FIFO
and socket identity/type without opening or connecting either. Exact Read and
final per-entry Write remain required. Tests cover no connection, replacement
drift, separate missing regular/FIFO/socket Write and retention before mutation.
Checkout passes 45/45 (66711), the socket inventory test passes separately,
and Git all-target Clippy passes (21999). Independent `review_prepared` security
review finds no new blocking issue. Real MCP socket coverage passes (93230),
`/private/tmp/i300-socket-mcp.log`, with success, denied-node retention and replay
refusal through the shared public permission gate.
Devices and link deletion remain refused; this does not close T7.

FIFO-version standard gate passed (2026-10-15): approved session 9204 exited 0;
`/private/tmp/i300-fifo-preflight-approved.log` ends `release preflight: passed`,
including workspace tests, doctests and both external SDK modes. This gate
predates the socket production change above; it is not current-final evidence.

FIFO standard-gate environment checkpoint (2026-10-15): session 13206 exited
101 with tools 691/692 passing; the sole failure is the existing local Push
process-observation fixture returning EPERM in the sandbox. No other test
failure was reported. The approved identical outside-sandbox standard rerun is
live in session 9204, log `/private/tmp/i300-fifo-preflight-approved.log`.
Await its exit before claiming current-source standard validation.

FIFO public-entry checkpoint (2026-10-15): the real MCP Checkout matrix now
includes a FIFO in its nonempty child deletion fixture on Unix. Allowed execution
restores the target regular file, denied execution retains the FIFO, and replay
still refuses. The full 19-case matrix passes (23059), log
`/private/tmp/i300-fifo-mcp.log`. Formatting and diff checks pass (25559).
The standard preflight (13206) remains running; this focused result does not
establish its outcome or close T7.

Unix FIFO deletion checkpoint (2026-10-15; unpublished): nonrecursive Checkout
child deletion now captures FIFO identity and type without opening its content.
Every FIFO requires exact Read discovery and final exact Write admission;
missing Write retains it and refuses before mutation. FIFO inode, regular-file
and symlink replacements all fail revalidation (30551). Real prepared Checkout
also deletes an admitted FIFO-containing directory successfully. Checkout tests
pass 44/44 (78586), Git all-target Clippy passes (25346), and independent
`review_prepared` security review finds no new blocking issue. This supersedes
the FIFO refusal statement below only; links/devices/sockets remain refused.
The earlier standard preflight predates this change and is not final-candidate
evidence. T7 and the full serial queue remain incomplete.

Native special deletion characterization (2026-10-15): isolated Git fixtures
for nonrecursive gitlink-to-file checkout each contain a file symlink, directory
symlink or FIFO under the child directory. All three native checkouts exit 0,
replace the child directory with the target regular file and preserve an
external sentinel. This establishes deletion-entry semantics, not authority to
follow links or open FIFO content. Current `checkout_deletion::Inventory` still
refuses those entries. Completing support requires admitted entry metadata and
identity evidence, exact deletion effects, drift tests and security review;
the config-link metadata proposal is not generic deletion authorization.

Current standard gate passed (2026-10-15): approved session 29306 exits 0 and
`/private/tmp/i300-current-preflight-approved.log` ends `release preflight:
passed`. This covers workspace locked check/Clippy/tests, tools 691/691, doctests
and both independent runtime SDK consumer modes, plus standard governance and
boundary validators. It supersedes the sandbox-only process-observation failure
below. No production code changed during this rerun. Link referent admission,
Pull, transport/recursive/platform residuals and exact-head publication gates
remain incomplete; no completion commit or T7 completion is claimed.

Standard preflight observation checkpoint (2026-10-15): session 29032 ended
101 solely at tools `prepared_local_push_runs_all_eight_admitted_stages` because
process observation returned EPERM inside the sandbox. Tools passed 690/691;
workspace check/Clippy and preceding Agent/CLI/MCP/Runtime suites passed, but
the full preflight did not pass and SDK validation was not reached. The approved
outside-sandbox rerun is live in session 29306, with output at
`/private/tmp/i300-current-preflight-approved.log`; await its actual exit before
citing success. This is environment recovery, not permission bypass in Talos.

Target resource-limit admission checkpoint (2026-10-15): a real prepared
Checkout matrix now covers oversized individual source bytes, cumulative bytes
across individually bounded sources, record exhaustion, cumulative requested
paths, and repeated-include expansion payload. All five cases pass (80919),
requiring target-discovery refusal before mutation and preserving HEAD, index,
worktree and config bytes with no new branch. Final locked tools all-target
Clippy passes (84286). Link metadata acceptance remains pending; this additional
boundary evidence does not activate that capability or complete T7.
Project governance and Collaboration Claim checks pass with zero warnings
(59819). Standard workspace preflight is running in session 29032 with output
at `/private/tmp/i300-current-preflight.log`; it is not yet passing evidence.

Link discovery design checkpoint (2026-10-15): independent reviewer
`review_prepared` found no new static bypass in the current ordinary-directory
`..` implementation. It identified a real authority gap for `read_link` because
Path tokens canonicalize aliases. ADR-089 now records a proposed, disabled
`git:configuration-link-metadata` capability; maintainer acceptance is pending.
A native regression confirms a symlink config's relative include uses the
spelled origin rather than the canonical referent directory (93479). A separate
parser regression confirms include traversal preserves spelling without
relaxing other configured paths (54519). These are focused passing tests, not
production symlink support or final delivery evidence.
After those test additions, target-config tests pass 29/29 (14288), locked
all-target tools Clippy passes (76917), and project governance and Collaboration
Claim validation pass with zero warnings (4525). The maintainer decision was
requested; a question submission receipt is not acceptance.

Parent traversal continuation checkpoint (2026-10-15): Sources retains raw
include spellings and every cancelled ancestor, requires the entire frontier's
Read admission before capture, and revalidates ancestor identities before leaves.
Traversal unit regressions pass 3/3 (49730). The real target-hook admission
matrix now uses `traversal/../target.conf` in six cases, including successful
final mutation and cancelled-ancestor substitution denial; it passes (32125).
Initial discovery needed a dedicated include-path parser; other configured paths
and receiver include traversal retain their previous syntax restriction.
The first tools-library run passed 330/332: one old early-refusal assertion was
updated to require the actual final overlap refusal; process observation in the
other case was sandbox-denied. The approved rerun passes 332/332 (91531), log
`/private/tmp/i300-traversal-tools-approved.log`. Final tools all-target locked
Clippy passes (31998), as do diff whitespace and project governance validation
(zero warnings); Collaboration Claim validation exits 0 (10874). Symlink referent
admission, independent security review and all remaining T7 gates remain open.

Overlap evidence correction and hook coverage (2026-10-15): the previous
missing-config overlap fixture used a `..` spelling and a generic refusal
assertion; it could stop in path discovery, so session 73067 did not prove final
overlap enforcement. It now uses the canonical absolute missing source and
requires the exact final `Checkout rewrites an effective configuration or
auxiliary source` error; the regression passes (session 14407). A new real
guessed-branch test covers both creation of a missing late hook and replacement
of an existing non-executable hook by an executable target. Both require that
same final overlap error and preserve HEAD/index/hook bytes/branch absence with
no hook marker (session 32490). Its initial fixture failed native checkout because
index executable mode differed from worktree mode; setting the fixture's actual
mode fixed preparation. Ordinary config symlink/`..` referent admission remains
incomplete; these tests do not claim to support it. All original T7 gates remain.

Public continuation final checks (2026-10-15): updated MCP config/branch
assertions pass (session 22680); combined talos-tools/talos-mcp/talos-runtime
all-target locked Clippy passes (session 6610,
`/private/tmp/i300-public-target-clippy.log`); diff check and Collaboration Claim
validation pass. These are local checks only, not the final workspace preflight.

Public target continuation evidence (2026-10-15): real MCP Checkout transport
matrix now includes allow/deny guessed remote branches with target-only
conditional config, both ordinary file replacement and nonempty uninitialized
gitlink-directory deletion. The 19-case matrix passes (session 51594), including
request replay denial. Runtime's model-driven Agent path additionally covers
target-conditional branch creation with separate approvals and initial denial;
the focused locked regression passes (session 45010), preserving index and
worktree. A real guessed Checkout now exercises target-tree creation of its own
missing config source through the full prepared continuation and refuses before
HEAD/index/branch/source changes (session 73067). This supersedes reliance on
injected Sources for that overlap test. Added final MCP assertions are being
revalidated separately. Hook-overwrite and resource-limit matrices, config-path
referent support, Pull, and original final delivery gates remain open. No
completion or publication is claimed.

Final target drift/deletion regression (2026-10-15): the real final-admission
fixture now records the actual final profile in PreparedStageHistory, covers
target config drift and Missing hook creation after final approval, and verifies
refusal preserves symbolic HEAD, commit, index and worktree. Five cases pass
(session 53866). The real gitlink reversal fixture additionally guesses a remote
topic branch with conditional config, crosses TargetConfiguration then
CheckoutDeletion without any Write facets, and refuses a missing exact child
Write while preserving child bytes and branch absence (session 65355). These
focused locked Git-feature tests pass. Public-entrypoint and full overlap/deletion
success matrices, Pull, and original delivery gates remain open; no completion
or publication is claimed.

Final target admission continuation (2026-10-15): supersedes the historical
final-effect refusal checkpoints below. After exact-admitted hook capture and
revalidation, target discovery now retains every Read and generates a fresh plan
identity. It either proceeds through the read-only deletion frontier or appends
the original mutation facets for fresh mutation admission. Execution retains
source/hook drift and overlap checks. Independent review of Git 2.54 cache and
lazy-fetch behavior and the actual transition found no new safety defect.
The real prepared regression covers missing hook Read, stale discovery proof
denial, and fresh-proof Checkout success, with HEAD/index assertions.
Checkout-filtered tests pass 41/41 (session 59390,
`/private/tmp/i300-final-transition-checkout.log`); `git diff --check` passes.
Complete target-conditional deletion/drift/overlap and public-entrypoint matrices,
Pull, and all original delivery gates remain outstanding. This is local progress,
not completion or publication evidence.

Native missing-object reporting regression (2026-10-15): added
`native_checkout_missing_reporting_objects_do_not_spawn_fetch`, covering missing
upstream commit / missing ancestor crossed with upstream / push comparison.
All four isolated non-promisor native Git cases record no Trace2 child_start and
no remote-helper sentinel execution. Missing-ancestor reporting fails after HEAD
moves; the regression preserves this partial-mutation observation rather than
claiming atomic failure. The focused locked talos-tools Git-feature test passed
(session 96247), and `git diff --check` passed. This supplies durable evidence for
the reporting path only; target configuration final effect binding, independent
review, Pull, and the original delivery gates remain open. Production final
mutation refusal remains in place; no completion or publication is claimed.

Missing hook-directory follow-up (2026-10-15): supersedes the missing-directory
residual below. Hook parents use optional directory contexts with retained None
identity; required gitdir contexts retain their existence requirement. Exact
frontier admission precedes metadata capture. Later directory creation invalidates
the captured absence. Independent review also found optional-absence→required
upgrade could bypass the requirement; required requests now refuse captured None,
with regression coverage. The prepared frontier test covers both a missing Read
token before invalid leaf inspection and an all-approved missing hook directory
reaching the final effect-binding refusal, preserving HEAD/index/branch absence.
Both cases reserve final mutation capacity with the core stage history.

Checkout tests pass 40/40 (/private/tmp/i300-hook-missing-checkout.log);
final target-config tests pass 25/25 and Git all-target Clippy passes
(/private/tmp/i300-hook-missing-final-{target-config,clippy}.log). Ancestor-checkpoint
governance/claims passed zero warnings
(/private/tmp/i300-late-hook-guard-{governance,claims}.log).
Final target mutation remains disabled. Next retain and bind the complete
cache-specific evidence to a fresh final mutation admission, then complete Pull
and original T7 delivery gates. No completion or publication is claimed.

Late-hook ancestor guard follow-up (2026-10-15): independent review identified
that direct hook recapture could inspect an external leaf after parent symlink
substitution. Although final refusal made that continuation unreachable, the
guard was repaired before enabling execution: candidate parents now join the
existing exact-admitted directory frontier, and Sources ancestor revalidation
runs before hook recapture. Nonempty hook evidence requires complete Sources.
A parent-to-external-symlink regression checks the source-identity refusal occurs
before the invalid external leaf is inspected. Checkout tests pass 40/40 and
Git all-target Clippy passes, logs /private/tmp/i300-late-hook-guard-{checkout,clippy}.log.
Independent follow-up confirms the static substitution gap is closed; ordinary
check/open TOCTOU residual remains. Missing hook directories currently refuse
at directory-context capture; supporting their absence remains required before
claiming full target-hook behavior. Final effect binding and mutation stay disabled.
Previous checkpoint governance and claims passed with zero warnings, logs
/private/tmp/i300-late-hook-{governance,claims}.log.

Late checkout hook frontier checkpoint (2026-10-15): target configuration now
produces a separate exact-Read continuation for traditional post-checkout and
reference-transaction candidates without inspecting their contents. A non-null
source hooksPath retains its independently cached source directory; otherwise
the target directory is admitted conservatively. All candidate tokens must pass
before any leaf capture; executable, linked and nonregular hook leaves refuse.
Captured evidence is wired into revalidation and target replacement/deletion
protection, but the final target effect-binding refusal remains in place.

Checkout-filtered tests pass 40/40 and Git all-target Clippy passes, logs
/private/tmp/i300-late-hook-final-{checkout,clippy}.log. The exact-token regression
uses the shared PreparedStageHistory and reserves final mutation capacity for its
ordinary-chain fixture; it is not evidence for arbitrary-depth chains or actual
final mutation. A Missing-to-created hook regression verifies retained-evidence
refusal. Initial read-only review found no new admission-order defect with the
final gate retained; evidence/protection follow-up review is tracked separately.
The first evidence regression failed because its fixture had no initial commit;
the fixture was corrected before the successful complete checkout rerun.
Next complete cache-specific effect binding and final admission, then production
Pull and the original T7 delivery gates. No completion or publication is claimed.

Target-source effect protection checkpoint (2026-10-15): captured target
configuration paths, including retained Missing sources, now join checkout's
existing tree-replacement and recursive-deletion protection. Directory-only
admission dependencies and pending sources are excluded. A real commit-tree
regression compares ordinary file creation (allowed by the overlap check) with
the same path captured as an absent target config source (refused), and verifies
deletion overlap refusal. This is direct protection-helper evidence; the final
target-stage mutation continuation remains disabled.

Checkout-filtered tests pass 36/36 and Git all-target Clippy passes (64512),
logs /private/tmp/i300-target-protection-{checkout,clippy}.log. Independent
read-only review found no new static admission bypass; existing canonical-path
revalidation TOCTOU limitations remain. Next complete source/target cache-specific
effect generation and final admission, then the outstanding Pull and original T7
delivery gates. No completion/publication or serial-queue advancement is claimed.

Target-source revalidation and cache partition checkpoint (2026-10-15):
AddPlan now revalidates captured target configuration after the repository location
guard. Only already-admitted ancestors/files are revisited; pending sources are
not read. A target-only file drift regression confirms refusal while HEAD/index
and absent topic branch remain unchanged. Independent boundary review found no
new admission-order defect. Checkout-filtered tests pass 34/34 and Clippy passes
(88822), /private/tmp/i300-target-drift-{checkout,clippy}.log.

Native Git 2.54 cache characterization now covers eight tracked/empty-index,
source-hooksPath present/absent and malformed/valid target-chain combinations.
Malformed target config fails after HEAD changes when source core.hooksPath is
set; without that setting these fixtures succeed. Neither target file hooks nor
target configured hook commands execute. Git repo_config_clear clears ordinary
configset only, while configured hook lists retain their independent cache;
a missing hooksPath can cause ordinary config to reload before HEAD moves.
This is a bounded characterization, not proof for all later readers. The new
native regression passes within target-config 24/24; final all-target Clippy
passes (14285), /private/tmp/i300-cache-partition-{tests,clippy}.log.
The earlier malformed-target fixture was rerun and retains its always/inherit
failure result; its source hooksPath explains the distinguishing condition.
Final target effects/admission remain unfinished and mutation still refuses.
Next bind cache-specific readers and auxiliary effects, preserving source hook
semantics and target-source evidence; Pull and all original delivery gates remain
open. No completion/publication or serial-queue advancement is claimed.

Physical tracking overlay checkpoint (2026-10-15): target discovery now predicts
local config setters from captured exact-origin evidence before ordered include
expansion. The overlay preserves native section placement, duplicate remote/rebase
setter behavior, duplicate merge unset behavior, empty sections and standalone
versus inline comments. Thirteen real Git checkout fixtures compare both physical
record order and expanded include order. Independent review found and verified
closure of frontmatter record misalignment; section tokens now enforce the
16,384-item bound even for empty sections. Oversized write payloads also refuse.
Final local verification passes target-config 23/23, Checkout-filtered 33/33 and
Git all-target Clippy (5258); logs /private/tmp/i300-overlay-final.log,
/private/tmp/i300-overlay-checkout.log and /private/tmp/i300-overlay-final-clippy.log.
Read-only review found no remaining issue within overlay scope.

This does not enable the final target mutation continuation. Next implement the
source/target config-cache partition, bind post-switch auxiliary/effect facets,
admit their reads and revalidate before original argv dispatch. Production Pull,
other original T7 behavior and fixed-candidate publication gates remain open.
No completion commit, publication or advancement of the serial queue is claimed.

Tracking prediction and stale-source checkpoint (2026-10-15): retained invocation
configuration now predicts local branch.remote/merge/rebase write intentions from
native Git 2.54 tracking modes and the resolved starting reference. Create uses
admitted symbolic HEAD (detached means no start); guessing uses the retained
remote mapped reference. Execution recomputes intentions and revalidates before
original argv dispatch. Native differential coverage passes 40 local/guessed
merge/rebase combinations plus detached true/always/inherit; Checkout passes
32/32, Git all-target Clippy passes, logs
/private/tmp/i300-tracking-{matrix,detached,checkout,clippy}.log. Independent
read-only review found no specific defect or increased authority, explicitly
excluding physical config overlay and final conditional effects from its conclusion.

The same-OID two-remote regression now also admits the discovered plan, changes
checkout.defaultRemote, and consumes that exact proof. Execution refuses before
continuation while HEAD, index, current branch and absent topic branch remain
unchanged. This exercises snapshot drift refusal, not isolation of the later
source-comparison check from configuration revalidation. Focused test and latest
Git all-target Clippy pass (16560/69371), logs
/private/tmp/i300-tracking-stale-{source,clippy}.log; diff check passes.

Native setter experiments on Git 2.54.0 show replacement retains an existing
key's position, but insertion uses the last matching section, including an empty
section across an include. Duplicate remote fails its setter; duplicate merge
unset returns positive 5. Actual checkout with autoSetupMerge=always continues
past duplicate merge, preserving old merges and appending the new one (exit 0
with warning). branch.c checks setter results for negative values, so assuming
all nonzero results abort would be incorrect. Raw setter experiment artifact:
/private/tmp/i300-tracking-config-order.json. Implement a section-aware local
config overlay, verify full checkout parity including duplicates and empty
sections, then evaluate the target chain with source/target cache partition and
bind final effects. Appending intention records to expanded config is invalid.
Target-stage mutation still explicitly refuses; Pull, Windows execution and all
original publication/delivery gates remain open. No Complete or publication is
claimed; the serial queue is unchanged.

Remote-guess source checkpoint (2026-10-15): checkout resolution now retains
the selected remote and mapped reference alongside its commit, rather than only
a guess flag. Final execution compares the full retained source and commit,
then revalidates repository/config evidence. A two-remote same-OID regression
prepares both defaultRemote selections, proves their captured sources differ,
and confirms discovery creates no local branch. This is source-capture evidence,
not a direct stale-source execution regression or completed tracking prediction.
Checkout passes 30/30 (33878), Git all-target Clippy passes (98472), logs
/private/tmp/i300-tracking-source-{checkout,clippy}.log; diff check passes.
Independent read-only review found no new flaw or expanded read/network scope,
and explicitly limits the new regression to capture. Native Git 2.54 branch.c
confirms remote/merge/rebase writes depend on tracking mode and resolved starting
reference; local/create and remote matching prediction still require implementation.
Final target mutation refusal, Pull and original delivery gates remain open.
GIT-001's standing gix tracking was refreshed against published 0.89.0 and
pinned upstream sources; it already contained prior research, contrary to the
preceding user-facing status statement. No publication/completion is claimed.
Both governance and Collaboration Claim validators exit 0 (55213/39722), logs
/private/tmp/i300-tracking-{governance,claims}.log.

New-branch name binding checkpoint (2026-10-15): create and remote-guess
paths now run native check-ref-format --branch inside admitted config discovery,
retain its bounded single-line name, use that name for target onbranch context,
and re-interpret/compare it before executing the original argv. Real regression
creates the deleted previous branch via -b @{-1}, checks the retained normalized
name, and rejects bad..name before mutation. Checkout passes 29/29 and Git
all-target Clippy passes (85783), logs
/private/tmp/i300-new-branch-{checkout,clippy}.log; diff check passes. Independent
read-only review confirms native check_branch_ref interpretation, discovery Read/
Execute/config-chain coverage, bounded output and execution/ref-log revalidation,
with no new finding. Target Sources uses the accepted normalized name; tracking
writes and their post-write predicted configuration/cache split are still not
implemented, so final target mutation refusal remains. Symlink/component support,
final effects, Pull and original delivery gates remain open. No completion or
publication is claimed.

Attached-HEAD review correction (2026-10-15): independent review found
HEAD and @ preserve the current attached branch in native checkout, rather than
always detaching. These exact inputs now obtain symbolic local context alongside
previous-checkout expressions; revision suffixes/fully qualified revisions are
not broadly converted to branch switches. Real onbranch execution regression
covers both HEAD and @ preserving source branch/config. Final Checkout 28/28 and
Git all-target Clippy pass (56288), logs
/private/tmp/i300-head-context-{checkout,clippy}.log; diff check passes. The preceding
previous-expression 28/28 and Clippy checkpoint remains valid. Review otherwise
found no additional previous-expression flaw and confirmed reflog identity
binding. This correction has focused verification; final stable-candidate review
and full workspace gates remain required. Create-name normalization, predicted
tracking config, symlink contexts and final effects remain open. No completion or
publication is claimed.

Previous-checkout expression checkpoint (2026-10-15): non-create exact
@{-digits} expressions now obtain the symbolic local reference just like '-' and
bind the local target OID before configuration preparation. Expressions with
revision suffixes remain detached contexts. The real onbranch Checkout regression
now alternates '-' and '@{-1}'; the same-OID reflog-change refusal regression uses
explicit '@{-1}', proving OID equality does not admit changed history. Checkout
passes 28/28 and all-target Git Clippy passes (43344), logs
/private/tmp/i300-previous-expression-{checkout,clippy}.log. Diff check passes.
Independent classification review remains pending. Create-name normalization and
tracking/source-target configuration prediction still need closure, along with
symlink support and final effects. No task completion or publication is claimed.

Target-branch classification checkpoint (2026-10-15): checkout resolution
now retains whether the OID came from remote candidate guessing or direct revision
resolution, and execution rechecks that distinction as well as the OID. Target
Sources has an explicit optional local branch: create/guessed paths select the new
local name, existing local paths select the refs/heads suffix, and direct detached
revisions have no onbranch context. Native differential tests confirm detached
HEAD does not match *, **, empty or HEAD patterns. Target tests pass 21/21,
Checkout 28/28 and all-target Git Clippy pass (52062), logs
/private/tmp/i300-target-branch-{tests,checkout,clippy}.log. Diff check passes.
Independent follow-up requested; result pending. This is not complete branch-name
normalization for arbitrary previous-checkout expressions or tracking semantics.
Post-tracking config prediction, source/target cache partition, symlink contexts
and final effects remain open; production target-stage mutation still refuses.
No publication or task completion is claimed.

Linked-gitdir context checkpoint (2026-10-15): target Sources now requests
the invocation gitdir and every ancestor explicitly as directory identity Read
facets, sharing the cumulative request/path-byte budget. Whole-frontier admission
precedes root-to-leaf capture; no directory enumeration or child-file read occurs.
Completion waits for both file and directory frontiers, and captured identities
are revalidated. This removes the previous dependency on enabled/present worktree
config for linked-gitdir matching. Regression covers zero capture on denied
gitdir, malformed unrequested config.worktree left unread, successful conditional
matching, and replacement-directory drift. Target tests pass 20/20 (83504),
/private/tmp/i300-directory-context-tests.log; Checkout passes 28/28 (80167),
/private/tmp/i300-directory-context-checkout.log; Git all-target Clippy passes
(58265), /private/tmp/i300-directory-context-clippy.log. Independent read-only
review found no new permission/budget gap in either the preceding host-root work
or this directory-context increment. Existing ancestor/path-access TOCTOU remains.
Symlink contexts, actual target branch identity, post-tracking predicted config,
cache partition and final effects remain incomplete; final mutation refusal stays.
No publication or task completion is claimed.

Ordered host-root checkpoint (2026-10-15): prepared target discovery now
retains native system/global roots separately from the potential include-source
set and appends repository config in read order. Completed capture evaluates
those ordered roots instead of repository config alone. Worktree config is
requested only after admitted direct repository-config records enable the
extension; global/include declarations do not enable it. New worktree bytes wait
for a fresh exact Read frontier. Native Git comparisons exercise repeated global
sources via a system include, XDG/user order, direct/indirect extension selection,
and final false override. Target tests pass 19/19 (12475),
/private/tmp/i300-host-roots-tests.log; Checkout passes 28/28 (55618),
/private/tmp/i300-host-roots-checkout.log; all-target Git Clippy passes (47859),
/private/tmp/i300-host-roots-clippy.log. Diff whitespace check passes. Independent
review requested; no result yet. Linked gitdir without an enabled worktree config
still needs a separately captured directory-context frontier. Symlink contexts,
actual target branch selection and final effect binding remain open; mutation is
still refused. No completion or publication is claimed.

Admitted gitdir matching checkpoint (2026-10-15): ordered expansion accepts
an explicitly supplied gitdir only when retained non-symlink ancestor evidence
binds that directory. Matching uses captured context without filesystem calls,
with relative-origin literal prefixes, implicit **/ prefixes, trailing /**,
HOME interpolation and case-insensitive variants. Native Git differential cases
cover wildcard characters in the source parent, relative/absolute/HOME patterns,
case and directory suffixes; an unadmitted gitdir is rejected. Pure lexical dot
and repeated-separator normalization preserves the bound no-link context.
Hasconfig URL comparisons additionally share a 4 MiB comparison-input budget,
with an amplification regression. Target tests pass 18/18 and Git all-target
Clippy passes (37896), /private/tmp/i300-gitdir-target-{tests,clippy}.log.
Checkout passed 28/28 (74750) before the final lexical normalization adjustment,
/private/tmp/i300-gitdir-checkout.log. Independent security review remains due.
Symlink contexts are still unsupported; ordered host roots, actual target branch
selection and final effects remain incomplete. Production still refuses mutation.
No T7 completion or publication is claimed.

Hasconfig evaluation checkpoint (2026-10-15): ordered target expansion now
performs bounded remote-URL lookahead over captured bytes, enables hasconfig
includes during scanning, forbids URLs in conditional descendants, and matches
URL patterns with pathname semantics. All passes share source-byte, output-byte,
visit and processed-record budgets. Native Git differential regressions cover
later URL declarations, slash matching, unmatched conditional URL prohibition,
indirect descendants and non-path conditional keys. The last case exposed and
corrected a missing lookahead trigger. Target-config tests pass 16/16 (52938),
/private/tmp/i300-hasconfig-condition-tests.log; Checkout passes 28/28 (10370),
/private/tmp/i300-hasconfig-checkout.log; all-target Git Clippy passes (3035),
/private/tmp/i300-hasconfig-clippy.log. This checkpoint has not received independent
review. Production target-stage mutation remains disabled; gitdir context, host
root ordering, symlink/component admission and final effect binding remain open.
No completion or publication is claimed.

Deletion-plan follow-up (2026-10-14): existing recursive snapshot support is used for ordinary checkout sources, but gitlink evidence currently retains only child metadata/marker and does not enumerate every untracked descendant under a present child directory. Enabling gitlink→file deletion would therefore require a fresh bounded recursive directory inventory, explicit deletion-effect facets and revalidation before spawn. The conservative refusal remains correct until that separate design is implemented; no recursive delete authority is inferred from the repository-wide Checkout Write facet.

Initialized-child deletion residual (2026-10-14): native Git can remove untracked or initialized child contents when converting a present gitlink directory to an ordinary file, even with nonrecursive settings. Exact-path Read/Write evidence currently retained by Talos does not safely model that recursive deletion set, so prepared Checkout continues to refuse present child directories and metadata. The refusal regression passed (59465); no permission widening or silent deletion was introduced.

Initialized-child type-transition boundary (2026-10-14): native Git experiments show gitlink→ordinary-file checkout can delete untracked files and initialized dirty child repositories under nonrecursive settings. Prepared Checkout therefore continues to refuse present directories/child metadata; a regression now verifies refusal preserves parent branch/index and child HEAD/dirty bytes (59465), /private/tmp/i300-initialized-kind-refusal.log. This documents the required deletion-effect boundary rather than widening authorization.

MCP prepared Git regression complete (2026-10-14): full `git_prepared` integration suite passed 6/6, covering checkout, commit, local push and network push approval/denial/replay; test log /private/tmp/i300-mcp-git-prepared-full.log. Test target Clippy passed, /private/tmp/i300-mcp-git-prepared-clippy.log. This strengthens real admission-chain evidence only; production Pull, unsupported transports, Windows execution and final publication remain open.

MCP Checkout integration evidence (2026-10-14): `real_checkout_transport_denial_success_and_replay` now covers ordinary-file and gitlink target switches under allow/deny permissions, plus replay refusal; the four-case real MCP transport test passed (69642), /private/tmp/i300-mcp-gitlink.log. This confirms the new type-transition path through the shared permission gate, not only direct tool tests. Pull mutation and final T7 delivery gates remain open.

## Current Delivery State — 2026-10-14

Ordered target-config expansion checkpoint (2026-10-14): captured sources now
expand in declaration order, preserving include directives, repeated includes,
origin paths, implicit/empty values and onbranch selection. Expansion reads only
bound bytes and never opens a file. Native Git comparisons cover insertion at
include positions, ten-level depth, missing eleventh include and cycles; a removed
source still expands captured bytes and separately fails revalidation. Independent
review identified repeated comment-only parsing amplification; corrected with
separate cumulative 4 MiB parsed-source and output-payload budgets plus 4096
visits, all charged before work/allocation. Parser/source/expansion tests pass
14/14 (72176), /private/tmp/i300-ordered-target-config.log. Checkout passes 28/28
(46221) before the subsequent expansion-budget correction. Final all-target Clippy
passes (40772), /private/tmp/i300-ordered-target-clippy.log. Independent follow-up
confirms shared visits/source-byte/output budgets close the reported amplification
gap, with no new correction finding.
Production TargetConfiguration currently exercises repository-config expansion
only and still refuses mutation. Full ordered host root selection, gitdir and
hasconfig context/lookahead/prohibition, component/symlink paths and final effect
binding remain open. The potential-source set cannot substitute for those semantics.
No publication, Completion Commit or T7 completion is claimed.

Admitted target-config source stage checkpoint (2026-10-14): conditional branch
creation now reaches a separate Read-only TargetConfiguration phase instead of
the earlier blanket refusal. Roots are retained source configs, repository roots
and native system/global path declarations. Before any target-source lookup,
Sources checks exact admission for the complete file/ancestor frontier. Directory
identities are checked root-to-leaf, files retain regular/missing evidence, and
new nested includes require a fresh next-stage identity/admission. Shared budgets
cover 4096 requested paths, 4 MiB path bytes and 4 MiB file contents. Unsupported
parent traversal and symbolic ancestors/leaves refuse rather than being followed;
this currently includes common host symlink layouts and is not full compatibility.
Source evidence is revalidated across advances, including absent-file creation.
Tests pass 10/10 (91095), /private/tmp/i300-admitted-target-sources.log; Checkout
passes 28/28 (93490), /private/tmp/i300-target-source-checkout.log. Latest
all-target Git Clippy passes (90140), /private/tmp/i300-admitted-target-source-clippy.log;
governance (19087) and Claim checks (14269) pass with zero warnings, logs under
/private/tmp/i300-admitted-target-source-{governance,claims}.log. Independent
read-only review found no new admission bypass or cumulative-budget gap. The
existing metadata/open TOCTOU window remains; this is evidence revalidation, not
race-free confinement. Potential-source deduplication is not ordered include
expansion/depth proof. Stage completion still refuses mutation because ordered
conditional/hasconfig evaluation, final effects and original-argv execution gates
are not implemented. No T7 completion, publication or completion SHA is claimed.

Independent lexical-frontier review correction (2026-10-14): gix wildmatch's
64-level recursion guard returns false and logs the source pattern. Some(branch)
now rejects patterns with 32 or more star bytes before calling the dependency,
counted after trailing-slash expansion and conservatively including escaped stars.
This prevents guard exhaustion from being interpreted as a missing dependency;
it is an explicit complexity refusal, not full support for complex patterns.
Native-compatible source discovery with None never invokes the matcher. The new
65-star regression and parser suite pass 7/7 (81827),
/private/tmp/i300-target-frontier-recursion.log. Full library tests passed 299/299
(77343), /private/tmp/i300-target-lexical-frontier-all-tools.log, before this guard.
Independent read-only review confirms both recursive calls consume a star, so
this conservative guard prevents the dependency limit and pattern logging. Latest
all-target Clippy passes (72326), /private/tmp/i300-target-frontier-recursion-clippy.log.
Target-chain integration remains open; no completion or publication is claimed.

Lexical target-config frontier checkpoint (2026-10-14): admitted-source include
extraction now shares a pure declared_paths function. It preserves declaration
order and repeats, resolves only lexical origin/HOME paths, and performs no
existence/stat/read lookup. Optional target-branch selection applies Git-compatible
onbranch pathname wildmatch, including trailing-slash ** expansion; patterns and
branch names are bounded to 4096 bytes before dependency matching. Other condition
kinds remain potential Read dependencies, not evaluated semantic results.
Eight native Git onbranch comparisons and missing-path/order regression pass;
complete parser tests pass 6/6 (7897), /private/tmp/i300-target-lexical-frontier.log.
All-target Git Clippy passes (97282),
/private/tmp/i300-target-lexical-frontier-clippy.log. The production capture caller
uses no branch filter, retaining every declared dependency. Actual target-chain
file admission/traversal and final effects remain unimplemented, and production
conditional branch creation remains refused. Previous 297/297 library result
predates this extraction refactor; it is not final-source broad-test evidence.

Bounded pure configuration parser checkpoint (2026-10-14): captured admitted
source bytes now retain ordered repeated include declarations using the locked
pure gix parser, without opening include targets. Independent review identified
allocation amplification from repeated subsection prefixes and dropped valid
pre-section records. Both are corrected: cumulative 4 MiB key/value reservations
precede cloning/normalization, and frontmatter uses the same record decoder as
sections. Independent read-only follow-up confirms both corrections with no new concrete
parser blocker. The 4 MiB budget bounds decoded payload, not total process RSS;
parser events, buffers and collection capacity add bounded overhead. Record Debug
redacts keys and values. Native Git differential fixtures
cover frontmatter, case folding, empty/implicit values, escapes and continuations;
parser tests pass 4/4. All talos-tools library tests pass 297/297 in an approved
unsandboxed run (80997), log /private/tmp/i300-target-parser-all-tools-unsandboxed.log.
The first sandboxed run passed 296/297; its sole failure was process observation
EPERM, not parser behavior. Git all-target Clippy passes (86699), log
/private/tmp/i300-target-parser-frontmatter-clippy.log. No automatic include
traversal is enabled. Target-chain frontiers, conditional/hasconfig parity and
final effect binding remain to implement; creating-branch conditional Checkout
continues to refuse. These are local checkpoint results, not T7 completion or
final-candidate publication evidence.

Creating-branch conditional-config investigation (2026-10-14): isolated native
Git fixtures cover `checkout -b created` with source/target conditional hook
directories and `branch.autoSetupMerge=false/true/always/inherit/simple`, plus
`branch.autoSetupRebase=always`. All five select only the invocation's source
reference-transaction/post-checkout hooks; `always` writes a local upstream and
`inherit` copies the source upstream, both with rebase enabled. Evidence:
`/private/tmp/i300-create-onbranch-matrix.py` and
`/private/tmp/i300-create-onbranch-matrix.json`. These observations identify
configuration writes that a complete plan must authorize; they do not prove
absence of other target-config readers. Independent source/design review in existing reviewer `review_prepared`
identified Git 2.54
`remote.c::format_tracking_info` reading `status.comparebranches` via
`repo_config_get_string` after HEAD changes, which can reload repository config
cleared by tracking setters. Source-cached hook selection therefore cannot
justify permitting the target chain without discovery. Production creation
under conditional config remains closed pending complete chain evidence and
implementation; forcing tracking off is not a full solution.
The malformed-target native comparison confirms the distinction: false/true/simple
return success, but always/inherit return 128 after HEAD already becomes
`refs/heads/created`, reporting the target include's parse error
(`/private/tmp/i300-create-onbranch-reload.py`,
`/private/tmp/i300-create-onbranch-reload.json`). Git's config options have no
hypothetical branch override. Locked gix-config 0.60.0 has a branch-name context,
but its automatic include traversal performs unbounded file copies without an
admission callback; enabling that traversal would not meet Talos's evidence and
budget boundary. Reusing its pure parser requires a separately reviewed bounded
traversal, not automatic discovery. Follow-up independent boundary review found
that normal exact Path Read frontiers cover target-only files without expanding
the named configuration-chain capability. ADR-089 now records this implementation
route and its concrete parity/admission/effects gates. No additional maintainer
authorization is needed for these ordinary per-file permissions. The previous
claim that a new target-chain capability is required is superseded by this route.

Commit automatic-maintenance execution correction (2026-10-14): independent
review traced Git 2.54 commit → maintenance → gc → pack-objects to shell execution
of `gc.recentObjectsHook`. Parent Commit now refuses this configured command when
maintenance is enabled, matching the receiver boundary; disabled maintenance
retains the inactive configuration. Focused regression passes, Commit-filtered
suite passes 14/14, and Git-enabled all-target Clippy passes
(`/private/tmp/i300-commit-recent-hook-enabled.log`,
`/private/tmp/i300-commit-recent-hook-suite.log`,
`/private/tmp/i300-commit-recent-hook-clippy.log`). Independent review found no
concrete gap in this correction. The regression tests the guard and disabled
maintenance, not the native GC threshold; hook reachability is based on Git 2.54
source review. Standard preflight 56638 exited 0 with `release preflight: passed`
but predates this correction and cannot prove final-source validation. No
completion or publication evidence is claimed.
Post-correction standard preflight session 70906 exited 0;
`/private/tmp/i300-commit-recent-hook-standard-preflight.log` ends with
`release preflight: passed`, including both external SDK fixture modes.
Separate governance and Collaboration Claim checks also pass with zero warnings
(`/private/tmp/i300-commit-recent-hook-governance.log`,
`/private/tmp/i300-commit-recent-hook-claims.log`). This is local checkpoint
evidence; original T7 implementation and exact-head publication gates remain.

Previous-local-branch conditional Checkout extension (2026-10-14): `checkout -`
can now use branch-conditional configuration when an admitted, bounded native
`rev-parse --symbolic-full-name` query resolves `@{-1}` to `refs/heads/...` and
captured local branch commit agrees with native target resolution. Existing full
reference and reflog capture/revalidation remains mandatory. Detached previous
targets do not obtain this exception. The real regression exercises two reverse
switches with conditional markers; same-OID reflog-name drift now runs under an
active conditional include and still refuses. Checkout 28/28 and all-target
Clippy pass (`/private/tmp/i300-previous-conditional-suite.log`,
`/private/tmp/i300-previous-conditional-clippy.log`, session 49062). Independent
review found no concrete defect in this limited extension; prior preflight
predates this production change. The reflog regression proves drift after
discovery, while the separate active-config regression injects drift after admission.

Inactive conditional-source protection regression (2026-10-14): real prepared
Checkout now directly tests both empty and populated `includeIf.onbranch:changed`
destinations rewritten by the target tree. Both refuse before mutation, retaining
source bytes, index and branch and never creating the configured marker. Switching
to an existing branch that leaves those sources unchanged succeeds; the original
unconditional-source create-branch coverage is retained. Focused regression and
Git-enabled all-target Clippy pass (`/private/tmp/i300-inactive-include-protection.log`,
`/private/tmp/i300-inactive-include-clippy.log`, session 88038). This closes the
specific coverage limitation from the preceding review; no production change or
broader conditional-target support is claimed.

Real MCP conditional-config acceptance (2026-10-14): Checkout's transport matrix
now has 15 cases, adding approved/denied existing-local-branch switches with
external source/target `includeIf.onbranch` files. Success selects the target
configuration; denial preserves source branch/config/index; both preserve
external config bytes and reject replay. The matrix passes
(`/private/tmp/i300-onbranch-mcp.log`, session 46039), and test-target Clippy
passes (`/private/tmp/i300-onbranch-mcp-clippy.log`, session 20941). These test-only
additions are verified separately from the running standard preflight.

Existing-local-branch conditional configuration extension (2026-10-14): explicit
existing local branch Checkout now supports `includeIf.onbranch` under retained
initial configuration. Native target resolution must match the captured local
branch commit; full ref/config/source revalidation remains in force. Creating,
guessing and nonlocal target forms still refuse conditional
configuration. The real regression covers benign source→target→source switches,
active conditional-config drift after discovery, inactive target hook preservation,
and refusal once that hook becomes active. Checkout-filtered tests pass 28/28
(`/private/tmp/i300-onbranch-checkout-suite.log`, session 58994). Independent
follow-up review confirmed the hook-command correction below and found no
remaining concrete defect in this limited slice (Git 2.45.0/2.54.0 source basis;
not final candidate approval). Git-enabled tools all-target Clippy passes after
the correction (`/private/tmp/i300-onbranch-hook-clippy.log`, session 3048).
Standard preflight 23429 failed at process observation with sandbox EPERM;
post-correction unsandboxed preflight 72934 exited 0 and ends with
`release preflight: passed` (`/private/tmp/i300-onbranch-hook-standard-preflight.log`),
including tools 648/648 and both external SDK fixture modes. The test-only MCP
matrix extension was verified separately as recorded above. Governance and
Collaboration Claim validators pass with zero warnings
(`/private/tmp/i300-onbranch-governance.log`, `/private/tmp/i300-onbranch-claims.log`).
T7 remains Active; these local gates are not publication or completion evidence.

Checkout hook-command guard (2026-10-14): independent Git 2.54 review found
that active `hook.<name>.command` entries can execute post-checkout or
reference-transaction commands even when no hook file exists. Parent repository
configuration capture now refuses these entries before any Checkout mutation,
including when they arrive through an active branch-conditional include. The
28-case Checkout suite passes after this guard
(`/private/tmp/i300-onbranch-hook-checkout.log`, session 77469); the full workspace
preflight must be rerun from this source checkpoint.

Branch-conditional configuration characterization (2026-10-14): isolated native
Git 2.54.0 source→target, target→source and new-branch checkout experiments select
reference-transaction and post-checkout hook paths from the invocation's initial
branch configuration. Git config invoked inside those hooks observes the new
branch after HEAD changes. Evidence: `/private/tmp/i300-onbranch-characterize.py`
and `/private/tmp/i300-onbranch-characterize.json`. This disproves assuming that
all checkout effects switch configuration together. It does not prove the absence
of other post-switch configuration readers; the blanket refusal is superseded
only by the explicit existing-local-branch extension above. Other target forms
remain refused. No target-config shortcut
or temporary repository projection has been authorized or introduced.

Preparation-depth acceptance (2026-10-14): the real MCP Checkout matrix now has
13 scenarios, including an over-depth child deletion inventory with all ordinary
permissions allowed. The unchanged runner stage budget refuses before mutation;
deep sentinel, untracked file, index and branch remain unchanged, and replay is
rejected (`/private/tmp/i300-mcp-depth.log`). This verifies safe budget refusal,
not support for arbitrary-depth deletion. Standard workspace preflight session
49959 exited 0; `/private/tmp/i300-deletion-standard-preflight.log` ends with
`release preflight: passed`, including the external runtime SDK fixture. This is
local validation evidence; remaining implementation and exact-head delivery gates
still prevent T7 completion.

Full Git regression after deletion inventory (2026-10-14): 291/291 tools library
tests pass outside the sandbox (`/private/tmp/i300-delete-full-git-unsandboxed.log`).
The sandbox run passed 289/290 but process observation failed with EPERM; the
rerun verifies that case without weakening it. New inventory coverage proves the
byte limit is cumulative across discovery frontiers and counts the root in the
4,096-entry total, with no deletion on budget rejection. Authorized target/debug
cleanup and compact rebuild recovered disk space. This source checkpoint passes
standard workspace preflight; final stable-candidate validation and exact-head
gates remain outstanding.
Tools/MCP all-target Clippy also passes after the budget regression
(`/private/tmp/i300-delete-full-clippy-final.log`); formatting and diff checks pass.

Checkout deletion inventory (2026-10-14): gitlink→ordinary target now discovers
nonempty child entries through separately admitted exact Read frontiers. Completed
plans request exact Write for every file/directory and retain identity, directory
listings and bounded file evidence (4,096 entries / 16 MiB, unchanged eight-stage
runner budget). Native checkout performs the deletion; no preemptive removal or
automatic retry occurs. Missing child Write and late files refuse before mutation.
Sender/child effective configuration and auxiliary overlaps, symlinks/reparse
points and special entries refuse; embedded child `.git/config` remains protected.
Nonempty uninitialized and external-gitdir initialized layouts pass real MCP
allow/deny/replay cases, with external metadata preserved (12-case matrix,
`/private/tmp/i300-mcp-delete.log`). Checkout 26/26 pass
(`/private/tmp/i300-delete-final.log`), the strengthened admission regression passes
(`/private/tmp/i300-delete-admission.log`), and tools/MCP all-target Clippy passes
(`/private/tmp/i300-delete-clippy.log`). Independent source review found no new
concrete defect. Windows object identity and filesystem TOCTOU limits remain;
this supersedes historical blanket nonempty-child refusal, not final T7 gates.

Unix tracked-symlink→uninitialized-gitlink checkout is implemented with an exact
stage-zero index mode check. External and dangling referents remain unchanged;
dirty/late-changed links and protected auxiliary sources refuse with branch/index
preserved. Checkout tests pass 24/24 and Git all-target Clippy passes
(`/private/tmp/i300-symlink-checkout-suite.log`, `/private/tmp/i300-symlink-clippy.log`).
Independent source review found no new referent read or missing effect facet.
Windows reparse behavior is not enabled by this extension. Real MCP Checkout
passes the eight-case allow/deny and replay matrix including the external symlink
case (`/private/tmp/i300-mcp-symlink.log`); final delivery gates remain open.

Empty uninitialized gitlink restoration: preparation retains directory identity
and verifies emptiness again before execution. The late-child regression preserves
the added child and current branch on refusal. Checkout tests pass 23/23
(`/private/tmp/i300-empty-checkout-reviewed.log`) and Git all-target Clippy passes
(`/private/tmp/i300-empty-checkout-clippy.log`). Independent review found and
confirmed the correction of a read-order gap: parent and directory identity
checks now precede empty-directory enumeration. The Unix replacement-directory
regression passes. Windows PathIdentity currently compares resolved paths only,
so same-path directory-object replacement detection remains unproven there;
these checks do not remove the existing filesystem TOCTOU residual.
This supersedes historical refusal of every present directory; nonempty and
initialized child deletion remains unsupported.

MCP reverse Checkout evidence: the real transport matrix now covers six cases
(ordinary, file→gitlink, missing gitlink→file; each allowed/denied) and replay
refusal. Test 75414 passed, /private/tmp/i300-mcp-reverse-gitlink.log. Denied
reverse transitions preserve missing worktree state and original index; approved
transitions restore content and regular-file index mode.

Same-kind multiple-destination Push review (2026-10-14): independent read-only
review found no new concrete gap in ordered/duplicate destinations, URL rewrite,
full-profile token consumption, configuration drift checks or partial-failure
reporting. This review did not execute tests and is not an exact-head candidate
approval. Mixed local/network, SSH/HTTP and helper/hook effects remain open.

Windows CI now explicitly enables the git feature for both executable guard
regressions (real Git startup and replacement denial). This is a configured gate,
not Windows execution evidence. Current standard preflight 69747 exited 0;
/private/tmp/i300-checkout-current-preflight.log ends with release preflight: passed.
Production sources stayed unchanged during this invocation. Directory restoration
Clippy passed (52930). The added Windows step passes YAML and PowerShell syntax
parsing locally, checks test-name presence and propagates each exact test failure;
actual Windows execution is still required.

Directory restoration runtime evidence: the prepared checkout regression now also
restores file/child from a missing gitlink, verifies branch and bytes and absence
of child .git, and passes (35397; /private/tmp/i300-gitlink-directory.log). This
adds runtime evidence to the prior source review; it does not cover initialized
children or recursive checkout.

This table supersedes historical running/refusal checkpoints below for current
status only. Published scope and acceptance remain unchanged. No implementation
commit, PR, exact-head CI or completion evidence has been produced for this slice.

| Area | Current authoritative state | Remaining delivery work |
|---|---|---|
| Add / Commit | Prepared discovery and mutation dispatch are active; historical “mutation disabled” notes are superseded | Final stable-candidate real Agent/MCP evidence, review and publication |
| Checkout | Ordinary switches including explicit existing local branches with conditional configuration; nonrecursive gitlinks, tracked regular-file/Unix symlink→gitlink; reverse transitions support missing/empty and exactly admitted nonempty child inventories including external-gitdir layouts | Conditional config for creating/guessing/other targets, embedded effective configuration deletion, symbolic/special deletion entries, recursive checkout and remaining configured behavior; final delivery gates |
| Push | Direct credential-free git:// and local receiver mutation are implemented, including same-kind multiple destinations | SSH/HTTP/custom helpers/hooks; mixed local/network receive-pack semantics; platform and final integration evidence |
| Pull | Prepared configuration, graph/source evidence and policy/cleanliness checks run; `Phase::PullFetch` still returns an explicit unimplemented error | Maintainer decision on proposed metadata capability, then production fetch and separately admitted integration, failure/cancellation and real Agent/MCP coverage |
| Authorization | Accepted configuration, foreground maintenance, Trace2, receiver references and receiver metadata contracts have implementation | Pull `git:pull-repository-metadata` remains proposed and compiler-rejected; never infer acceptance from a question submission receipt |
| Platform | macOS local verification exists | Windows process lifecycle, executable identity and native tests/CI remain unproven |
| Gates | Post-hook-guard standard preflight 72934 passed, tools 648/648 and both SDK modes; subsequent test-only MCP and inactive conditional-source extensions and Clippy pass separately; governance/claims pass | Full stable final-candidate validation, independent exact-head security/API review, CI, CAS, merge and owner completion SHA |

Immediate implementation sequencing: finish supported Checkout boundary evidence
without enlarging permissions; separately design mixed Push/native transport
semantics. Pull write implementation cannot be activated before its outstanding
contract decision. T8 and all later serial handoff items remain unactivated;
passing any row above does not close T7 or the full handoff.


Reverse Checkout review complete (2026-10-14): independent review confirms missing/uninitialized gitlink→ordinary file or target subtree is safe under retained path/source/parent evidence; initialized children, existing directories and nonregular markers still refuse. Full checkout-filtered suite and all-target Clippy passed (16698), logs /private/tmp/i300-reverse-checkout.log and /private/tmp/i300-reverse-clippy.log. Governance/claim checks remain clean from 79370. This closes the bounded Checkout type-transition slice only; Pull mutation, transport effects, cross-platform gates and final publication remain open.

Missing gitlink reverse implementation (2026-10-14): supersedes the refusal-only audit below. Checkout permits reverse type changes only when retained gitlink evidence is Missing with no child metadata; present directories/files/links still follow existing refusal. Target-effect protection now includes gitlink→ordinary file. Real forward/dirty/config-protected fixture also restores the original regular file from a missing gitlink and passes (61297), /private/tmp/i300-reverse-gitlink.log. Broader checkout suite/Clippy running at /private/tmp/i300-reverse-checkout.log and /private/tmp/i300-reverse-clippy.log; independent review requested. Original T7 scope and full closure gates remain open.

Reverse gitlink-type audit (2026-10-14): native Git restores a regular file when an uninitialized gitlink is checked out in the opposite direction, but the current prepared path conservatively refuses all existing-gitlink→file transitions because initialized child metadata/content and recursive behavior require separate admission. This remains an explicit compatibility residual; no unsafe widening was made.

Checkout extension validation (2026-10-14): complete Git-enabled tools suite passed 287/287 unsandboxed (46512), log /private/tmp/i300-checkout-final-git-suite.log. Latest all-target Clippy passed (16911); governance and Collaboration Claim validators passed with zero warnings (79370). This validates tracked regular-file→uninitialized gitlink support, dirty preservation and effective include-source rejection locally. Standard preflight 26739 passed before this extension; final stable-candidate preflight/exact-head review/CI remain required. Pull metadata contract remains proposed, actual fetch/integration disabled, mixed transport/SSH/HTTP/helpers/hooks and other original residuals remain open. No completion commit or publication evidence exists.

Checkout protected-source regression (2026-10-14): real file-to-gitlink fixture now configures include.path=../file, verifies effective-configuration refusal, and retains branch/file/index before removing the include and exercising dirty refusal/clean success. Focused regression passed (19298), /private/tmp/i300-gitlink-config-refusal.log. Prior broader checkout suite and Clippy passed (50392); latest test-only Clippy passed (16911), /private/tmp/i300-gitlink-config-clippy.log. Independent review confirms target-effect inclusion is required and now present. Production Pull and other original T7 gates remain incomplete.

Checkout file-to-gitlink implementation (2026-10-14): nonrecursive checkout now permits a target gitlink replacing only an already tracked regular-file source. Existing directories, symlinks and other added-path states still refuse. Config/hook target-effect detection now includes this replacement rather than excluding all target gitlinks. Native Git experiment and prepared regression (79376, /private/tmp/i300-file-gitlink.log) confirm clean success to uninitialized directory and dirty rejection preserving branch, bytes and index. Independent review checked native nonrecursive child behavior; final diff review and broader checkout suite/Clippy remain required. Mixed Push test-only Clippy passed (35818). Prior standard preflight 26739 passed before this production extension and is not final-source gate evidence. No T7 completion claimed.

Stable standard gate passed (2026-10-14): 26739 exited 0; /private/tmp/i300-stable-capacity-preflight.log ends with release preflight: passed, including workspace tests and both SDK consumers. Implementation sources stayed unchanged during this run. Subsequent test-only mixed-Push regression passed (98607), /private/tmp/i300-mixed-refusal.log. Reviewer corrected the proposed mixed-transport implementation: global --receive-pack embeds the bound local Git command, but git:// sends it as daemon service, so merely removing the mixed rejection is invalid. Preserve rejection until per-destination semantics are implemented and reviewed. New test exercises both local/network URL orders and verifies neither receiver acquires the target ref. Production Pull and original delivery scope remain incomplete; no publication/completion SHA.

Stable-source preflight restart (2026-10-14): 66523 terminated exit 101 on sparse cumulative-output budget, having compiled the intermediate 16 MiB probe cap. Corrected 1 MiB tree probe passed the same 300-file/six-stage regression (27792) and all-target Git Clippy (31503). Full standard preflight restarted after terminal confirmation in 26739, log /private/tmp/i300-stable-capacity-preflight.log; keep implementation sources unchanged while it runs. No full preflight pass claimed. Independent reviewer requested to identify an accepted-contract Push implementation step while Pull metadata decision remains pending.

Sparse capacity validation in progress (2026-10-14): sparse fixture now includes 300 additional identical-content files, exceeding old 16 KiB ls-tree output without expanding object count. Initial 16 MiB command cap exhausted the unchanged 64 MiB cumulative worst-case stdout/stderr budget; adjusted only ls-tree probe cap to 1 MiB, retaining scalar 16 KiB and all cumulative/stage limits. Retry 27792 at /private/tmp/i300-large-sparse-retry.log passed the real 300-file sparse fixture, including unchanged six-stage reservation, fresh descendant admission and unchanged index. Git all-target Clippy passed (31503), log /private/tmp/i300-large-sparse-clippy.log. Standard preflight 66523 at /private/tmp/i300-reviewed-standard-preflight.log is live; sources changed during this invocation, so it cannot alone prove final stable-source validation. Prior reviewed Pull suite and Clippy 73152 exited 0. Production Pull and unanswered metadata contract remain open.

Reviewed Pull graph checkpoint (2026-10-14): independent git_plan_review found no new admission-order bypass after moving exact-admitted inventory advance ahead of full-refs readiness. Added explicit local inventory revalidation before/after sparse ls-tree per review. Prior full Pull regression passed 38/38 (4082); final revalidation change tests and Clippy run in 73152, logs /private/tmp/i300-reviewed-pull.log and /private/tmp/i300-reviewed-clippy.log. SDK lock repair validation 97926 passed both default and coding consumer runs. Governance and claims passed with zero warnings (35425). These supersede running-state notes above; full standard preflight must still be rerun successfully, and production Pull/remaining T7 acceptance remains open.

Pull replacement-symbolic correction (2026-10-14): early sparse ls-tree now requires the complete refs subtree in both metadata roots and current exact Read tokens, because replacement symbolic refs can target deep branch refs. Move the already-admitted inventory advance before readiness evaluation; unchanged six-stage sparse reservation test passes (78384). Added deep symbolic-target frontier regression (40928); focused log confirms pass. Broader Pull regression running in /private/tmp/i300-full-refs-pull.log. Standard preflight 48514 ended exit 1: workspace tests passed, external SDK fixture failed stale locked gix-hash SHA-256 dependency. Updated fixture lock offline without version upgrades; SDK validation retry 97926 remains running at /private/tmp/i300-sdk-lock-repair.log. No full preflight success or production Pull completion claimed. Pending Pull metadata decision unchanged.

> Document status: Active
> Published plan date: 2026-10-11
> Planned objective: deliver the complete T7 structured Git repair from the session handoff.
> MVP deliverable: approved literal two-file staging through real Agent admission and faithful staged/unstaged built-in diffs.

## Collaboration Claim

Order-file verification (2026-10-14): config tests 17/17 and Git Clippy passed (84571); external orderFile prepared-flow fixture verifies exact admission, retained evidence and invalidation after content drift (96819). Original production/delivery gates remain outstanding.

Diff external-effect audit (2026-10-14): clean probes disable external diff/textconv/renames and index refresh, bind `diff.orderFile`, and retain frozen environment. Clippy/governance/claim validation pass (47575); production Pull mutation remains disabled.

Diff order-file boundary (2026-10-14): Pull clean comparison now admits configured `diff.orderFile` as an exact external read before `numstat`; Pull suite 36/36 (73774). Production fetch/integration and full security review remain open.

Decision receipt correction (2026-10-14): async question submission was not treated as approval; provisional Pull metadata permission was reverted. Rejection regression test, governance and claim validators pass (32219). ADR-089 remains proposed and production Pull mutation remains disabled.

Git-enabled package regression validation (2026-10-14): full locked talos-tools Git-feature suite passed unsandboxed (98071) after sandbox EPERM on maintenance-process observation. This does not establish workspace or final publication gates; production Pull fetch/integration remains unimplemented.

Gitlink prefetch validation (2026-10-14): ninth cleanliness scenario stages a gitlink and passes preparation with unchanged index (36897), matching the audited prefetch ignore rule. Integration semantics and full task delivery remain incomplete.

Prepared-history verification (2026-10-14): replacement-ref guarded Pull suite 36/36 and Git Clippy pass (77756). Sparse fixture now consumes actual PreparedStageHistory including configuration and reserves both future stages (71056); their real effect profiles and execution remain unimplemented, so this is fixture capacity evidence only.

Sparse stage-budget repaired (2026-10-14): early tree-name discovery requires complete, individually admitted object descendants and shares remaining reference continuations. Unchanged six-stage sparse tracking assertion passes (30162); Pull 34/34 and Git Clippy pass (45623). This supersedes the budget failure checkpoint, not the remaining security/production/delivery work.

Sparse tracking budget failure (2026-10-14): strengthened real origin/Topic sparse fixture requires seven preparation stages, leaving insufficient capacity for both fetch and integration (46281). Keep failing reservation assertion; coalesce discovery without changing lifecycle bound. Prior Clippy passed (41153); no sparse delivery claim.

Sparse preparation progress (2026-10-14): admitted tree enumeration now generates descendant source/attribute facets and fresh continuation; real sparse fixture confirms admission boundary and unchanged index. Pull suite 34/34 (85523). Sparse stress/stage-budget/security review, current validation gates and production fetch/integration remain open.

Sparse control-file update (2026-10-14): `info/sparse-checkout` is now exact Pull evidence with absent/create/change/link invalidation; Pull suite 34/34 (47281). Sparse tree expansion and production fetch/integration remain open.

Numstat edge evidence (2026-10-14): expanded prepared-flow fixture passes binary, deletion and executable-mode changes in addition to clean/touch/staged/dirty/autostash (2151). All retain original index bytes. Sparse expansion and original delivery scope remain open.

Sparse boundary evidence (2026-10-14): real sparse-index prepared rebase fixture passes (43727), proving existing refusal before sparse source expansion. This is not implemented sparse support; admitted tree expansion and descendant facets remain required. Clean suite 32/32 and Clippy passed in 67029.

Touch-only repair evidence (2026-10-14): numstat content comparison with index refresh disabled passes real clean/touch/dirty/staged/autostash prepared-flow fixture (88270), retaining index bytes. Pull suite/Clippy run 67029; sparse, submodule/config edge coverage, security review and production execution remain outstanding.

Open regression (2026-10-14): native clean comparison implementation is provisional; new touch-only fixture fails because diff.autoRefreshIndex=false avoids index mutation but produces a dirty result. Repair without writes under Read admission before publication. Details and resume path are in the session handoff; no completion claimed.

Filter runtime verification (2026-10-14): Pull suite after production filter check passed (85325); real helper-refusal fixture passed (15321), with no helper marker or FETCH_HEAD. Native clean comparisons and original scope remain incomplete. Clippy log: /private/tmp/i300-source-attribute-clippy.log.

Execution update (2026-10-14): admitted Pull attribute snapshots and source snapshots passed 30 Pull tests (64983); prior governance/claims passed (76456). Filter inspection was subsequently added and is being validated in 85325 (/private/tmp/i300-pull-filter.log). Remaining native clean comparisons, sparse expansion, helper-negative fixture, security review and all original delivery gates remain open.

Execution checkpoint (2026-10-14): Pull prefetch retains admitted tracked-source snapshots with parent identity checks before/after capture and revalidation. Content/missing-source/parent-link tests added; verification pending in session 65412, log /private/tmp/i300-pull-source-evidence-compact.log, after ENOSPC recovery by authorized target/debug cleanup. Attribute/filter/clean comparisons, sparse expansion and production fetch/integration remain open; no completion evidence added.

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | T7 structured Git typed effect resources, same-family scope defects and staged/unstaged diff fidelity; shared Auto prepared-command admission required for native Pull activation, with associated tests and documentation. Other built-in Auto coverage remains PERM-007-H. |
| Claimed At | 2026-10-11 |
| Source Issue | None |
| Governance Claim PR | #755 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized serial implementation and gated merges, accepted ADR-089, and permitted non-overlap with I293/I294. No separate maintainer reviewer is available; independent Agent security/API review, exact-head CI, validators and CAS are required. |
| Implementation PR | #735 (Draft; T7 incomplete) |
| Last Updated | 2026-10-23 |
| Handoff / Release Condition | No implementation before accepted contract and target-branch claim. |

## Published Baseline

Pull attribute-path admission review (2026-10-14): independent review found no new authorization defect. The new path set includes worktree ancestor .gitattributes, shared common_dir/info/attributes (correct for linked worktrees), and Git var GIT_ATTR_SYSTEM/GLOBAL results; the latter follows effective core.attributesFile/XDG defaults. Relative custom attribute paths remain conservatively refused by existing absolute-path helper. No attribute contents are read until fresh exact Read admission. Git-enabled tools Clippy passed (1987); Pull budget fixture passed (99417). Remaining source/attribute capture, filter checks, worktree comparisons, sparse expansion, and production fetch/integration stay open.

Late-path continuation correction (2026-10-14): a completed inventory frontier that adds source or fixed evidence Read facets now returns a fresh continuation; inspection runs only after a subsequent proof consumes those exact tokens. This prevents newly discovered worktree paths from being read under the prior graph-only proof. Pull-filtered suite remains 26/26 (44015). Sparse expansion, source/attribute capture, worktree comparisons and production fetch/integration remain open.


Early pure-index discovery verification (2026-10-14): ordinary source-name facets now share existing inventory continuations after exact index Read checks; real rebase stage-budget test passes without extending eight stages (63793). Current Pull-filtered suite passes 26/26 (80571). Independent review found no new authorization bypass; source discovery remains lexical, and unapproved paths at a completed frontier are refused. Follow-ups: return a fresh continuation for such late-added paths rather than an error, narrow source requests to relevant policy, then complete attribute/source binding and cleanliness checks. Early object-format stderr handling now matches later probes. Pure decoder validates nonzero SHA1/SHA256 checksums while allowing skipHash zero trailers (28486); its 16MiB input/single-allocation limits are not a total-memory claim. No production fetch/integration enabled.

Pure-index/SHA-256 checkpoint (2026-10-14): Pull index decoding now retains the exact RegularEvidence used for parsing and requires PullEvidence to match it before accepting conflict/rebase decisions. The decoder is bounded, catches panics, rejects shared-index links, and does not invoke Git sparse expansion. gix sha256 support was explicitly enabled so SHA-256 index decoding is available; locked cargo check for talos-tools passed (95275), with the expected two-line Cargo.lock feature resolution change. The existing rebase stage-budget failure remains; sparse directory expansion and worktree clean comparisons remain unimplemented. No fetch/integration capability is enabled.

Sparse-index read-boundary finding (2026-10-14): independent Git 2.54 source review shows repo_read_index may call clear_skip_worktree_from_present_files, probe worktree metadata and expand sparse trees even for ls-files --cached --sparse. Therefore existing Pull ls-files probes are not proven index-only in sparse configurations; do not advance or enable fetch on that basis. Next implementation must replace them with bounded pure index decoding (gix::index::State::from_bytes accepts bytes without implicit filesystem expansion), retain mode/OID for sparse directories, and admit tree/worktree descendants separately. This also enables ordinary index-name discovery to share an earlier inventory continuation and fix the preserved rebase budget failure. No stage limit change is authorized; existing failing rebase test remains a required repair.

Rebase stage-budget regression (2026-10-13): source and missing tracking evidence facets now coalesce into one fresh continuation, avoiding a needless extra stage. Pull suite passed 25/25 before strengthening the budget fixture. The real shared-stage fixture now explicitly sets pull.rebase=true and FAILS (32168): preparation cannot reserve both fetch and integration within eight stages. Preserve this failing assertion; do not widen the lifecycle contract or claim stage reachability. Next: coalesce source discovery with an already necessary admitted index frontier/config continuation, or implement accepted metadata discovery only after its pending maintainer decision. Path enumeration still uses 16 KiB and needs a deliberate bounded inventory limit for normal larger repositories. No worktree content read or mutation is enabled.

Pull clean-worktree path-discovery checkpoint (2026-10-13): production prefetch now identifies when rebase without autostash requires a clean worktree, then parses only ls-files --cached -z names into safe relative workdir paths. It requests exact Read facets and a fresh continuation before any worktree source capture; it rejects absolute, dot, dot-dot, empty-component, backslash and unterminated names without touching the worktree. This is only the admission/inventory half: content snapshots, attribute/filter checks, separate HEAD/index and index/worktree comparisons, touch-only semantics and full stage-budget evidence remain. Pull suite remains 24/24; literal-path parser focused test passed (38971).

Pull checkout identity checkpoint (2026-10-13): prefetch initial and upstream-discovery probes now compare both branch and HEAD via same_checkout, while allowing newly discovered fork-point evidence. Focused tests cover unborn/born, same-branch OID changes, detached transitions and fork-point addition (1235). Independent review found no concrete defect and confirmed 16 KiB probe overflow propagates an error rather than a truncated clean result; the limit is unchanged. Current Git all-target Clippy passed (26224). Production clean-worktree admission and fetch/integration remain incomplete; no new permission contract is enabled.

Pull rebase clean-worktree implementation boundary (2026-10-13): independent review identified the next production step: enumerate tracked names solely from the already admitted index, then request exact file and ancestor/system/global attribute Read in a fresh continuation before source capture. Do not call tracked_commit_sources before admission: literal_add_paths/capture_gitlink inspect the worktree. Reuse capture_tracked_sources and existing check-attr filter checks after admission; preserve promisor/fsmonitor refusal and independently compare HEAD/index and index/worktree. Native require_clean_work_tree ignores submodule worktrees; staged gitlink OID changes still require native comparison. Verify touch-only changes, no index writes, dirty/autostash policy and the eight-stage budget. Missing/empty/staged initial-index positive/negative fixture passed 24081; timestamp fix received final independent review with no new concrete defect. This records the concrete next implementation, not completed clean-worktree support.

Pull unborn-index timestamp correction (2026-10-13): independent review found Git 2.54 is_index_unborn rejects an existing empty index based on nonzero index mtime, even with no entries. PullEvidence now retains exact index existence and mtime evidence, rechecks it during capture/revalidation, and applies the native seconds-to-u32 nonzero rule before the rebase initial-index probe. Real staged and existing-empty-index rejection scenarios pass; timestamp-only drift invalidation passes; Pull-focused suite passed 24/24 in earlier run, and current talos-tools Git Clippy passed (88118). Full preflight must be rerun for this source change; metadata contract and production fetch/integration remain pending.

SDK recovery completed (2026-10-13): validator retry 46519 exited 0, with both default and coding external-consumer executions passing after space recovery. Combined local evidence now includes workspace check/Clippy/tests from 60116 (tools group 627/627; not the total workspace count) and the successful SDK retry. The original standard preflight still exited 1 on ENOSPC; do not relabel that invocation as passed or claim fixed-head publication gates. No build remains live from these sessions. T7 is still Active: production Pull fetch/integration and all documented transport/security/semantic residuals remain open, as do subsequent serial tasks.

Full workspace and SDK recovery checkpoint (2026-10-13): sandbox-external standard preflight 60116 completed workspace check, Clippy and workspace tests, including tools 627/627 and the previously EPERM-blocked eight-stage maintenance-cancellation test. SDK default execution passed; coding compilation then failed in arborium-cpp archiving with errno=28 (disk full), so the standard preflight exited 1, not passed. Under existing target/debug cleanup authorization, only completed test executables named in that log were removed (1,844,082,960 bytes); dependencies and source were retained. SDK validator retry is live as 46519 (/private/tmp/i300-sdk-space-retry.log), default mode already passed again. Do not restart the live retry. Separate fixture target cleanup remains unapproved and untouched. Full fixed-candidate delivery gates and all T7 scope remain open.

Full preflight sandbox result (2026-10-13): session 86088 ended with exit 101. Workspace check/Clippy completed; tools workspace test group passed 626 and failed one maintenance-cancellation fixture because spawning ps returned macOS EPERM at prepared_add.rs process observation. No skip or weakening applied. The tool escalation was approved and the standard full preflight is now running outside the sandbox as session 60116, log /private/tmp/i300-preflight-unsandboxed.log. Corrected Pull-filtered tests passed 23/23 (97952). Neither this mutable-worktree rerun nor its pending result establishes exact-head final delivery.

Pull index authorization correction (2026-10-13): independent review found the new unmerged probe needed exact index Read. git_dir/index is now fixed evidence and a configuration-continuation facet; absent/changed index is bound by revalidation. Reviewer verified Git 2.54 ls-files does not refresh/write the ordinary index; existing RepositoryState sharedindex.* refusal excludes split-index implicit reads/freshening. Focused conflict regression passed (14879), and corrected Pull-filtered verification finished (97952; see /private/tmp/i300-pull-index-authorized.log). No dynamic Pull capability enabled. Full preflight 86088 remains live in workspace tests, not final exact-head evidence.

Pull conflict-index production checkpoint (2026-10-13): admitted prefetch inspection now runs a read-only, owned ls-files --unmerged -z probe and refuses conflict entries before fetch. A real conflict-stage index regression is added; its focused locked verification is live in session 14879 (/private/tmp/i300-unmerged.log), waiting behind the confirmed live full preflight 86088. No pass is claimed yet. Independent review of MERGE_HEAD confirms exact token checks precede capture and no new continuation is added. Remaining rebase cleanliness, dynamic metadata decision, transport/integration and original T7 gates are unchanged.

Pull pending-merge production checkpoint (2026-10-13): MERGE_HEAD is now a fixed exact Read resource retained by PullEvidence. Prefetch preparation refuses any existing marker (including empty content); creation after evidence capture invalidates revalidation. Pull-filtered locked tests pass 22/22 (27237). Unmerged-index and rebase cleanliness checks remain outstanding; no fetch mutation or pending metadata capability is enabled. Full standardized preflight is live in session 86088, log /private/tmp/i300-preflight-current.log; do not restart it merely for an observation timeout or treat its mutable-worktree result as exact-head final evidence.

Pull unborn semantic evidence (2026-10-13): a real native comparison now verifies empty-tree read-tree coordination followed by compare-and-swap HEAD update for an initial pull. Clean and disjoint staged files succeed with identical HEAD/index/worktree/diffs; conflicting untracked contents are preserved and both routes fail. Focused locked test passed (16421). This test is preparation for production integration, not enabled Pull execution; metadata contract acceptance and complete effect binding remain outstanding.

Pull reconciliation selector checkpoint (2026-10-13): the test-only integration selector now returns an explicit Reconcile plan when fetch moves the checked-out born HEAD, before candidate parsing or policy rejection. The real clean/disjoint/conflicting fixture executes that plan before selecting merge; an empty FETCH_HEAD assertion verifies reconciliation ordering (focused locked test passed, session 70131). Independent read-only review found no concrete defect in ordering or original/current HEAD selection. Production must bind successful reconciliation evidence to HEAD, index/worktree and FETCH_HEAD and revalidate before fresh integration authorization; the test-only boolean is not an execution credential. Dedicated Pull metadata acceptance remains pending, no fetch mutation is enabled, and T7/I300 remains Active with all original delivery gates outstanding.

Pull current-branch semantic checkpoint (2026-10-13): while the dedicated Pull metadata contract remains pending maintainer response, a real native comparison now covers fetch updating the checked-out branch with --update-head-ok. Clean, disjoint dirty and conflicting dirty worktrees compare native pull against fetch plus read-tree -u -m old new plus merge. The fixture compares HEAD, file bytes, index entries, status and staged/unstaged diffs. All three pass (9652); all-target Git Clippy passes (61931). Conflict leaves the fetched HEAD update in place while preserving local bytes, proving a partial-effect reporting requirement. This is semantic evidence only: production reconciliation still refuses pending its complete admitted effect plan, and no pending permission contract was activated. Next is explicit reconciliation/target binding after maintainer decision, not treating this fixture as delivered Pull.

Pull dynamic-metadata contract checkpoint (2026-10-13): inspection confirmed exact Path grants cannot authorize unknown fetched object names, locks or maintenance outputs; accepted receiver metadata remains git_push-only. ADR-089 now contains a proposed, inactive git_pull-only metadata Read/Write scope paired with exact identity-bound gitdir/common roots and a bound Git Command. Independent review requested explicit index/merge/rebase/autostash/local tracking-config coverage; the proposal now names it and requires a fresh integration Write plan separate from fetch. Worktree/external paths/helpers/network remain separately admitted. Maintainer acceptance was requested asynchronously; no reply or activation is recorded here. No fetch mutation is enabled. Governance/claims passed (29480/37518) before the final scope wording addition. Next: consume the maintainer decision without re-requesting it; if accepted, implement the dedicated compiler/executor checks, replace the per-level production inventory, then complete fetch/integration and real Agent/MCP gates. Original scope and Active status remain unchanged.

Pull stage compression checkpoint (2026-10-13): a completed admitted inventory frontier now proceeds to prefetch inspection under its consumed exact-path proof rather than returning an identical-profile continuation. Fixed evidence paths (logs/HEAD, shallow and info/grafts) are requested in the configuration continuation even when absent; the completed graph set still checks every path token before probes. Dynamic missing tracking evidence still produces a fresh identity/admission. Independent review confirmed the retained token checks and found no new concrete defect. Pull-filtered tests pass 19/19 (59970); ordinary origin/Topic tracking with a real core PreparedStageHistory passes (47492). The strengthened test reserves both future fetch and integration positions; this does not execute those operations or prove arbitrary deep reference capacity. All-target Git Clippy passes (7939) before the final capacity assertion. Remaining next work is actual fetch transport/effects and integration admission, real Agent/MCP execution, deeper-layout stage policy and all original T7 delivery gates. No completion or publication is claimed.

Pull exact-frontier checkpoint (2026-10-13; supersedes the no-producer checkpoint below): production configuration continuation now seeds a bounded exact-path inventory. Each admitted frontier enumerates directory names only, returns fresh identity/facets, and defers child metadata to the next admission. Existing path identities and directory listings are retained; new entries and symlinks refuse. PullEvidence exposes a shared required-path set; absent reflog/grafts paths are added without probing and require a separate fresh admission before capture. Pull-filtered tests pass 19/19 (45082), including unapproved-child, added-entry, symlink and absent-evidence checks; Git-enabled all-target Clippy passes (85713) before the final assertion additions. Independent review confirms the frontier/absent-path mechanism but identifies unresolved eight-stage capacity: ordinary tracking reflog depth plus configuration, supplemental evidence and fetch/integration can exhaust the runner limit. The empty-repository test now uses core PreparedStageHistory; this is not typical-repository or complete Agent/MCP reachability evidence. Do not widen the bound silently or claim Pull delivery. Next: compress admitted discovery stages and verify typical tracking repositories through the real runner budget, then complete fetch/integration effects and all original T7 gates. No implementation commit/publication or task completion is claimed.

Pull production configuration checkpoint (2026-10-13): GitPullTool now
prepares an owned, identity-bound configuration discovery stage using the accepted
closed configuration-chain capability. It grants no Write/Network authority and
requires fresh continuation admission. Ordered Pull configuration conversion,
owned prefetch inspection and evidence retention are implemented internally, but
independent review found root Read does not authorize descendant reflog/object
bytes. Production prefetch therefore refuses before probes until a complete
bounded graph-resource inventory is independently admitted; the private inventory
has no producer yet. Direct inspection fixtures are semantic evidence only.
Review confirms this guard has no production bypass. Tests cover tool preparation
and cleanup receipt, missing/stale proofs, changed configuration, configured
remote retention, reflog changes and promisor rejection. Full Git-enabled tools
library passes 264/264 (4916); the first sandbox run could not execute ps for an
existing maintenance test, so the permitted rerun verified that case. Graph
inventory/admission, actual fetch/integration and all prior T7 delivery gates
remain open. This intermediate stage does not close Pull or T7.

Repository-location and Pull-evidence checkpoint (2026-10-13): production
AddPlan revalidation now checks retained workdir/gitdir/common identities and
location pointers before repository rediscovery, preventing reads through a
changed commondir or replaced .git root. PullEvidence remains test-only: it
binds bounded reflogs, shallow/grafts and post-fetch FETCH_HEAD bytes, rejects
parent symlink mappings, and detects absent-leaf creation. The additional shared
file-evidence budget is 16 MiB. Independent review found no further specific
regression after root identity binding; non-Unix identity strength and path
TOCTOU remain limitations, not confinement guarantees. Prior execution evidence:
Git-enabled tools library 260/260 passed (12658), all-target Clippy and targeted
location tests passed (36520). Production Pull stage wiring, current-branch
reconciliation, unborn mutation, transport/helper effects and full delivery
gates remain open. No implementation commit or T7 completion is claimed.

Pull split-integration checkpoint (2026-10-13): test-only BeforeFetch now
selects an executable merge/rebase continuation from bounded FETCH_HEAD and
owned-runner graph probes. It preserves ff-only precedence, unspecified-policy
divergence refusal, rebase fast-forward optimization, retained fork-point/octopus
selection, rebase mode and selected autostash. Twelve real scenarios compare
explicit fetch plus selected integration against native Pull for status, final
tree and commit subjects; both paths verify dirty edit restoration and stash
state. Probe infrastructure failure propagates. Post-fetch symbolic HEAD/commit
movement refuses explicitly; native current-branch reconciliation is still to
be implemented, not silently approximated. Independent review identified this
boundary and missing split-worktree assertions; both are now explicit/covered.
Eight Pull-state tests and all-target Clippy pass (79647). Production Pull still
requires configuration/fetch/integration stage wiring, ref/reflog/FETCH_HEAD
snapshot binding, unborn mutation and recursive/transport/helper effects.
T7 and all subsequent handoff tasks remain incomplete.

Pull missing-upstream checkpoint (2026-10-13): pre-fetch probes now quietly
resolve the configured tracking commit before fork-point traversal. A normally
absent reference skips fork-point; malformed successful evidence and process
failures still refuse. The real native Pull fixture deletes its tracking ref
before inspection and verifies successful rebase with no retained fork-point.
Eight Pull-state tests (2717), owned-runner budget/movement regression and
all-target Git-enabled Clippy (85755) pass. Production Pull, reference/reflog
snapshot binding and all remaining T7/handoff delivery gates remain open.

Pull upstream-mapping prototype (2026-10-13): tracking_reference now derives
fork-point input from the matching branch.remote, first branch.merge and first
matching positive fetch mapping; negative fetch mappings do not erase existing
upstream lookup. gix validates Fetch refspec syntax inside catch_unwind before
mapping. Native source remote.c set_merge/branch_get_upstream was inspected.
Eight Pull-state tests pass (10091), including a real refs/custom/Topic mapping
compared to Git @{upstream} before actual Pull; Clippy passes (61824). Explicit
other remotes do not reuse branch upstream. Local '.' full refs are covered;
local abbreviated/DWIM refs still require admitted discovery, not guessed paths.
Test-only status remains: production fetch authorization and integration are not
wired, and these checks do not close T7 or later handoff tasks.

Pull remote-selection prototype (2026-10-13): verified upstream v2.54 remote.c
remote_for_branch semantics (local source /private/tmp/i300-remote-2.54.c).
fetch_remote preserves explicit input, branch subsection case/last remote value,
sole configured remote fallback, then origin. It preserves '.' and remote names
containing dots; pushDefault does not influence fetch. Unrepresentable remote
names refuse rather than silently changing destination. Seven Pull-state tests
pass (84326), including actual native Pull using branch remote Up.stream and
matching fork-point tracking reference; Clippy passes (8188). Selection remains
test-only, not an admitted production transport plan. Original T7/full handoff
scope, fetch effects, integration effects and publication gates remain open.

Pull actual probe prototype (2026-10-13): BeforeFetch::probe now executes
symbolic-ref, quiet HEAD commit lookup and (for configured rebase with a born
branch) merge-base --fork-point through a caller-owned bounded runner. Only
quiet exit 1 denotes absent; transport/collection failures propagate. It rereads
branch/HEAD and rejects movement across inspection. All nine pull-filtered tests
pass (76470); real owned-runner unborn/budget/branch-movement test passes (39431).
Clippy passes (23312 before the added movement fixture, final run 8762).
The rewritten-upstream baseline now obtains retained evidence from actual probes.
Still test-only: production configuration-stage permission binding, reference/
reflog snapshots and post-admission revalidation, fetch/integration continuation,
and Windows remain outstanding. No Pull delivery or T7 completion claim.

Pull retained pre-fetch prototype (2026-10-13): BeforeFetch captures owned
branch/original HEAD/fork-point/config policy from completed admitted probe
outputs. It distinguishes detached HEAD and unborn branches, validates object
width/nonzero hex/newline framing, and rejects fork-point without a branch and
original HEAD. Six filtered Pull tests pass (26723), including retained pre-fetch
values in the real rewritten-upstream fixture; all-target Clippy passes (17404).
This remains test-only: production probes, snapshot revalidation, authorized
fetch and integration continuations are not wired. Next work must bind this state
to the admitted executor rather than treating the prototype as delivered Pull.

Probe integration verification (2026-10-13): complete Git-enabled tools library
passes 253/253 before the checkout caller migration (48303). Independent reviewer
git_plan_review found no definite output-collector regression; non-Unix descendant
cleanup parity remains open. Checkout now uses a quiet commit-target probe:
only exit 1 with both streams empty permits candidate guessing; budget/spawn/
timeout/diagnostic failures propagate instead of being swallowed by fallback.
All 22 checkout-filtered tests pass (89971); missing-reference versus exhausted
budget regression passes (49105); all-target Clippy passes (33189). Follow-up
independent review found no definite caller-migration regression. Production Pull remains
unimplemented; this shared probe is preparation, not Pull delivery evidence.

Pull process-boundary checkpoint (2026-10-13): AddPlan now separates bounded
owned process collection (run_command_output) from success-only command handling.
Existing mutation/discovery callers retain their success-only behavior; probes
can inspect status/stdout/stderr without converting legitimate exit 1 into a
process error. The same shared allowance, hardening, deadline and owned operation
remain applied. Non-Unix collector also retains Output, but Windows validation
remains outstanding. Real missing Git config probe, nonexistent executable and
timeout test passes (17943); three cumulative budget tests pass; all-target
Git-enabled Clippy passes (13267). Production Pull and fork-point probe wiring
remain outstanding; no completion or publication claim is made.

Pull rewritten-upstream baseline (2026-10-13): the real native Pull fixture now
includes upstream history replacement. Before fetch it confirms fork-point is
the old upstream tip; after pull it verifies the new upstream is an ancestor,
the obsolete tip is not, and exactly the base/new-upstream/local commits remain.
All five filtered Pull tests pass (46476); all-target Git-enabled Clippy passes
(74970). This is native semantic baseline evidence only, not prepared execution.
Integration inspection identifies AddPlan::discover as configuration capture and
run_command as the owned process boundary. Fork-point probing needs exit-status
preservation: a legitimate no-fork-point result must not be confused with process
failure, timeout or cancellation. Production Pull integration remains outstanding.

Pull policy verification checkpoint (2026-10-13): test-only policy capture now
preserves branch subsection case and branch.rebase/pull.rebase precedence,
pull.ff and pull.autostash precedence. Upstream Git v2.54 rebase.c confirms
implicit rebase values use Git boolean semantics. Five filtered tests pass
(25183), including real divergent pulls proving branch-selected merge/rebase
topology, ff-only refusal without HEAD movement, and explicit pull.autostash
on/off overriding rebase.autostash while preserving uncommitted edits.
Git-enabled all-target Clippy passes (49647). These are prototype and native Git
baseline checks, not production prepared-Pull execution. Next: retain pre-fetch
HEAD/fork-point and bind admitted fetch and integration continuations. T7 remains
Active; all later handoff tasks and publication gates remain outstanding.

Pull fetched-target prototype (2026-10-13): test-only pull_state parses bounded
FETCH_HEAD evidence into ordered merge candidates, excluding not-for-merge
records and leaving credential-bearing/non-UTF-8 descriptions uninterpreted.
It validates SHA-1/SHA-256 identifiers and rejects malformed/truncated/oversized
evidence. Three tests pass (18597), including an actual fetch with multiple remote
branches and configured branch.local.merge selecting only upstream. Strict
evidence refusal is intentional and is not a claim of accepting arbitrary manually
edited FETCH_HEAD files. The module is not yet compiled into production: fetch
admission, retained pre-fetch state, config policy, target-effect binding and
merge/rebase execution are still required before integration. This checkpoint
does not enable Pull or satisfy its delivery acceptance.

Pull implementation investigation (2026-10-13): current full Git-enabled tools
library passes 247/247 after maintenance cancellation corrections (6272,
authorized outside the sandbox for process observation). Git v2.54 builtin/pull.c
was inspected directly (upstream source, local copy /private/tmp/i300-pull-2.54.c).
The prepared Pull implementation must retain original HEAD and the rebase
fork-point before fetch, preserve branch/pull rebase and pull.ff precedence,
autostash precedence and FETCH_HEAD not-for-merge filtering. Fetch may itself
update the checked-out branch, requiring worktree/index reconciliation; an empty
HEAD and multiple merge candidates have separate behavior. Recursive submodule
fetch and post-merge/rebase updates have independent child effects. A naive
fetch-then-new-rebase plan would lose these semantics and is not the implementation
target. Next code slice is retained Pull pre-fetch/configuration evidence and an
admitted fetch continuation, followed by fresh fetched-target effect admission.
Pull is still unavailable; source inspection and push tests do not prove Pull
delivery. Existing accepted contract and full acceptance scope remain unchanged.

Maintenance-phase cancellation checkpoint (2026-10-13): the real eight-stage
local push test now creates 17 loose receiver refs, enables automatic pack-refs
and holds packed-refs.lock with a 30-second configured timeout. Process-tree
observation positively identifies pack-refs under maintenance under the unique
receiver's receive-pack, after the pushed ref has updated. Cancellation awaits
the operation cleanup receipt; the observed child is absent or zombie, the
fixture-owned lock survives, and releasing it causes no late packed-refs write.
All nine scenarios pass (85061). The initial sandboxed run (48356) failed solely
because process observation via ps was forbidden; the authorized unsandboxed run
passed. Independent review confirmed the positive process ancestry observation
and identified ambiguous failure handling in the final ps check. The corrected
assertion distinguishes absent/zombie from command failure; test 72880 passes.
Independent review confirmed this specific correction. This does not close
Windows lifecycle, other transports, Pull or complete publication gates.

Local cancellation/maintenance checkpoint (2026-10-13): the real eight-stage
push regression now holds receive-pack at a pre-existing receiver reference lock,
waits for transferred objects, cancels and awaits its operation cleanup receipt.
The original lock remains intact; releasing it after confirmed cleanup does not
produce a late reference update. Independent review confirms this exercises the
actual receiver, but cancellation occurs before automatic maintenance begins.
A further scenario enables receiver loose-object maintenance and verifies a pack
exists before push returns (12219, all eight scenarios pass). Maintenance-phase
cancellation and full platform/publication gates remain open; no completion claim.
All-target Git-enabled Clippy passes after these test changes (67597). Independent
review recommends a real automatic pack-refs lock-wait cancellation fixture with
positive maintenance descendant observation; this follow-up is not yet implemented.

Local receiver execution checkpoint (2026-10-13; supersedes earlier disabled
transport checkpoints): the candidate now executes local/file push after eight
fresh admitted stages. The real prepared-stage test passes for bare, normal,
linked and file URL destinations (29677). Real MCP success, Write denial and
duplicate-request replay protection pass (4893). System shell support is bounded
to `/bin/sh` with the reviewed macOS or Linux dash/bash canonical implementation;
startup environment overrides still refuse. Independent review confirmed these
shell restrictions and the added rr-cache/MERGE_RR metadata coverage, and found
no new definite namespace omission in Git 2.54 automatic maintenance paths.
This is local candidate evidence, not full security approval or completion.
Cancellation, partial multi-destination effects, prepared Pull, remaining
transports and Windows lifecycle validation remain open. Current full tools
validation passed 247/247 (92388), with all-target Clippy passing (47702).
The subsequent expanded eight-stage test also passes for two local pushurl
destinations (5084), checking both receivers stay untouched before final admission.
MCP compilation/test 4893 is terminal and passed.
The same real eight-stage test now also covers partial failure (26449): the first
local receiver updates, the second refuses a divergent reference, its original
reference remains unchanged, and the result explicitly reports possible partial
remote updates. Governance/claim validators pass with zero warnings (22133/30900).
Next: cancellation/descendant cleanup and remaining transport/Pull implementation;
retain the full acceptance scope and obtain complete candidate security review.

Windows executable guard candidate (2026-10-13): ProgramEvidence now retains
a read-only File opened with FILE_SHARE_READ and OPEN_REPARSE_POINT. It reads
the bounded snapshot through that same handle, rejecting reparse points before
content access. This avoids unstable std identity APIs and does not lock mutable
refs/config evidence. A Windows-only regression checks write/delete/rename
refusal while either evidence handle survives and successful rename after both
drop. It has not run on this macOS host. Same-file and cap-std alternatives were
inspected; their 64-bit Windows identity paths were not adopted as a claimed ReFS
identity guarantee. No new library or Talos unsafe was introduced.
The macOS Git-enabled library suite passes 231/231 (44582). Windows compilation,
native Git/shell startup compatibility, ancestor mapping and cancellation/process
lifetime ownership remain delivery gates; the retained handle currently follows
the plan lifetime and does not prove protection after a detached child outlives
that plan. Independent implementation review is pending. Receiver mutation stays
closed; no cross-platform completion is claimed.

Executable leaf-open correction (2026-10-13): independent review found
capture_program used a general snapshot reader whose ordinary open could follow
a concurrently substituted leaf symlink or block on a FIFO. It now uses
capture_regular, retaining outer program identity checks. Review confirms the
specific Unix leaf window is closed by O_NOFOLLOW/O_NONBLOCK and opened-file
identity checks; ancestor/content races remain. Full tools library tests pass
231/231 after this correction (18376, then 97856).
Windows capture_regular now requests FILE_FLAG_OPEN_REPARSE_POINT and rejects
reparse attributes before reading. An attempted standard-library Windows identity
implementation was removed after review confirmed volume_serial_number/file_index
are unstable in Rust 1.99.0. Windows stable identity binding and native validation
remain required; this is not cross-platform closure. Final receiver mutation and
the pending reference-tree contract are unchanged.

Receiver child executable checkpoint (2026-10-13): sender discovery now queries
the admitted Git for its compiled shell path and derives receive-pack from the
reported exec-path. These become fresh exact Read facets; receiver-root discovery
captures them only after admission, retaining path mapping, bytes/permissions and
Unix file identity. Every later receiver layout revalidation checks this evidence.
Regressions cover link retargeting, equal-byte inode replacement, missing Read
admission through execute_stage, and replacement between production stages.
The full Git-enabled tools library suite passes 231/231 (20724); all-target
Clippy with warnings denied passes (87601). Governance and Collaboration Claim
validators pass with zero warnings (69314/61782); git diff --check passes.
The earlier filtered run passed
181 tests (13891) and is not the full library result.

This is executable discovery and evidence binding only: it grants no child
Execute authority and does not enable receiver mutation. Missing exec-path
receive-pack or an unavailable/nonabsolute compiled shell refuses; PATH fallback
and shell startup/environment effects are not yet covered. Independent review
of this addition remains pending. The reference-tree capability still awaits
maintainer acceptance. Sibling worktree state, maintenance contexts/effects,
final mutation facets/dispatch, prepared Pull, remaining transports, full
preflight and publication remain open. No task completion is claimed.

Receiver reference authorization checkpoint (2026-10-13): auxiliary evidence now
retains regular-file identity as well as bytes/permissions, repairing the
identical-content replacement gap recorded below. The files reference backend
retains bounded shared/private refs trees and packed-refs; chained symbolic
retargets, inode replacement, stable symlinks and unsupported backends have
regressions. Git-enabled tools pass 228/228 (session 93235), including rejection
through execute_stage when exact directory tokens lack the closed tree capability.
The earlier 227/227 result (73288) predates that authorization correction.
Permission validation also passes: 143 unit tests, three public API integration
tests and one doctest (10514). Tools/permission all-target Clippy passes with
warnings denied (17533); both governance validators pass with zero warnings
(96625/16033), and git diff --check passes.

Independent review rejected treating exact directory Read tokens as subtree
authority. The candidate now adds closed Read(Remote
git:receiver-reference-tree) plus independently checked exact refs roots; the
one-use prepared proof binds both. The grant compiler restricts the name to
git_push with an absolute refs Path facet; it does not itself prove prepared
identity. Discovery and all later read-only stages retain the capability, and
missing capability refuses before tree reads. Review confirms that specific
binding, conditional on maintainer acceptance of the new scope; it is not
mutation approval. ADR-089 records the candidate. Concurrent ancestor exchanges
can still cause a read before postcheck rejection; no OS isolation is claimed.

Next: obtain the concrete tree-scope decision after validation; bind receive-pack
and shell executable identity, sibling worktree HEAD/state and maintenance
configuration/effects, then final mutation facets and dispatch. Git v2.54.0
config.c resolves the HEAD symbolic chain for onbranch, while receive-pack.c
enumerates all refs and checks get_worktrees/find_shared_symref. connect.c uses
a shell for local receive-pack. These are required execution/read effects, not
covered by the current reference snapshot. Prepared Pull, remaining transports,
full preflight, exact-head review/CI and all later handoff tasks remain open.
Authorized target/debug cleanup restored 4.5 GiB; subsequent builds use
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
CARGO_BUILD_JOBS=1. No completion commit, publication or task closure is claimed.

Receiver auxiliary retention checkpoint (2026-10-13): auxiliary capture now
retains its path/entry evidence across a fresh identity transition into the still
Read-only ReceiverMutation gate. That gate revalidates sender, layout, configuration,
all auxiliary path mappings and recaptured content/permissions/missing entries
before its explicit mutation refusal. Real bare/normal/linked regressions now
verify an absent hook becoming a regular file invalidates this evidence.
Alternate-store detection uses exact inventory paths, not a filename heuristic.
Git-enabled tools pass 223/223 (session 56137); Clippy passes (92851).
Independent review found no new retention-stage blocker. Auxiliary snapshots
still lack cross-stage inode identity: identical-byte/permission replacement
can pass, unlike the separately retained layout pointers. This remains a
mutation prerequisite. This supersedes the discarded
evidence limitation below, but does not enable writes: ref/backend evidence,
maintenance/child contexts, mutation facets and full delivery gates remain open.

Receiver auxiliary inventory checkpoint (2026-10-13): receiver configuration
now contributes fresh Read facets for conventional hooks, receive.fsck.skipList,
object alternates and shallow evidence, without opening those targets. The admitted
auxiliary stage checks all receiver path mappings then uses bounded regular-file
capture. Executable hooks and nonempty alternates refuse explicitly; configuration
hook commands, alternateRefsCommand and updateInstead refuse during inventory.
Maintenance-enabled recentObjectsHook/repackFilterTo are explicit unsupported
helper/external-write cases. No setting is silently disabled; ordinary automatic
maintenance remains required scope, not replaced by a maintenance-off subset.
Independent review found no new auxiliary-discovery blocker, but evidence retention
and final revalidation, maintenance child contexts, other-worktree effects and
child executable identity still block mutation. The auxiliary stage remains terminal.
Metadata now also retains/rechecks HEAD bytes and leaf identity. Review confirms
this binds the HEAD file only: downstream symbolic refs can still change onbranch
matching and require retained ref/backend evidence. A real-host 12-case
bare/normal/linked gitdir/onbranch comparison matched
config inventory with receive-pack advertiseAtomic output across two HEAD names;
this is bounded condition evidence, not full receive-pack equivalence.
Git-enabled tools pass 223/223 (session 23905). Full preflight, remaining transports,
prepared Pull and later serial handoff tasks remain open; no completion/publication.

Receiver configuration discovery checkpoint (2026-10-13): source review of
Git v2.54.0 setup.c corrected the earlier global/include bootstrap prerequisite.
read_repository_format and read_worktree_config use git_config_from_file without
include expansion; global/include worktreeConfig cannot activate config.worktree.
A real-host regression confirms this distinction. Direct layout hints still need
Read admission because ordinary config setup may resolve core.worktree.
The fresh ReceiverConfiguration stage now admits Execute plus the existing closed
configuration-chain capability, runs a fixed bounded config inventory with an
absolute gitdir, verifies identical repeated output and retained layout, then
returns fresh source Read facets and ReceiverAuxiliary. The latter still refuses;
no receiver mutation or complete receive-pack equivalence is claimed.
Review found relative-origin ambiguity and recursive configuration snapshots.
Native capture now rejects nonabsolute receiver origins before source access;
configuration snapshots use bounded regular-only/no-follow reads with opened-file
and post-read identity checks. HOME/rooted-suffix guards cover discovered nested
include declarations too. Ancestor/content races and weaker non-Unix opens remain.
Git-enabled tools pass 219/219 (session 3936); all-target Clippy passes (80649).
Independent review confirms the discovery read blockers repaired. Conditional
include/HEAD and cwd comparison matrix, receiver auxiliary effects, final mutation,
full preflight and publication remain open. All later handoff tasks remain in scope.

Receiver include inventory checkpoint (2026-10-13): local/shared and enabled
worktree configuration now inventory all include/includeIf path declarations,
including duplicates and inactive conditions, without target reads. Each candidate
becomes a fresh Read facet resolved against its source directory or frozen HOME.
Independent review identified ambiguous HOME parent traversal and rooted ~/ suffixes;
both now refuse before joining, with integration regressions and reviewer recheck.
Implicit/empty path declarations also have regression coverage; the suspected gix
implicit-value defect was disproved by parser inspection, not patched around.
Git-enabled tools pass 214/214 (session 8022); all-target Clippy passes (92503).
After verification, authorized target/debug cleanup restored 4.9 GiB free space.
Effective global/include layout
closure, receiver configuration, auxiliary effects, final mutation and full
preflight remain open. This does not enable local push or complete I300.

Receiver worktree bootstrap checkpoint (2026-10-13): explicit local
extensions.worktreeConfig=true now captures already-admitted gitdir/config.worktree
and adds its core.worktree candidate alongside the shared candidate. Both config
snapshots are rechecked before returning, and retained for next-stage comparison.
Disabled local flags leave worktree config unread; include/global activation is
still unresolved and the effective-configuration stage remains closed. The gix
boolean API mismatch found at compile time was corrected to Result<Option<bool>>.
Independent review found no new blocker under the current closed execution gate;
its stale-comment and same-capture consistency suggestions were applied.
Sixteen receiver tests pass, and the pre-review-tweak Git-enabled suite passed
211/211 (session 11821). Full preflight, effective-chain closure, auxiliary effects
and mutation admission remain unfinished. No completion or publication claimed.

Receiver local-layout bootstrap (2026-10-13): the admitted Common wave now
captures common/config with the existing 16 KiB regular-file/no-follow pointer
reader and retains its evidence. A catch_unwind-guarded gix byte parser extracts
only the local core.worktree hint without include expansion or Git execution;
that path is a fresh continuation Read facet. This is not effective configuration:
worktree-specific/global/include-driven overrides still require closure before
repository-setup commands. Quoted/duplicate keys, absent includes, changed bytes
and oversize refusal have a focused regression. Fourteen existing receiver tests
passed after integration; the new focused regression is tracked separately.
Full preflight and independent review remain required. Local push stays disabled.

Receiver configuration prerequisite audit (2026-10-13): independent source
review of Git v2.54.0 found receive-pack enters the actual gitdir and sets
GIT_DIR=. before repo_config, whereas ordinary git config has its own repository
setup. Therefore neither worktree-cwd config nor gitdir-cwd --git-dir=. has yet
been proved equivalent for core.worktree and conditional includes. A local
isolated fixture with nonexistent core.worktree returned exit 0 for receiver-cwd
explicit-file, neutral-cwd explicit-file and gitdir-cwd effective probes. Those
exit codes do not prove absence of filesystem access and do not establish a
startup failure; do not claim the experiment reproduced one. Before enabling
configuration execution, inspect layout keys using a bounded Rust parser without
include expansion, admit newly named paths, and close include-driven layout
paths before any repository-setup command. Validate bare/normal/linked, relative
core.worktree and includeIf gitdir/onbranch against receive-pack semantics.
No command override of core.worktree/core.bare is authorized as a substitute.
Current ReceiverConfiguration remains fail-closed; this records a concrete
implementation prerequisite, not configuration or push delivery.

Receiver real-layout and alias correction (2026-10-13): the integrated
layout pipeline now has a regression using real host-Git-created bare, normal
and linked-worktree repositories through all four admitted layout waves.
Independent review identified a symlink-parent plus leading-parent backlink
ambiguity; backlink parsing now uses the already-admitted canonical marker
parent instead of its raw spelling. A dedicated regression verifies canonical
resolution without consulting the lexical alternative. Fourteen receiver tests
and all 209 Git-enabled tools tests pass (session 33298, exit 0). Configuration
command semantics and fresh review remain open; no receiver execution is enabled.

Receiver backlink checkpoint (2026-10-13): ReceiverConfiguration now verifies
an independently admitted backlink target against the retained canonical gitdir
and directory identity before its existing configuration-stage refusal. Tests
cover missing/wrong admission, inconsistent backlink, matching target and Unix
same-path directory replacement. Twelve receiver-focused tests pass. Configuration
discovery, auxiliary effects, final mutation, fresh independent review and full
preflight remain open; local push is still disabled and no completion is claimed.

Receiver common wave checkpoint (2026-10-13): admitted shared directories
and linked-worktree markers now retain evidence in ReceiverCommon. Marker bytes
are opened only after their parent/leaf mapping matches the current admission;
leaf symlinks are rejected. Backlink targets remain unresolved next-stage Read
facets. ReceiverConfiguration currently revalidates the retained layout then
refuses: backlink consistency, configuration-chain execution, auxiliary evidence
and final mutation authorization remain unfinished. Eleven receiver-focused
tests pass (session 25200, exit 0). This change has not received fresh independent
review or full preflight and does not enable local push. Existing TOCTOU and
non-Unix limitations remain. No completion or publication is claimed.

Receiver metadata wave checkpoint (2026-10-13): fresh ReceiverMetadata
admission now permits bounded commondir/gitdir pointer capture in each admitted
gitdir, retaining directory identity and pointer bytes/Unix leaf identity.
New targets are only Read facets on the fresh ReceiverCommon continuation;
previous root/gitdir scopes remain present for revalidation. ReceiverCommon
revalidates retained evidence then explicitly refuses pending common/worktree,
configuration and auxiliary effect binding. No receiver mutation is enabled.
Independent review found no new blocker within the documented path-based
TOCTOU/non-Unix limitations. Ten receiver-focused tests and all 205 Git-enabled
tools tests pass with pinned Rust 1.99.0 and --locked (session 37793, exit 0).
The focused test's initial macOS raw/canonical alias assertion was corrected;
production authorization continues comparing normalized paths. Next implement
the admitted common/worktree wave, config-chain discovery, auxiliary evidence
and final all-destination mutation gate. Publication and full preflight remain
open; all later serial tasks retain their original scope and order.

Receiver root evidence checkpoint (2026-10-13): root discovery now retains raw
spelling, admitted canonical mapping, directory identity, .git marker kind,
Unix leaf identity and bounded gitfile bytes. The fresh ReceiverMetadata proof
rechecks retained roots before its existing terminal refusal. Tests cover marker
retargeting, same-byte inode replacement, root symlink retargeting, directory
marker replacement and the production stage refusal. Seven receiver-related
tests pass with the pinned toolchain and --locked. Independent review found
marker canonicalization could query a concurrently substituted symlink target;
marker identity now uses symlink_metadata instead, without canonicalizing that
leaf. Unix pointer opens use O_NOFOLLOW/O_NONBLOCK and fstat identity checks.
Ancestor/content TOCTOU and weaker non-Unix identity/open guarantees remain;
this is not OS confinement. Metadata/commondir/backpointer waves, configuration,
auxiliary evidence and final mutation admission remain unfinished. No local/file
transport activation, completion transition or publication is claimed.

Receiver root discovery checkpoint (2026-10-13): local destinations now enter
an admitted Read-only ReceiverRoots phase, bounded to 64 receiver roots. It
checks the admitted normalized root before inspecting its .git entry; gitfiles
are bounded to 16 KiB and external gitdir targets become fresh continuation Read
facets without target access. ReceiverMetadata still explicitly refuses: layout
identity retention/revalidation, commondir/backpointer waves, configuration,
auxiliary evidence and final mutation remain unimplemented. Independent review
found no immediate blocker within the existing path-based TOCTOU limitations.
Tests cover stale/fresh proof and an absent external target; Git-enabled tools
pass 198/198. The legacy-remote regression now uses an explicit direct Git URL
so it still reaches mutation revalidation after local paths gained extra stages.
This is partial integrated discovery, not local/file push support.

Explicit phase checkpoint (2026-10-13): AddPlan now separates sender configuration
discovery from mutation dispatch with an internal phase enum. Configuration
presence alone no longer selects mutation, and mutation rejects even a valid
discovery proof before repository revalidation or child execution. A regression
checks refusal and unchanged index; stale-source/index tests now use the real
discovery-stage entrypoint. The Git-enabled tools suite passes 197/197 with
`--locked`. This prepares the receiver integration point but does not implement
receiver layout/configuration stages or enable additional transports.

Receiver path prerequisite (2026-10-13): local receiver URL classification rejects
ambiguous parent traversal after ordinary components; leading relative parents
remain supported. This prevents declaring a lexically normalized path different
from Git's symlink-aware traversal. The focused classifier test passes. Actual
receiver discovery/execution remains unfinished; no transport was enabled.

Command-budget diagnostic correction (2026-10-13): discovery wrappers now
preserve cumulative budget exhaustion instead of rewriting it as an unavailable
Git installation or configuration inspection failure. The targeted regression
checks config, config-path and attribute-path queries. The complete Git-enabled
tools suite passes 196/196 with `--locked`; both governance validators pass with
zero warnings. This is diagnostic accuracy, not a new receiver capability.

Runner budget verification (2026-10-13): both real Agent and MCP runners now
exercise a first-stage profile exceeding 16 MiB (zero executions), and three
8 MiB stages (two executions then cumulative refusal). MCP also proves sticky
request-ID refusal; Agent replay checks prove no additional effects, with initial
oversize refused before ledger reservation. Agent normal_prepared tests pass 5/5;
MCP prepared_replay passes 6/6. This strengthens the production-path evidence
without claiming receiver support or closing the remaining concurrency matrix.
The maintainer explicitly accepted Trace2 suppression; ADR-089 and the migration
inventory reflect acceptance rather than a pending decision.

Execution checkpoint (2026-10-13, cumulative resource bounds): Agent/MCP now
reserve at most 65,536 authorization facets and 16 MiB of UTF-8 resource and
description bytes per invocation before cloning prepared profiles. The Git
plan carries a shared reservation budget of 128 Talos-spawned commands and
64 MiB of combined stdout/stderr capacity through discovery and mutation.
No refunds occur after failure/cancellation. Core budget tests and all 195
Git-enabled tools tests pass. Independent review closed an initial first-stage
clone bypass; Agent focused tests pass 5/5 after the fix. Git descendants,
evidence bytes, additional concurrency cases and receiver support remain open.
This appends execution evidence and does not replace the published acceptance.

Execution checkpoint (2026-10-13, prepared continuation prerequisite): the local
Agent/MCP runner candidate permits eight unique prepared stages and refuses a
ninth, retaining fresh admission and proof consumption for each stage. It uses
one 600-second invocation deadline and a separate existing permission deadline
shared across admissions; Git's admitted child still has its 300-second bound.
Independent review found and closed a permission/execution deadline conflation.
Focused Agent prepared tests pass 5/5, including deterministic cumulative-time
tests; MCP transport tests pass 6/6, including 8/9-stage boundaries, identity and
request replay, cancellation/disconnect, and admitted execution surviving its
permission deadline. Cumulative byte/helper budgets, additional concurrency
evidence, receiver implementation and the remaining published T7 acceptance
stay open. See the [bounded-stage proposal](../proposals/i300-bounded-prepared-stage-extension.md).

Execution checkpoint (2026-10-12): push discovery now distinguishes local receiver
Path Read/Write authority from network destinations and rejects unbound custom
remote helpers. Receiver contents are not inspected during this discovery stage.
Git tools tests pass 179/179. Actual push transport execution, receiver effect
binding, pull migration and final publication gates remain incomplete.

The Active/Claimed values above are proposed atomically by #704 and are ineffective
until its finalized head merges into main. No implementation branch or committed
implementation may start before that merge. PERM-007-H remains Unclaimed.

### Selection And Dependencies

Selected requirement: T7 in the [handoff](../tasks/2026-10-10-session-todo-handoff.md),
under the existing [GIT-001](../backlog/active/GIT-001-embedded-git-tools.md) capability.
Do not reopen historical GIT-001 or TOOL-020 acceptance as though it covered this defect.
I299 is Complete/Closed through implementation #701 and closure #703. ADR-089 is
Accepted. At initial plan publication this was planning only; #704 now proposes
atomic activation, effective only after main merge.

Inventory against main `444a8137`: I293 and I294 Active, retain other-session Search
ownership under the existing explicit non-overlap authorization; I277 Review with
deferred human checks, retain; I249 Planned, retain dependency-pilot deferral;
I164 Paused, retain superseded layout. I299 and I296-I298 remain terminal. No open
PR was found at selection after #696/#702/#703 merged. Recheck before claiming.

### Scope And Acceptance

- Identify the tested executable/build, reproduce the two-file permission error
  through production Agent admission and record actual cause, not only source inference.
  Identify the current tested build; do not pretend an unavailable historical
  executable has been reproduced.
- Implement [ADR-089](../decisions/089-structured-git-effect-authorization.md) after
  acceptance: literal paths and actual repository/index effect binding, approved
  success, denied no-mutation, invalid-resource failure and escape prevention.
- Review and repair same-family Git mutation scope defects without a permission
  bypass, blanket workspace grant, new destructive operation or automatic shell fallback.
- Preserve ordinary configured Git behavior through an explicitly authorized
  execution path; explain and track unsupported configurations without silent bypass.
- Correct staged HEAD/index and unstaged index/worktree diffs using the complete
  ADR regression matrix. Preserve ref comparisons and original content fidelity.
- Run the exact historical two-file operation in an isolated fixture through the
  real production chain, preserving unrelated staged content. Historical publication
  already succeeded via explicitly authorized host Git; verify its existing SHA
  rather than recommitting the historical documents as new implementation evidence.

### Non-Goals

No MODEL-014, compaction, model validation/agile tools, memory-limit parameter,
Search implementation, release, credential/network probing, grant-policy relaxation
or new persistent permission schema. No change to another session's owner.

### Planned Validation And Documentation

- Focused locked Git and real Agent/Runtime integration regressions, including
  approved/denied/invalid resource and the full filesystem/configuration matrix.
- Standard `./scripts/release_preflight.sh`, pinned Rust 1.99.0, `--locked`;
  Windows and Unix CI for platform-specific boundaries. Never skip a failing test
  without a tracked and authorized boundary.
- Independent security/permission/API review, both governance validators and
  exact-head CI plus merge-time CAS.
- Update user-facing built-in Git documentation in README/README.zh-CN and the
  GIT-001 tool reference; document literal-path migration and configured
  execution behavior. Update GIT-001 with bounded corrective evidence, this owner,
  handoff T7, iteration/decision indexes and Board owner-first.
- Native manual acceptance only for a changed UI interaction that automated real
  admission tests cannot establish; do not repeat unchanged I299 observations.

### Risks And Rollback

Index writes are repository metadata effects, not writes to source filenames.
Hooks/filter/config and external linked-worktree metadata cannot be hidden behind
ordinary path approval. Path rechecks alone do not eliminate filesystem races.
Use existing permission/process boundaries and revert only this bounded repair if
authorization fidelity regresses. Preserve user files/index and other-session work.

## Actual Execution And Evidence

Final receiver plan checkpoint (2026-10-13): ReceiverMutation now retains the
admitted Storage snapshots and constructs a fresh-identity Mutation plan with
the preserved sender facets, receiver exact Write roots, closed metadata Write
and child Execute facets. Final execution retains consumed authorization tokens,
checks receiver Write roots and capability, and compares current storage with
the approved evidence both at entry and immediately before dispatch. The fixed
foreground receiver command is wired into push arguments. Real bare/normal/linked
regression covers stale proof, missing Write and post-approval object creation.
Full Git-enabled tools library passes 245/245 (7378). The first receiver run
failed because the old test still expected discovery to terminate; it was updated
to assert the new final-plan transition and authorization checks. Local dispatch
is still blocked by the transport validator pending shell-baseline and complete
execution review/tests; these changes do not claim successful production push.
Independent review confirms the final-dispatch recheck closes the reported
timing gap and found no new wiring defect. Git-enabled all-target Clippy passes
with warnings denied (29970); governance and claim validators pass without
warnings (43655/94418), and diff whitespace passes. Local host evidence identifies
the compiled shell as /bin/sh, reporting GNU bash 3.2.57; this observation alone
does not establish the supported-shell contract or authorize arbitrary startup.

Receiver storage inspection checkpoint (2026-10-13): the metadata Read wave
now inspects bounded metadata structure only after dedicated capability and
exact-root admission. Real bare/normal/linked stage-chain tests cover missing
capability and missing root authorization. Full Git-enabled tools library passes
244/244 (37939). A subsequent Unix hard-link regression passes with the other
storage tests (87272); all-target Git-enabled Clippy passes with warnings denied
(33737). Missing leaves are rechecked before returning, including parent mapping;
independent review confirmed that fix. Unix regular metadata with nlink != 1 is
conservatively refused, and nlink is rechecked with inode identity. This can refuse
local clones sharing hardlinked objects and does not claim Windows equivalence.
The final nlink recheck also passes all three storage tests (41154).
Independent review identifies the remaining local-push work as final sender and
receiver write facets, retained/revalidated Storage evidence, production fixed
receiver command and Execute facets, shell baseline, and real admitted mutation,
denial, change, cancellation and partial-failure tests. Accepted metadata scope
requires no further permission split. Mutation remains disabled; no completion
or publication is claimed. Existing path-check TOCTOU remains a documented limit.

Receiver metadata authority accepted (2026-10-13): maintainer accepted the
git_push-only closed `git:receiver-metadata` Read/Write capability. Grant
compilation now requires matching absolute receiver roots, a bound absolute
Git Command and the push tool; runtime prepared identity still binds ownership.
ADR-089 records the scope and exclusions (worktree, external stores, hooks and
arbitrary programs remain outside it). Permission regression passes. This
acceptance does not enable mutation while child/environment/write effects remain
unbound.

Receiver child execution checkpoint: receive-pack, compiled Git executable and
transport shell are retained as exact Read evidence. Revalidation now requires
receive-pack and maintenance Git to match the sender discovery executable.
Git source inspection found local transport strips `local_repo_env` configuration
parameters; receiver dispatch therefore needs an explicit fixed receive-pack
command carrying the accepted foreground maintenance settings. The command
builder quotes absolute executable paths and has a shell-injection regression;
its focused test passes. No mutation dispatch has been enabled.

Latest receiver checkpoint (2026-10-13): full clean Git-enabled tools library
passes 239/239 (15092), including main-worktree HEAD/config.worktree mutation and
authorization tests, sibling private refs, relative sibling backlink verification,
rebase occupancy and aggregate reference-budget coverage. Main-worktree HEAD
symbolic targets use the same retained-namespace validation as other receiver
HEAD files. Sibling common/marker targets now have independent exact Read checks;
marker backlinks are only parsed in Auxiliary, emitted as next-wave Read facets,
then resolved and identity-checked after fresh admission. Independent reviews
found no new definite blocker in these additions. Complex symlink/parent-relative
alias spellings may safely refuse; filesystem check/open TOCTOU and Windows
identity/lifecycle limitations remain. No mutation activation is claimed.

The clean rebuild followed disk-exhaustion failure (98377); no Cargo/rustc process
was live before authorized target/debug removal (67285). Governance and claim
validators pass with zero warnings (87877/64556); diff whitespace check passes.
Git-enabled all-target Clippy passes with warnings denied (45407). Local receiver execution/maintenance/write
effects, object-store effects of detached rebase branch resolution, prepared
Pull, remaining transports, Windows lifecycle and full delivery gates remain open.
Earlier test-fixture and pending-reference-acceptance checkpoints below are
historical; maintainer accepted reference-tree scope and the fixture was repaired.

Receiver reference contract accepted (2026-10-13): maintainer explicitly
accepted the dedicated receiver-reference-tree Read scope with independently
admitted roots, 65,536 entries and 16 MiB reference-file content. Earlier
pending-acceptance checkpoints are historical. Implementation remains Active;
acceptance does not authorize skipping receiver effect binding or delivery gates.

Rebase and sibling evidence follow-through (2026-10-13): rebase-apply and
rebase-merge directory existence and fixed occupancy files now receive exact
Read facets before capture, are retained and revalidated. Independent review
identified discarded admitted mappings and a generated/consumed root mismatch;
both were corrected and re-reviewed without a new definite blocker. Real-layout
tests remove individual directory/file admissions and create previously absent
rebase directories; unit tests cover each file, stable links, and parent retarget.
The full Git-enabled tools library passed 236/236 (7691); the subsequent parent
retarget regression passed with the other two rebase tests (72523).

Sibling private refs now receive exact root facets and use the accepted closed
tree scope. Shared receiver references and sibling trees share a retained-evidence
budget across all destinations. Independent review found no new definite
authorization/budget blocker; repeated roots conservatively count again and
existing filesystem TOCTOU limitations remain. The multi-destination bisect
grouping regression passed (35406). A new full-suite run (89532) found a fixture
restoration error: the test removed an originally existing refs directory;
restoration now preserves pre-existing directories. Final validation of that
correction and the shared-budget regression is in progress. Local receiver
mutation, pointer-target/main-worktree configuration binding, child/maintenance
effects, prepared Pull, remaining transports, Windows lifecycle and delivery
gates remain open; no completion or publication is claimed.

Bisect coverage correction (2026-10-13): the real-layout regression now creates
a distinct second linked worktree and asserts three unique metadata roots (main,
receiver, sibling). For BISECT_LOG and BISECT_START at each root it checks exact
Read removal refusal, newly created state invalidation, and restored-state success.
The expanded regression passes (9072), closing the prior missing common/sibling
direct coverage. Multiple receiver grouping still needs a dedicated regression;
this does not complete rebase/private refs/object effects or enable mutation.

Bisect evidence checkpoint (2026-10-13): configuration inventory adds exact
BISECT_LOG/BISECT_START Read resources for receiver, common and belonging sibling
gitdirs, without opening them. Existing auxiliary admission/capture/revalidation
retains bounded regular evidence. Review found no new definite safety blocker;
its cross-receiver over-enumeration finding was corrected by filtering sibling
ownership. Real bare/normal/linked pipeline test passes (95641), directly changing
the receiver gitdir's bisect files. Common/sibling-specific mutations and multiple
receiver grouping still need direct regression coverage. Clippy passed before
that grouping correction (73549). Rebase state, private refs, object reads and
final mutation remain unfinished; no completion is claimed.

Sibling follow-through (2026-10-13): independent review found no new definite
authorization bypass in the metadata wave. Sibling root symlinks resolving to an
explicitly admitted canonical target are accepted, unlike inventory-root links;
do not claim blanket rejection. Auxiliary capture now revalidates siblings again
before returning. Real-layout regression (92006) proves removal of exact sibling
HEAD Read authority refuses despite retained directory authority, and retains
the HEAD replacement regression. Git v2.54.0 wt-status.c shows detached-HEAD
occupancy checks access rebase-apply/applying/patch/head-name/onto,
rebase-merge/interactive/head-name/onto, BISECT_LOG and BISECT_START; directory
existence also matters. get_branch may abbreviate an object ID, so these files
alone do not close object-store read effects. These remain implementation
prerequisites, not completed scope. Local receiver mutation stays closed.

Sibling metadata checkpoint (2026-10-13): common discovery now emits exact
HEAD/gitdir/commondir Read facets for each inventoried sibling. Configuration
entry checks these authorizations before bounded Metadata capture, retains the
results, and lists pointer targets as fresh resources without opening them.
Configuration completion and both auxiliary entry/revalidation check retained
siblings. Receiver suite passes 34/34 (59972); expanded real bare/normal/linked
pipeline regression passes (3332), including sibling HEAD replacement rejection.
Private refs, rebase/bisect state, main-worktree configuration, pointer-target
verification and receiver write/child effects remain required. Independent review
is pending; local mutation remains closed and no completion is claimed.

Worktree inventory review correction (2026-10-13): independent review found
that a directory-to-symlink exchange before PathIdentity capture could accept an
external listing. Inventory capture now binds the initial lstat leaf identity
and parent identity, checks canonical mapping against admission before read_dir,
and repeats mapping/type/identity checks afterward, including missing roots.
Receiver tests pass 34/34 (73652); expanded same-name directory replacement and
root-link regressions pass 2/2 (80355). Read_dir ancestor/instantaneous exchange
TOCTOU remains; no OS containment or complete sibling-state binding is claimed.
Independent recheck confirms the specific persistent-redirection defect repaired;
same-name inode replacement detection is Unix-only. Clippy passes (4632).
Final local push remains closed.

Receiver worktree inventory checkpoint (2026-10-13): the metadata wave now
names the shared worktrees directory as an exact Read resource. After admission,
the common wave captures at most 1024 immediate registration names without child
metadata/content reads, emits fresh child Read candidates, and retains the
inventory for later common revalidation. Stable root symlinks refuse; child
symlinks remain names only. Original Git-enabled library tests pass 231/231
(83639); the two new inventory regressions pass (69101). Independent review is
pending. Sibling HEAD/private refs, gitdir targets, rebase/bisect state, main
worktree config and execution/write effects remain required before local push
activation. This inventory is not complete receiver effect binding and final
mutation remains closed.

Maintainer closure clarification (2026-10-13): accepted decisions must be
implemented and verified in full; no partially disabled operation may substitute
for delivery. The original T7 report is narrower than the accepted ADR-089 and
published I300 acceptance, but that does not revoke the latter. Retain ordinary
add/commit/checkout/push/pull, configuration discovery, owned cleanup, diff fidelity
and composition-root migration in the completion audit. Local push's final
refusal, absent prepared Pull, and Windows lifecycle gaps remain required work,
not optional follow-up. The reference-tree amendment remains a pending proposal,
not an accepted decision inferred from this clarification.

Real MCP transport integration passes 5/5 (78360, exit 0): approved and denied
add/commit/checkout, direct Git-daemon push success/network denial and replay
refusal. This does not establish local push, other transports or prepared Pull.
The production Agent two-file add tests pass 3/3 (71337, exit 0): approved staging
preserves unrelated index entries, full denial preserves index bytes, and approved
discovery followed by denied mutation preserves the index. These are focused
integration results, not full preflight or completion evidence.
Added a Windows-only regression
that retains executable evidence while starting real Git through the bounded
runner. It has not run on this macOS host; existing Windows workspace CI is the
required native validation route. No implementation completion is claimed.

2026-10-11: #704 proposes atomic claim and activation after maintainer acceptance
of ADR-089. Current main is `444a8137f054b33802f26bffdb55fb0f83a7a861`;
the inventory above remains applicable. No overlapping PR was present at selection.
Historical Planned baseline above is preserved; activation is effective only on merge.

#704 subsequently merged as `55fef9f9` and is the effective target-branch claim.
The implementation branch starts at that commit. This updates execution facts,
not the published baseline above.

Read-only source investigation found missing typed facets in add/commit/checkout,
workspace/repo mismatch in pull, unbounded host helper lifetime and incorrect diff
enumeration/baselines. Independent Agent investigation corroborated these findings.
No production reproduction, code fix, effective claim or completion is claimed.

## Residuals

Status evidence follow-up: continuation now declares repository worktree Read
for host status directory enumeration/ignore rules. Admitted execution captures
bounded names/types and .gitignore contents, excluding Git metadata and separately
bound gitlinks, without reading untracked file bodies. Undiscovered nested
repositories refuse. Child external excludes follow child config and frozen
XDG/HOME defaults. Parent/child evidence is revalidated; 140 tools tests pass.
Independent review and actual Commit activation remain outstanding. Full workspace
tests/doctests passed in preflight 8197, which then failed on SDK fixture lock
drift; its single missing dependency edge is repaired. Full preflight is not green.

Commit activation checkpoint: parent config/hook/filter inspection now runs in
both commit modes, with child evidence retained until final shared revalidation.
137 tools tests and Clippy pass. Git 2.54 no-change commit failure invokes status,
including untracked directory enumeration and nested ignore reads; these effects
are not yet bound. Native no-op detection must account for the post-`--all` index,
or status listing/ignore authority must be implemented before mutation enablement.
See the migration inventory. Cleanup public API remains pending maintainer decision.

Review corrections (2026-10-12): parent configuration now binds file and ancestor
identity, rejecting identical-byte replacements and ancestor retargets. Children
discover their own system/global attribute paths instead of inheriting the
parent's override. Promisor refusal includes partialCloneFilter and
extensions.partialClone and applies to commit/child object inspection even when
maintenance is disabled, because lazy fetching is not maintenance-specific.
These corrections supersede the narrower promisor checkpoint below. Tools tests
pass 136/136 and Clippy passes; actual commit mutation and cleanup API integration
remain unfinished, with the public cleanup contract still awaiting decision.

Promisor-network checkpoint: Git v2.54.0 `pack-objects.c` can call
`promisor_remote_get_direct`, which writes to a fetch subprocess and may perform
network I/O while repacking. Discovery now refuses enabled `remote.*.promisor`
when automatic maintenance is active; explicit false values do not trigger the
fallback. The regression verifies no commit message or mutation occurs. Source:
`/private/tmp/i300-pack-objects.c`, `/private/tmp/i300-promisor.c`. Tools tests
pass 134/134 and Clippy passes. This is a configured fallback, not a complete
network authority implementation; commit execution remains disabled.

Git child-resolution audit: Git v2.54's `run-command.c::prepare_cmd` resolves
`git_cmd` children through `prepare_git_cmd` and PATH; built-in commands listed
in `git.c` (`gc`, `repack`, `pack-objects`, `status`, maintenance helpers, and
others) are dispatched internally rather than through dashed external helpers.
An isolated fixture with trap `git-<subcommand>` binaries confirmed no trap ran
for status, maintenance, pack-refs or repack. This bounds the maintenance
executable review to the admitted Git binary and its internal configured helper
effects; arbitrary dashed helpers are not silently assumed safe. Source copies:
`/private/tmp/i300-git-2.54.c` and `/private/tmp/i300-run-command-2.54.c`.
Commit lifecycle and public cleanup receipt integration remain incomplete.

Maintenance host-evidence checkpoint: an isolated Git 2.54 fixture with two packs,
gc.autoPackLimit=1, maintenance.gc.enabled=true and gc.repackFilterTo reproduced
automatic gc→repack→pack-objects writing external .pack/.idx/.rev files. Trace:
/private/tmp/i300-maint-pack-6b2xnbj3/retry.json. This substantiates the destination
effect rather than relying on a self-confirming refusal test. Maintenance entry
identity is now rechecked after child/repository inspection; a late symlink
retarget regression fails closed. Tools tests 133/133 and Clippy pass. The
complete helper-resolution/lifecycle gates and actual commit enablement remain
open; no completion or publication evidence is claimed.

Maintenance object-destination checkpoint: Git v2.54.0 builtin/gc.c reads
gc.repackFilterTo and passes it as repack --filter-to. Commit discovery now
refuses a nonempty destination while maintenance.auto is enabled, since the
current plan has no destination write authority. Explicit maintenance.auto=false
keeps the inactive setting discoverable. This is a conservative fallback even
when a maintenance threshold would not fire; precise task applicability remains
a compatibility residual. Real-config no-mutation regression passes; locked
tools tests 132/132 and Clippy pass. Commit execution remains disabled.

Maintenance source audit: Git v2.54.0 builtin/gc.c's task table gives prefetch
no auto_condition; maybe_run_task rejects such tasks under --auto. Thus commit's
automatic maintenance is not equivalent to scheduled prefetch. No generic
maintenance.*.command override was found in that implementation. An attempted
rule rejecting those invented command keys and its self-confirming test were
removed before publication. Actual maintenance child executable/configuration
effects still require binding; this finding does not enable commit execution.
Source: https://github.com/git/git/blob/v2.54.0/builtin/gc.c (task table and
maybe_run_task); local inspected copy /private/tmp/i300-gc-2.54.c.

Retained child evidence checkpoint: filter inspection returns attribute and
tracked-source snapshots instead of discarding them. The commit path retains
and revalidates them after repository checks; regression detects edits occurring
after inspection returns. Locked tools tests pass 131/131. This is still before
the deliberate mutation refusal; final dispatch and lifecycle remain unfinished.
The pending cleanup-receipt maintainer decision was presented explicitly again;
automatic continuation is not acceptance of that public-interface amendment.

Child tracked-source checkpoint: admitted filter inspection snapshots tracked
entries before host inspection and compares both index path membership and entry
content afterward. Directory replacements are recorded as directory kind only,
without recursively reading untracked descendants. Same-length edit and nested
untracked directory regressions pass; locked tools tests 130/130 and Clippy pass.
This evidence spans inspection only; it does not complete final mutation binding,
recursive status, untracked listing/ignore effects or cleanup receipt integration.

Child attribute snapshot correction: collect local .gitattributes for every
tracked path ancestor before filter inspection, alongside declared external
sources, and compare snapshots afterward. Previously only external sources were
included in this check. Locked tools tests 129/129 and Clippy pass. An isolated
untracked-child fixture yielded parent commit exit 1 and child status exit 0;
that trace proves invocation, not untracked file-body reads. Broader status and
lifecycle gates remain open; commit execution remains disabled.

Ordinary-commit failure-path correction: isolated Git 2.54 trace2 showed a
commit without staged changes exits 1 after invoking child `status --porcelain=2`.
Successful default/verbose/submodule-summary examples only spawned maintenance;
those success traces were insufficient to omit child discovery for ordinary
commit. Both commit modes now discover child configuration/location and inspect
filters before reaching the temporary execution refusal. The filter no-helper
regression covers all=false and all=true; tools tests 129/129 and Clippy pass.
Evidence: `/private/tmp/i300-normal-child-pq334b3y/no-staged.json`. Recursive
status, untracked child inspection and remaining lifetime effects remain open.

Child status-effect checkpoint: isolated Git showed same-length child edits can
invoke a configured clean filter during commit-all's child status path. The
admitted plan now snapshots child attribute sources and runs child `check-attr`
before status; configured filters cause pre-mutation refusal, while helper output
is never executed. Nested gitlinks are refused pending their own discovery.
Regression verifies no helper marker and no parent commit message. Locked tools
tests pass 129/129 and Clippy passes. Child status/stat semantics, ordinary
commit confirmation, helper executable identity and lifecycle remain open.

Admitted child metadata checkpoint: continuation execution now captures child
RepositoryState only after proof consumption, requires checkout/gitdir/common-dir
to equal the discovered locations, and checks configuration/location evidence
before and after. A second capture detects intervening HEAD/index byte changes.
External separate-git-dir regression verifies discovery reaches continuation
without consuming the child index and denied continuation leaves state untouched.
Snapshots bind index bytes, not index semantic validity. Locked tools tests pass
128/128; all-target Clippy passes. Child status source/filter/helper execution,
ordinary-commit coverage and production lifecycle remain incomplete; mutation
is deliberately still disabled.

Child location checkpoint: admitted all=true discovery captures canonical gitdir
and common-dir identities, reading only the already-declared gitdir's commondir
pointer. It checks the location before/after configuration discovery and on later
revalidation, and adds discovered common-dir Read authority to continuation.
Regressions cover pointer retarget, identical-path directory replacement, pointer
symlink refusal and deferred shared contents. Child HEAD/index/source inspection
after continuation admission, helpers and lifecycle remain unfinished; mutation
is still closed. Unix identities include device/inode; Windows remains limited
to canonical paths and requires its own platform evidence.

Child configuration identity checkpoint: origin evidence now retains canonical
resolution and, on Unix, device/inode identity for the file and canonical
ancestors. Capture/revalidation check identity before and after content reads.
Equal-content ancestor-symlink retarget and identical-byte file replacement
regressions cover the review finding. Windows currently binds canonical paths
only; this is not race-free containment. Child repository/common-dir identity
and remaining execution effects still need implementation; commit stays closed.

Admitted child configuration checkpoint: commit all=true discovers each present
gitlink checkout's configuration through the fixed config inspection command
only after discovery proof consumption. Effective values/origins and source bytes
are retained internally, checked twice and revalidated; child origins become
Read facets on the fresh continuation. An external include configuring child
fsmonitor is refused before status/helper execution; explicit false remains
discoverable and continuation denial leaves repository state untouched. Locked
tools tests pass 123/123 and all-target Clippy passes. Child common-dir/HEAD/index
capture, attributes/filter/nested status effects, helper executable binding and
production cleanup remain incomplete; commit mutation is still disabled.

Child-read ordering correction: preparation no longer invokes RepositoryState
on child repositories. It snapshots only the checkout kind and local .git marker,
parses the target pathname without reading destination contents, and declares
its Read facet. A regression uses invalid external configuration to prove those
contents are deferred; marker retarget invalidates the plan. This supersedes
the earlier pre-admission child HEAD binding: child state/configuration capture
after admission is still to be implemented. Locked tools tests pass 122/122 and
all-target Clippy passes. Commit mutation remains explicitly disabled.

Typed gitlink checkpoint: commit sources now distinguish mode 160000 from
ordinary files. Index OID, checkout kind and child repository metadata are bound
without recursively capturing child worktree content. Regressions cover HEAD
movement with unchanged parent index, untracked nested content, missing and
uninitialized checkout; tools tests pass 121/121. This remains preparation work,
not enabled commit execution. Independent review identified premature reads of
external child metadata and missing child configuration/source evidence.
Isolated Git 2.54 trace2 with a dirty child observed no child status command for
ordinary commit, but `git status --porcelain=2 -uno` for `commit -a`. Thus all=true
requires child configuration/helper analysis rather than parent-only assurance.
Evidence: `/private/tmp/i300-child-status-6h_cj193/{normal,all}.json` (ephemeral
local trace, not publication evidence). Next: defer external child metadata
capture until admission and represent the actual child status effects.

Metadata type checkpoint: commit scalar metadata (including MERGE_RR) must be
regular files or absent; logs/cache/sequencer roots must be directories or absent.
Type validation precedes recursive capture, preventing malformed scalar paths
from causing subtree reads. Wrong-type and symlink regressions pass; locked
tools git-feature library tests pass 119/119. Commit execution remains disabled;
this does not complete typed gitlink or lifecycle work.

Commit follow-up: rerere discovery now distinguishes disabled/inert configuration
from enabled active MERGE_RR records; active replay refuses before mutation.
Unmerged index entries refuse before host discovery. The fixed admitted
`--exec-path` query declares child executable Read/Execute facets on the fresh
continuation; only canonical identity with the bound Git program is currently
represented. Locked tools git-feature tests pass 117/117. Typed gitlinks,
additional maintenance helper resolution, stat/source scope and lifecycle
integration remain open, so the commit mutation refusal remains in place.

Commit migration checkpoint (2026-10-12): direct GitCommit execution now refuses;
the prepared path binds the tool name, tracked sources, merge/sequencer state
and commit-specific hooks, then uses independently admitted discovery. Mutation
remains explicitly disabled until maintenance executable resolution, active
rerere worktree effects, typed gitlink evidence and lifecycle integration are
complete. The maintainer accepted visible foreground-maintenance overrides;
ADR-089 records the scheduling contract. Independent isolated Git 2.54 trace2
verification observed `maintenance run --auto --no-quiet --no-detach` with both
overrides, with child and parent exiting 0. This verifies scheduling on that
installation, not arbitrary executable resolution or completed commit support.
Locked tools git-feature library tests pass 115/115; the permission compiler's
closed discovery test passes for Add/Commit and refuses unrelated tools.
The maintainer authorized deleting rebuildable `target/debug` artifacts after
free space fell to 116 MiB; cleanup recovered about 11 GiB and the tests rebuilt
successfully. No source or Git data was deleted. Full gates remain open.

Reference evidence checkpoint (2026-10-12): repository snapshots bind bounded
shared and per-worktree loose references in addition to packed-refs. Real Git
regressions verify loose movement without new objects, packed movement observed
from a linked worktree, private reference isolation, and symlink refusal. Locked
tools git-feature library tests pass 112/112; all-target tools Clippy passes with
warnings denied. Shared Add snapshots conservatively invalidate on unrelated
reference movement; operation-specific scope requires review as commit migrates.
This is supporting evidence only: GitCommit still uses its legacy route, and
full preflight, independent final review and publication remain outstanding.

Atomic claim/activation is next. All T7 acceptance remains
pending; all six other numbered handoff requirements retain their existing owners
and serial order. This plan does not close the whole handoff.

### Authorization carrier finding - 2026-10-11

Independent API/security review confirmed that current execution authorizations
carry Path facets only; they cannot bind the actual Git index, configured helper
execution or the reviewed request as one invocation. The attempted source-file
Write(Path) approach was discarded before submission because it misstates the
effect. The maintainer accepted ADR-089's additive normal-tool one-shot plan and
migration contract on 2026-10-11. Git mutation implementation may proceed only
after wrapper inventory and configured-helper enforceability checks; the accepted
contract does not waive either check. The read-only diff repair and its focused tests may
continue independently; they do not satisfy the write-tool acceptance.

Local validation checkpoint: the diff candidate resolves supplied refs to object
IDs before host dispatch and rejects missing partner refs or option-looking refs.
`cargo test -p talos-tools --locked --features git --lib git::tests` executed
8 tests: 8 passed, 0 failed, 0 ignored. Formatting validation passed. An initial
compile found a non-Send gix repository held across await; resolution is now
synchronously scoped, and the corrected candidate compiled and passed. Full
preflight, missing/binary/unreadable distinctions, mode/symlink handling,
truncation and bounded host execution remain outstanding. No candidate was
committed or pushed. Disk had approximately 1.9 GiB available; target occupied
approximately 14 GiB and was not deleted.

Subsequent local diff checkpoint: content readers distinguish absent entries from
read failures, preserve binary bytes, and read final symlink entries rather than
their targets (including broken links). Added/deleted text uses `/dev/null` and
unified headers no longer duplicate their prefixes. The same focused command
executed 11 tests, all passed; coverage includes unborn HEAD additions, staged
and unstaged deletion, unchanged binary, read errors and Unix symlink entries.
Mode-only changes, empty-file metadata, conflicts/submodules, broader path safety,
truncation and host-process lifetime still need completion and review. No full
preflight or whole-T7 completion is claimed; API implementation remains pending.

Further local diff checkpoint: staged empty-file additions now retain the `new
file mode` record, and Unix staged/unstaged mode-only changes retain old/new
mode without an empty text patch. `cargo test -p talos-tools --locked --features
git --lib git::tests` executed 13 tests: 13 passed, 0 failed, 0 ignored;
`cargo fmt --all -- --check` passed. The matrix is still incomplete, notably
conflicts/submodules, non-UTF-8 names, host ref-diff output bounds and
cross-platform mode behavior. No implementation commit or full preflight yet.

### Maintainer scope addition — 2026-10-11

The maintainer accepted ADR-089 and requested Auto approval for all built-in tools,
not only script execution. Track that broader outcome in
[PERM-007-H](../backlog/active/PERM-007-H-all-builtin-auto-review.md), scheduled
immediately after I300 in the handoff. It requires its own iteration/claim and
must reuse I300's truthful Git effects rather than broaden this Git repair silently.

## Native Pull Auto prerequisite claim amendment — 2026-10-23

This governance-only amendment is ineffective until its reviewed exact head
merges into main. The original #704 claim and published acceptance remain intact.
Implementation PR #735 remains an incomplete, production-gated candidate.

The maintainer accepted native Pull as one frozen command and its transitive
Git/helper effects, with shared model-assisted invocation-local approval and
explicit Deny/human-only policy preserved. Independent review of implementation
`496893c25` found an activation dependency cycle: I300 must prove Auto success,
but the shared classifier still excludes git_pull and PERM-007-H was scheduled
only after I300. Move only the required shared prepared-command assessment and
Pull integration into I300's effective Work Slice; do not activate all of T8 or
claim its completion. This is a prerequisite sequencing correction for the
already accepted Pull outcome, not a new independently deliverable feature.

### Executable deliverable and acceptance

- Native Pull reaches the existing isolated shared assessor with bounded typed
  operation evidence, user intent, frozen command/configuration identity and
  disclosed transitive effects. Never label it read-only or path-confined.
- High-confidence model approval can authorize the one prepared invocation,
  including its network and mutating effects. Explicit Deny and human-only Ask
  remain authoritative; uncertainty, insufficient evidence and assessor failure
  fall back to the existing human path. No persistent authority is created.
- Reuse the shared evaluator, deadline, cancellation, digest and stale-result
  checks. Do not introduce a Git-specific evaluator or a shell-only name bypass.
  Older assessor adapters must reject evidence they do not understand; document
  additive API adaptation and any exhaustive-match migration before API edits.
- Test actual Agent/Runtime/MCP admission and execution, model allow/refusal,
  assessor error/cancellation, human-only policy, stale/replay and changed command,
  configuration or repository identity. Preserve native recursion, conflicts,
  cancellation and partial-effect reporting. No test-only production bypass.
- Standard locked preflight, independent permission/security/API review,
  current-head CI and native Windows evidence remain mandatory before activation
  and merge. cargo xwin is compilation evidence only.
- Update Auto and structured-Git user documentation, ADR-064/089, I300 and the
  session handoff. The full PERM-007-H registry inventory, other built-in tools
  and its higher-risk policy reconciliation remain uncompleted subsequent work.

### Selection inventory and non-overlap

Target main `444c8ff1c`, refreshed 2026-10-23: I293/I294 Active retain Search;
I306 Active retains Responses provider work; I300 Active owns this prerequisite;
I277 Review retains deferred Desktop human acceptance; I302 Review retains native
resource attribution; I249 Planned retains dependency pilot; I164 Paused retains
superseded layout. No Blocked iteration was found in the current owner inventory.
I162 is Complete (its header mentions a historical Review outcome), not active.
Open PRs #754 (I293), #749 (I306), #705 (diagnostics) do not overlap this scope;
#735 is the existing I300 implementation carrier. PERM-007-H remains unclaimed
for its remaining full built-in coverage. No other session's claim is changed.

Authorization derives from the maintainer's accepted whole-command Pull and
shared Auto decisions and instruction to finish the complete handoff. No separate
maintainer reviewer is available; independent Agent security/API review,
exact-head CI, both governance validators and merge-time CAS remain required.

### Local output-bound checkpoint — 2026-10-11

Ref-to-ref host diff now bounds each output pipe to 256 KiB and shares one
10-second deadline between pipe reading and waiting for exit. Oversized output
returns an explicit error and kills the direct child; this is not evidence of
complete configured-helper/process-tree containment. The git feature explicitly
enables Tokio io-util after a git-only build exposed the missing feature.
Native diff max_lines now counts file headers and mode metadata as well as patch
lines. Empty-file fixtures cover zero, partial, exact and excess line budgets.
`cargo test -p talos-tools --locked --features git --lib git::tests` passed
15 tests, zero failures/ignored and no compiler warnings; `cargo fmt --all`
completed. No full preflight, implementation commit, publication or write-tool
completion is claimed. Remaining: actual one-shot authorization implementation,
real admission reproduction, conflicts/submodules/non-UTF-8 diff fidelity,
configured helpers and process lifetime, full validation and independent review.

### Agent admission reproduction candidate — 2026-10-11

Added `i300_real_git_add_two_files_rejected_before_mutation` to the Agent tests,
registering the real GitAddTool with the production permission pipeline and a
human-approval resolver. Its characterization asserts the untyped-facet error
before approval and an unchanged empty index. This test is NOT yet validated;
convert it to the approved-success regression after implementing effect binding.
The tools dependency is test-only. Inspection also found RuntimePermissionAwareTool
in runtime/lib.rs is cfg(test), not a production composition root; do not cite
that wrapper as production admission evidence.

Attempted `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p talos-agent
--locked --lib i300_real_git_add_two_files_rejected_before_mutation` was deliberately
interrupted with exit 130 during dependency compilation when free space fell below
1 GiB. No test result was produced. Maintainer permission to clean this workspace's
rebuildable target cache has been requested; source and Git data must be retained.
Earlier 15 Git tests and focused tools Clippy passed, but neither proves this new
Agent regression. Full preflight remains pending.

Production-entrypoint inventory is recorded in
[I300 Git authorization migration](../reference/I300-GIT-AUTHORIZATION-MIGRATION.md).
Both CLI legacy wrappers, as well as the Runtime legacy wrapper, are cfg(test).
Agent and standalone MCP dispatch are the production admission paths requiring
prepared-plan integration. This inventory is not an implemented API.
The ref-diff candidate additionally disables textconv and color and selects
literal pathspec interpretation; tests cover a configured unavailable converter
and pathspec magic. These latest tests are unexecuted pending disk recovery.

### Verified local checkpoint — 2026-10-11

After the authorized target-cache cleanup, both previously outstanding commands
completed with exit 0 using `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0` and `CARGO_BUILD_JOBS=1`:

- `cargo test -p talos-agent --locked --lib i300_real_git_add_two_files_rejected_before_mutation`:
  1 passed, 0 failed. This reproduces the current admission defect, not a fix.
- `cargo test -p talos-tools --locked --features git --lib git::tests`:
  17 passed, 0 failed, 0 ignored on macOS, including explicit unresolved-conflict
  rejection and display-path escaping. The Linux-only non-UTF-8 filesystem
  fixture remains unverified on Linux.

The write-tool preparation/admission API and complete Git effect binding remain
unimplemented; full preflight and independent final review remain pending.
PERM-007-H retains the full built-in Auto requirement and follows I300 serially.
The maintainer accepted pre-mutation refusal of configured helper behavior that
cannot be represented and enforced by the reviewed plan, with an explanatory
separately authorized shell Git alternative. This is not automatic shell fallback
or blanket refusal of ordinary Git configurations. Supported behavior, exact
boundary detection and refusal tests remain implementation work.

The local Git write host runner now uses a single five-minute deadline across
process exit and output-pipe EOF, limits each output pipe to 256 KiB, nulls stdin
and requests child termination on timeout, over-limit output or future drop.
Errors explicitly warn that a mutation may have partially happened and must be
inspected before retry. This bounds the direct child and caller wait only; it does
not isolate configured helpers or guarantee process-tree termination. Focused
`cargo test -p talos-tools --locked --features git --lib git::` passed 20 tests,
including exact/excess output and inherited-pipe deadline cases, on macOS.
The write authorization plan, production entrypoint migration, Linux-specific
fixtures, full preflight and independent security/API review are still pending.

An additive local `PreparedNormalToolInvocation` interface now exposes one
owned permission profile and consuming execution method. Agent and standalone
MCP use that object for both admission and dispatch; the default tool behavior
remains unchanged. `cargo check -p talos-agent -p talos-mcp --locked` passed, and
`cargo test -p talos-agent --locked --lib
normal_prepared_operation_requires_pipeline_and_refuses_replay` passed 1/1.
No Git write tool implements the interface yet; the original two-file Git
failure remains the expected characterization. This is an API migration
checkpoint, not Git write acceptance or whole-I300 completion.

Independent incremental security/API review found that the public consuming
`PreparedNormalToolInvocation::execute` method still accepts an empty Path
authorization slice from any Rust caller. Agent/MCP admission checks do not
protect that direct API, and Path-only grants cannot attest admitted Command or
Remote facets. The prepared-normal API is therefore **not approved for Git
mutation**. Before enabling it, replace the unbound execution method with an
unforgeable, non-clone, whole-invocation admission capability; bind the exact
tool, input, plan and all facets, consume it once, and test direct-call bypass
as well as Agent/MCP denial and replay. No final independent approval, full
preflight or implementation publication is claimed by this checkpoint.

The subsequent permission capability checkpoint passed `cargo test
-p talos-permission --locked`: 139 unit tests, three public API integration
tests, and the doc test all passed. This verifies the new one-use proof's
permission-crate behavior only. Git write tools still use the legacy direct
execution path and do not consume the proof, so the I300 authorization defect
remains open; no complete preflight or merge approval is implied.

Standalone MCP source inspection found a separate replay gap: `call_tool`
currently ignores the `RequestContext` ID and admits each repeated request
afresh. The exact connection/request-ID boundary and negative tests are now
specified in the Git authorization migration inventory. No MCP replay defense
has been implemented or verified yet.

Subsequent local implementation adds a per-handler prepared-request ID ledger
before permission admission. IDs remain reserved after errors or cancellation;
4096 reservations and 1024-byte string IDs bound memory without evicting prior
identities. The production stdio path creates one handler per connection; public
API documentation now requires that lifecycle. Distinct request IDs remain
independent. `cargo test -p talos-mcp --locked --lib replay_tests` passed all
three tests, including concurrent duplicate reservation. This is not yet real
transport replay or Git mutation acceptance; those integration gates remain open.

The standalone MCP transport regression then passed `cargo test -p talos-mcp
--locked --test prepared_replay` (1/1). It confirms that a completed request's
same ID cannot execute again, whether arguments match or change, while a new
ID can execute. It does not establish cancellation-in-flight behavior or Git
mutation acceptance; those gates remain open.

An attempted transport-level in-flight duplicate fixture timed out at its
10-second deadline while the first operation was held at an execution barrier.
The fixture was removed because it did not establish whether rmcp dispatched
the duplicate concurrently; no protection claim is inferred from the timeout.

The MCP handler now selects on rmcp's request cancellation token during prepared
admission and execution, while retaining the request ID reservation. A real
stdio transport test holds execution at a barrier, sends the MCP cancellation
notification, observes execution-future drop, confirms the connection still
answers a separate request, then confirms the old ID cannot execute again.
`cargo test -p talos-mcp --locked --test prepared_replay` passed 2/2 and the
ledger's `replay_tests` passed 3/3. `cargo clippy -p talos-mcp --all-targets
--locked -- -D warnings`, both governance validators (0 warnings), and
`git diff --check` also passed. These checks do not establish a Git write
effect plan, process-tree termination, or full preflight. The canceled Git
mutation outcome remains potentially partial and must not be auto-retried.

### Prepared-plan binding checkpoint

Added ephemeral `PreparedPlanIdentity` in core and additive prepared admission /
consumption APIs in permission. Equal request arguments cannot substitute a
different plan identity; legacy consumption rejects prepared proofs, and prepared
consumption rejects legacy proofs. `cargo test -p talos-permission --locked
prepared_proof_rejects -- --nocapture` passed 1/1. This API is not yet connected to
the Agent/MCP prepared composition roots or Git effect plans; trusted provenance
and full execution-boundary tests remain required.

Latest GitAdd path checks passed 4/4 with `cargo test -p talos-tools --features git
--locked host_boundary_tests::literal_add -- --nocapture`, including contained
absolute paths, deleted directory entries, literal wildcard names and symlink
boundaries. Earlier filters `git_add` without the feature and `git_write::tests`
selected zero tests and are not validation evidence. These path tests do not prove
authorization or config/index snapshot integrity. No implementation commit,
remote publication, independent approval or completion is claimed here.

Agent and standalone MCP production prepared routes now pass trusted provenance
to preparation and require the resulting plan identity at admission. The proof
must be consumed for that identity and request source; legacy prepared admission
and missing-identity operations fail closed. `cargo check -p talos-agent -p
talos-mcp --locked` passed; Agent prepared regression passed 2/2 and standalone
MCP real-transport replay/cancellation passed 2/2. The permission negative test
passed 1/1 including cross-plan and cross-source refusal. Both governance
validators passed with 0 warnings, and `git diff --check` passed. This is
integration evidence for the generic prepared route only, not Git's effect-plan
or final I300 acceptance. No branch/PR was published.

Additional negative coverage exercises missing plan identity through both the
Agent tool path and a real standalone MCP transport request. Agent targeted tests
passed 2/2; MCP transport tests passed 2/2. These are local generic-route checks,
not proof that any Git write tool has migrated.

A bounded Git source-entry snapshot component is now local work in
`git_write/snapshot.rs`. It records file bytes and permissions, symlink link text
without dereferencing its target, missing entries, and sorted directory entries;
one shared entry/byte budget refuses an oversized selection. Nested repositories
are explicitly refused instead of omitting their gitlink state. Its four focused
tests pass. The current GitAdd execution path now captures selected sources
twice immediately before host Git and refuses if they differ; `cargo test -p
talos-tools --features git --locked git_write:: --lib` passed 11/11. This is an
execution precheck, **not** an approval-time binding: it remains unwired to a
prepared Git operation, permission facets, repository/index/config identity or
the user's approval. Therefore the actual GitAdd authorization defect remains
open; full preflight, implementation PR and security review remain pending.

Standalone library Clippy (`cargo clippy -p talos-tools --features git --lib
--locked -- -D warnings`) exposed missing `tokio/macros` in the Git feature,
previously masked by test dependency feature unification. The manifest now
declares it directly. After integrating the source precheck, `cargo clippy -p
talos-tools --features git --lib --locked -- -D warnings` passes. This targeted
result does not replace workspace Clippy or prove the prepared Git plan. Both
governance validators passed with 0 warnings before this checkpoint.

### 2026-10-12 independent authorization-consumption review

Independent incremental security/API review found a blocking Session mutex in
`AdmittedInvocation::consume_matching_request`: contention could block the Tokio
worker beyond Agent deadlines or MCP cancellation. Consumption now uses
`try_lock` and returns `StateUnavailable` without executing. A regression holds
the actual Session fence while consuming both legacy and prepared proofs.
`cargo test -p talos-permission --locked` passed: 141 unit tests, three public
API integration tests and one doctest. Builds used debug=0, incremental=0 and
one job. The independent reviewer rechecked the change and closed that finding;
this is dirty-tree incremental review, not final exact-head approval.

Git write preparation, approval-time source/repository/index/config binding,
helper effect enforcement and owned host process-tree cancellation remain open.
No implementation commit, PR, full preflight or T7 completion is claimed.
Disk inspection found approximately 1.2 GiB free and 1.8 GiB under
`target/debug/incremental`; permission to remove only that rebuildable cache was
requested under the handoff resource policy. The maintainer then authorized that
cleanup; `rm -rf target/debug/incremental` exited 0, with approximately 3.0 GiB
free afterward. No source, Git data or other build artifacts were removed.

The GitAdd execution precheck now also compares bounded local repository
snapshots: canonical worktree, actual Git/common directory and index paths,
index bytes, HEAD, local/worktree config, packed refs and linked-worktree
discovery files. Metadata symlinks fail closed rather than binding only link
text while Git consumes their target. Native repository inspection has a panic
boundary. Focused `cargo test -p talos-tools --features git --locked git_write::
--lib` passed 15/15, including linked-worktree index isolation and detection of
index/HEAD/local-config changes. This remains an execution precheck, not an
approval-time prepared plan or complete effective-config/environment snapshot;
the authorization defect remains open. Loose refs, configured helpers and
process-tree ownership still need their operation-specific plan treatment.

The updated Git library passed `cargo clippy -p talos-tools --features git --lib
--locked -- -D warnings` after correcting two needless borrows introduced by the
repository snapshot integration. Both local governance validators and the
installed skill validator passed with 0 warnings; `git diff --check` passed.
All observed validation processes exited. Resume by implementing the immutable
Git plan's effective configuration/environment and execution boundary, then
binding its facets and retained snapshots to `AdmittedInvocation`; the current
prechecks alone do not authorize mutation. Preserve this uncommitted candidate
and obtain final security review only after the full real-wrapper matrix passes.

### Prepared GitAdd integration checkpoint

The maintainer corrected repeated unsupported tool-unavailability reports;
`functions.exec` successfully inspected the preserved checkout. No actual failed
invocation had supported those reports; the correction is recorded in EVOLUTION.

GitAdd now prepares an owned plan, admits source Read / Git metadata Write /
absolute host-program Execute facets, consumes the exact prepared proof, and
revalidates repository/source evidence before dispatch. Legacy direct execution
fails closed. Tests cover forged and exchanged proofs and post-approval source
and index changes. The actual Agent two-file approved fixture preserves unrelated
staged content, and a denied fixture preserves index bytes. Those two tests
passed before the subsequent effective-config integration. The denial fixture
now additionally requires reaching the approval resolver, so preparation failure
cannot masquerade as tested denial.

The plan freezes a process environment after existing loader hardening, binds a
canonical executable and bounded bytes, uses `env_clear`/`envs`, rejects Git
redirection overrides, and permits captured GIT_PAGER with explicit --no-pager.
Focused Git tests passed 19/19 at that intermediate point. Subsequent native
configuration evidence and post-admission host configuration parity are now
integrated locally, but **current tests are failing**: 15/20 passed, five staging
success fixtures rejected a host/native configuration mismatch. A focused retry
confirmed extra host occurrences for credential.helper and LFS configuration;
values were not printed, and temporary key diagnostics were removed. Do not
ignore the difference or claim the earlier green tests cover this newer tree.

Next: resolve system/include config origins and host/native semantic parity
without stripping normal Git config; validate relevant attributes/ignore sources
and filter applicability. The current per-key ordered values preserve duplicate
precedence but still need mixed implicit/explicit handling. Host config output
currently passes through lossy UTF-8 and must become byte-preserving. Complete
program/platform identity, metadata/object topology, helper refusal tests, owned
process-tree cancellation, same-family migrations, Runtime/MCP real Git tests,
documentation, full preflight and independent exact-head review remain required.
This is an uncommitted intermediate candidate, not T7 closure or publishable code.

Current targeted Git library Clippy passed after correcting the new constant
chunk iteration to the pinned toolchain's `as_chunks` API. Both governance
validators passed with zero warnings. These checks do not override the five
failing staging-success fixtures or establish full preflight. All observed
validation processes exited; no build is left as an unverified wait.


### Separately Admitted Configuration Discovery (2026-10-12)

Maintainer accepted two decisions in-session: independently authorized read-only
configuration discovery before fresh mutation admission, and a closed capability
covering Git's normally parsed system/global/include chain without generic file
read authority. ADR-089 records the exact boundary. Core's additive stage result,
Agent and MCP run at most two independently admitted stages with fresh identities,
normal policy/final hooks and one request replay reservation across both stages.

Host config inspection now preserves bytes, origins and ordered values (including
mixed implicit/explicit occurrences), and suppresses credential-bearing inspection
diagnostics. Discovery captures only actual config origin contents. Independent
review found premature reading of configured auxiliary files; attributes/excludes/
hook paths now become explicit continuation Read facets, and their contents are
inspected only after that admission. The regression uses an external directory
as the configured attributes source to prove discovery does not inspect it.

Current evidence: focused Git suite 23/23 passed; actual Agent two-file approval,
discovery denial and discovery-approved/mutation-denied fixtures 3/3 passed.
The Unix host boundary now uses the existing process-group launcher and owns
cleanup on timeout/cancellation; the added real descendant marker test passed.
Unix executable evidence additionally binds device/inode/ctime and rejects an
identical-byte replacement. No new unsafe site was introduced.

Remaining: full attribute/ignore ancestor/default evidence, special object/index
layouts, helper refusal coverage, Windows process ownership and identity,
remaining Git mutation family, real Runtime/MCP Git scenarios, stage-limit/replay
regressions, updated public documentation, full preflight and independent final
security/API review. Permission/MCP suites are currently being run; their result
is not yet claimed here. No implementation commit or publication; T7 remains Active.


Subsequent review rejected the experimental Unix process-group cleanup: it could
signal a reused PGID after leader reaping, and Drop-spawn cleanup depended on a
live runtime. The experimental cleanup and its test were removed; the existing
leader-only bounded host runner remains. Its descendant marker pass is historical,
not current-tree process ownership evidence. A single supervisor must coordinate
cancellation, termination and reap before this gate can pass.

Permission/MCP locked suites passed: permission 141 unit + 3 integration + 1 doc;
MCP 9 unit + 13 integration, including real stdio replay/cancellation. Both governance
validators passed with zero warnings. A new closed-capability scope test was added
after that permission suite; targeted validation is pending. Broader Clippy found
only the new test module's placement; it has been moved to the end for retry.

### 2026-10-12 Subsequent Local Security Checkpoint

The current Unix host runner now reuses ADR-082's sole-owner supervisor, retaining
the leader unreaped through the final process-group signal. Per-stream output is
bounded; cleanup errors preserve the original overflow reason and remain errors.
Locked sandbox unit tests passed 42/42; focused Git tests passed 28/28, including
real cancellation after leader exit and output overflow. Real Agent Git approval,
discovery denial and mutation denial passed 3/3 after these changes. The closed
configuration capability scope regression passed, and all-targets Clippy for
tools/agent/MCP/permission/sandbox passed before the subsequent stage-limit test.

Independent incremental security review found missing system attributes and
object namespace redirection. Fixed discovery now asks the bound Git executable
for `GIT_ATTR_SYSTEM/GLOBAL` paths under the frozen environment, deferring content
reads until continuation Path Read admission. Directory nodes explicitly supply
their own attributes/ignore paths. The complete bounded object namespace is bound
and rejects symlink/special entries; nonempty alternates and sharedindex layouts
refuse with a separately authorized fallback. Real topology refusal tests passed.
The linked-worktree regression now correctly checks an unchanged main index but
invalidated shared-object evidence after linked staging.

Remaining lifecycle gate: the current private per-call receipt is unavailable to
production cancellation/shutdown. A concrete additive tool-receipt proposal is
in the migration inventory, pending maintainer decision. No success on dropped
futures or runtime teardown is inferred. Windows ownership/identity, remaining Git
writes, real Runtime/MCP Git coverage, public documentation and full exact-head
publication gates remain open. I300 remains Active; no completion SHA exists.

Additional locked regressions passed: focused Git 30/30, including frozen-home
interpolation and empty/symlink-only directory rule coverage; normal-prepared
Agent tests 3/3, including third-stage refusal, identity reuse before a second
effect, and replay after terminal/failed continuation. Subsequent Agent
all-targets Clippy passed. Both governance validators passed with zero warnings.

### 2026-10-12 Real Host Entry-Point Checkpoint

Added real Git coverage through RuntimeBuilder and standalone MCP duplex JSON-RPC
transport. The locked Runtime focused test passes across three scenarios:
discovery denial, approved discovery followed by mutation denial, and both stages
approved. It checks approval counts, explicit tool success/permission-denial
results, unchanged index bytes on denial, exact selected staged content, and
preservation of unrelated staged content. The locked standalone MCP focused
test also passed after final assertion changes. Standalone MCP checks explicit write
permission refusal, response IDs, same-ID replay refusal after mutation, and a
new-ID intentional retry staging updated content. Its fixture uses explicit
policy scopes, not an invented interactive resolver. Ordinary test shutdown
is cleanup only and provides no cancellation/descendant lifecycle evidence.
Independent review identified overly broad denial assertions; these were tightened.
README and README.zh-CN describe the current candidate's literal paths, separate
configuration discovery/mutation permissions and unsupported-layout fallback.
The cleanup-receipt decision remains pending; no Complete or publication claim.

### 2026-10-12 Diff Gitlink And Broader Regression Checkpoint

Staged gitlink additions, changes and deletions now render `Subproject commit OID`
from tree/index identity without loading the submodule commit from the parent
object database. A real Git fixture with intentionally absent child objects
verifies all three cases. This does not establish unstaged submodule dirty-state
support. The broader Git suite exposed lost actionable missing-Git guidance after
Unix supervisor integration; host execution errors now retain the underlying
cause, installation/read-only guidance and conservative partial-effect warning.
Locked `cargo test -p talos-tools --features git --locked --lib git::tests`
passed 18/18 after correction. The complete library suite for that feature set
then passed 97/97. This remains local evidence, not full preflight or release proof.

### 2026-10-12 File-Mode Configuration Fidelity

A real Git comparison reproduced a false executable-mode change when content also
changed under `core.fileMode=false`. The native diff now applies that setting only
to regular worktree executable-bit comparisons, retaining staged mode changes and
file/symlink type changes. Invalid boolean values remain errors. The regression
failed before the correction; focused Git tests passed 19/19 afterward, including
a staged-mode test with fileMode disabled after staging. GIT-001 now records the
bounded corrective candidate and explicitly preserves historical acceptance.

Independent local review additionally identified owner-executable-bit semantics.
Host comparison fixtures for 0644→0654 and 0644→0645 reproduced false mode
changes; worktree mode detection now tests 0100, matching Git. Both fixtures
passed after correction; focused Git tests now pass 20/20. Review confirmed the
staged-gitlink/fileMode fixes while retaining unstaged-submodule and whole-I300
lifecycle/publication residuals. Both governance validators passed with zero
warnings before this final evidence-only append; diff whitespace check passed.

### 2026-10-12 Unstaged Submodule Fidelity

Native diff now uses gix child status to render checked-out submodule OIDs and
`-dirty` for child index/worktree modifications without trying to read a directory
as blob content. Host comparison exposed two differences from gix's status
convenience defaults: Git diff defaults to ignoring untracked child files, and
per-module ignore overrides global diff.ignoreSubmodules. The diff path selects
those rules explicitly and uses gix for state inspection; no new host execution.
Real fixtures cover clean, untracked/default-ignore, ignore=untracked,
tracked/index dirty, ignore=dirty, local-none/global-all override and changed HEAD.
Independent review found stale .gitmodules entries could resurrect removed
index entries; scanning now requires stage-zero mode160000 and applies the
requested path filter before child status. A staged gitlink removal regression
passes. Final focused Git tests passed 21/21. The earlier complete git-feature
library run passed 100/100 before the final stale-entry correction; no broader
result is attributed to the final tree. Missing/deinitialized checkout and type
replacement matrices still need host comparison. Lifecycle/API decision, remaining
write-tool migration and full publication gates remain open.

### 2026-10-12 Submodule Checkout And Replacement Evidence

Additional host-comparison tests pass for an absent gitlink checkout, an empty
uninitialized checkout directory, and replacement of a gitlink with an ordinary
file. After staging the replacement, further unstaged edits remain ordinary file
content despite the retained .gitmodules entry. Focused Git tests pass 23/23.
These fixtures do not prove every deinitialization layout or patch-byte parity;
full preflight, protected-boundary review and publication remain outstanding.

### 2026-10-12 Resumed Positional Argument Verification

Execution tools are available again. Git push remote/refspec, pull remote and
checkout branch inputs reject empty, leading-option and NUL values before
repository lookup or host execution. Ordinary remote URLs and refspecs retain
their syntax. This input correction does not establish exact effect admission
for the remaining unmigrated write tools.

Current candidate validation with pinned Rust 1.99.0 and
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`:
- `cargo test --locked -p talos-tools --features git --lib`: 104 passed,
  zero failed or ignored, including both positional argument tests.
- `cargo clippy --locked -p talos-tools --features git --all-targets -- -D warnings`:
  passed.

No Cargo/rustc/preflight process was observed before validation; both commands
completed. Disk remaining afterward was approximately 731 MiB. No further cache
cleanup was performed. The cleanup receipt amendment remains pending maintainer
acceptance; independent review, full workspace gates and publication remain open.

Both governance validators subsequently passed with zero warnings and
`git diff --check` passed. Independent positional-input review confirmed ordering
and coverage but identified a compatibility regression: checkout's valid `-`
previous-branch shorthand is rejected. Restore it only for non-creating checkout
by normalization to `@{-1}` and add real previous-branch/create rejection coverage
before considering this input correction finished. Other leading-option values
must remain refused; remote/refspec effect admission is still separate work.

### 2026-10-12 Previous-Branch Compatibility Correction

The real-repository regression failed before the correction with the positional
argument error. Non-creating checkout now normalizes only `-` to `@{-1}` before
validation. The fixture switches topic→main→topic via the shorthand and verifies
`create=true` rejects `-` without changing the branch. The early-error matrix also
covers creating checkout. With the same pinned locked low-disk build settings,
the complete git-feature tools library suite passed 105/105, none ignored.
This repairs the preceding review finding, not the outstanding authorization or
shutdown lifecycle gates.

### 2026-10-12 Git Boolean Configuration Fidelity

Host Git accepts `core.fsmonitor` false spellings `false`, `0`, `off`, `no`,
empty and numeric zero, case-insensitively. The prepared staging plan previously
rejected `off`/`no` as unsupported; it now matches those values and still refuses
true, nonzero and command-valued fsmonitor configurations before effects. A
real-host fixture covered all disabled spellings and staged content successfully.
The complete git-feature tools library suite then passed 107/107 and tools
all-target Clippy passed with warnings denied. This is configuration fidelity
evidence only; the remaining Git-family migration, lifecycle API decision,
security review, full preflight and publication gates remain open.

Independent review found no helper execution bypass in this correction and
recommended reusing gix Boolean parsing; the implementation now does so and
107/107 tests pass again. Coverage is limited to the enumerated spellings:
host Git also accepts `0k` as false, while gix rejects that numeric suffix.
Such unsupported values still fail closed and remain a compatibility residual,
not evidence of complete Git boolean parity. Both governance validators passed
with zero warnings before this evidence append.

The implementation then used gix integer parsing for unit-suffixed numeric
values, so host-compatible zero forms such as `0k` are disabled while nonzero
forms remain refused. The focused suite and tools Clippy passed again. The
permission resolver's named configuration-chain exception is intentionally
`git_add`-only; broadening it for other Git operations requires a separate
effect-plan review and is not inferred from this test.

### 2026-10-12 Worktree Diff Ancestor Boundary

A real repository regression demonstrated outside content disclosure when a
tracked directory was replaced by a symlink before unstaged diff. Worktree blob
and mode reads now inspect ancestor components and refuse symlink/non-directory
ancestors before reading the leaf. Missing ancestors remain deletions, preserving
ordinary directory removal; final symlink entries still expose link text only.
The initial signature migration had compile errors, now repaired. The complete
git-feature tools library suite passed 108/108 and all-target tools Clippy passed
with warnings denied, using the pinned locked low-disk settings above.
These path checks address pre-existing substitutions, not concurrent replacement
between inspection and opening; race-free containment and independent security
review remain open. No complete T7 or publication claim is made.

The worktree reader also now refuses pre-existing special files before opening
them. A Unix FIFO fixture with no writer verifies prompt rejection, preventing
the prior unconditional file read from blocking on that file type. Final symlink
entry handling remains intact. The git-feature tools library suite passed
109/109 and all-target tools Clippy passed with warnings denied. This does not
close the inspection/open race or the remaining family migration/lifecycle gates.

### Status evidence review continuation (2026-10-12)

Requested outcome: repair the two independent status-reader findings while
preserving existing implementation and the published I300 baseline. Artifacts:
status_state.rs, this owner, the migration inventory and session handoff.
Validation: focused Git tools tests, Clippy, governance validators and stable
preflight; residuals remain owned here. No lifecycle/API approval is inferred.

Ignore capture now rejects non-regular entries before reading, without recursive
snapshot calls. Each traversed directory retains path/identity evidence checked
at enqueue, before/after reading and at scan completion. Independent reviewer
confirmed both reported defects repaired; tests cover budget-preserving ignore
directory rejection, same-path directory replacement and ancestor links.
Tools tests pass 142/142; all-target Git-feature Clippy and diff checks pass.
The leaf metadata/open race (including symlink/FIFO substitution), weaker Windows
identity and conservative traversal of ignored/build directories remain open.
Real commit success/cancellation acceptance and cleanup receipt API remain
unfinished; mutation stays disabled. Preflight session 5976 is still running in
the SDK fixture phase; edits during that run require a subsequent stable check.

Follow-up: Unix ignore opens use O_NOFOLLOW/O_NONBLOCK with opened-file type
and device/inode verification. Independent security review accepted the leaf
change; ancestor races and concurrent content writes remain, and Windows does
not have equivalent flags/identity. Updated tools tests pass 143/143. Both
governance validators pass with zero warnings. Session 5976 exited 0 with full
preflight/SDK fixture success, but overlapped edits; stable rerun is required.

### Stable status-reader validation and symlink diff evidence (2026-10-12)

Standard locked preflight session 8828 exited 0; log is
`/private/tmp/i300-preflight-status-reviewed.log`. It validates the hardened
status reader, workspace tests and both SDK fixture configurations. A test-only
symlink diff regression was added during the run and separately passed using
`cargo test --locked -p talos-tools --features git git_diff_symlink_targets`.
It compares host Git and the built-in tool for staged HEAD/index and unstaged
index/worktree link-target changes, preserves no-newline markers and excludes
target file bodies. All-target Git-feature Clippy and diff checks pass after
that addition. No production changes followed the preflight start. The added
test still needs inclusion in final candidate preflight before publication.
Cleanup receipt decision and real Commit activation remain outstanding; I300
and the full serial handoff remain incomplete.

### Checkout branch/path ambiguity correction (2026-10-12)

A real fixture reproduced `git_checkout` interpreting a missing branch name as
a tracked path and reporting branch-switch success after restoring worktree
content. The branch-only structured invocation now ends with an explicit `--`.
Regression verifies failure preserves file bytes, index bytes and current branch;
create=true can still create a branch with the same name. Existing previous-branch
coverage and new local-only remote-tracking guess coverage pass. Independent
security review found no new issue in the argument fix. Tools tests pass 146/146.
The GIT-001 reference now accurately describes branch-only input and create.
This is an actual same-family defect fix, not prepared checkout migration or
cleanup lifecycle completion. Final candidate preflight remains required after
this production change; full preflight 8828 covers the preceding candidate.

Checkout-boundary stable preflight session 21275 exited 0. Log:
`/private/tmp/i300-preflight-checkout-boundary.log`. Standard pinned/locked
workspace checks, Clippy, tests/doctests, governance validators and both external
SDK fixture configurations passed. No code changed during this run. Cleanup
receipt public API decision, full Git family migration, Commit success/lifecycle
acceptance and publication remain open; this result does not complete T7.

### Cleanup lifecycle decision accepted and implementation started

Maintainer acceptance supersedes earlier pending-decision checkpoints. Additive
core receipt/owner interfaces and default tool/prepared hooks are implemented
locally; Unix sandbox command owners retain aggregate and isolated operation
receipts. Git/Agent/Runtime/MCP integration and end-to-end cancellation/shutdown
acceptance remain unfinished. Core tests pass 111/111. Independent review and
focused sandbox validation are in progress. No Commit activation or completion
is claimed; the full serial handoff remains active.

Cleanup integration checkpoint: SandboxCleanupReceipt now exposes typed,
content-free wait_confirmed evidence and implements the core wait contract.
OwnedCommandExecutor implements owner sealing/waiting. Unsupported platforms
return Unconfirmed through the new typed boundary; the legacy wait API remains
compatible. ToolRegistry retains distinct owner handles across replacement and
exposes them for composition-root registration. Tests verify retained owners,
shared-owner deduplication, typed timeout and sticky failure. Core 112/112 and
sandbox 45/45 tests pass; both crates' all-target Clippy and diff checks pass.
Runtime finalizer registration, pre-cancel seal ordering, Git operation scopes
and MCP owner shutdown are still unimplemented; no end-to-end claim is made.

Runtime cleanup wiring checkpoint: registry owner handles are registered before
Actor startup in a tool_commands finalizer. Interrupt seals before cancellation;
FinishCurrent retains its grace then seals. Owner wait shares the finalizer cap
and remains bounded by the coordinator's original deadline. Public owner seal
panics are isolated and retained as failure while other owners are sealed.
Independent review confirmed normal session cancellation already aborts/joins
the actual Agent task before its terminal barrier; outer Actor ordering remains
unchanged. Abnormal Ok(false) barriers no longer count as completed cleanup.
Runtime check and 61 library tests pass; all-target Clippy passed before the final
review corrections. Final candidate checks and real Git/MCP lifecycle scenarios
remain required. These are basic Runtime hooks, not completed T7 acceptance.

Git receipt wiring checkpoint: Add/Commit instances expose their stable Unix
command owner; prepared plans bind an invocation-local operation retained across
discovery and continuation. All host commands in these plans use that operation.
A regression seals the owner after discovery and verifies continuation/new
discovery rejection with no index creation, then waits both local and aggregate
evidence. The first assertion expected a raw sealed diagnostic; it was corrected
to the existing content-withholding configuration error, without behavior change.
Tools tests pass 147/147. Agent cancellation receipt registration and MCP owner
shutdown remain open; Commit mutation stays disabled.

Agent cancellation checkpoint: each prepared operation's local receipt is
registered before stage execution and deduplicated across continuations. Session
cancellation aborts and joins the Agent task, then waits the tool receipts using
one five-second remaining budget; normal/error completion also waits before
announcing terminal output. Receipt wait panics become content-free Unconfirmed
errors. Agent library tests pass 467/467, including bounded timeout and panic
containment. MCP transport owner shutdown and full end-to-end cancellation
scenarios remain open.

Agent receipt isolation correction: cleanup registration/wait now keys on the
internal HookContext TurnId, so concurrent turns sharing an Agent do not wait on
one another. Confirmed receipts are released after waiting; timeout/failure
receipts remain retained. Regression verifies one pending turn cannot block a
completed second turn, successful entries are released, and timed-out entries
remain. Agent check and both targeted cleanup tests pass. MCP integration and
real process cancellation acceptance remain open.

MCP request-level cleanup checkpoint: prepared execution now leaves the select
scope (dropping the actual invocation future) before awaiting its local receipt
on success, error or cooperative cancellation. A five-second outer timeout and
panic catch report content-free unconfirmed cleanup, preserving replay IDs.
Locked MCP check and real transport git_prepared (1) / prepared_replay (2) tests
pass. This does not close connection-level shutdown: rmcp service lifetime and
active-handler joining still require retained owner integration and acceptance.

MCP connection checkpoint: handler shutdown now fences new calls, seals retained
owners, waits active handler guards and then owner receipts under one deadline.
CLI MCP mode retains an Arc handler and calls explicit shutdown after transport
termination, including transport-error paths. MCP/CLI locked checks pass; three
transport regressions pass, including an extended cancellation fixture proving
execution Drop precedes receipt waiting, waiting remains pending until released,
and replay stays refused. Connection teardown failure/sticky result semantics,
shared-owner scope and independent lifecycle security review remain required;
this is not yet complete shutdown acceptance.

MCP review follow-up: request-level unconfirmed cleanup is sticky through owner
shutdown, and every continuation receipt is now retained instead of only stage
one; all waits share one five-second deadline. MCP 11 unit and 3 transport tests
pass. Independent review still requires cross-connection shared-owner isolation
and EOF/transport-error cancellation evidence. Clippy identified Agent receipt
type complexity; a private TurnReceipt alias fixes it, with recheck in progress.

MCP EOF acceptance checkpoint: a real duplex transport closes while a prepared
invocation waits. The fixture observes execution Drop before cleanup wait, keeps
cleanup blocked until released, and verifies explicit connection shutdown returns
success. Focused EOF test passes in ~5 seconds (rmcp EOF response-drain period).
This proves the observed EOF path only, not arbitrary transport I/O failure or
real subprocess descendant reaping. Shared registry/owner lifetime isolation
remains a required correction before delivery.

Real process cancellation checkpoint: a Git AddPlan run_command fixture starts
a shell with a delayed-writing descendant, aborts/joins the execution, waits its
operation receipt, and verifies no delayed file appears. Focused test passes.
This validates the prepared plan process adapter, not the full Agent/MCP Git
transport. A trial removal of MCP owner sealing was reverted because it did not
solve shared ownership and broke seal-failure reporting; restored MCP 11 unit
and 3 replay/EOF transport tests pass. Shared registry lifecycle remains open.

MCP shared-lifecycle checkpoint (2026-10-12): default connection shutdown fences
only its own admission and waits retained invocation receipts; the exclusive CLI
stdio composition root explicitly opts into sealing/waiting aggregate owners.
Exclusive shutdown also retains local receipts for tools without an aggregate
owner. A ConnectionReset transport fixture now complements EOF coverage. A real
prepared-process MCP fixture cancels a shell with a delayed-writing descendant,
confirms cleanup, verifies no delayed write, and proves closing a second handler
sharing the registry neither waits the unrelated process nor seals its owner.
CLI reports simultaneous transport failure and unconfirmed cleanup explicitly.
Independent read-only review found no new prepared-Git MCP blocker and identified
the documented boundary: non-prepared aggregate-only tools remain the registry
host's cleanup responsibility.

Validation: locked MCP library tests (13), prepared replay/transport tests (4),
and real process cleanup test (1) passed; locked CLI check and MCP/CLI all-target
Clippy with warnings denied passed. These use the pinned toolchain with dev/test
debug disabled, incremental off, and one build job. Owner remains Active: full
Agent/Runtime real descendant acceptance, real Git commit/--all acceptance,
remaining mutation migration, fresh standardized preflight and publication gates
are still required. Git commit temporary execution refusal remains in place.

Commit enablement checkpoint (2026-10-12): real Agent Session Interrupt and
Runtime shutdown regressions now launch an owned shell with a delayed-writing
descendant. Both pass; Session waits before emitting Cancelled, Runtime reports
complete and rejects later owner registrations, and neither leaves the delayed
write. Agent all-target locked Clippy passes. These complement MCP acceptance.
Independent read-only review found no remaining blanket-refusal justification
for ordinary no-helper commits, while requiring tracked-only parent capture and
later maintenance executable validation; both corrections are implemented.
The temporary commit execution refusal is removed on this local candidate.

Real prepared commit tests pass: ordinary commit preserves unstaged content and
commits only index state; --all updates tracked content without adding untracked
files; no-change fails without moving HEAD. The Git-enabled tools library suite
passed 149 tests before one added maintenance regression; that focused regression
also passes, forcing configured loose-object automatic maintenance and observing
a pack before operation completion. Earlier statements that commit is disabled
are historical and superseded by this checkpoint. Remaining work includes
push/pull/checkout prepared migration, expanded commit edge acceptance, fresh
full preflight, independent candidate review and publication gates. No completion
or implementation commit is claimed.

Checkout implementation checkpoint: independent source review identified the
next concrete migration path within existing ADR-089, without a public API
amendment. Extend the private operation plan and reuse two-stage admission and
owned execution, but bind checkout-specific evidence: current index/target tree
path union, resolved target commit/tree and branch/guessing refs, conflict paths
and nearest existing parent identity. Preserve branch execution (not detached
OID substitution), previous-branch shorthand and terminal empty pathspec.
Repository-wide worktree and actual metadata Read/Write effects must be explicit.
Target .gitattributes and new-path smudge/process filters need inspection beyond
current-worktree check-attr; retain normal conversion semantics and refuse only
unrepresentable helper execution. Inspect post-checkout/index/ref hooks and
submodule.recurse; recursive child mutation cannot inherit read-only child grants.
Required real acceptance includes switch/create/remote guessing, uncommitted
preservation/conflicts, symlink/type changes, target filter/hook refusal, linked
worktrees and target-ref mutation after approval. This records the implementation
sequence, not completed migration. Push and pull remain subsequent required work.

Latest tools verification: all 151 Git-enabled library tests passed, including
ordinary tracked-file-to-directory deletion and nested-repository refusal with
HEAD preservation. Nested repository refusal remains explicit discovery scope,
not claimed parity for all nested configurations.

Recovery checkpoint: Runtime real descendant cleanup test now covers both
Interrupt and FinishCurrent with an expiring grace period; both paths passed
before cache cleanup. A collaborator accidentally started a default-debug build,
which exhausted disk; that build exited 101, then the already-authorized
rebuildable target/debug directory was removed after confirming no live build.
Subsequent Cargo commands must retain dev/test debug=0, incremental=0 and jobs=1.
A new low-debug MCP git_prepared test run is rebuilding dependencies; its result
is pending. Added real commit transport coverage checks denial without HEAD/index
mutation, index-only success and same-ID replay refusal; no pass claimed yet.

Checkout production migration is still absent. The collaborator returned only
TargetTree header-budget corrections; the primary agent owns the remaining
integration. Config checkout hook/submodule checks are currently test-only,
as is target-tree evidence. These prototypes must not be published as a repaired
checkout route. Current old push/pull/checkout execution still requires migration.

### Checkout production checkpoint (supersedes the preceding recovery checkpoint)

Checkout now uses the production two-stage prepared route with configuration-chain
discovery, target tree/ref/reflog evidence, parent identity checks, current/index/
target attribute inspection, and actual worktree/gitdir/common-dir write facets.
Direct execution cannot bypass admission. Same-path nonrecursive gitlinks capture
child configuration and status sources, bind the child Git executable, and declare
child gitdir writes for possible index refresh. Real acceptance preserves dirty
child contents and HEAD; configured child fsmonitor refuses before parent mutation
or helper execution. Linked-worktree acceptance verifies both metadata write facets
and preserves the main worktree HEAD.

Verification: all 164 Git-enabled library tests and Git-enabled all-target Clippy
passed on this checkpoint. The earlier MCP git_prepared rebuild
completed with both add and commit transport tests passing. No final-candidate
workspace preflight or publication is claimed. Added/removed/type-changing gitlinks,
recursive submodule checkout, remaining checkout compatibility acceptance, push/pull
migration, and delivery gates remain open. I300/T7 and the serial queue stay active.

### Checkout compatibility and target-effect review continuation

Nonrecursive checkout now accepts new gitlinks at missing paths and removal of
existing gitlinks without deleting dirty child contents. Existing-path additions,
gitlink type transitions and recursive submodule effects remain unsupported.
File/directory transitions now distinguish a missing target descendant behind a
regular file (ENOTDIR) from a failed read; conflict acceptance preserves dirty
content and the index. Real MCP checkout transport acceptance covers write denial,
successful branch/content switch, replay refusal and explicit owner cleanup.

Independent review found that a target tree can introduce an executable hook or
rewrite configuration that a later Git child reloads. Checkout now checks target
effects against effective hooks/configuration/auxiliary sources, including empty
or absent include destinations and system/global/local/worktree configuration
entrypoints. Fixed Git configuration-path queries discover installation defaults;
continuation facets admit their reads. Alias checks retain each symlink hop and
traversed ancestor, including ancestors followed by `..`, and conservatively
handle case/normalization ambiguity. Branch-conditional includes refuse explicitly
until target-branch configuration discovery is implemented. The new checks do not
claim filesystem race freedom. Final validation/review evidence follows below;
push/pull production migration, remaining compatibility and all delivery gates
remain open. No implementation completion commit exists yet.

Checkpoint validation: all 170 Git-enabled library tests, all three real MCP Git
transport tests, and tools/MCP all-target Clippy passed with locked low-debug,
non-incremental, single-job builds. Both governance validators passed with zero
warnings. Independent read-only review confirmed the specific target-hook, alias,
and missing-configuration-entry findings repaired; it is not final-candidate
security approval. Workspace preflight remains stale and must rerun after the
remaining production migrations, before publication.

Remote migration checkpoint: an ordered destination-resolution prototype now
matches real Git for multi-URL push, empty pushurl resets, explicit pushurl versus
pushInsteadOf, longest-prefix rewrites, and fetch's first URL. Two focused tests
passed without network access. The prototype remains test-only; Push/Pull still
use their old production route. Preserve configuration record ordering when
integrating it (the existing values map alone cannot resolve equal-length rewrite
ties faithfully). Legacy remotes/branches files and transport-specific effects
remain to be bound. The requested independent push review did not run: its agent
service returned an account-quota error. This is missing review evidence, not a
workspace-tool failure or a reason to claim completion.

Push production continuation supersedes the preceding prototype-only checkpoint:
GitPushTool now prepares the same admitted configuration-discovery/continuation
route and exposes its owned process lifecycle. Discovery retains ordered remote
records and resolves destination facets; old discovery proofs cannot authorize
the fresh continuation, and direct execution refuses bypass. Credential-bearing
or ambiguous destinations refuse before returning review facets. Push mutation
is temporarily closed until transport-specific helper, local-path, legacy-remote
and object-transfer effects are implemented and reviewed; this is an incomplete
migration, not a supported push implementation or an acceptable final deliverable.
Pull still uses its old route. Git-enabled library tests pass 173/173, including
real permission discovery and stale-proof rejection without network or mutation.

Push follow-up: destination rewrite ties now follow Git's first base-group order
(verified against host Git with interleaved declarations), and unrelated implicit
boolean remote options no longer invalidate URL extraction. Push discovery now
enumerates pre-push/reference-transaction hooks for later admitted inspection and
rejects unrepresented signing, recursive submodule execution, custom receive-pack,
remote helpers and Git proxy programs. Disabled signing/recursion retain discovery.
All 176 tools Git library tests and all-target Clippy pass. Transport execution
remains closed and incomplete; these checks do not establish full push effect
coverage or replace the missing independent transport review.
Pull child configuration continuation checkpoint (2026-10-16, `88246a89`):
configuration-chain discovery now has explicit state. The first continuation
records Git-reported include sources and returns for fresh exact Read admission;
the next continuation verifies every source. Correction (`b11c91ee`): retain
the original snapshots, revalidate them, and compare Git-reported values and
origins before accepting the second discovery. A changed include chain refuses
before capturing new origins. Shared preparation regression tests pass 129/129;
configuration identity/origin tests pass 3/3. `8a312b3e` also restores the distinct
Commit/Checkout gitdir interpretation during shared child revalidation.
Broader validation: `cargo test --locked -p talos-tools --all-features --lib`
passes 779/779. This is crate-library evidence, not workspace or delivery closure.
`98dff9b9` supersedes the helper-only external-gitdir prototype: separate marker,
gitdir/fixed-pointer, and common-root waves retain shared Root/Metadata evidence.
The prepared-plan test consumes fresh permission-engine proofs at each wave,
leaves targets absent until their own wave, and rejects subsequent marker and
commondir redirection. The duplicate marker parser was removed; Pull/Checkout
share exact child Read validation. Tools library tests pass 779/779.
`2edba07e` reuses ordered fetch destination resolution for child URLs and binds
configuration declaration order. Local sources request Read facets rather than
Network facets. Configuration tests pass 25/25; all-target Clippy passes; final
`cargo test --locked -p talos-tools --all-features --lib` passes 780/780.
`6f68807f` (rebased from `0cd8ae36`) binds the first recursive child fetch to the admitted HEAD branch,
ordinary fetch default-remote rules, and retained ordered destinations. It reuses
the parent remote resolver; non-branch symbolic HEAD chains refuse pending
separate resolution admission. Git v2.54.0 `submodule.c` and `remote.c` confirm
that this first-fetch path differs from missing-object retry remote selection.
Tools library tests pass 781/781 outside the sandbox; the sandboxed run passed
780 with the existing process-observation EPERM in the eight-stage Push test.
All-target Clippy passes. This is preparation evidence, not recursive execution.
`bb8e854c` reuses bounded graph inventory for child objects/references. Each
new descendant frontier requires fresh exact Read admission; retained inventories
are revalidated before later preparation. A prepared-plan regression consumes
permission-engine proof and rejects reuse of the previous frontier's tokens.
Tools library tests pass 782/782 outside the sandbox; all-target Clippy passes.
This establishes directory admission only. `dd66155d` additionally captures
admitted `objects/info/alternates` and `http-alternates`, refusing nonempty
external-store redirects. Empty-file bytes and absent-leaf directory listings
are retained and revalidated, including in-place edits and newly created leaves.
Tools library tests pass 784/784 outside the sandbox; all-target Clippy passes.
`5d31624d` binds admitted child HEAD, packed-refs and loose reference contents
under a shared 4096-entry/4-MiB budget. Revalidation detects in-place changes,
including a symbolic replacement declaration's target inside the refs tree;
capture never recursively opens paths absent from the admitted inventory.
Tools library tests pass 785/785 outside the sandbox; all-target Clippy passes.
`5e630e6f` reuses receiver symbolic-target validation to reject pseudo refs and
cross-worktree aliases outside retained refs namespaces; no separate parser was
introduced. Tools library tests pass 786/786 outside the sandbox; all-target
Clippy passes. This does not resolve symbolic chains or authorize other targets.
Object bytes, replacement resolution and actual child probes remain incomplete;
these checks do not establish recursive execution or final stage-budget reachability.
Missing-object retry remote selection, refspecs and transport/source effects,
object admission, recursive fetch, integration, runner stage capacity, and
delivery gates remain open; Production Pull remains disabled.

Pull child bootstrap checkpoint (2026-10-17): child configuration discovery now
uses the shared explicit `--git-dir`/`--work-tree` bootstrap probe instead of
ordinary repository setup, so configuration-provided `core.worktree` does not get
resolved during discovery. After exact Read admission for the fixed common and
worktree config files, bounded startup evidence retains raw local layout targets
and emits them as separate Read facets; revalidation detects edits. A regression
with a nonexistent configured worktree verifies the target remains unopened.
`cargo test --locked -p talos-tools --all-features --lib` passes 787/787 outside
the sandbox; all-target Clippy passes. Sandbox execution still has the known
process-observation `EPERM` in the existing eight-stage Push test. This checkpoint
covers startup discovery only: admitted target identity retention for execution,
objects/replacement semantics, child transport and retry behavior, recursive
fetch, parent integration and stage-budget reachability remain open. Production
Pull remains disabled; T7 and the original handoff remain incomplete.

Pull child identity continuation checkpoint (2026-10-17): after independent
Read admission, configured startup worktree directories now retain identity
and reject same-path replacement on Unix during subsequent plan revalidation. The
post-fetch path without attribute rules now shares the existing selected-child
identity transition rather than skipping directly to integration; it adds no
attribute-only approval stage. A real post-fetch fixture exercises explicit
activity across worktree/index/HEAD/absent module declarations, verifies fresh
child-directory/marker Read facets, and rejects the old admission tokens.
Locked all-feature tools library tests pass 787/787 outside the sandbox
(`/tmp/talos-t7-routing-tests-unsandboxed.log`); all-target/all-feature Clippy
passes (`/tmp/talos-t7-routing-clippy.log`). The sandboxed suite passed 786 and
failed the existing process-observation test with EPERM, then was rerun outside
the sandbox. A final focused routing run passes 1/1. Object probing/replacement
semantics, child transport/retries, recursive fetch, parent integration,
eight-stage runner reachability and final delivery gates remain open.
Production Pull remains disabled; I300/T7 and the handoff stay incomplete.

Linked-worktree reference closure update (2026-10-17): independently admitted
common targets now publish exact `refs`/`packed-refs` frontiers before descendant
access. External reference bytes share the existing 4096-entry/4 MiB evidence
budget. Main `common/config.worktree` and registered worktree fallback
`packed-refs` contents are also bound. Locked Pull tests pass 90/90; all-target,
all-feature Clippy passes with warnings denied. Independent final review found
no new blocker after the fallback packed-refs correction;
missing-relative target compatibility and the remaining object, transport,
integration, runner and delivery requirements remain open. Pull stays disabled.

Linked-worktree target admission update (2026-10-17): raw targets parsed from
retained pointer bytes receive new exact Read facets and a fresh plan identity
before resolution. Binding checks all grants first, verifies canonical versus
approved normalized mappings, and retains target identities. Absolute gitdir
backlinks do not trigger target access; relative spellings retain `..` until
admitted resolution. Pull-focused tests pass 88/88, including old-frontier
denial, wrong-mapping rejection, target replacement and pruned absolute backlink
coverage. All-target/all-feature Clippy passes; independent review found no new
blocker. Missing relative/forgiving
targets, external common reference descendants and remaining native-probe reads
remain activation blockers; production Pull and T7 remain incomplete.

Linked-worktree pointer evidence update (2026-10-17): admitted registration
`gitdir` and `commondir` file contents now share the reference evidence budget,
with pointer bytes kept separate from symbolic-ref validation. The 13/13
inventory regressions cover fresh frontier admission and in-place pointer
modifications; Pull-focused tests pass 86/86 and all-target/all-feature Clippy
passes. Independent review found no new blocker in the bounded evidence change.
Resolved-target authorization remains an explicit native-probe activation
blocker; production Pull and the broader T7 objective remain incomplete.

Linked-worktree graph inventory update (2026-10-17): child graph discovery now
admits `worktrees` through the existing bounded exact-path frontier; registered
worktree HEAD/refs contents are retained in the shared reference evidence budget.
Inventory regressions pass 13/13, including descendant admission denial and
in-place linked HEAD/private-ref changes. A real Git 2.54 detached-worktree
fixture confirms those HEADs participate in `rev-list --all` unless explicitly
restricted with `--single-worktree`. Object probing, recursive execution and
production activation remain incomplete.
The full Pull-focused suite passes 86/86; all-target/all-feature Clippy and
project/collaboration governance validators pass. Fetch found no new main
commits requiring rebase at this checkpoint.
Independent review found no new blocker for the inert frontier increment.
Before native child probing, linked registration `gitdir`/`commondir` still
require bounded content evidence and resolved-target restriction/admission:
Git's worktree/ref backend reads those pointers, so HEAD/refs evidence alone
is insufficient. This remains an explicit activation blocker.

Windows path identity implementation update (2026-10-17): maintainer accepted
Windows/Git-only same-file 1.0.6 (ADR-089 amendment). PathIdentity now retains
panic-guarded safe handles for canonical leaf and ancestors; configured child
worktree binding checks fresh Read admission and retains identity. Locked tools
tests pass 790/790 on macOS; Windows GNU library cross-check passes with the
existing run_host_command dead-code warning. CI explicitly selects ancestor/
leaf replacement and configured-worktree regressions. Incremental independent
review found no new blocker. Native Windows runtime evidence remains pending;
ReFS 128-bit identity, RegularEvidence's separate Windows leaf identity and
aggregate handle resource bounds remain tracked residuals. This supersedes the
pending dependency decision below, without closing T7 or enabling Pull.

Final stage validation: project governance and collaboration checks pass with
zero warnings. After changing the configured-worktree fixture to use `git config`
for portable path quoting, its focused locked regression passes. Windows GNU
test cross-check is environment-blocked by missing `x86_64-w64-mingw32-gcc`
(`/tmp/talos-t7-windows-identity-tests-check.log`); native Windows test execution
remains a delivery gate, separate from the successful library cross-check.

Independent review residual (2026-10-17): shared `snapshot::PathIdentity`
retains ancestor device/inode identity on Unix, but only canonical paths on
Windows. Same-path Windows directory replacement therefore remains undetected;
the new replacement regression is not verified on Windows and must not be
treated as cross-platform evidence. Direct Windows-only `same-file` use is
pending maintainer acceptance under ADR-089's dependency restriction. No new
dependency has been introduced. This stage remains uncommitted pending that
decision and the resulting implementation/review; the Unix checks above do not
close the Windows residual or authorize Production Pull.

Current-index child routing checkpoint (2026-10-17, `c0eae3f3`): identity discovery now
uses one shared candidate list, with already-selected current-index paths first
and active historical paths afterward, deduplicated by path. This restores
current `On` candidates even when no new gitlink changes exist; their selection
does not depend on historical activity. Git v2.54.0 `submodule.c`
`fetch_task_create` / `get_fetch_task_from_index` establish this requirement.
A regression initializes a real child repository, verifies exact child/marker
Read facets and rejection of previous-stage tokens, and confirms that metadata
resolution still waits for another admission. Locked all-feature tools library
tests pass 788/788 outside the sandbox
(`/tmp/talos-t7-current-child-tests-unsandboxed.log`); sandbox execution passed
787 with the known process-observation EPERM. All-target/all-feature Clippy,
formatting and whitespace checks pass. This fixes routing only: initialized-task
name suppression, uninitialized/legacy children, object reachability, child
transport/retries, integration and runner reachability remain incomplete, as does
the Windows identity decision above. Production Pull remains disabled.
Independent reviewer `review_prepared` found no new authorization blocker in
this increment. The reviewer confirmed fresh exact-path admission precedes root
capture, and distinguished path deduplication here from native initialized-task
name suppression, which remains open. The review did not cover actual recursive
execution or stage-budget reachability.


Child metadata scope checkpoint (2026-10-17): initialized child graph discovery
now uses the accepted dedicated metadata Read capability and separately approved,
identity-bound child gitdir/common roots to batch bounded directory traversal in
one continuation, rather than consuming an approval stage per directory depth.
External paths and raw linked-worktree pointer targets retain separate exact-path
admission. A real prepared-plan regression covers 24 directory levels, missing
capability/root refusal and reference-content mutation. Child default-remote
selection now strips config origin fields using the same ordered-record extraction
as parent Pull policy; configuration origin evidence remains retained.
Locked all-feature Pull library tests pass 90/90; all-target/all-feature Clippy
with `-D warnings` passes (`/tmp/talos-t7-child-scope-{tests,clippy}.log`).
Independent reviewer `review_prepared` found no new authorization blocker.
This establishes discovery batching, not full runner reachability: child object
semantics, transport/retries, recursive fetch, parent integration, eight-stage
end-to-end validation and final delivery gates remain open. Production Pull is
still disabled; I300/T7 remains Active and incomplete.


Windows regular-file evidence checkpoint (2026-10-17): `RegularEvidence` now
retains a Windows leaf handle using the already accepted Windows/Git-only
same-file dependency, closing the separate identity placeholder that treated
identical-byte replacement as unchanged. Dependency errors and panics reject
evidence capture. The existing identical-byte replacement regression now applies
to Windows and is explicitly selected by Windows CI. Its locked macOS execution
passes; all-target/all-feature Clippy passes. Independent reviewer
`review_prepared` found no new blocker, including retained-handle lifetime and
shared-delete behavior. Native Windows execution remains required; ReFS identity
and aggregate handle budgets remain residuals. All-feature Windows cross-check
cannot run on this host because x86_64-w64-mingw32-gcc is unavailable, so it is
not runtime evidence. Production Pull remains disabled.
The preceding child-scope stage also passes all 795 all-feature tools library
tests outside the sandbox (`/tmp/talos-t7-child-scope-all-tests-unsandboxed.log`);
sandbox execution passed 794 and hit the known process-observation EPERM.
Git-feature Windows GNU library cross-check passes
(`/tmp/talos-t7-windows-leaf-git-check.log`), with the existing unused
`run_host_command` warning; this is compilation evidence only.


Missing linked-checkout checkpoint (2026-10-17): after independent exact target
admission, relative backlinks may resolve through ordinary missing suffixes.
Evidence retains the resolved path and nearest existing ancestor identity;
creation of missing components or replacement of that ancestor invalidates the
plan. Commondir targets still must exist. Git 2.54 abspath.c confirms the native
forgiving resolver permits multiple missing components. Dangling symlinks and
missing paths containing unresolved parent components remain conservative
refusals, not full native equivalence, and require follow-up before claiming
ordinary-path coverage complete. No native object probe is activated here.
Locked all-feature Pull tests pass 93/93; all-target/all-feature Clippy passes
with warnings denied (`/tmp/talos-t7-missing-worktree-{pull-tests,clippy}.log`).
Independent reviewer `review_prepared` found no new blocker. Object/replacement
semantics, child transport/retries, recursive fetch, parent integration, runner
stage-budget success and final delivery gates remain incomplete. T7 stays Active
and production Pull stays disabled.


Reftable read-evidence checkpoint (2026-10-17): child inventory now includes
`reftable/` in each admitted gitdir/common root and binds table manifests and
files under the existing shared reference-content budget. Linked-worktree
private backends receive the same treatment; external common backends become
fresh exact-path frontiers before any inspection. Regressions reject stale
frontier admission and in-place table/list changes. Locked all-feature Pull
tests pass 96/96 (`/tmp/talos-t7-reftable-tests.log`). Manifest entries
are conservatively limited to ASCII single filenames, rejecting path separators,
Windows device names, special/control characters and trailing dot/space aliases.
This closes the native reader's unchecked manifest path join; nonstandard names
are refused rather than followed outside the admitted backend. No native child object
probe or fetch is enabled. Independent design review also identified in-place
object/acceleration-file mutations as an activation prerequisite: directory
identity alone cannot bind those bytes, so retained object decisions require
content evidence or a justified fresh-probe strategy. Object type/replacement
semantics, recursive transport/retries, parent integration and final delivery
remain open; Production Pull stays disabled and T7 remains Active.

Historical recursion-policy checkpoint (2026-10-17): active historical children
with effective `fetchRecurseSubmodules=false` no longer enter identity discovery
or add child path Read facets. Current-index selections remain independent, so
a current `On` task is not suppressed by a historical `Off` policy. Focused
regression covers Off, On, OnDemand and current-index precedence. Independent
review found no blocker in this correction. Object observations must be freshly
re-probed at their final consumer after every approval or object-store mutation;
no cross-stage existence/reachability cache may drive fetch decisions. Layout,
configuration, reference and redirect evidence remains mandatory. T7 stays Active.

Cross-stage validation checkpoint (2026-10-17, implementation `68b5adf4`):
locked all-feature Pull tests pass 97/97 and Clippy passes with warnings denied.
The complete locked all-feature talos-tools library suite passes 802/802 outside
the sandbox, including process-observation tests
(`/tmp/talos-t7-tools-all-current.log`). Both governance validators pass with
zero warnings. Incremental cache cleanup used the previously accepted scope;
subsequent builds disable incremental caching. This is tools-library evidence,
not full-workspace preflight, native Windows runtime or publication evidence.
GIX tracking was rechecked against the live sparse index: latest non-yanked
0.89.0 (published 2026-10-08, MSRV 1.88), Talos locked at 0.87.1. No newer release
or host-Git removal date is established. Remaining T7 execution/integration
and all subsequent handoff deliverables remain open.

Child common-root stage convergence checkpoint (2026-10-17): metadata discovery
continues under the current proof when every resolved child common root already
has exact Read admission. Ordinary gitdir==common layouts therefore use two
identity continuations rather than three; no configuration values are read at
this point. External common targets still return for fresh admission and their
canonical mappings are checked before ChildLocation capture. The real current-
child regression asserts this stage reduction; the external target regression
and all 97 locked all-feature Pull tests pass. This does not establish complete
runner success within eight stages or enable production Pull.

Parent graph content evidence checkpoint (2026-10-17): both prefetch and
postfetch now bind alternates and reference contents on the completed parent
inventory before subsequent object/graph inspection. This reuses the child
inventory's bounded evidence instead of introducing another snapshot layer.
The real tracking-plan regression rejects an in-place branch ref edit, then
accepts restored bytes. All 97 locked all-feature Pull tests and warnings-denied
library Clippy pass; independent incremental review found no blocker. Parent
linked-worktree target discovery/admission and external reference-root closure
remain prerequisites before native child probes or production activation.
Fetch found origin/main unchanged (118 ahead, zero behind before this commit).
T7 remains Active; no handoff completion or full-workspace validation is claimed.

Parent linked-worktree admission checkpoint (2026-10-17): prefetch and
postfetch now share bounded parent-inventory convergence. Raw linked targets
require exact grants and normalized identity matching; external reference roots
and descendants remain fresh frontiers. Successful fetch discards the old graph
inventory once; postfetch continuations retain the new evidence across approval.
The real local-fetch fixture rejects pointer edits between postfetch approvals
and retains FETCH_HEAD. Its deep external-ref case accounts for preparation and
fetch in PreparedStageHistory, stops when the shared budget refuses continuation,
and verifies untouched fetched bytes and an unvisited leaf. This fixture manually
mirrors the fetch transition; it is not full execute_stage integration success.
All 98 locked Pull tests and library Clippy pass; independent review found no
new blocker. Production activation, ordinary recursive success within eight
stages, child probes/fetch/retries, integration and final delivery remain open.

Child commondir parsing checkpoint (2026-10-17): ChildLocation now removes all
trailing CR/LF, matching receiver metadata parsing and Git's nonempty-pointer
rule while preserving path spaces. New regression covers repeated mixed line
endings, a directory with a trailing space, and the retained empty-pointer
refusal (the latter remains deliberately stricter than Git). All three locked
child-location tests pass; independent review found no blocker. Original pointer
bytes remain bound. Object probing still requires a supervised input path or an
equivalent library implementation; child fetch/integration and all original T7
closure gates remain open. This is a compatibility fix, not Pull activation.

Supervised input checkpoint (2026-10-17): OwnedCommandOperation now offers
explicit finite owned stdin delivery alongside the existing null-stdin API.
Input writes share the output loop, timeout, cancellation and process-group
cleanup; successful collection requires completed delivery. Independent review
identified an early-exit race, fixed by requiring input_done before success.
Regressions cover duplex pipe-capacity overflow, EOF, blocked-input timeout,
caller cancellation and early exit with undelivered bytes. All 48 locked sandbox
library tests and warnings-denied all-target Clippy pass. ADR-082 documents the
additive contract and caller-owned input authorization/size budget. This is the
input prerequisite for Git batch object probing, not an enabled probe or Pull
execution; the remaining T7 and handoff scope is unchanged.

Child batch object-type checkpoint (2026-10-17): after child configuration,
startup and graph/linked-target admission converge, selected historical gitlink
OIDs are checked using supervised finite stdin and Git cat-file batch protocol.
Only explicit missing records mean absent; failed execution, malformed responses
and non-commit objects refuse. Tags are not peeled and native replacement lookup
is preserved. Presence results are discarded immediately and cannot skip a later
fetch across approval; reachability and actual fetching remain unimplemented.
Input is capped at 256 KiB and output reserves the existing shared command budget.
All 100 locked Pull tests and all-target/all-feature warnings-denied Clippy pass;
independent incremental review found no blocker. Tests include actual discovery-
stage missing/noncommit cases and real Git replacement-to-tree/tag checks. Old plans correctly reject
changed replacement metadata, so those fixtures prepare new evidence first.
Non-Unix supervised input currently refuses explicitly; Windows support remains
required. Production Pull remains disabled and all original closure gates and
later handoff tasks remain open.

Child reachability checkpoint (2026-10-17): the child probe now follows native
submodule_has_commits with exact commit-type/presence checking followed by
rev-list -n 1 --stdin --not --all. Missing or unreachable commits return false;
nonzero traversal status follows native false semantics while owned execution
failures remain errors. Two probes are not an atomic graph snapshot and the
result is discarded in discovery; the future fetch consumer must freshly probe.
A shared child command builder keeps cwd/gitdir/worktree/environment consistent.
Real Git coverage includes reachable, orphaned, mixed and missing OIDs plus an
orphan made reachable solely by a linked worktree's detached HEAD. All 100 locked
Pull tests and all-target/all-feature Clippy pass; independent review found no
blocker. This prepares fetch selection but does not yet change fetch behavior.
Child fetch/retries, parent integration, Windows input, complete runner evidence
and all original T7/handoff closure gates remain open.

Windows Job command-line preparation checkpoint (2026-10-18): the existing native
launcher now uses a shared UTF-16 argument encoder with no lossy conversion,
rejecting embedded NUL and command lines above 32,767 UTF-16 units including the
terminator. Separate platform-independent tests pass 3/3 with all features,
including unpaired UTF-16 units, quoting and expansion limits. A default-feature
run selected zero tests and is not counted as evidence. Frozen OsString command
and environment extraction, stdin and foreground ownership/cleanup integration
remain open; Windows structured Git remains refused. The original 49cc215f full
release preflight completed successfully, but does not certify these later edits.

Windows Job environment checkpoint (2026-10-18): the existing background launcher
now snapshots vars_os rather than panicking on non-Unicode environment values.
A shared explicit-snapshot encoder uses std Command's Windows key ordering and
case-equivalent override handling without spawning; UTF-16 values are preserved,
invalid names/NUL rejected, and empty/nonempty blocks terminate with double NUL.
Tests are separate: six combined environment/command-line tests pass on macOS;
all-feature/all-target locked Clippy passes. The Windows-specific key-override
test is not executed on macOS. Full-feature Windows cross-check failed before
Talos compilation because x86_64-w64-mingw32-gcc is missing; this is not Windows
validation evidence. Git-only cross-check is tracked separately. Foreground Git
still requires exact frozen-environment transfer, stdin and Job ownership/receipt
integration; this helper does not claim end-to-end non-Unicode launch support.

Environment review correction: independent review identified that std Windows
key ordering may panic before validation for an empty key. Snapshot now validates
all UTF-16 key/value pairs before Command insertion and catches ordering panics
as errors. Windows snapshot regression assertions include invalid keys. Git-only
Windows cross-check passes with three unused fallback warnings; it is compile
evidence only. Full-feature cross-check still needs the missing MinGW compiler.

Windows native launch input checkpoint (2026-10-18): the existing background
entrypoint now prepares lossless OsStr/OsString argument buffers and an explicit
environment block before any native handles are allocated. The same native
create/assign/resume/cleanup implementation consumes only those buffers and
never re-reads PATH or parent environment. Background executable resolution
and inherited-environment selection remain in its compatibility wrapper; original
argv[0] spelling is retained. A separate Windows-only input regression is added
but has not run on Windows. Git-only Windows cross-compilation passes; foreground
Git ownership, finite stdin and receipts are still incomplete and remain refused.

Finite Windows stdin primitive checkpoint (2026-10-18): the shared native Job
launcher accepts an optional RAII pipe prepared before raw launch allocation.
Only the child read endpoint joins the inherited-handle allowlist; the parent
writer is explicitly non-inheritable. Failure drops both endpoints; successful
launch closes the local child endpoint and transfers the writer to the caller.
Background launch continues to request no input. A separate Windows-only test
checks inheritance flags, binary bytes and EOF, but is not executed on macOS.
Git-only Windows cross-compilation passes. Foreground input delivery/deadlines,
cancellation and terminal Job receipts remain unimplemented; no activation or
Windows runtime verification is claimed. The no-input foreground NUL handle and
full setup-fault/resource matrix also remain required before activation.

Stdin review correction: the inherited-handle buffer now lives outside the
launch closure in both production and tests, through failure-path attribute-list
deletion. Independent review identified premature Vec drop in the initial
production-only branch; that divergent branch was removed. Windows Git-only
cross-check passes after correction. Real Windows lifecycle evidence remains open.

Windows Job cleanup confirmation checkpoint (2026-10-18): the shared control now
queries JobObjectBasicAccountingInformation after termination and returns success
only when ActiveProcesses is zero. Query errors and a five-second deadline return
unconfirmed cleanup errors. Independent security review found no blocker in the
limited delta; background signal/force plus grace can now take up to fourteen
seconds and foreground ownership must account for that bound. Separate empty,
invalid-handle and live-process tests were added but have not run on Windows.
Git-only Windows library cross-check and macOS all-feature/all-target locked Clippy
pass. Descendants with closed stdio, cancellation/reconfirmation, query-failure and
timeout runtime evidence remain open; this does not complete foreground integration.

Windows no-input checkpoint (2026-10-18): the shared native launch command owns
a read-only NUL handle for no-input calls, sets its inheritance flag, and includes
it in the existing explicit handle allowlist. Finite stdin remains a separate
mutually exclusive pipe. This also changes background stdin from an invalid handle
to immediate EOF; programs can observe that difference. Independent review found
no blocker in this limited delta. Windows Git library cross-check and macOS locked
all-feature/all-target Clippy pass. Separate local-handle and real-child EOF tests
are added but not executed on Windows. Existing eight-resource fault accounting
does not cover this RAII NUL handle; return-time release evidence and native fault
matrix remain required. Foreground lifecycle integration remains incomplete.

Windows stdin ownership simplification checkpoint (2026-10-18): commit 1d7fe170
replaces two independent optional handles with one Pipe/Null enum, so neither
missing nor simultaneous stdin modes are representable. Native startup and the
allowlist use the same child handle; successful launch consumes the enum to close
the local read side and transfer only a finite writer. Independent review found
no blocker; early-error local stdin closure precedes outer native cleanup, while
the attribute-list backing buffer still outlives deletion. Git-only Windows
cross-check, all-feature/all-target locked Clippy, project governance and claim
validators pass. Native failure/release evidence and foreground integration remain.

### Windows foreground integration checkpoint (2026-10-18)

The foreground executor now reuses the assigned-before-resume native Job launch
boundary, retaining IO workers through cancellation/timeout and requiring Job
termination plus joined IO before confirming cleanup. IO worker panics and launch
failures produce sticky unconfirmed receipts. Native launch panics take the same
raw-owner cleanup path; only initialized attribute lists enter that cleanup.
Tests live separately in `process_boundary/windows_foreground_tests.rs`. Windows
CI explicitly requires six named lifecycle/finite-input/timeout/cancellation
regressions. These tests still require Windows compilation and execution.

Evidence correction: earlier Git-only Windows checks excluded the native module
through `lib.rs` cfg. The corrected gate now compiles the actual foreground/native
Job code. Locked Windows GNU library checking passes with unused fallback and
background-helper warnings; macOS locked all-feature/all-target Clippy passes.
Neither result is Windows runtime evidence. The non-Unix preparation refusal stays
in place until runtime/failure-path validation and final review are complete.
T7 remains Active; this checkpoint is not feature delivery or handoff completion.

The standard release preflight exited 0 for this candidate, including locked
workspace check, Clippy, tests and the external SDK fixture; log:
`/tmp/talos-i300-foreground-preflight.log`. Protected native inventory now includes
`windows_foreground.rs`, with updated baselines after independent protected-source
review. Its three mutation/baseline tests pass. No Windows runtime, exact-head CI
or merged delivery is claimed.

Pull retained-byte discovery checkpoint (2026-10-18): pointer candidate parsing
no longer performs implicit filesystem revalidation. Both production callers
first bind/revalidate admitted references; binding still independently rejects
changed registration evidence. A regression removes the retained pointer source,
proves discovery still parses only the retained bytes, and proves subsequent
source validation/binding refuse. Inventory tests pass 28/28. Independent targeted
review found no new bypass in this delta; raw-target Path normalization and the
invocation-wide accounting/resolution capability remain open and production Pull
remains disabled. This is not final T7 security approval or completion.

CI correction validation checkpoint (2026-10-18): source cd9755be passed the
standard locked release preflight outside the sandbox (session 78771 exited 0;
`/tmp/talos-i300-ci-correction-preflight-unsandboxed.log`), including workspace
checks, Clippy, tests and both external SDK fixture runs. Commits 83a0521f and
cd9755be respectively restrict obsolete host helpers to tests and apply accepted
foreground maintenance settings to the post-fetch test fixture. The earlier
sandbox run failed process observation with EPERM; this successful run supersedes
that local validation failure, not the outstanding remote Windows CI. The fixture
race cause remains tentative. PR #735 still requires fresh exact-head CI and
review; production Pull and non-Unix preparation remain disabled, and T7 Active.

Test separation checkpoint (2026-10-18): moved the 28 Pull inventory unit tests
to `pull_inventory_tests.rs`, preserving their module path and production source.
The extracted body was checked against the original before rustfmt; formatting
only joins two expressions after deindentation. All 28 inventory tests pass with
locked all-feature tools testing. This reduces the production inventory file
from 1314 to 553 lines; it is organization, not new Pull capability or completion.

Pull directory revalidation checkpoint (2026-10-18): retained sorted names are
checked incrementally with a bounded seen bitmap instead of recapturing a full
second path vector. Missing names, unknown names and duplicates refuse; entry
identity checks remain separate. A regression covers deletion, equal-count
replacement and restoration without changing the retained baseline. Locked
inventory tests pass 29/29, all-feature tools library Clippy with warnings denied
and workspace format checking pass. Independent security delta review found no
new blocker or read-scope expansion. This removes duplicate path retention only:
bitmap, directory handles and temporary platform path allocations still require
invocation-wide accounting. Production Pull remains disabled; T7 remains Active.

Pull file revalidation checkpoint (2026-10-18): alternate/reference evidence now
compares retained bytes through an 8192-byte stack buffer rather than capturing
a second whole-file snapshot with a reset budget. Checks retain missing-state,
leaf identity, permissions and content semantics, and compare opened-file
identity, length and EOF. Shared regular opening retains existing no-follow /
nonblocking Unix and open-reparse Windows flags. Independent review found a lost
Windows post-read reparse rejection; the leaf identity helper now rejects
reparse metadata before both identity checks, and follow-up review closed that
static blocker. Snapshot tests pass 11/11, inventory tests 29/29, Pull-prefixed
tests 72/72, and all-feature tools library Clippy passes with warnings denied.
The corrected Windows Git-only library cross-check passes; it is not runtime
evidence. Fixed scratch and temporary handles still need invocation-wide quota
reservation, and existing path-based TOCTOU residuals remain. No activation or
T7 completion is claimed.


Windows CI test-control correction (2026-10-18): run 39117998283 on
`e20e89d8` passed Windows workspace Check/Clippy but failed test compilation:
the direct lifecycle control still imported removed `quote_windows_arg` and
used fallible `windows_environment` as a buffer. The control now reuses the
pure UTF16 encoder and propagates environment errors before its resource
baseline. Independent protected-source review found no new blocker; direct
Win32 ownership/acquisition/cleanup remains independent of the production
launcher. The reviewed source fingerprint was refreshed. Locked all-feature
encoder tests pass 3/3; standalone edition-2024 native inventory tests pass
3/3; formatting and diff checks pass. These macOS checks do not compile or run
the Windows control. Fresh Windows test-target/runtime CI and current-head full
preflight remain required. Production Pull remains disabled; T7 is not complete.

Current-source full validation (2026-10-18, implementation `42492944`):
standard locked release preflight exited 0 outside the sandbox; tools tests
861/861 and external runtime SDK fixtures passed. Log:
`/tmp/talos-i300-control-fix-preflight-unsandboxed.log`. The earlier sandboxed
run failed only on process observation EPERM and is superseded for this gate.
Fetch found no origin/main-only commits (ahead 165 / behind 0), so no rebase
was needed. Publish this reviewed batch to PR #735 for fresh Windows test
compilation/runtime evidence. This does not complete T7 or enable Pull.

Pull retained-reference checkpoint (2026-10-18): repeated reference binding
now revalidates existing bytes/leaf identities in place and captures only new
paths. Existing evidence consumes the same per-inventory file allowance before
new capture; no second full copy of retained file bodies/handles is created.
Independent review found no blocker. Inventory tests pass 29/29, including
repeated binding and subsequent drift refusal; all-feature library Clippy with
warnings denied and formatting pass. Logs:
`/tmp/talos-i300-retained-reference-{tests,clippy}.log`. This is not invocation-wide
accounting: shared parent/child/continuation quotas, temporary resource peaks,
resolution authorization and full Pull delivery remain open. Production Pull
stays disabled and T7 remains Active.

Pull cumulative reference allowance checkpoint (2026-10-18): AddPlan now owns
one shared reference-file allowance cloned into all four production parent/child
inventory construction sites. New file captures consume cumulative entries and
bytes across inventories, replacements and continuations; repeated retained-file
revalidation does not recharge, and dropping an inventory does not refund.
Independent review found no blocker. Inventory tests pass 31/31, Pull-prefixed
tests 118/118, all-feature library check/Clippy and formatting pass. Logs:
`/tmp/talos-i300-reference-quota-{check,tests,pull-tests,clippy}.log`.
This only closes reference-file allowance resets: directory/path/link evidence,
alternates and Windows temporary/retained-handle peaks still require the unified
invocation quota. Production Pull remains disabled; T7 stays Active.
Windows CI 39122382667 compiled the previously failing test target and ran
foreground tests, but path_identity_retains_leaf_and_ancestor_identity failed
when renaming a parent directory with retained handles (Access denied). This
must be resolved and revalidated on Windows; no platform completion is claimed.

Windows metadata-handle correction (2026-10-18, `f4c3ba0d`): dependency inspection
shows same-file's path constructor requests content-read access. PathIdentity
now opens metadata-only handles with explicit read/write/delete sharing, then
retains the same-file identity. All original ancestor/leaf replacement assertions
remain. Independent security review found no blocker; local snapshot tests pass
9/9, formatting passes, and Windows GNU Git-only library check passes with seven
existing feature-only dead-code warnings. These checks do not prove the Windows
runtime failure is resolved. Full standard preflight is running in session 48484,
log `/tmp/talos-i300-metadata-handle-preflight.log`; fresh native Windows CI is
still required. Production Pull remains disabled and the full handoff stays open.

Shared alternate evidence checkpoint (2026-10-18, `d830597a`): the four fixed
alternate-file candidates now consume the same invocation file allowance as
references. Rebinding revalidates and reuses retained evidence instead of opening
another copy or resetting a separate pool. Nonempty alternate files still refuse;
the 256 KiB capture ceiling remains. Independent review found no blocker;
inventory tests pass 33/33 and all-feature library Clippy passes with warnings
denied. Full preflight session 48484 failed because sandbox execution of the CLI
fixture returned PermissionDenied; approved external session 96720 now runs the
standard gate against the shared-alternate source. Log:
`/tmp/talos-i300-shared-alternates-preflight-unsandboxed.log`. This is not proof
of full path/directory/temporary-resource accounting or native Windows success.

Stage synchronization (2026-10-18): fresh fetch succeeds; `d830597a` has no
origin/main-only commits (ahead 170 / behind 0), so no rebase is necessary.
Both governance validators pass with zero warnings. Remote run 39122382667 is
terminal: Linux standard workspace gate passed; Windows identity runtime failed
as recorded above. That run predates the metadata-handle correction and does not
validate it. Await external preflight session 96720 before publishing this batch.

Stable-candidate gate result (2026-10-18): approved external session 96720
exited 0; its log ends `release preflight: passed`. This validates implementation
`d830597a` through the standard locked workspace check/Clippy/tests (tools
865/865) and independent external runtime SDK fixtures. Publish the reviewed
batch to #735 for native Windows evidence. This does not activate Pull or close
T7; the original handoff queue remains authoritative.


Pull pointer-authorization integration (2026-10-18, `36853e60`): retained
registration pointers now request the dedicated resolution capability before
component inspection; resolved targets receive fresh exact Path Read facets
before reference discovery. Parent and child inventories share the resolver's
cumulative allowance with reference/alternate captures, including failed
captures; retained revalidation does not recharge. The old canonicalize-based
worktree target resolver is removed. Independent security review found no new
authorization bypass. All-feature Pull tests pass 123/123, the permission scope
matrix passes 1/1, library Clippy with warnings denied and formatting pass.
Logs: `/tmp/talos-i300-resolution-pull-final.log`,
`/tmp/talos-i300-resolution-grant-tests.log`, and
`/tmp/talos-i300-resolution-final-clippy.log`.

Windows CI 39124767443 still fails the parent-rename identity test despite
metadata-only handles. Commit `0f445a84` retains every original identity assertion
and live handle, but moves the same leaf out before replacing the empty parent,
then restores it. The targeted local test passes; native Windows evidence is
still required. Current-source standard preflight session 57864 exited 101:
LLVM stripping ran out of disk and left the CLI binary as a non-executable
3376-byte file; all 15 config_unset cases then failed to execute it. This is not
a passing gate. Within prior cleanup authorization, removed rebuildable examples,
incremental and the damaged CLI outputs, restoring approximately 1.8 GiB free.
Approved external standard preflight session 75918 then exited 0, ending
`release preflight: passed`; the standard locked workspace gate and external
Runtime SDK fixture passed for implementation `36853e60`. Log:
`/tmp/talos-i300-resolution-stage-preflight-unsandboxed.log`. Both governance
validators also pass without warnings. Publish the reviewed candidate to #735
for fresh native Windows validation; no platform completion is implied.

Remaining: directory/path/identity/temporary peak quota unification, recursive
Pull and final integration execution, real wrapper acceptance and native Windows
validation. This stage does not prove full ADR-089 resource accounting. Production
Pull and non-Unix preparation remain disabled; T7 and the full handoff remain open.

Resolution queue checkpoint (2026-10-18, `055f65e9`): reserve initial and
link-expanded component text before allocating owned queue components; remove
the duplicate link target and intermediate component vector. Resolver tests pass
7/7, including refusal before metadata discovery and refusal at link expansion
when only queue text exceeds the remaining allowance. Evidence:
`/tmp/talos-i300-resolution-queue-final-tests.log`. All-feature library Clippy
passes (`/tmp/talos-i300-resolution-queue-clippy.log`). This is conservative
cumulative queue charging, not completed temporary-peak or directory/identity
accounting. The previous stable stage was pushed through `32dda28a` to #735;
CI 39127288380 is still running, including native Windows identity validation.
No full preflight or remote exact-head evidence is claimed for `055f65e9` yet.

Native Windows result (2026-10-18, CI 39127288380): the Windows job is
terminal failure. The original path identity ancestor/leaf replacement test now
passes, as do real Git startup, executable replacement guard, regular-evidence
replacement and focused foreground supervision tests. The next test,
`pull_child_bootstrap_defers_worktree_and_retains_startup_evidence`, fails at
`prepare_pull`: production preparation still deliberately rejects non-Unix
platforms while foreground Job integration acceptance is incomplete. Thus the
identity repair has native evidence, but Windows preparation remains a delivery
blocker; neither skipping this test nor opening preparation without full review
is closure. Job log: `/tmp/talos-i300-windows-39127288380.log`, job 117675006764.

Windows lifecycle acceptance extension (2026-10-18): add required native CI cases
for an infinite output writer exceeding the allowance and a leader exiting while
its acknowledged descendant loops with separately redirected stdio. Both require
a successful cleanup receipt; the descendant observer fails and explicitly stops
any survivor. Independent review of the first fixture identified a missing child
acknowledgement; the revised script retains a ready marker and live-child check
before leader exit. Formatting and Git-only Windows library cross-check pass
(`/tmp/talos-i300-windows-foreground-lib-check.log`, seven existing feature-only
warnings). Cross-checking test targets is blocked by absent
`x86_64-w64-mingw32-gcc`; neither new test has native runtime evidence yet.
Review found no static spawn bypass, but opening the common preparation gate
also requires native Add/Commit/Checkout/Push authorization/execution coverage,
concurrent-operation isolation, large-input/two-output-stream and failure cleanup
acceptance. Keep the gate closed pending those results.

Windows concurrency/IO acceptance extension (2026-10-18): add native CI cases
that cancel one acknowledged live operation while another remains blocked until
explicit release, and mirror 512 KiB of binary stdin into both stdout and stderr.
The concurrency case requires a cancelled JoinError, independent cleanup receipt,
and exact successful output from the surviving operation. Static independent
review identified the missing cancellation-result assertion; it is now retained
and checked after cleanup. These tests still require native Windows execution.
Fresh fetch finds origin/main behind 0 / implementation ahead 180 before this
batch, so no rebase is required. Standard preflight is running in session 12231,
log `/tmp/talos-i300-windows-lifecycle-preflight.log`.

CI 39127288380 is terminal failure, not only a Windows failure: the main full
workspace test job fails `pull_postfetch_rebuilds_evidence_before_selecting_integration`
at the linked-budget frontier with an inventory InvalidInput. The current local
isolated test passes 1/1 (`/tmp/talos-i300-postfetch-repro.log`); this does not
invalidate the remote failure or establish its cause. Investigate before merge.
Remote job log: `/tmp/talos-i300-linux-39127288380.log` (job 117675006825).

Post-fetch fixture correction (2026-10-18): independent investigation confirms
the remote failure ran on macOS with Git 2.55.0, but has not established its root
cause. The linked-budget assertion now uses the same canonical external root as
the commondir pointer, avoiding a false negative through `/var` aliases. Fixture
Git calls fix foreground maintenance scheduling to match the accepted production
policy. Test-only inventory diagnostics identify the rejecting source location
without printing repository paths or reference contents; frontier failures also
report the source case and stage number. The focused post-fetch test passes
locally (`/tmp/talos-i300-postfetch-diagnostics.log`); this is not proof that the
remote failure is resolved.

Standard preflight session 12231 exited 101: tools 868 passed / 1 failed because
the local-push fixture's process observation received OS PermissionDenied. The
approved external standard preflight is running in session 82402, log
`/tmp/talos-i300-lifecycle-preflight-external.log`. It began before the fixture
correction above, so do not treat it as exact-candidate evidence for that change.
T7 remains Active; production Pull and non-Unix preparation remain disabled.

External preflight result (2026-10-18): session 82402 exited 0 and ends
`release preflight: passed`. Tools passed 869/869, and both external Runtime SDK
fixture variants passed. This closes the local process-observation environment
failure; it does not establish the cause of the earlier remote frontier failure
or provide native Windows evidence. Independent review of `68cd246a` found no
production behavior change or authorization bypass; its diagnostics remain
partial (some helper failures still combine path, OS and quota causes). Both
governance validators and formatting pass. Fresh origin/main fetch has behind 0 /
ahead 182, so no rebase is needed. Publish the stable stage to #735 for native
Windows lifecycle results and remote post-fetch diagnostics; do not merge while
preparation and full Pull acceptance remain incomplete.

Streaming identity revalidation (2026-10-18): Pull inventory now compares each
retained ancestor identity without constructing a second full PathIdentity.
Windows uses the same metadata-only, rename-compatible handle opener for initial
capture and comparison; each comparison handle is dropped before the next
ancestor. Missing entries, leaf links, type changes and replaced parents still
refuse. Independent static review found no authorization bypass or weakened
identity predicate. Inventory tests pass 34/34 and the shared identity replacement
test passes 1/1; Windows Git-only library cross-check passes with seven existing
feature-only warnings. Logs: `/tmp/talos-i300-inventory-stream-revalidate.log`,
`/tmp/talos-i300-stream-identity.log`,
`/tmp/talos-i300-stream-identity-windows.log`. Native execution is still required.
The fresh external standard preflight runs in session 92511, log
`/tmp/talos-i300-stream-identity-preflight.log`; no terminal result yet.

Quota scope audit: the four graph inventories are not the entire Pull invocation.
PullEvidence still has its own 16 MiB capture budget and reconstructs evidence on
revalidation; repository/configuration/child-location evidence also retains
identities independently. Full shared cumulative and peak accounting must cover
these paths as well as graph discovery and pointer resolution. The streaming
comparison above is a prerequisite, not a claim of completed quota enforcement.
The preceding stable candidate is pushed as `290628dc`; CI 39129652021 is running.

Windows operation acceptance checkpoint (2026-10-18): external standard
preflight session 92511 exited 0; the stream-identity log ends
`release preflight: passed`. This supersedes its running status above and covers
786d9c2a, not subsequent unit-harness changes. Native Windows job 117682063849
passed the new output-exhaustion, descendant-cleanup, concurrent-cancellation and
large duplex-I/O regressions, then failed the child-bootstrap fixture at the
still-closed preparation gate. This is primitive lifecycle evidence, not full
prepared-operation acceptance.

The next candidate opens preparation only under Windows cfg(test), adds seven
existing Add/Commit/Checkout acceptance cases to the Windows CI inventory, and
adds an integration test linking the production library to assert all five write
tools still reject before repository discovery. No production activation is
claimed. Formatting passes. Local prepared-operation tests initially passed
138/139 with the local-push process observer denied by the sandbox; the approved
external rerun passes 139/139, log
`/tmp/talos-i300-windows-unit-gate-external.log`. The new Windows-only integration
test still requires native CI; a host run cannot validate its body. Remaining
Pull functionality, unified quota, Windows operation acceptance and publication
remain owned here; T7 remains Active.

Candidate f29e4ed0 standard preflight exited 0 (session 82228), log
`/tmp/talos-i300-windows-gate-preflight.log`; both external Runtime SDK variants
passed. The candidate is pushed and native CI 39135084930 is running. Fresh
origin/main fetch has behind 0; no rebase is needed. Previous CI 39129652021
finished with the main workspace job passing and Windows preparation fixture
failing as described above.

Pull evidence streaming checkpoint (2026-10-18): revalidation now compares the
retained metadata bodies with the existing fixed 8192-byte scratch buffer instead
of recapturing a second PullEvidence. It reselects and validates the closest
existing parent before opening each leaf, brackets each comparison with retained
parent identity checks, retains both index timestamp checks and compares the
complete RepositoryState at the end. Immutable tracking/fetched fields used only
for reconstruction are removed. Initial post-fetch capture semantics are intact.
Independent static review found no lost predicate or authorization bypass.
Pull-state tests pass 25/25, including an independent test file covering newly
created intermediate directories, a changed second body chunk, restored contents
and identical-byte inode replacement; log
`/tmp/talos-i300-pull-evidence-stream.log`. This reduces PullEvidence duplicate
bodies/handles only: RepositoryState still recaptures and whole-invocation quota
accounting remains incomplete. T7 stays Active and production Pull stays closed.

Shared metadata allowance checkpoint (2026-10-18): both production PullEvidence
captures now use the AddPlan reference ledger shared with graph inventories and
path resolution, replacing their independent 16 MiB body allowance. The default
capture wrapper is test-only; production must supply the ledger. Pre/post-fetch
captures cumulatively consume the same 4 MiB allowance; a post-fetch exhaustion
still returns through the existing retained-effects error path. This intentionally
rejects operations whose combined metadata exceeds the bound, even when each
individual capture would previously fit.

Independent review also identified and verified a fix for failed-read charging:
regular snapshot reads now debit observed bytes before propagating partial I/O
errors, and a growing-file overrun exhausts the allowance rather than preserving
it. Separate tests cover partial read failure, overrun, and a 3 MiB metadata
capture followed by revalidation/drop/replacement without refund. Snapshot tests
pass 12/12; before the shared-helper correction, Pull state passed 26/26 and
prepared Pull tests passed 41/41. Logs: `/tmp/talos-i300-failed-budget.log`,
`/tmp/talos-i300-pull-metadata-quota.log`,
`/tmp/talos-i300-pull-metadata-prepared.log`. Full final-candidate validation
remains required. Directory/path/identity serialization, retained/transient
handles, independent RepositoryState capture and index decoding remain quota
residuals; this is not whole-invocation accounting or T7 completion.

Final focused checkpoint: all Pull graph/resolution/state tests pass 81/81
(`/tmp/talos-i300-metadata-quota-all-pull.log`); the revised failure Reader handles
short buffers and its final test passes 1/1. Windows Git-only library cross-check
passes with the seven existing feature-only warnings. Both governance validators
pass. Native candidate f29e4ed0 Windows job 117697889352 now passes the production
preparation gate, child-bootstrap and selected Add acceptance cases, then fails
`real_commit_preserves_index_semantics_and_all_excludes_untracked`: maintenance
executable differs from bound Git. The remaining later operation cases were not
run. Log `/tmp/talos-i300-windows-117697889352.log`. Investigate executable binding;
do not weaken the guard or claim Windows operation acceptance.

Maintenance binding diagnostic checkpoint (2026-10-18): the native f29e4ed0
candidate's Format/Check/Clippy/Test job finished successfully, but the Windows
job failed at the maintenance binding guard described above. The next candidate
accepts CRLF as well as LF framing for the path-only `--exec-path` response and
adds test-only rejected-child/canonical/bound path diagnostics. This is a narrow
line-protocol normalization (including the trailing-CR ambiguity on Unix), not
evidence of the Windows root cause. The canonical diagnostic is a second
observation after rejection, not an atomic snapshot of the failed comparison.
Fresh child Read/Execute admission and the canonical executable identity guard
are unchanged; independent review found no authorization-boundary relaxation.
The focused local Commit test passes 1/1
(`/tmp/talos-i300-maintenance-crlf.log`). Native Commit/Checkout/Push acceptance
and production Windows preparation remain outstanding.

Final local stage validation: the pinned-toolchain standard
`./scripts/release_preflight.sh` exited 0 with incremental compilation disabled,
debug info disabled and two build jobs. Log:
`/tmp/talos-i300-metadata-maintenance-preflight.log`. It covers 2134fb19,
24449b2c and the maintenance path/diagnostic source changes, including workspace
tests and both external Runtime SDK fixture variants. This is local validation;
fresh remote evidence is still required for this stage and no merge is claimed.

Directory discovery allowance checkpoint (2026-10-18): Inventory construction
is now fallible and charges initial frontier spellings before joining/retaining
them. Discovery charges visited keys, directory keys, directory-entry names
before joining, and next-frontier clones through the same cumulative
ReferenceBudget as regular metadata bodies and pointer resolution. All four
production construction sites propagate failure and retain the invocation's
shared ledger. Deduplicated initial spellings still consume discovery allowance;
failed advances preserve pending and never refund observed work. This tightens
the accepted 4096-entry/4-MiB refusal behavior across parent and child inventories.

Independent review found no new authorization bypass or lost frontier. Pull
library tests pass 131/131 (`/tmp/talos-i300-directory-all-pull.log`); the final
four separate directory-budget tests also pass after adding a directory/body
joint-ledger assertion (`/tmp/talos-i300-directory-final.log`). They cover shared
construction exhaustion, byte exhaustion, failed advance/retry/drop/rebuild and
revalidation without rediscovery. Existing tiny-body-budget tests isolate their
body ledger after discovery only in test code. Windows Git-only library
cross-check passes with the seven existing feature-only warnings
(`/tmp/talos-i300-directory-windows.log`); it is not native runtime evidence.
Temporary paths()/exact_paths() copies, worktree-frontier copies, identity
serialization and retained/transient handles remain quota residuals, as do
RepositoryState/index/config capture. Full stage validation remains required;
production Pull and Windows preparation stay closed and T7 stays Active.

Validation follow-up (2026-10-18): directory-stage preflight exited 101 at
`prepared_local_push_runs_all_eight_admitted_stages`, with macOS process
observation returning PermissionDenied / Operation not permitted (875 tools
tests passed). This is not a passing full preflight. The standard command was
restarted with explicit sandbox escalation; log
`/tmp/talos-i300-directory-preflight-unsandboxed.log`, exited 0 including both
external Runtime SDK variants. This validates directory-stage fb7bb553.

Native Windows run 39136878491, job 117703093508, confirms the maintenance
refusal persists at 7fde190a. Test diagnostics show the bound entry at
`C:\Program Files\Git\bin\git.exe` and canonical child at
`C:\Program Files\Git\ucrt64\libexec\git-core\git.exe`. CRLF normalization did
not resolve the different-program layout. Log:
`/tmp/talos-i300-windows-117703093508.log`. Independent review confirms fresh
child Read/Execute admission permits identity capture but does not prove its
configuration semantics match the original entry. The proposed next repair is
bounded re-preparation from the separately admitted internal Git, discarding
old program-specific evidence and rediscovering location/configuration/exec-path.
This proposal is not implemented or validated; the existing guard stays intact.

The same remote run's main validation job 117703093505 also failed independently:
`loose_reference_changes_invalidate_snapshot_without_new_objects` observed a
transient `objects/maintenance.lock` in only one object-layout snapshot. Its
fixture Git helper now sets both accepted foreground-maintenance options, so
setup commits complete their maintenance before comparing repository snapshots.
The object-layout assertion and production capture are unchanged. Log:
`/tmp/talos-i300-main-117703093505.log`. Focused repository-state validation
passes 14/14 (`/tmp/talos-i300-repository-maintenance.log`). Both governance
validators pass with zero warnings. Fresh full-stage validation and native
Windows acceptance remain outstanding.

Borrowed Pull inventory checkpoint (2026-10-18): production path and exact-path
queries now borrow inventory entries rather than cloning full path vectors.
Retained graph inventory copies charge the invocation ReferenceBudget before
each clone; failed partial captures do not refund consumed allowance. Test-only
owned query helpers preserve existing fixture assertions. Independent static
review found no authorization-set change or retry refund. Pull regression tests
pass 131/131 (`/tmp/talos-i300-borrowed-paths-final.log`); the additional focused
exhaustion test passes 1/1 (`/tmp/talos-i300-borrowed-budget-final.log`).
Source/required-path
extensions, worktree-target copies, identity serialization and handles remain
quota residuals; this does not close the full resource bound or enable Pull.

Borrowed-path full validation (2026-10-18): standard preflight initially stopped
at the real Push fixture's process observation (macOS PermissionDenied; 876 tools
tests passed). The unchanged standard command then exited 0 with explicit
sandbox escalation, including workspace tests and both external Runtime SDK
fixtures (`/tmp/talos-i300-borrowed-preflight-unsandboxed.log`). This validates
27af3fa1; it does not validate subsequent worktree quota changes. Fetch confirmed
no origin/main-only commits, so no rebase was necessary.

Linked-worktree quota checkpoint (2026-10-18): binding validates each existing
Resolution directly instead of building a temporary raw-to-normalized path map.
Authorization-result copies, raw target joins, common-target/reference-root
copies and reference frontier joins charge the same invocation ledger before
allocation. Failed partial binding retains only charged state and never refunds;
retries cannot invoke the grant-path callback after exhaustion. Independent
static review found no authorization widening or early descendant reads. Pull
regression tests pass 133/133 (`/tmp/talos-i300-worktree-budget-final.log`),
including the separate binding exhaustion/retry test. Reference-evidence path
copies, source/required-path extensions, identities and handle accounting remain
open; no full quota closure or production activation is claimed.

Reference-path quota checkpoint (2026-10-18): reference evidence now charges
its retained path copy before cloning through the existing invocation ledger.
Reference/boundary/worktree classification borrows relative path components
instead of repeatedly allocating joined prefix paths. Existing body-budget
fixtures now include retained path cost; a separate test proves that successful
body capture with no remaining path bytes refuses publication and cannot refund
on retry. Independent static review found the classification and authorization
sets unchanged. Pull tests pass 134/134
(`/tmp/talos-i300-reference-path-budget-final.log`); tools all-target/all-feature
Clippy passes with warnings denied (`/tmp/talos-i300-reference-path-clippy.log`).
Source/required-path extensions, serialized identities and peak identity handles
remain quota residuals. Full-stage validation and original T7 functional gates
remain outstanding; Pull remains disabled.

Remote candidate 27af3fa1 run 39139847519 completed: main
Format/Check/Clippy/Test passed (117711870239), confirming the fixture maintenance
race correction on that candidate. Windows job 117711870368 still fails Commit
at the maintenance identity guard: bin/git.exe differs from
ucrt64/libexec/git-core/git.exe. Log:
`/tmp/talos-i300-windows-117711870368.log`. This is the existing distinct-program
binding blocker, not Windows acceptance. Standard full preflight for
eeec0ced exited 0 (`/tmp/talos-i300-reference-stage-preflight.log`), ending
`release preflight: passed`, including locked workspace gates and both external
Runtime SDK fixtures. This validates the two local quota increments, not native
Windows acceptance or complete Pull execution.

Windows rebind implementation boundary (2026-10-19): independent static review
requires re-preparation immediately after first sender discovery, before any
Checkout-derived continuation. Commit/Checkout may use a separate discovery
stage; local Push must reuse the start of ReceiverRoots to retain eight stages.
The internal executable needs fresh exact Read/Execute and configuration-chain
admission. Re-discovery must preserve the invocation command budget and process
supervisor, start from original tracked inputs, and rebuild all sender evidence.
Before installation, compare ordered configuration/source contents, child
locations/configuration, target OIDs/modes/tracking writes, deletion/new-gitlink
classification, auxiliary sources, destinations, receiver roots and executable
paths as well as semantic permission facets. No receiver capture precedes this
comparison. Do not rerun discover on the populated plan or equate two programs
by installation/version. This is reviewed implementation guidance, not an
implemented rebind or a completed Windows gate. Fetch found no upstream-only
main commits at this checkpoint (193 local-only commits).

Windows sender-first binding checkpoint (2026-10-19): Commit/Checkout now query
only the entry Git's exec-path before first sender discovery, then require a new
prepared proof retaining original Read frontiers plus internal executable
Read/Execute and configuration-chain admission. After exact path binding, the
internal Git must report the identical exec-path candidate and canonical self;
no second migration is accepted. Only then does full sender discovery run for
the first time. This avoids dual populated discovery plans and preserves the
same command budget and supervisor. The original executable evidence remains
bound for drift checks. Independent static review found no security blocker.
Separate sender-program tests pass 2/2, including stale-proof refusal, missing
Read refusal and deliberately reduced budget retention. All-target/all-feature
Clippy passes; Windows Git-only library cross-check passes with the existing
feature-only warnings. Prepared Git regression passes 140 tests but its real
Push process-observation case fails with sandbox PermissionDenied; this is not
a full passing suite. The unchanged authorized standard external preflight
then exited 0 for implementation 1be6f4f9, including all workspace tests and
both Runtime SDK fixtures (`/tmp/talos-i300-sender-program-preflight.log`).
Both governance validators also pass. Native Windows evidence remains required.

This supersedes the dual-discovery comparison guidance above only for
Commit/Checkout: no old sender configuration or derived effects exist when the
program changes. Push is unchanged and remains subject to the earlier
ReceiverRoots re-preparation requirement. Windows always takes an extra admission
even for a same-path installation; complex Checkout eight-stage coverage,
distinct-path/entry-replacement regression cases and native Windows acceptance
remain outstanding. Production Windows preparation and Pull remain disabled.

Windows Push binding candidate (2026-10-19): local Push rebinds the internal
Git at the start of ReceiverRoots, before receiver capture; remote-only Push
uses a separate PushProgram admission. Both require fresh exact executable
Read/Execute and configuration-chain proof, retain the original executable
evidence, invocation budget and supervisor, and compare complete configuration,
ordered destinations, local roots and child executable paths before continuing.
The temporary configuration-chain grant is removed before mutation admission.
Separate regression tests pass 2/2, including changed-destination refusal and
exact candidate Execute facets without receiver reads. All-target/all-feature
Clippy and Windows Git-only library cross-check pass; independent incremental
review found no blocker. These are local candidate checks, not native Windows
or whole-T7 acceptance. Standard external preflight session 80821 exited 0 for the Push candidate
(before rebase), including workspace tests and both Runtime SDK fixtures;
output is at `/tmp/talos-i300-push-program-preflight.log`. Implementation
commit after rebase is `4f7a199a`. The branch now includes origin/main
`45cf9eb9`; the EVOLUTION append conflict retained both independent records.
Post-rebase validation remains required. Older CI run 39182330307 additionally
failed Runtime conditional Checkout with source drift; local preflight passing
does not explain or close that CI failure, which remains a reproducibility item. Native Push/eight-stage proof, Pull integration and the
previously recorded delivery residuals remain open. No completion is claimed.


Post-rebase diagnosis checkpoint (2026-10-19): Runtime conditional Checkout
passes in a focused locked local run (session 40805). This does not reproduce
or resolve the older Linux CI failure. Revalidation now distinguishes environment,
executable and repository-metadata drift while preserving short-circuit checks
and the same refusal conditions. The Windows CI required-test list now explicitly
selects all four sender/Push program-binding regressions and rejects missing test
names. The first post-rebase preflight stopped at formatting while diagnosis was
being edited; formatting is corrected. Standard external preflight session 11898
is running against this candidate, with output in
`/tmp/talos-i300-drift-diagnostics-preflight.log`; no pass is claimed yet.
Native Windows execution, real Windows Push and the original Pull/T7 residuals
remain required.

Pull typed integration work-in-progress (2026-10-19): replace the opaque
Command vector with Merge/Rebase variants retaining merge candidate OIDs,
fast-forward policy, autostash, rebase mode, onto and upstream. Unborn and
Reconcile remain distinct. Command rendering is test-only for native Git parity
fixtures; this does not authorize or enable production integration. Locked
pull_state tests pass 26/26 (session 1030), output
`/tmp/talos-i300-typed-integration.log`; prepared Pull tests pass 41/41
(session 53705), output `/tmp/talos-i300-typed-prepared-pull.log`.
The strengthened post-fetch candidate-OID assertion passes (session
15126), output `/tmp/talos-i300-typed-head-binding.log`. All-target/all-feature
Clippy passes with warnings denied (session 82353), output
`/tmp/talos-i300-typed-integration-clippy.log`. Standard preflight 11898 was already
running when this subsequent source work began; its eventual result must not
be cited as exact-head validation of the typed-plan change. Fresh stable-source
validation and effect discovery/admission/execution remain required.


Typed-plan stable candidate validation (2026-10-19): preceding standard
preflight session 11898 exited 0, including workspace tests and both Runtime
SDK fixtures, but source changed during that run and it is not final exact-head
proof. Windows Git-only library cross-check session 15246 exited 0 with the
existing seven feature-only warnings (`/tmp/talos-i300-typed-windows.log`).
Implementation `edca7e14` is frozen for standard external preflight session
98420 (`/tmp/talos-i300-typed-final-preflight.log`), which exited 0 with
`release preflight: passed`, including locked workspace checks, Clippy, tests
and both external Runtime SDK fixtures. Only this owner document changed during
that run. Both governance validators subsequently passed with zero warnings.
Fresh origin fetch succeeded; HEAD is ahead 199 / behind 0 origin/main, so no
additional rebase is needed. Existing
independent reviewer git_plan_review completed read-only static review of the
drift-diagnostic, Windows test-selection and typed-integration delta with no
blocker: short-circuit read order, candidate ordering, policy fields and native
parity command semantics are retained. That review did not run builds/tests or
validate native Windows or integration mutation authority. Native Windows
runtime, Pull effects/execution, recursive fetch residuals and final delivery
remain open; no Complete status or completion evidence is claimed.

Maintainer execution guidance (2026-10-19): when a compatibility, behavior or
scope choice would create a disproportionately long implementation chain, bring
the concrete alternatives, recommendation, behavior impact and remaining cost
to the maintainer promptly. This is design guidance, not runtime authorization:
it does not replace effect admission, repository identity validation or owned
process cleanup, and it does not silently narrow the published handoff scope.
Accepted decisions remain accepted; do not ask for them again. The next
functional delivery target remains Pull integration effect discovery, fresh
admission and supervised execution, rather than enabling the current refusal
branch without those effects.

Pull state test separation checkpoint (2026-10-19): the 24 previously inline
tests now live in six `git_write/pull_state_tests/` files (evidence, policy,
activity, merge, reconciliation and unborn HEAD). The existing two separate
evidence tests remain. The production file decreases from 2934 to 1323 lines;
each new test file is at most 388 lines. A source comparison verified all moved
test bodies/attributes modulo formatting and module qualification, and unchanged
production code modulo whitespace and test module declarations. Locked
all-feature Pull state tests pass 26/26 after restoring the Unix-only attributes
to their moved tests (`/tmp/talos-i300-pull-test-split-final.log`). All-target,
all-feature locked Clippy passes with warnings denied
(`/tmp/talos-i300-pull-test-split-clippy.log`). The initial extraction left two
leading Unix attributes behind; Clippy detected this, and the final equivalence
check includes the entire leading attribute set rather than starting at cfg(test).
This addresses the requested test layout, not Pull activation or T7 completion.
The maintainer accepted dedicated one-Pull-integration worktree scope
authorization on 2026-10-19. ADR-089 records the accepted boundary; compiler,
executor and independent security validation remain required before activation.
Production integration remains disabled.

Pull integration capability compiler checkpoint (2026-10-19): implemented the
closed `git:pull-integration-worktree` Read/Write facet for git_pull only, with
matching-nature absolute Path root and absolute Execute Command prerequisites.
The dedicated test file rejects other tools, missing roots/commands, relative
roots/commands, nature mismatches, arbitrary scope names and Execute capability
substitution. `cargo test --locked -p talos-permission` passes all 147 unit,
11 integration and one doc tests (session 66438); locked all-target Clippy with
`-D warnings` passes (session 2526). Generic Path compilation is unchanged.
This is compiler preparation only: runtime worktree identity/stage consumption,
external-effect discovery and supervised integration remain unfinished. No
production activation or completed security review is claimed.
Independent static security review by git_plan_review found no compiler-boundary
blocker; it confirmed the capability remains Remote text and generic Path
matching is unchanged. The reviewer did not rerun tests and explicitly requires
runtime worktree/Git identity and stage binding before activation.
Remote run 39185762993 on 0b42e580 is now terminal: Linux standard workspace
validation, Linux Desktop explicit feature, installer and issue reconciliation
passed. Windows workspace failed in Desktop auto-approval runtime testing before
its Git-specific steps; native Windows Git execution remains unverified. This
run predates the test separation and capability compiler changes.

Pull integration hook discovery work-in-progress (2026-10-19): integration
selection now publishes a separately admitted hook Read frontier with a fresh
prepared identity, after child scheduling completes. Candidate parsing uses
retained configuration only, respects external/relative core.hooksPath, and
does not open candidates. The new stage removes Write/Network facets, checks
the complete exact Read frontier before capture, and revalidates before and
after capture. Failures report retained fetch effects. Executable hooks remain
unsupported pending separate execution authorization; no hook is disabled.
Two separate-file regressions cover old-proof rejection, external candidate
discovery without opening it, missing Read denial before type inspection, and
nonregular-file refusal after admission. Independent incremental static review
found no blocker for this inert discovery stage, without rerunning tests.
Initial full Pull tests exposed three old fetch-specific diagnostic assertions;
the shared hook error now says Pull, and those assertions were updated.
Full integration effects (including filters/drivers/editors), admitted worktree
mutation, eight-stage success capacity and all original T7 gates remain open.
Production Pull remains disabled; this stage is not a completion claim.
Final-source validation for the hook frontier: locked all-feature Pull tests
pass 136/136 (session 96006, `/tmp/talos-i300-integration-hooks-final2.log`);
locked all-target/all-feature Clippy with warnings denied passes (session 37898,
`/tmp/talos-i300-integration-hooks-clippy-final.log`). Both governance validators
pass with zero warnings (sessions 62979/68782). The Windows rerun remains an
external, earlier-head validation and cannot validate these source changes.

Windows rerun evidence (2026-10-19): run 39185762993 attempt 2 on 0b42e580
is terminal and failed in Structured Git executable and path identity guard
tests. Desktop explicit feature validation passed on this attempt. All ten
selected foreground supervision tests passed, followed by real Git startup,
executable replacement protection and fresh sender Read admission tests.
`sender_program_binding_preserves_invocation_budget` failed during first sender
discovery with `Internal Git exec-path does not bind to its admitted executable`.
The current combined check does not distinguish textual path mismatch from
canonical executable mismatch; the cause is unproven. Diagnose both comparisons
on native Windows before changing identity policy. Later selected tests did not
run, and this earlier-head run cannot establish current-head Windows support.
Failed-step evidence is retained at
`/tmp/talos-i300-ci-39185762993-attempt2-failed.log`. T7 remains Active, with
production Windows preparation and Pull still disabled.

Binding diagnostic follow-up: the two sender binding checks now share a helper
that distinguishes spelling drift, resolution failure and canonical-target drift.
It preserves the original short circuit: a differently reported path is rejected
before canonicalization, so diagnostics do not open an unadmitted replacement.
The new regression exercises all three failures and an unchanged binding; it
passes with locked all-feature tools tests, as do the two sender-program tests.
All-target/all-feature locked Clippy with warnings denied passes. Independent
static security review found no blocker or enlarged authority; it did not run
tests. Native Windows diagnosis remains required; no Windows fix is claimed.

Stable candidate validation (2026-10-19): rebased onto origin/main 2b015a6b
without conflicts; Git source is unchanged by that rebase. Full standard
`./scripts/release_preflight.sh` on frozen 0cdfb928 passes outside the sandbox
(session 15707, `/tmp/talos-i300-binding-full-preflight-approved.log`, exit 0,
ending `release preflight: passed`). This includes locked workspace check,
Clippy, tests/doctests, governance/claims and both external Runtime SDK modes.
The initial sandbox run failed only at process observation EPERM in the local
eight-stage Push test; that test passes in the approved full rerun. Independent
static authorization review of the Git/permission delta from 0b42e580 to
0cdfb928 found no code blocker, without running tests. Current-head remote CI,
native Windows diagnosis, actual Pull integration execution and the original
T7/handoff completion gates remain outstanding. No production Pull activation
or completion is claimed.

Follow-up (2026-10-19, c541320a / be5e98ff): integration hook path computation
now happens before returning the next approval frontier, saving one stage while
retaining fresh exact hook Read admission. Pull tests pass 137/137, including
the actual child-completion dispatch and stale-proof rejection; this is not
complete eight-stage integration evidence. Windows CI run 39189835232 at
f9c22a76 failed sender binding because Git reported `//?/C:/...` after an
initial `C:/...` candidate. The fix compares only the retained canonical
spelling with slash conversion and re-resolves only the admitted path; no
prefix, case or dot-component alias expansion is introduced. Six focused
program tests and all-target/all-feature locked Clippy pass locally. Independent
static authorization review found no blocker in either change. Windows runtime
confirmation and full preflight on be5e98ff remain pending; production Pull
and Windows preparation remain disabled. Log evidence:
`/tmp/talos-i300-fold-hook-frontier.log`,
`/tmp/talos-i300-bound-spelling-tests.log`,
`/tmp/talos-i300-ci-39189835232-windows.log`.

Validation follow-up: run 39189835232 is terminal; all jobs except Windows pass.
The first local full preflight stopped at four CLI registration header assertions,
also reproduced outside the sandbox. Bounded fixture maintenance 6923d481 fixes
HTTP name-case comparison without changing product behavior or credential values;
all eight registration tests pass. Standard full preflight passed outside
the sandbox at frozen source 6923d481 (session 53843, exit 0,
`/tmp/talos-i300-canonical-full-preflight-approved.log`, ending
`release preflight: passed`). Locked workspace check, Clippy, tests/doctests,
governance/claims and both external Runtime SDK modes passed. Subsequent fetch
found origin/main ae6a0791 (#737); rebase and validation of the resulting
candidate remain required. Native Windows confirmation, actual Pull integration
execution and the original T7/handoff delivery gates remain outstanding.

Rebase validation checkpoint (2026-10-19): rebased without conflicts onto
origin/main ae6a0791 (#737), producing frozen candidate 34115871. The tree delta
from the pre-rebase candidate is exactly the six incoming search files; the Git
implementation is unchanged. Standard full preflight passed outside the sandbox
(session 13599, exit 0, `/tmp/talos-i300-rebased-34115871-preflight.log`, ending
`release preflight: passed`), including locked workspace checks, Clippy,
tests/doctests, governance/claims and both external Runtime SDK modes. PR #735
describes the bounded CLI fixture maintenance exception and remaining delivery
gaps. Fresh exact-head remote CI, native Windows repair confirmation and final
authorization review remain required; T7 stays Active.

Standing gix tracking recheck (2026-10-19): the GitHub releases API's latest
100 releases still identifies `gix-v0.89.0` (2026-10-08) as the newest main
crate release when filtered with `^gix-v[0-9]`; `gix-validate` and `gix-utils`
are separate crates. Cargo.lock remains on 0.87.1. The release notes add
linked-worktree capabilities and change public error APIs; they do not establish
complete replacement of Talos's structured Git operations. This confirms the
existing source-audit baseline rather than a new capability assessment. No
dependency upgrade or host-Git removal date is claimed. crates.io returned HTTP
403, so this check uses GitHub release evidence only:
https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.89.0 .

Pull module extraction checkpoint (2026-10-19, 210b7d88): moved existing
prepared Pull discovery, child-fetch and integration-hook methods from the
large dispatcher into three private modules under `prepared_add/`. Method-body
comparison after namespace, parent-only visibility and formatting normalization
found no behavioral changes. Locked all-feature Pull tests pass 137/137
(`/tmp/talos-i300-pull-module-tests.log`), and locked all-target/all-feature
Clippy passes with warnings denied (`/tmp/talos-i300-pull-module-clippy.log`).
This reduces the dispatcher by approximately 2350 lines without adding a
parallel implementation or changing authorization. The extraction remains local;
published f92b64a8 CI does not validate it. Integration scope consumption,
external-effect admission and actual integration execution remain unfinished.
Independent static review of the complete four-file extraction found no new
blocker: all 27 moved methods preserve behavior and the remaining parent file
is unchanged apart from private module declarations. The reviewer did not run
tests. Published Windows job 117882260178 is terminal failed: sender discovery
now passes executable spelling validation but fails later at configuration
inspection with values withheld. The underlying configuration failure remains
unproven; do not claim native Windows completion.

Current-head validation checkpoint (2026-10-19): implementation `64cc4674`
adds bounded configuration-failure diagnostics (exit status and output byte counts,
never configuration contents) and a regression test with a secret-bearing invalid
configuration fixture. Targeted diagnostic and sender-binding tests and all-target,
all-feature Clippy passed; independent static review reported no blocker. This is
failure classification, not a fix for the native Windows configuration failure.
Remote run 39197138949 at `f92b64a827109a44d21c843a7bfdded3f18f5043`
is now terminal: Linux Format + Check + Clippy + Test, Linux Desktop, installer,
and remote owner reconciliation passed; Windows Rust workspace failed. Current
source standard preflight session 89793 remains live, with check and Clippy passed
and test compilation in progress; log:
`/tmp/talos-i300-modules-diagnostic-preflight.log`. Poll that session before starting
another full validation. No current-head full pass or Windows completion is claimed.
The accepted Pull integration worktree scope still lacks its runtime consumer and
actual integration dispatcher; production Pull remains disabled. All original T7
acceptance and the subsequent handoff queue remain required.

Validation correction (2026-10-19): session 89793 terminated with exit 101;
tools tests passed 930/931, with the sealed-owner regression still expecting the
old configuration-error wording. Actual execution was refused with the new
redacted runner-failure diagnostic. Update both stale string assertions while
retaining the rejection, absent-index and cleanup-receipt assertions, then rerun
the focused test and standard full gate. This supersedes the live-session note
above; no passing full gate is claimed for `64cc4674`.
Focused sealed-owner test now passes 1/1 with all original effect/cleanup checks
retained (`/tmp/talos-i300-sealed-owner-diagnostic-test.log`); formatting passes.

Stable-batch full gate (2026-10-19): standard preflight session 21926 exited 0
for source `35693ea1`, ending `release preflight: passed`. Locked workspace checks,
Clippy, tests/doctests, governance/claim gates and both external Runtime SDK modes
passed. Log: `/tmp/talos-i300-sealed-diagnostic-full-preflight.log`. This validates
the private Pull module extraction, redacted config diagnostic and corrected
sealed-owner assertions locally; native Windows remains unproven. A fresh
`git fetch origin` succeeded; this source is ahead 217 / behind 0 origin/main,
so no rebase was needed. Publish the reviewed stable batch to #735 for fresh CI;
production Pull integration and all remaining handoff gates remain open.

Pull integration scope consumer checkpoint (2026-10-19): after separately admitted
hook inspection, the prepared continuation now publishes a fresh integration
scope stage. Read/Write worktree authority and metadata authority each require
their dedicated Remote capability and exact identity-bound root Path facets.
The displayed operation contains selected OIDs and a SHA-256 digest of retained
FETCH_HEAD bytes, never its potentially credential-bearing remote descriptions.
The stage consumes its own immutable prepared proof before checking both natures
and all workdir/gitdir/common roots; retained evidence drift is refused. It still
terminates without mutation until external-effect admission and supervised
integration execution are implemented. This is infrastructure preparation, not
new user-facing Pull behavior or production activation.

Independent static review found no new security blocker, and explicitly retained
the eight-stage successful-flow residual. Pull tests passed 139/139 before the
additional continuation test; all-target/all-feature Clippy passed. The final
focused scope suite passes 3/3, including real admitted hook-to-scope continuation
and terminal no-mutation assertions, stale proof/evidence, missing capability,
and hidden FETCH_HEAD descriptions. Logs: `/tmp/talos-i300-scope-pull-tests.log`,
`/tmp/talos-i300-scope-clippy.log`, `/tmp/talos-i300-integration-scope-tests.log`.
These checks do not prove the full Pull stage budget, external effects, execution,
or final runtime acceptance. Native Windows run 39199780959 at published
`c1ba37d669550eca0b7ebc099507a08cdd7369b0` remains live; poll that run rather than
restart it. No T7 completion or current-scope full workspace pass is claimed.

Pull integration command convergence (2026-10-19): the retained integration now
generates the command vectors used by both the approval description and native
parity tests. Merge/rebase keep their existing argv; unborn checkout generates
read-tree followed by compare-old update-ref, and reconciliation generates
read-tree before a separately selected/admitted next operation. Invalid, empty,
option-looking and mixed-format object identities are refused. SHA-1 and SHA-256
unborn fixtures cover clean, disjoint staged and conflicting untracked files.
Hashing and command preparation have panic boundaries. No integration process
execution is enabled by this change.

Current-source locked Pull tests pass 141/141 and all-target/all-feature Clippy
passes with warnings denied. Logs:
`/tmp/talos-i300-command-final-pull-tests.log` and
`/tmp/talos-i300-command-final-clippy.log`. Independent review of the command
generator identified missing hash panic boundaries; those are now present in
both command generation and the FETCH_HEAD approval digest. Full stable-batch
preflight and exact-head remote gates remain required before publication/merge.

The previously live Windows run 39199780959 is terminal failed at job
117890194603: sender configuration inspection exits 128 after 200 stdout bytes
and 39 stderr bytes. The diagnostic withholds all configuration values and
paths. This proves Git started and failed during config inspection, not its
underlying cause. Native Windows completion and actual Pull execution remain
open; production Pull stays disabled and T7 remains Active.

Windows configuration isolation follow-up (2026-10-19): a separate test now
compares entry/internal Git across default/system/global/local configuration
sources with the same frozen environment, cwd and owned runner, after candidate
Read and configuration-chain admission. It collects all eight results before
asserting default discovery success; only fixed labels, exit codes and stream
lengths are exposed. Windows CI selects it before the existing failing binding
test. The focused locked all-feature test passes locally (1/1,
`/tmp/talos-i300-sender-config-matrix-test.log`); independent static review found
no blocker. This is diagnosis, not a Windows repair or platform completion.

Standard preflight session 43993 failed: tools 934/935, with the real local-push
test denied process observation by the sandbox. Authorized external preflight
session 44449 is running at
`/tmp/talos-i300-command-stage-preflight-unsandboxed.log`; poll it rather than
restart. The sandbox failure is not a passing workspace gate. Fetch succeeded,
with current implementation ahead 220 / behind 0 origin/main; no rebase needed.

Stable-batch validation superseding the running checkpoint (2026-10-19):
authorized external session 44449 exited 0 with `release preflight: passed`.
The standard pinned-toolchain/locked workspace checks, Clippy, tests, doctests,
governance checks and external SDK fixtures passed; tools tests pass 936/936.
The log explicitly includes the new sender configuration isolation test from
`95baec5f`, as well as the retained command and fresh-scope changes from
`30a74c2a` and `ef37e810`. Evidence:
`/tmp/talos-i300-command-stage-preflight-unsandboxed.log`.
This clears the local standard gate for this batch only. Native Windows
diagnosis, exact-head remote gates, actual supervised Pull integration and
the original T7/handoff acceptance remain open; T7 stays Active.

Pull signing-effect checkpoint (2026-10-19): integration hook discovery now
refuses configured commit signing for merge/rebase and signature verification
for merge before publishing hook paths. Git 2.54 merge/rebase configuration
callbacks consume these booleans independently of hooks; worktree scope does
not grant signer/verifier execution. Every declaration is parsed, including
invalid early values later overridden by false. Final false values proceed;
read-tree/update-ref-only unborn/reconciliation operations are unaffected.
Tests live in a separate `config_state_tests/pull_signing.rs` file and cover
implicit/numeric booleans, overrides and secret-bearing invalid values.
Independent static review found no new blocker. This is a conservative external
execution boundary (even a fast-forward may be refused when signing is enabled),
not full native compatibility or a complete effect validator. Branch merge
options, attributes/drivers/filters, editors, strategies, rerere and actual
integration execution remain open. Production Pull remains disabled.

Locked all-feature configuration tests pass 29/29, Pull-focused tests pass
142/142, and all-target/all-feature Clippy passes with warnings denied. Logs:
`/tmp/talos-i300-pull-signing-config-tests.log`,
`/tmp/talos-i300-pull-signing-pull-tests.log`,
`/tmp/talos-i300-pull-signing-clippy.log`. These targeted checks do not extend
the preceding full-workspace gate to this new implementation. Published
`44b03138` remains under exact-head CI run 39201923094; do not restart that
live run or treat its earlier-head result as approval of this signing change.

Pull integration approval simplification (2026-10-19): hook candidate paths and
the retained integration worktree/metadata scope now share one fresh plan.
Path computation opens no hook contents. The consumer first consumes the new
proof and validates the scope, then checks every exact hook Read token before
capturing any hook. The obsolete PullIntegrationHooks state and its extra
approval round are removed. Regressions cover stale proof, external candidate
non-reading, missing exact hook Read despite granted scope, and completion of
this local frontier in one admitted stage. Full Pull-focused tests pass 143/143
and all-target/all-feature Clippy passes with warnings denied:
`/tmp/talos-i300-coalesced-pull-tests.log`,
`/tmp/talos-i300-coalesced-clippy.log`. Independent security review found no new
blocker. This removes one local round, not proof of a complete eight-stage Pull;
actual integration still stops at the existing no-mutation guard. Remaining
external effects and the original T7 delivery acceptance are unchanged.

Windows matrix result superseding the live-job checkpoint: job 117896536817
in run 39201923094 failed the new matrix. With the same frozen environment/cwd,
entry/default, entry/system and entry/local exit 0; internal/default exits 128
(200 stdout/39 stderr bytes), internal/system exits 128 (0/102), and
internal/local exits 128 (0/68). Both explicit global probes exit 128 (0/95).
Evidence: `/tmp/talos-i300-windows-matrix.log`, fetched from the completed job
API while other workflow jobs were still running. No stream contents or paths
are published. This narrows investigation to internal executable rebinding or
startup differences; it does not establish the underlying cause. Do not loosen
identity checks or mark Windows complete on this evidence.

Windows invocation-spelling follow-up: the matrix now also probes the retained
internal executable's already-admitted original spelling, revalidating the
binding before each executable group. All twelve probes retain the same frozen
environment, cwd, runner and redacted diagnostics. This tests the canonical
verbatim-prefix/startup hypothesis without changing production execution or
reading any newly reported executable. The focused locked all-feature test
passes locally (1/1, `/tmp/talos-i300-windows-spelling-matrix-test.log`), and
independent security review found no blocker. The underlying Windows failure
is still unknown. Authorized full standard preflight session 49774 is running
at `/tmp/talos-i300-coalesced-full-preflight.log`; poll that handle rather than
restart. Earlier run 39201923094 is terminal failure, not a current-batch gate.

Current-batch standard gate (2026-10-19): session 49774 exited 0 and
`/tmp/talos-i300-coalesced-full-preflight.log` ends with
`release preflight: passed`. The pinned-toolchain locked workspace check,
Clippy, tests (including tools 938/938), doctests and external SDK fixture
passed for implementation through `f4eed9eb`. This supersedes the running
checkpoint above. Fetch confirmed no origin/main-only commits. The three
implementation increments have independent review evidence recorded above;
publish this stable batch for fresh Windows invocation-spelling evidence.
Windows runtime, actual supervised Pull integration and all original handoff
acceptance remain open; T7/I300 stays Active.

Interactive integration execution boundary (2026-10-19): hook discovery now
refuses retained interactive rebase before publishing the combined integration
scope. That worktree scope does not grant sequence-editor execution. The new
split prepared-plan regression checks the discovery refusal and unchanged HEAD;
noninteractive operation matching remains unchanged. Locked all-feature
Pull-focused tests pass 144/144, all-target/all-feature Clippy passes with
warnings denied, and formatting passes. Logs:
`/tmp/talos-i300-editor-pull-tests.log`,
`/tmp/talos-i300-editor-clippy.log`. Independent static authorization review
found no new blocker. This is not complete editor/effect coverage or actual
integration execution; those residuals remain owned here. The preceding full
workspace gate does not cover this later change. Published head `4f8610cb` is
under CI run 39205382544, confirmed running; do not restart it for this local
increment.

Pull test layout checkpoint (2026-10-19): split the 2,377-line prepared Pull
test file into fetch command behavior, prefetch configuration, fetch admission,
post-fetch selection, graph/source admission and rebase tests. The 31 existing
test blocks and platform attributes are preserved; same-module includes retain
their helpers and test names. Only separator blank lines at file ends changed.
The original file is now 374 lines and each extracted file is at most 542 lines.
Locked all-feature Pull tests remain 144/144 passing
(`/tmp/talos-i300-split-pull-tests.log`); formatting and whitespace checks pass.
This addresses test maintainability, not production Pull activation or T7
completion. Integration/external-effect and native Windows residuals remain.

Windows invocation binding checkpoint (2026-10-19): completed Windows job
117906997821 (run 39205382544) shows default configuration inspection succeeds
through the entry and admitted internal spellings, but fails through the
canonical internal spelling. Preserve the admitted spelling for Windows
invocation while retaining the canonical executable evidence and original path
identity for revalidation, including Push rebinding. The canonical diagnostic
probe remains visible but is not a production startup requirement. Independent
review identified two canonical-only evidence captures; both now use the
retained canonical target and the reviewer confirmed that blocker is resolved.
Locked all-feature program-binding tests pass 6/6 and the configuration matrix
passes 1/1; all-target/all-feature Clippy with warnings denied and whitespace
checks pass. Logs: `/tmp/talos-i300-spelling-final-tests.log`,
`/tmp/talos-i300-spelling-final-configuration.log`, and
`/tmp/talos-i300-spelling-final-clippy.log`. Native Windows confirmation and a
fresh full workspace gate remain required. This does not activate production
Pull or close T7; actual integration execution and the full handoff remain open.

Full-gate follow-up (2026-10-19, implementation `56784d58`): standard preflight
session 57232 exited 101. Tools passed 938/939; the remaining test
`prepared_local_push_runs_all_eight_admitted_stages` failed because its process
observation returned sandbox `Operation not permitted`, not a passing cleanup
assertion. Log: `/tmp/talos-i300-spelling-full-preflight.log`. The unchanged
standard command was rerun with approved external execution as session 35403;
its successful result is recorded below. Read-only integration design review
confirms existing scope,
command generation, evidence and owned runner are reusable, but the complete
external-effect admission and configuration-reload boundary must precede guard
removal. Merge/Rebase/Unborn/Reconcile execution, conflict/cancel/partial effects,
successful stage-budget evidence, native Windows and all original handoff
deliverables remain required.

Superseding full-gate result (2026-10-19): approved external session 35403
exited 0 and `/tmp/talos-i300-spelling-full-preflight-unsandboxed.log` ends
`release preflight: passed`. This covers implementation `56784d58`, including
the preceding interactive-editor boundary and Pull test split: pinned locked
workspace check, Clippy, tests/doctests, tools 939/939, both governance validators
and both external Runtime SDK modes. The failed sandbox observation above is
retained as historical evidence, not a source failure or passing gate. Publish
this stable stage for native Windows verification; final exact-head CI/review,
actual integration execution and all original handoff acceptance remain open.

Configuration test split (2026-10-19): moved the 28 inline configuration tests
into six same-module files for child layout/configuration, checkout, receiver
and Push, parsing, and Pull. Each new file is 120–257 lines; the production
file is now 1,964 lines. Exact comparison confirms all test blocks and platform
attributes are preserved after removing their module indentation, and the
production prefix is byte-identical. The existing signing include remains in
the same module. Locked all-feature configuration tests pass 29/29
(`/tmp/talos-i300-config-split-tests.log`), all-target/all-feature Clippy with
warnings denied passes (`/tmp/talos-i300-config-split-clippy.log`), and format
and whitespace checks pass. This is test maintenance only; it neither closes
integration execution nor changes I300/T7 status. Published candidate `2f6a1a5b`
is under native CI run 39207944752; its evidence does not include this later
test-only split.

Pull branch merge-option evidence (2026-10-19): integration discovery now checks
only the retained current branch's effective last `mergeOptions` declaration.
A small Git-style byte tokenizer preserves quoting, literal comments/assignments,
backslash-newline and empty arguments; flags with unmodeled external effects,
value arguments or abbreviations refuse without exposing configuration values.
The admitted flag subset does not replace complete hook/editor/driver validation.
The first native comparison exposed an incorrect VT/FF whitespace assumption;
Git 2.54 `ctype.c` confirms only space/tab/LF/CR. The corrected implementation
passes native alias/direct-argv differential cases. Locked all-feature config
checks pass 31/31, integration checks 8/8 and all-target/all-feature Clippy passes
with warnings denied (`/tmp/talos-i300-merge-options-{tests,integration-tests,clippy}.log`).
Independent static review found no new blocker in this guarded increment;
production integration remains disabled and full effect execution is still open.

Native Windows follow-up: job 117914809238 in run 39207944752 is terminal failed.
It passed the earlier executable binding checks and real commit fixture, then
`checkout_nonrecursive_gitlink_preserves_dirty_child_worktree` failed because
Git rejected a verbatim `\\?\C:\...\child\.git` argument. Full job log is
`/tmp/talos-i300-windows-binding-stage.log`. Child explicit-root invocation spelling
needs correction while preserving canonical identity evidence; no Windows pass
or T7 completion is claimed. Linux main job was still running at this observation.

Windows explicit-directory argument correction (2026-10-19): all five explicit
`--git-dir`/`--work-tree` arguments now use a Git-compatible conventional Windows
spelling after canonical target and retained ancestor identity validation.
Canonical permission/evidence paths remain unchanged; Unix borrows its original
argument. Verbatim-only trailing-dot/space names, colon/ADS and DOS device
components refuse before conversion; unsupported names do not broaden authority.
The child filter command now propagates conversion failure. Two native Windows
regressions (Unicode/spaces, rejected verbatim-only names, real explicit-root Git)
are required by CI. Incremental independent static security review found no new
blocker; it did not execute tests or establish filesystem confinement.

Local verification: locked all-feature dirty-child checkout and effective-root
filter regressions pass, all-target/all-feature Clippy with warnings denied
passes, format/whitespace checks pass, and Git-only Windows GNU library check
passes with seven existing feature-only warnings. Logs are
`/tmp/talos-i300-directory-{checkout,filter,clippy}.log` and
`/tmp/talos-i300-directory-argument-windows-lib.log`. Full-feature cross-check
cannot complete because this host lacks `x86_64-w64-mingw32-gcc`; native Windows
runtime and configuration-origin compatibility remain unproven. Standard full
preflight is running in session 31795 with log
`/tmp/talos-i300-directory-full-preflight.log`; no success is claimed yet.
Previous run 39207944752 is now terminal: Linux main and Desktop passed, Windows
workspace failed as recorded above. Fetch completed; HEAD has no commits behind
origin/main. Actual Pull integration execution, native acceptance, final review
and the original T7/handoff deliverables remain open.

Directory-stage gate follow-up: implementation `ed3d9e36` is committed. Both
standalone governance validators pass with zero warnings. Standard session 31795
exited 101: tools 940/941 passed; `prepared_local_push_runs_all_eight_admitted_stages`
could not observe processes because the sandbox returned `Operation not permitted`.
This is not passing cleanup evidence. Approved external execution of the unchanged
preflight is running as session 20275, log
`/tmp/talos-i300-directory-full-preflight-unsandboxed.log`; its result is pending.

Superseding directory-stage full gate: approved external session 20275 exited 0;
`/tmp/talos-i300-directory-full-preflight-unsandboxed.log` ends
`release preflight: passed`. Implementation `ed3d9e36` passes pinned locked
workspace check, Clippy, tests/doctests (tools 941/941), both governance validators
and both external Runtime SDK fixture modes. The earlier sandbox failure remains
recorded separately. Publish this converged stage to #735 for native Windows
verification; no final integration execution or T7/handoff completion is claimed.

Pull sequencer-format effect checkpoint (2026-10-19): isolated native Git 2.54
reproduction configured `rebase.instructionFormat` with `%s%nexec` and observed
its helper create a marker during noninteractive `rebase --rebase-merges`.
Git v2.54.0 `sequencer.c` around line 6151 reads that format into todo generation;
worktree scope does not authorize the resulting commands. Integration discovery
now refuses nonempty custom instruction formats for Rebase before publishing the
mutation plan. Empty formats retain Git's default; Merge does not consume this
setting. Refusal does not print configured values. This conservative unsupported
configuration still needs a separately effect-authorized path; no silent config
override or general executable-todo approval is introduced.

Locked all-feature focused regressions pass 2/2 for On/Merges refusal, unchanged
HEAD/no helper marker, redaction, dormant Merge and default empty format; all-target
all-feature Clippy with warnings denied and formatting/whitespace checks pass.
Logs: `/tmp/talos-i300-instruction-format-final-tests.log` and
`/tmp/talos-i300-instruction-format-clippy.log`. Production integration remains
disabled until the remaining external effects and owned execution are complete.
Published stage `9d5fd3bb` has CI 39211752607 running; native Windows job
117926517309 is live and its result is not claimed by these local checks.
Incremental independent static security review found no new blocker. The existing
integration-focused suite also passes 8/8; both governance validators pass with
zero warnings. No fresh full workspace gate is claimed for this later increment.

Windows auxiliary traversal follow-up (2026-10-19): native Windows job
117926517309 terminated with a failure in the nonrecursive dirty-child checkout
fixture: explicit-directory argument tests now pass, but auxiliary path capture
rejects an unavailable path. Component traversal inspected an incomplete Windows
prefix before its RootDir. Skip only that incomplete prefix; retain inspection
of the complete root, every subsequent component and every symlink hop. Add a
Windows existing/missing-path regression and require its exact name in CI.
The same dirty-child checkout regression passes locally (1/1); this Unix result
does not prove the Windows fix. Native confirmation remains pending. Prior
follow-up all-feature Clippy and Windows GNU library cross-check exited zero;
logs: `/tmp/talos-i300-directory-followup-clippy.log` and
`/tmp/talos-i300-directory-followup-wincheck.log`. T7 remains incomplete.
Independent static security review found no blocker: full-root inspection,
symlink hops and pre-`..` ancestors remain retained. Review did not execute Windows
and the new fixture does not directly cover UNC or symlink paths.
Both project governance and Collaboration Claim validators pass with zero
warnings for this follow-up.

Recursive Pull directory argument checkpoint (2026-10-19): implementation
`81232433` routes the two remaining explicit child Git directory/worktree
command constructors through the existing identity-preserving host-path adapter.
All three production callers propagate conversion errors before process startup;
cwd, historical `--work-tree=.` scheduling, environment and authorization remain
unchanged. Focused locked all-feature child regressions pass 22/22
(`/tmp/talos-i300-child-directory-tests.log`); formatting and whitespace checks
pass. Independent static security review found no blocker or missing caller;
native Windows validation remains required.

The standard sandboxed full gate for `2cecc982` exited 101: tools 942/943 passed,
and the sole failure was process observation denied by the sandbox in
`prepared_local_push_runs_all_eight_admitted_stages`. Log:
`/tmp/talos-i300-root-followup-preflight.log`. Approved external full gate
session 72915 now validates the source including `81232433`; log
`/tmp/talos-i300-child-directory-preflight-unsandboxed.log`. Its result remains
pending. No full-gate success, Windows completion or T7 completion is claimed.

Superseding full-gate result: approved external session 72915 exited zero and
its log ends `release preflight: passed`. Pinned locked workspace checks, Clippy,
tests/doctests (tools 943/943) and both external Runtime SDK fixture modes pass
for implementation `81232433`. Native Windows exact-head CI remains outstanding.

Pull dynamic-source checkpoint (2026-10-19): after integration scope and exact
hook Read admission, reject effective configuration, stable auxiliary or hook
paths whose retained spellings traverse the mutable worktree outside strictly
spelled bound metadata roots. This avoids reloading a source that this operation
may rewrite. Existing auxiliary symlink-hop/ancestor inspection is reused; this
is no proof of arbitrary external-source immutability or full effect validation.
An independent review caught misuse of conservative potential-path matching for
metadata exemptions; replaced it with strict component-prefix matching and added
a `.GIT` regression. Static re-review found the blocker resolved. Split regression
also covers default metadata hooks, missing worktree hooks, external sources and
Unix external aliases into the worktree. Focused integration tests pass 9/9
(`/tmp/talos-i300-integration-sources-tests.log`). Execution remains disabled.

Published head `eb7bfc8e` CI 39220084842: native Windows job 117953134257
failed before Git regressions in Desktop Auto approval flow: decision/report and
ToolStarted arrived, but no ToolResult before watchdog timeout (105/106 passed).
Log `/tmp/talos-i300-root-windows-ci.log`. This does not confirm or falsify the
Git path fixes; native confirmation remains pending and the timeout needs
investigation. Linux full job was still running at last observation.
Dynamic-source follow-up all-target all-feature Clippy with warnings denied,
formatting, whitespace and both governance validators pass (zero warnings).
No fresh full-workspace gate is claimed for this later increment.

Windows evidence isolation checkpoint (2026-10-19): Desktop Auto's successful
assessment reaches ToolStarted but its outer 15-second watchdog expires before
PowerShell returns (tool default timeout is 120 seconds). Source inspection
confirms this path uses WindowsBackgroundLauncher, not the structured Git
foreground executor. Root cause remains unproven; no timeout relaxation or
passing claim is made. Make the independent Windows Git guard step run unless
the workflow is cancelled, even after Desktop failure. Earlier failures continue
to fail the job; this change only prevents unrelated failure from hiding Git
regression evidence. CI 39220084842 remains the prior-head evidence.

Resumed validation evidence: `/tmp/talos-i300-sources-full-preflight.log` ends
`release preflight: passed`, with tools 944/944 and both external Runtime SDK
fixture modes passing for implementation `b9075e47`. The interrupted execution
cell is no longer available, so its shell exit status was not recovered; the
standard script completion marker is the retained evidence. Both governance
validators pass with zero warnings. Prior-head CI 39220084842 is now terminal
failure; this does not validate the current increment.

Native integration-effect and Windows fixture checkpoint (2026-10-19):
`bc7fd24a` corrects `checkout_ref_movement_after_discovery_invalidates_plan`
to admit each actual discovery frontier, with the normal stage-history limit,
until a Write plan exists before moving the target reference. Windows job
117970282428 in run 39225369123 passed Desktop validation but failed this test
with "Git mutation requires a completed discovery phase"; the old fixture
assumed one continuation was sufficient. Original rejection and unchanged
HEAD/branch/file assertions remain. Local locked all-feature focused test passes;
this is not native Windows confirmation. Job log:
`/tmp/talos-i300-windows-39225369123.log`. The overall run remains live at this
checkpoint; do not restart it merely because its Windows job is terminal.

`6587407c` retains a native Git merge-editor experiment in the split merge tests.
On Git 2.54.0 with null stdin, an unset GIT_MERGE_AUTOEDIT did not execute the
configured GIT_EDITOR; `yes` executed it and `no` did not. Locked all-feature
regression passes, as does all-target/all-feature Clippy with warnings denied.
Closed stdin therefore cannot substitute for external-execution effect admission.
The ordinary no-editor case must not be rejected merely because an editor is
configured. This is behavioral evidence, not an editor authorization implementation.
Fetch succeeded: HEAD before these two commits was ahead 240 / behind 0
origin/main. Standard preflight is running in session 51544, log
`/tmp/talos-i300-discovery-editor-preflight.log`; inspect that handle before any
rerun. Pull final execution remains guarded; T7 and the complete handoff remain
Active/incomplete, including the original later task queue.
Source follow-up: `prepared_add::validate_environment` already refuses inherited
GIT_* overrides except GIT_PAGER, including both GIT_MERGE_AUTOEDIT and GIT_EDITOR.
The experiment's enabled environment therefore is not currently admitted in
production. Existing branch merge-option validation also handles editor options.
Do not add a redundant environment guard or silently force auto-edit off; retain
these existing checks when assembling final execution. Both governance validators
pass with zero warnings after this checkpoint.

Gate follow-up: session 51544 exited 101, tools 944/945 passed. Sole failure
was the existing eight-stage Push test's process observation returning sandbox
PermissionDenied/Operation not permitted. Approved external standard preflight
session 36855 is live, log
`/tmp/talos-i300-discovery-editor-preflight-unsandboxed.log`; inspect it before
rerunning. CI 39225369123 is now terminal failure: Linux workspace and Linux
Desktop passed, Windows failed the old checkout discovery fixture above.
The local correction has not yet received native Windows CI. Both governance
validators passed with zero warnings. No full-gate success is claimed yet.

Superseding standard gate result (2026-10-19): approved external session 36855
exited zero and `/tmp/talos-i300-discovery-editor-preflight-unsandboxed.log`
ends `release preflight: passed`. Locked pinned workspace checks, Clippy,
tests/doctests (tools 945/945) and both external Runtime SDK modes pass for
implementation `bc7fd24a` / `6587407c`. Publish this stable stage for fresh
Windows evidence; the previous-head failure is not a result for these fixes.
Pull integration execution and all original handoff residuals remain open.

2026-10-19 Pull integration runner checkpoint: the final admitted stage now
uses retained Integration commands through the existing owned process runner
(120-second command deadline, 1 MiB output reservation, shared invocation budget).
Commands stop on the first failure, preserve conflict/partial state, and recheck
repository root identity before each command and after success. Reconcile still
refuses until its fresh-evidence continuation exists. Production activation is
still explicitly disabled before any integration command; this is implemented
execution under test, not a released Pull capability. Focused locked all-feature
integration tests pass 11/11, including real fast-forward HEAD/worktree updates
and real merge conflict retention (HEAD unchanged, MERGE_HEAD and unmerged index
preserved). Independent static review found no new blocker for this test wiring;
external effects, other modes, cancellation and activation evidence remain open.
Standard external preflight session 10382 is running, log
`/tmp/talos-i300-integration-runner-preflight.log`; inspect before rerunning.
CI 39228852503 at the previous pushed head has Linux workspace failure in
`checkout_target_hook_missing_evidence_refuses_later_creation`: metadata changed
during discovery. Root cause is not established. Windows workspace remains live;
do not restart the run merely because another job failed. T7 and the original
handoff queue remain incomplete.

2026-10-19 superseding integration validation and reconciliation binding:
The standard external preflight log
`/tmp/talos-i300-integration-runner-preflight.log` ends
`release preflight: passed` for implementation `e438f6b4` / `654770d0`.
This supersedes the running-session entry above. Subsequent reconciliation
selection now binds the exact reconciled HEAD, including rejection if HEAD
moves back to the original commit. The native reconciliation regression passed;
all-target/all-feature locked tools Clippy passed with warnings denied.
Independent static review found no new blocker for this binding. This later
edit is not covered by the preceding full preflight. Prepared reconciliation
continuation, production activation and the full original handoff remain open.

2026-10-19 remote gate follow-up: CI 39228852503 is terminal; both Linux
workspace and Windows workspace failed. Windows failed-step log is retained at
`/tmp/talos-i300-ci-39228852503-windows.log`. Three Agent Git-add acceptance
cases explicitly encounter the production Windows structured-Git supervision
gate, rather than reaching approval/mutation. Numerous tools tests also report
failure; their individual causes are not yet established and must not be
collapsed into this one diagnosis. Linux checkout missing-hook regression was
rerun locally with the locked all-feature tools library filter: 1/1 passed.
This does not resolve the remote metadata-drift failure. Both local governance
validators completed with zero warnings for the preceding binding checkpoint.
Windows activation/runtime acceptance and both remote workspace gates remain
open; no rerun or publication is claimed.

2026-10-19 Windows evidence correction: downloaded full workspace artifact to
`/tmp/talos-i300-windows-39228852503/`. The Test workspace step timed out after
20 minutes, before the tools binary emitted its failure-summary diagnostics.
Thus individual tools causes cannot be inferred from names alone. Agent/MCP/
Runtime structured-Git acceptance still hits the explicit non-test Windows
preparation gate, although the unit path already uses the shared Job runner.
CI now requests libtest `--nocapture` so panic diagnostics survive a subsequent
hang. This changes evidence capture only; no Windows activation is implied.

2026-10-19 stable-stage validation/publication: standard external preflight
session 56255 exited 0 for source HEAD `784b1d8d`; log
`/tmp/talos-i300-binding-final-preflight.log` ends `release preflight: passed`.
Pinned locked workspace checks/Clippy/tests/doctests, tools 947/947 and both
external Runtime SDK modes passed. Commits `654770d0`, `e438f6b4`, `f8522f85`
and `784b1d8d` were pushed to the existing PR735 branch. Fetch confirmed
HEAD ahead 249 / behind 0 of origin/main before publication; no rebase needed.
Fresh remote exact-head gates and all original T7/handoff requirements remain
open. Production Pull and Windows structured-Git activation remain disabled.

Pull reconciliation continuation checkpoint (2026-10-19): the test-gated
executor now returns successful read-tree reconciliation to refreshed
repository/index/worktree evidence and a fresh integration approval, rather
than claiming an isolated reconciliation completes Pull. FETCH_HEAD bytes,
configuration, external/metadata attributes, inventory and reconciled HEAD stay
bound. Independent review found metadata info/attributes was incorrectly
refreshable inside workdir; excluding gitdir/common roots fixes it, and static
re-review confirms that finding closed. Real-repository coverage verifies new
tracked files, old-proof denial, separately approved success, and refusal of
post-command FETCH_HEAD/config/HEAD/metadata-attribute drift. Final tools tests
pass 948/948 and all-target/all-feature Clippy passes. Standard preflight also
passes (/tmp/talos-i300-reconcile-preflight.log); it started before the metadata
attribute correction, which is covered by the final full tools run
(/tmp/talos-i300-reconcile-final-tools.log). Both governance validators pass.
Git v2.54.0 builtin/pull.c confirms child fetch precedes reconciliation; do not
repeat completed fetches. Final recursive worktree updates, remaining effects,
native Windows/runtime acceptance and exact-head delivery gates remain open.
Production Pull stays disabled; T7 and the original handoff queue remain Active.
Fetch confirms HEAD is ahead249/behind0 origin/main; no rebase was needed.

Pull rebase backend checkpoint (2026-10-19): reconciliation continuation was
committed/pushed as 54a34618. The next uncommitted increment adds applypatch-msg,
pre-applypatch and post-applypatch to the exact hook frontier for ordinary
rebase with effective rebase.backend=apply. Real divergent-history tests pass
for merge and apply backends, asserting rewritten parentage, preserved local
and remote files, and a clean worktree. Each executable applypatch hook is
refused before mutation in the separate Unix regression. Independent static
review against Git 2.54 confirms backend selection and finds no blocker in
this increment; it does not establish complete external-effect safety or
production activation. The earlier 784b1d8d CI Linux workspace job passed;
Windows remains running. Current-head 54a34618 CI is run39236567890.

Rebase backend validation follow-up (2026-10-19): real execution additionally
covers duplicate backend declarations (last value wins) and explicit
--rebase-merges with backend=apply (merge backend, no applypatch frontier).
All Pull-focused locked tests pass 153/153; format and whitespace checks pass.
Full standard preflight is running at
/tmp/talos-i300-apply-backend-preflight.log. No activation or completion claimed.

Windows diagnostics checkpoint (2026-10-19): run39234578410 is terminal
(overall cancelled; Windows job failure), not still waiting. Nocapture now
preserves tools failure summaries: 825 passed/60 failed, with concrete target
configuration path failures and Job test assumptions in addition to the known
production activation gate. Ordered config expansion passes a verbatim Windows
path to Git, which reports unable to read that path; missing-component/..
fixtures also disagree with Windows resolution. The invalid-Job test's
active_processes().is_err() assertion fails. These are distinct pending fixes;
no cause beyond the observed diagnostics is yet established. Log retained at
/tmp/talos-i300-windows-784b.log. Do not remove production gates to hide failures.

Rebase backend stage validation (2026-10-19): standard locked release preflight
completed with exit 0 (/tmp/talos-i300-apply-backend-preflight.log). This covers
the final backend/hook changes above; Pull-focused tests pass 153/153 and
all-target/all-feature tools Clippy passes. Current committed-head CI
run39236567890 is terminal: Linux full validation, Desktop, issue reconciliation
and Windows compile/Clippy/focused guards pass; Windows workspace tests fail.
Production activation and exact-head cross-platform acceptance remain open.
Fresh origin fetch shows ahead250/behind0; no rebase is needed.

Windows Job null-handle correction (2026-10-19): Microsoft documents that
QueryInformationJobObject(NULL, ...) queries the calling process's Job. The
previous invalid-handle test therefore exposed an ownership-proof bug on a
Job-hosted runner, rather than a reliable API error. WindowsJobControl now
rejects NULL before both query and termination; existing empty/live Job tests
and the NULL regression remain enabled. Independent process-hardening review
found no blocker and confirmed ADR-068 compatibility. Format passes. The initial
GNU-target cross-check used the wrong validation route: this project uses cargo
xwin with x86_64-pc-windows-msvc. Missing MinGW is not a project blocker. The
correct xwin all-feature check now passes with the pinned MSVC standard
library (/tmp/talos-i300-null-job-xwin-check.log); native Windows CI remains
required for execution evidence.
Reference: https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-queryinformationjobobject
This does not activate structured Git on Windows or complete T7.
Windows shell/git feature library cross-check passes with --locked for
x86_64-pc-windows-gnu (/tmp/talos-i300-null-job-process-check.log). This compiles
the changed Job boundary; it does not run the Windows tests.

Windows native config fixture correction (2026-10-19): three Git config CLI
parity fixtures now retain tempfile's ordinary absolute Windows root instead of
passing a Rust canonical verbatim root to Git. Non-Windows roots stay canonical.
The tests still compare declaration/include ordering, depth handling and host
configuration selection; no production parser or permission behavior changes.
Locked target_config tests pass 29/29 on macOS
(/tmp/talos-i300-config-fixture-tests.log); format passes. Native Windows runtime
results and the distinct parent-traversal failures remain pending.
Windows MSVC test-target cargo xwin check passes with --locked and git feature
(/tmp/talos-i300-config-fixture-xwin.log); feature-reduced build emits existing
dead-code warnings, so this is compile evidence, not a warning-free Clippy gate
or native test execution.

Platform-specific parent traversal fixture correction (2026-10-19): Rust's
documented verbatim PathBuf join behavior removes dot components before Sources
receives the path. The two POSIX traversal tests and the POSIX missing-parent
assertion now run on Unix; a Windows replacement explicitly verifies normalized
source admission, denial without capture, admitted contents and drift refusal.
No production path handling changed. Independent review found no blocker and
confirmed this does not prove arbitrary raw Windows path traversal semantics.
Locked macOS target_config tests pass 29/29
(/tmp/talos-i300-traversal-tests.log); Windows MSVC test-target cargo xwin check
passes (/tmp/talos-i300-config-fixture-xwin.log), with existing reduced-feature
dead-code warnings. Native Windows execution remains pending.

Windows handle-cleanup fixture checkpoint (2026-10-19): mode 3 now uses the
existing AttributeList fault after native attribute-list initialization instead
of an invalid program rejected before native resource acquisition. Receipt
phase names match the exercised cleanup. Independent process-boundary review
found no blocker and no production change; mode 3 remains diagnostic, while
mode 0 retains per-resource audit responsibility. Locked all-feature MSVC test
target checking via cargo xwin passes. Native Windows runtime evidence remains
pending; this does not activate Windows structured Git or complete T7.

Cross-platform diff mode checkpoint (2026-10-19): Git diff now retains the
index mode when a gitlink is replaced by a regular worktree file on platforms
without Unix permission bits, so the `160000 -> 100644` type transition is
reported while ordinary content changes with `core.fileMode=false` do not gain
an executable-mode header. The replacement and mode-tracking regressions pass
24/24 in the locked tools diff suite; standard workspace preflight passes at
`0a017af1`. This is a focused parity fix and does not activate Pull or close
T7; native Windows runtime and the remaining Pull/effect gates remain open.

Windows nonempty new-gitlink discovery checkpoint (2026-10-19): bounded
new-gitlink directory inventory now applies Git's Windows component protections
instead of refusing every nonempty directory on Windows. It still rejects a
reparse-point root, never stats or opens descendants, and retains exact root
Read admission plus directory identity/name-list revalidation before mutation.
The fixture now requires preservation of ordinary nonempty directory contents
and checks Windows device/illegal-name rejection. Checkout focused tests pass
2/2 on macOS; all-target/all-feature locked Clippy and `cargo xwin check` for
the Windows MSVC test target pass. Independent static review found no blocker,
but explicitly requires native Windows CI before this path is considered runtime
validated. Production Windows preparation remains closed; this stage does not
close T7 or the initialized-child, Pull, recursive, and delivery gates.

Windows configuration fixture checkpoint (2026-10-20): commits `995da5d4`
and `de6ba2a3` replace POSIX-only HOME/XDG test roots with host-native absolute
paths and write startup-layout core.worktree values through Git config, which
quotes Windows backslashes. Windows fixtures retain ordinary absolute paths
for native Git rather than Rust verbatim canonical paths. The locked Git-feature
configuration suite passes 32/32 on macOS. These are fixture corrections, not
native Windows runtime evidence or production activation. The uncommitted
Windows preparation gate removal was withdrawn; the original gate is retained.
Current published b9c01b0d Windows workspace CI remains failed (tools 834 passed,
53 failed; Agent/MCP/Runtime also fail at the production gate). Other path,
executable-binding, CRLF, Pull-resolution and final delivery residuals remain.
T7 and the full serial handoff queue remain incomplete.

Current-source pinned locked cargo xwin MSVC test-target check passes after both
fixture commits; reduced-feature dead-code warnings remain. Formatting and
whitespace checks pass. This is cross-compile evidence only.

Checkout sender admission fixture checkpoint (2026-10-20): `58ddc8a9` routes
Checkout's direct-discovery fixtures through the production discover_sender
method with freshly consumed permission proofs, including Windows internal
executable re-admission. This fixes fixture bypass of the retained executable
binding; it does not relax production checks. Locked Git-feature Checkout tests
pass 49/49 on macOS (/tmp/talos-i300-checkout-sender-final.log), and pinned locked
cargo xwin MSVC test-target checking passes
(/tmp/talos-i300-checkout-sender-xwin.log). Native Windows execution remains
pending. These focused helpers are not full runner-stage-budget evidence.
Standard workspace preflight is running in session 84989, log
/tmp/talos-i300-sender-preflight.log; do not restart without polling that handle.
Windows preparation and production Pull remain closed. No T7 or handoff
completion, merge, publication or current-head full-preflight pass is claimed.


2026-10-20 validation checkpoint: `a4547db4` repairs native config fixture paths
and quoting; `c847f3ad` supplies maintenance fixture author identity; `279c29d3`
retains actual Checkout source bytes after native checkout and explicitly tests
repository-local autocrlf conversion. Locked macOS config tests pass 30/30,
maintenance authorization 1/1, and Checkout 49/49.
`e21618db` normalizes only admitted Windows path text for gitdir matching,
following Git for Windows normalize_ntpath; patterns retain glob escapes,
unsupported device namespaces fail closed, and authority/identity paths are
unchanged. Regressions are split into target_config_path_tests.rs, including
Unix backslash and literal HOME-dot preservation. Locked config tests pass
32/32; final-source cargo xwin MSVC test-target checking passes with existing
reduced-feature warnings. This is not native Windows execution evidence.
Preflight 84989 failed at sandboxed Push process observation (EPERM, 950/951).
Authorized outside-sandbox preflight 87720 passed, including tools 951/951 and
both SDK fixture modes, but predates the final production matcher change.
Final-source standard preflight at e21618db passes (session 96752 exit 0),
including workspace tools 953/953, locked checks/Clippy/tests, doctests and both
SDK fixture modes: /tmp/talos-i300-fixtures-preflight.log. Sessions 58738 and
95219 also exited 0; final-source MSVC recheck passes. Both governance validators
passed, zero warnings. Native Windows execution remains a separate CI gate.
Fresh origin fetch found no main-only commits (268 ahead/0 behind before the
matcher commit), so no rebase was needed. Windows production and Pull remain
closed; native runtime, remaining Windows failures, initialized-child Checkout,
Pull, independent delivery review and the full serial handoff remain open.


2026-10-20 Pull root-resolution checkpoint: defer identity capture of a
Windows prefix when an explicit RootDir follows. Rust gives VerbatimDisk an
implicit root, so the former traversal inspected an incomplete `\\?\C:`
before constructing the filesystem root. Component traversal, cancelled ancestor
evidence, missing-path handling and cumulative allowances remain enforced.
The separate Windows regression requires the first retained identity to be the
complete root and revalidates the result. Locked local resolution tests pass
7/7; cargo xwin MSVC test-target checking passes with the existing seven
reduced-feature warnings (/tmp/talos-i300-resolution-root-xwin.log). Native
Windows execution is not yet verified. Standard preflight is running in session
17900 (/tmp/talos-i300-resolution-root-preflight.log); poll before restarting.
Previous published-head CI 39349870653 is still in progress. Windows production
preparation and production Pull remain closed; T7 and the full queue remain open.


2026-10-20 verified root/sender stage: `f9a51485` retains complete Windows
root identities; `0be5b8d8` routes the Commit child-filter fixture through fresh
sender executable admission. Focused Commit regression passes and final-source
cargo xwin MSVC test-target checking passes. Sandboxed preflight 17900 exited
101 solely at Push process observation (EPERM; tools 952/953). Authorized
outside-sandbox preflight 24264 exited 0: locked workspace checks, Clippy, tests,
tools 953/953, doctests and both external SDK fixture modes pass
(/tmp/talos-i300-root-admitted-preflight.log). Governance 83941 and claims 54013
both exited 0 with zero warnings. Fresh fetch finds HEAD 272 ahead/0 behind
origin/main; no rebase needed. Published predecessor CI 39349870653 has passed
Linux full validation; native Windows workspace tests remain in progress. This
checkpoint does not establish native execution of the new root regression.
Windows production and Pull remain closed; original T7 and handoff residuals
remain open. Next: collect native CI failures and publish this verified stage.

2026-10-20 Windows Push fixture checkpoint (`0cecd803`): the network Push
mutation fixtures now pass through the production internal-Git-program admission
stage before asserting mutation denial/success, removing a Windows-only fixture
shortcut. The focused Push suite passes 15/15, the locked standard release
preflight passes (tools 953/953 plus workspace checks, Clippy, tests, doctests
and both external SDK fixture modes), and pinned `cargo xwin check --locked`
passes for `x86_64-pc-windows-msvc`. Formatting and whitespace checks pass.
The xwin result is compile evidence, not native Windows runtime evidence.
Published PR #735 still points at `3128117a`; CI run 39352700931 is in progress
on that older head, including its Windows workspace test job. Commit `0cecd803`
is local-only at this checkpoint and requires a fresh exact-head CI run after
publication. Remaining T7 work includes collecting native Windows execution
results and fixing any failures, completing initialized-child Checkout and the
authorized recursive Pull/fetch/integration path, then obtaining independent
delivery/security/API review, fresh exact-head gates, synchronized user-facing
docs and merge/completion evidence. Windows production preparation and
production Pull remain closed; T7 and the serial handoff remain Active.

Windows CI failure follow-up (2026-10-20): predecessor run 39352700931 for
`3128117a` completed with the Windows workspace test step failing and then
reaching its 20-minute limit. It recorded Agent 471/3, MCP 0/5, Runtime 70/2,
and tools 858/32 passed/failed. Agent/MCP/Runtime Git integration fixtures hit
the explicit Windows process-tree supervision refusal; the tools failures span
Windows path/configuration expectations, Checkout and Push admission fixtures,
and Pull resolution/integration evidence. These are diagnostic failures, not
evidence that production Windows Git is enabled. The Push fixture admission
failure is addressed in `0cecd803`; exact-head CI run 39355259779 for `77a8cf84`
is now running and must determine which failures remain. T7 remains Active, and
no Windows runtime or delivery gate is considered closed yet.

2026-10-20 latest Windows failure triage: exact-head run 39355259779 for
`77a8cf84` is terminal failure. Linux full validation, Desktop and governance
jobs passed; Windows tools recorded 860 passed/30 failed. Full failure artifact
is retained at `/private/tmp/i300-windows-77a8cf84/windows-workspace-tests.log`.
Two fixture corrections now use native `git config` to quote Trace2 target
paths and canonicalize both sides of the native worktree comparison (Windows
long/short path aliases). Each focused locked Git-only library test passes on
macOS; formatting and diff checks pass. Log:
`/private/tmp/i300-fixture-repair-tests.log`. These results do not prove native
Windows execution. Remaining failures include sender/receiver executable
admission shortcuts, hook/configuration expectations, Pull resolution and
integration, plus cross-layer tests refused by the production Windows gate.
T7 remains Active; no gate is removed and no completion is claimed.

2026-10-20 local fixture revalidation: `282c8da7` fixes native worktree
identity comparison and writes Trace2 fixture paths through Git config quoting.
Receiver direct-stage fixtures now explicitly admit the Windows sender program
before testing receiver roots; the missing-Read case isolates root capture after
that admission rather than accidentally failing sender rediscovery. The final
source passes locked Git-feature library tests 450/450 outside the sandbox
(`/private/tmp/i300-tools-fixture-recheck.log`). The initial sandboxed run was
449/450 because process observation via ps was denied; it was not a reproduced
Git failure. Receiver fixture changes are committed as `7cc19789`. Locked Windows MSVC
test-target checking via cargo xwin passes (seven reduced-feature dead-code
warnings; `/private/tmp/i300-receiver-fixtures-xwin.log`). This is compilation
evidence only; native Windows behavior, remaining failures and all original
delivery gates stay open.

2026-10-20 Commit discovery fixture checkpoint: `03fcbe34` routes child
index/fsmonitor, rerere, deferred hook and maintenance-redirection checks
through the existing admitted sender-discovery helper. Windows internal Git
admission must finish before these fixtures inspect configuration results.
The denial messages and no-write assertions remain. Locked Git-feature library
tests pass 450/450 outside the sandbox; Windows MSVC test-target checking via
cargo xwin passes with the same seven reduced-feature warnings. Logs:
`/private/tmp/i300-commit-discovery-tests.log` and
`/private/tmp/i300-commit-discovery-xwin.log`. Native Windows remains unverified.
Fetched origin/main `2d7f4b36` and rebased without conflicts; the tree delta
against pre-rebase `df559867` contains only the incoming search compatibility
change. Previous fixture commits are now `71d78bb8` and `144d86b3`. Standard
preflight is running in session 91790 with incremental compilation and debug
symbols disabled to limit disk use (`/private/tmp/i300-rebased-preflight.log`).
No publication, Windows/Pull activation or T7 completion is claimed. Next:
finish this gate, then continue native receiver executable/path, Checkout hook
and Pull failures; preserve the full original handoff queue.

2026-10-20 raw Pull path fixture checkpoint: `0280cc41` preserves the raw
`absent/..` and `missing/deep/../..` input text on Windows verbatim roots,
instead of allowing PathBuf::join to normalize those components before the
resolver is called. Both fixtures assert ParentDir is present and retain the
original absence/replacement failure checks. Locked Git-feature missing-path
regressions pass 25/25 (`/private/tmp/i300-raw-path-tests.log`); Windows MSVC
test-target cargo xwin checking passes (`/private/tmp/i300-raw-path-xwin.log`).
Native Windows runtime evidence is still required. Standard preflight session
91790 remains live; it started before this test-only change, so its result must
not be described as full validation of the final source. Disk pressure required
only the previously authorized target/debug/incremental cleanup; no broader
clean was used. Continue monitoring the live preflight and remaining Windows
receiver/Checkout/Pull failures. T7 and the original handoff remain Active.

2026-10-20 preflight terminal result: session 91790 exited 1. Workspace
check, Clippy, workspace tests and doctests passed, and the independent SDK
fixture default run passed. Its final coding-feature build failed with
No space left on device, so the standard preflight did not pass. Full log:
`/private/tmp/i300-rebased-preflight.log`. Previously authorized target/debug
cleanup removed only six completed test executables to recover space; source,
Git data and Windows target artifacts were retained. Receiver bootstrap fixture
now retains ordinary Windows paths for host Git and prints complete init
output on failure; focused validation is running in session 88143. Production
receiver shell validation still only supports the Unix /bin/sh baseline;
Windows local Push needs an explicitly bound Windows shell/effect solution,
not just sender fixture admission. T7 remains Active.

2026-10-20 receiver shell evidence checkpoint: Git for Windows upstream
run-command.c `git_shell_path()` uses `locate_in_PATH("sh")` under
GIT_WINDOWS_NATIVE, rather than the Unix compiled SHELL_PATH. Its mingw.c
`path_lookup` searches PATH and can also fall back to BusyBox applets. Thus
assuming an installation-relative usr/bin/sh.exe is not a supported resolution
rule. The Windows workspace CI now records Git build/version, exec-path, actual
GIT_SHELL_PATH and resolved git/sh application paths before tests, so the next
native run can distinguish path discovery failures from the explicit Unix-only
receiver baseline. This adds evidence collection, not Windows shell admission.
Production Windows and Pull gates remain unchanged. The resumed independent SDK
fixture session 87459 exited 0; its coding-feature consumer ran successfully
(`/private/tmp/i300-sdk-resume.log`). This repairs the previously failed SDK
subcheck only; final-source standard preflight and exact-head native validation
are still required. Remaining disk space was 317 MiB at this checkpoint.
Receiver bootstrap follow-up: the focused test completed 1/1 successfully
(`/private/tmp/i300-receiver-bootstrap-test.log`), committed in `9421da53`;
session 88143 above is historical, not a live wait. Governance and collaboration
claim validators both pass with zero warnings for this evidence checkpoint.

2026-10-20 Windows target-hook authorization correction: Checkout target-tree
inspection now treats ordinary Windows blobs (100644/100755) at effective hook
paths as executable independently of tree executable bits, matching existing
Windows hook capture. Previously a 100644 hook could be introduced by Checkout
without this rejection. The regression explicitly clears its index executable
bit on Windows. Directory and gitlink modes remain distinct. Focused regression
passes 1/1 and Checkout selection passes 49/49 on macOS; Git-feature library
Clippy with -D warnings and formatting pass. Windows MSVC test-target cargo xwin
check passes with seven existing reduced-feature warnings. Logs:
`/private/tmp/i300-target-hook-mode.log`, `/private/tmp/i300-target-hook-checkout.log`,
`/private/tmp/i300-target-hook-clippy.log`, `/private/tmp/i300-target-hook-mode-xwin.log`.
Independent static security review found no blocker and confirmed directory,
gitlink, symlink and ancestor protection; native Windows evidence remains open.
Initial linking failed for lack of space; previously authorized target/debug
cleanup removed only examples outputs before successful rerun. Production
Windows/Pull gates remain closed; this correction does not close T7.

2026-10-20 Pull line-conversion checkpoint: a new dirty-autocrlf case reproduces
native Windows' generic clean-worktree inspection error on macOS before the
fix (`/private/tmp/i300-pull-autocrlf-before.log`). Successful diff output with
change records now refuses as dirty even when Git also emits conversion
warnings. Clean admission still requires empty stdout and stderr; diagnostics
remain withheld. The rebase backend fixture compares complete worktree bytes
against native checkout-index output rather than assuming LF regardless of
configuration. Index/no-lock/no-FETCH_HEAD denial assertions remain intact.
Pull-focused locked Git-feature tests pass 153/153, Clippy -D warnings and format
pass. Windows MSVC test-target cargo xwin check passes with seven existing
reduced-feature warnings; native Windows execution is still pending. Logs:
`/private/tmp/i300-pull-autocrlf-suite.log`, `/private/tmp/i300-pull-autocrlf-clippy.log`,
`/private/tmp/i300-pull-autocrlf-xwin.log`. Independent static security review
found no blocker, no expanded clean admission and no diagnostic disclosure.
Windows/Pull production gates and all original handoff requirements remain.


2026-10-20 Windows refusal-fixture checkpoint: two Checkout assertions now
expect the earlier executable-hook refusal on Windows, consistent with its
ordinary-file hook semantics. All no-write, HEAD/index/worktree and hook
non-execution assertions remain. Recursive Pull child identity failures now
include fixture mode and original error; no production comparison changed.
Checkout tests pass 49/49 and cargo xwin MSVC test-target checking passes
(existing seven reduced-feature warnings); independent static review finds
no blocker. Logs: /private/tmp/i300-windows-hook-expectations.log and
/private/tmp/i300-windows-hook-expectations-xwin.log. Full standard preflight
38211 terminated 101 at talos-cli linking with ENOSPC, not a test assertion.
Previously authorized cleanup removed only target/debug/examples; full retry
81834 terminated 101: test execution found a corrupted, non-executable
3,920-byte todo_presentation_chain artifact after objcopy ENOSPC warnings
(/private/tmp/i300-stage-final-preflight-retry.log). This is not a test
assertion failure. Authorized target/debug cleanup removes rebuildable
artifacts before a fresh standard preflight; Windows target outputs remain.
Both governance validators pass with zero warnings. No final pass is claimed.
Windows production/Pull gates remain closed; native Windows, child Checkout,
recursive Pull integration and original delivery requirements remain open.


2026-10-20 conflict-fixture correction: native
Windows previously lacked MERGE_HEAD, but the fixture supplied identity only to
its history-building commands. Isolated host Git reproduction with system/global
configuration disabled and user.useConfigOnly=true exits 128 before MERGE_HEAD
without repository identity; adding repository user.name/email reaches conflict
(exit 1, MERGE_HEAD and unmerged index present). The test now supplies explicit
repository identity and retains every conflict/partial-effect assertion, adding
result content to missing-MERGE_HEAD diagnostics. Independent static review finds
no blocker; format and whitespace checks pass. Native cause remains provisional
until fresh Windows CI. Clean standard preflight 41606 has passed locked
workspace check/Clippy/tests and doctests, including tools 963/963 and the
modified conflict test. Its final external Runtime SDK validation is still
compiling at /private/tmp/i300-stage-clean-preflight.log; poll that handle
before restarting. Windows MSVC test-target cargo xwin check exited 0 (session
50608, /private/tmp/i300-conflict-identity-xwin.log; seven existing reduced-feature
warnings). This is not native Windows evidence. target/debug cleanup recovered
about 10 GiB; production Windows and Pull remain disabled. No final preflight
pass, T7 completion or subsequent-task activation is claimed by this checkpoint.


2026-10-20 clean stage validation: standard release preflight session 41606
exited 0 at implementation commit 02afc11f. The log
/private/tmp/i300-stage-clean-preflight.log ends `release preflight: passed`:
locked workspace check/Clippy/tests/doctests, tools 963/963, governance/claims
and both independent Runtime SDK consumer modes pass. Windows MSVC test-target
xwin checking also passed; native Windows execution is still pending. Fresh
origin fetch finds zero main-only commits; no further rebase is needed. Remote
feature head remains 77a8cf8412f5c4d01785454311a5580ef60f6ffb; patch-equivalence
comparison finds no remote-only patch missing locally after the earlier rebase.
Publish this stable candidate with an explicit lease on that feature head,
then collect fresh native CI. Windows/Pull production gates, initialized-child
Checkout, recursive Pull integration, independent delivery review and every
remaining handoff task remain open; this is not completion evidence for T7.


2026-10-20 native candidate result: PR #735 head 1a9ce740 was published using
an explicit lease. CI 39367016204 completed: Linux full validation, Desktop,
installer and issue-owner reconciliation passed; Windows tools reports 884
passed / 16 failed. Log: /private/tmp/i300-native-1a9ce740.log. The conflict
fixture now passes natively. Git reports shell C:/Program Files/Git/usr/bin/sh.exe.
Linked-child identity fails specifically in linked mode; remaining failures
include receiver executable/layout admission, Checkout stage/drift handling,
rebase CRLF expectations, daemon exports and retained-handle rename semantics.
Windows production and Pull remain closed; native CI is not green.

Daemon fixture correction: Git for Windows daemon.c concatenates base-path and
URL then compares exports with memcmp; mixed separators can reject the fixture
repository. Three test daemons now use consistently slash-separated Windows
base/export parameters, preserving their exact export roots and all permission
assertions. No production code changes. Independent static review finds no
blocker; format/diff checks pass. Git-enabled prepared tests pass 162/162
(/private/tmp/i300-daemon-fixture-git-tests.log, session 40534 exit 0). Initial
command without the optional git feature selected zero tests and is not evidence.
Native verification and full final-source preflight remain required before
publishing this correction. T7 and all subsequent handoff deliverables remain open.


2026-10-20 rebase fixture byte correction: isolated Git reproduction confirms
checkout-index --force preserves an existing stat-clean LF file even with
core.autocrlf=true; deleting the fixture file first produces CRLF on checkout.
Expected local/remote bytes now come from genuinely recreated index content,
before the tested rebase. All byte/history/cleanliness assertions remain.
Focused rebase test passes 1/1 (/private/tmp/i300-rebase-native-bytes.log);
independent static review finds no blocker. Full batch standard preflight is
running as 10879 (/private/tmp/i300-daemon-rebase-preflight.log), and MSVC test
check as 72103 (/private/tmp/i300-daemon-rebase-xwin.log). Poll those handles
before restarting. These fixes remain unpublished pending final gates; native
Windows verification and all original T7/handoff requirements remain open.

2026-10-20 validation continuation: MSVC test-target xwin session 72103
exited 0; full preflight session 10879 remains live and is executing workspace
tests. Native CI actually uses Git 2.56.0.windows.2, with exec-path
C:/Program Files/Git/ucrt64/libexec/git-core. Rechecked daemon.c at that exact
upstream tag: base-path concatenation and bytewise export comparison still
match the fixture correction rationale. Windows CI now records existence and
link targets for git.exe, git-receive-pack.exe and git-upload-pack.exe under
the reported exec-path, to distinguish missing executables from layout
assumptions before changing production admission. This diagnostic has not yet
run remotely; receiver executable availability remains unproven.

Final local batch result: standard preflight session 10879 exited 0 with
`release preflight: passed`; daemon fixtures committed as b7d38522 and native
rebase fixture bytes as 5ae25149. Both governance and collaboration validators
also exited 0. Native Windows rerun, production activation and the original
T7 residuals are still required; these commits do not complete T7.

2026-10-20 child backlink correction in progress: Pull and Checkout now
share an alias check that converts only the already admitted canonical child
root through the existing identity-checked Git directory argument helper.
The declared backlink is never resolved or read. Before conversion, exact
root and marker Read tokens are required and retained Root evidence is
revalidated; evidence is revalidated again after conversion. Canonical
spelling acceptance remains unchanged. Windows regression covers missing
root/marker proof, approved conventional spelling and unrelated declarations.
Independent static security review found no blocker; it did not run tests or
instrument zero-I/O behavior. MSVC xwin test-target check passed (50695);
initial sandboxed prepared tests reported 161 passed / 1 process-observation
PermissionDenied, so the same selection is running with host permissions as
15383 (/private/tmp/i300-child-backlink-tests.log). Checkout drift assertions
now identify their fixture mode; focused local test passed 1/1. This correction
is uncommitted and needs final validation and native Windows execution.
Published head 92494edf has active CI run 39371203732; do not restart it.

Host-permission rerun 15383 exited 0: prepared tests 162/162 passed.
Final batch preflight is live as session 22365, log
/private/tmp/i300-child-backlink-preflight.log. Poll that handle before
restarting; no full batch pass or native Windows success is claimed yet.

Session 22365 has now exited 0 with `release preflight: passed`. Child
backlink correction is committed as 489d2fa5; Checkout fixture diagnostics as
05783988, not yet published. Fresh fetch finds zero main-only commits.
Next local batch removes the redundant dashed git-receive-pack executable
binding: fixed receiver_command invokes the bound Git's builtin directly.
Git for Windows 2.54 and 2.56 git.c confirm builtin-first dispatch. A native
isolated push with empty receiver exec-path succeeds and verifies the receiver
ref. The production eight-stage fixture now adds a git-only exec-path case,
without an external receive-pack; focused test is running as 22469, log
/private/tmp/i300-builtin-receiver-test.log. Actual Git, shell and maintenance
identity/admission checks remain. Final batch validation/review remain pending.

2026-10-20 builtin receiver regression correction: the attempted eight-stage
fixture with GIT_EXEC_PATH override failed at preparation, as required by the
existing separate-invocation authorization guard (session 46789). The earlier
after-prepare override likewise correctly failed environment-drift validation.
Both fixture approaches are removed; no production guard was weakened. The
existing command-level real-push regression now uses an empty exec-path to
verify builtin dispatch, while the original full eight-stage authorization
fixture remains unchanged. This proves dispatch only, not support for
environment overrides in structured Git. Final checks and review are pending.

Builtin cleanup validation: command-level push passes 1/1; unchanged prepared
authorization suite passes 162/162 with host process-observation permissions
(/private/tmp/i300-builtin-prepared.log). Windows MSVC cargo xwin test-target
check passes with the seven existing reduced-feature warnings
(/private/tmp/i300-builtin-xwin.log). Independent static security review finds
no blocker; its comment precision suggestion was applied. Full standard
preflight is running as session 63768 (/private/tmp/i300-builtin-preflight.log);
poll before restarting. Native CI 39371203732 at published 92494edf ended in
failure: tools 885 passed / 15 failed, with additional Agent/MCP/Runtime
failures. Log: /private/tmp/i300-native-92494edf.log. This CI excludes local
489d2fa5, 05783988 and the current receiver cleanup. Production Windows/Pull
remain disabled and all original T7/handoff completion gates remain open.

Receiver cleanup is committed as f68c2ce1. Preflight 63768 remains live; its
production source is f68c2ce1, but the subsequent bootstrap fixture-only edit
was made while workspace tests were running, so do not describe this run as
final-source validation for that later edit. Next bootstrap fixture correction
canonicalizes the admitted gitdir for Metadata/Common while keeping ordinary
Windows spelling for host Git commands. The earlier focused variant passed
1/1 (/private/tmp/i300-bootstrap-canonical.log); final host-cwd spelling and
Windows validation remain pending. No new completion claim or activation.

Preflight 63768 terminated with exit 101: tools 962 passed / 1 failed solely
at the existing ps process-observation PermissionDenied sandbox boundary.
This is not a production regression. A host-permission full preflight rerun
was requested, log /private/tmp/i300-builtin-preflight-host.log; poll its
returned session before restarting. The bootstrap fixture edit remains
uncommitted and requires final focused/native checks.

2026-10-20 receiver batch validation result: host-permission full standard
preflight 47044 exited 0, ending `release preflight: passed` in
/private/tmp/i300-builtin-preflight-host.log (locked workspace checks, Clippy,
tests and both external Runtime SDK consumer modes). It ran after the final
bootstrap fixture source was fixed and supersedes sandbox process-observation
failure 63768. Focused final bootstrap fixture session 26686 also exited 0,
1/1 (/private/tmp/i300-bootstrap-canonical-final.log). The canonical admitted
gitdir and ordinary host Git cwd are now deliberately distinct in that test.
Windows native confirmation remains required; no activation/completion claim.


2026-10-20 hook-fixture follow-up: exact-head CI 39376337680 standard
validation (macos-14, not Linux) failed with tools 962 passed / 1 failed:
checkout_target_hook_missing_evidence_refuses_later_creation rejected repository
metadata drift during sender discovery. Local pre-change focused rerun passes;
the CI root cause is not established. The fixture commit now fixes
maintenance.autoDetach=false and gc.autoDetach=false so asynchronous maintenance
cannot outlive fixture setup; production revalidation is unchanged. A separate
hook-frontier assertion verifies the fixture's late-hooks/post-checkout path is
actually retained. Focused hooks pass 2/2; Git-feature tools library passes
491/491 outside the sandbox (/private/tmp/i300-hook-fixture-full-tools-host.log).
The sandbox run passed 490 and failed only ps process observation PermissionDenied.
Formatting and diff whitespace pass. Git-feature all-target Clippy is live as
session 23162 (/private/tmp/i300-hook-fixture-clippy.log); poll before restarting.
Latest fetch has zero main-only commits, so no rebase was needed. Native Windows
CI 39376337680 is still live in workspace tests; focused identity/process guards
and direct walkthrough passed. This is not full native success or T7 completion.

Hook-fixture validation update: session 23162 exited 0; Git-feature all-target
Clippy with -D warnings passes. Full standard preflight is now live as session
36418 (/private/tmp/i300-hook-fixture-preflight.log); poll before restarting.
No final preflight result or native Windows success is inferred.


2026-10-20 disk recovery checkpoint: preflight session 36418 exited 101
during compilation with ENOSPC (wast, talos-permission and other dependencies),
not a completed test result. After confirming no cargo/rustc/preflight process
remained, removed only target/debug under the maintainer's existing rebuildable
artifact cleanup authorization; available space recovered from 120 MiB to 11 GiB.
Full standard preflight is now running outside the process-observation sandbox
as session 70035, /private/tmp/i300-hook-fixture-preflight-host-lowdisk.log, with
CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=1 and dev/test debug info disabled. Poll
that session before restarting; no pass is claimed. Native Windows job
118475251274 in CI 39376337680 remains in workspace tests at this checkpoint.
Production Windows/Pull and all original T7/handoff completion gates remain open.


Windows rebase diagnostic follow-up: added test-and-Windows-only fixed stderr
category reporting plus output byte counts at integration failure. Raw output,
paths and configuration values are never printed by this diagnostic; production
behavior is unchanged. This is investigation support, not a rebase fix. The
source edit occurred during preflight 70035; it cannot establish Windows test
compilation or native execution for this edit. Fresh MSVC/native validation is
required before publishing it.


2026-10-20 native CI and admission-fixture checkpoint: exact-head CI for
987015ead29fc14886eccebd3b0320c5b2ea8fa3 completed with failures: Windows tools
891 passed / 10 failed; macOS tools 962 passed / 1 failed. Production Windows
and Pull remain disabled. Windows failures still include receiver shell support,
daemon timeout, rebase integration, child Checkout stage budget and identity
fixture handling; this is not a completion claim.

The Windows backlink approval fixture now retains its explicit Execute facet:
local Read alone is automatically allowed and cannot be approved with approve_once.
Receiver root/metadata tests now compare new permission facets against the
previous approved frontier: only Read may be added; already admitted execution
facets may remain. The root fixture completes Windows internal Git re-admission,
and the rewritten-network-destination fixture completes the existing Push program
binding stage before asserting mutation permissions and stale-proof rejection.
Focused host receiver-root regression passes 1/1
(/private/tmp/i300-root-discovery-fixture.log). Formatting and diff whitespace
checks pass. Current Push batch and MSVC cargo xwin test-target check are pending;
standard preflight 70035 remains live. These edits are uncommitted and require
final verification, native Windows execution and appropriate stage review.


2026-10-20 admission fixture stage validation: standard preflight 70035 exited 0
with `release preflight: passed`, including locked workspace tests and both
external Runtime SDK fixture modes
(/private/tmp/i300-hook-fixture-preflight-host-lowdisk.log). Current receiver
regressions pass 47/47 and Push regressions pass 6/6; the missing-hook fixture
passes locally again. Logs: /private/tmp/i300-receiver-admission-fixtures-host.log,
/private/tmp/i300-push-admission-fixtures-host.log and
/private/tmp/i300-hook-missing-local.log. Final MSVC cargo xwin test-target check
exited 0 (/private/tmp/i300-admission-fixtures-xwin-final.log), retaining seven
existing reduced-feature warnings. This is compilation, not native execution.

After rebase onto origin/main 1a469954, these stages are e55f6acb (approval
fixtures), 59a18f38 (Checkout hook fixtures), 0fddbe18 (Windows-only fixed-category
Pull diagnostics), and 7075aae2 (this evidence update).
The root fixture performs the Windows program-binding and root-discovery halves
exactly once; it does not re-enter the dispatcher after binding. Staged diffs
were inspected. No production gate was enabled. Native CI failures and remaining
original T7 acceptance requirements remain open; the macOS CI drift cause is not
proven by a locally passing fixture. Earlier pending/uncommitted checkpoints
above are superseded only for these validation and commit facts.


2026-10-20 post-fetch identity fixture follow-up: content evidence is explicitly
dropped after all reference-byte/pointer assertions and before renaming gitdir.
The original RepositoryState and its live directory identity handles remain
retained, so replacement must still fail before repository rediscovery. This
separates content-handle lifetime from the directory substitution assertion;
it is a candidate fix for native Windows AccessDenied, not yet native proof.
Focused host regression passes 1/1
(/private/tmp/i300-postfetch-release-content.log); MSVC cargo xwin test-target
check exits 0 (/private/tmp/i300-admission-fixtures-xwin-final.log), with seven
existing reduced-feature warnings. Production identity code and gates unchanged.
CI 39383710845 tests the preceding published head 260e9228 and remains running;
it does not contain this fixture follow-up. T7 and all later tasks remain open.


2026-10-20 include ancestor preservation checkpoint: relative configuration
includes now retain cancelled ancestors when their source uses a Windows verbatim
root. Raw spelling concatenation avoids PathBuf join normalization; Windows
relative separators are normalized before component inspection. Root-relative
and drive-relative ambiguous includes are refused. Absolute includes retain the
existing path. Tests cover both Windows separators, ParentDir retention and
ancestor inventory. Independent static security review found no blocking issue;
native Windows confirmation remains pending.
Config-state host regressions pass 33/33 and the Checkout exact-read/ancestor
replacement regression passes 1/1. Final cargo xwin MSVC test-target check exits 0
with seven existing reduced-feature warnings. Evidence:
/private/tmp/i300-include-config-tests-current.log,
/private/tmp/i300-include-hook-current.log and
/private/tmp/i300-include-final-xwin.log. This is not native execution evidence.
CI 39383710845 at 260e9228 is terminal: Windows tools 895 passed / 6 failed;
other jobs passed. This supersedes the preceding running-state observation.
Remaining Windows failures concern Checkout stage budget, include ancestor
identity, rebase unstaged-change detection, receiver shell support, post-fetch
handle lifetime and daemon timeout. The local include and handle-lifetime fixes
still need native confirmation. Production Windows/Pull remain disabled; T7 and
all subsequent handoff tasks remain open.


2026-10-20 Checkout bounded-refusal checkpoint: rebased onto origin/main
4e20ee95 without conflicts. Rebase fixture correction is now e479806e;
include ancestor fix is 617661eb and content-handle fixture is cbc4eb72.
Rebase execution now covers merge/apply/rebase-merges with core.autocrlf both
true and false. checkout-index --index refreshes stat data after native checkout;
the previous true-autocrlf failure reproduced locally before that correction.
Host rebase suite passed 4/4 (/private/tmp/i300-rebase-suite.log).
Checkout now refuses already-known deletion overlap with retained child metadata
configuration before recursively inventorying that metadata. It adds no reads,
keeps the eight-stage budget and retains the final alias-aware overlap check.
Independent static review found no blocking issue. A deep metadata regression
checks refusal without exhausting stages, preserving child HEAD, dirty file,
parent index and the deep metadata file. Host Checkout suite passed 50/50
(/private/tmp/i300-checkout-early-suite.log); MSVC cargo xwin test-target check
passed (/private/tmp/i300-checkout-early-xwin.log), with seven existing warnings.
The final extra metadata-preservation assertion is checked separately in
/private/tmp/i300-checkout-early-final.log. Native Windows confirmation remains
pending. Production Windows/Pull remain disabled; all T7/handoff residuals remain
open. Receiver Windows shell support and daemon timeout are next investigations.


2026-10-20 rebased stage validation: standard release_preflight.sh completed
with exit 0 on caaaed602a48013a76c8b8d67201b115785116f8 (session 17706).
Evidence: /private/tmp/i300-stage-preflight-oct20-unsandboxed.log. Workspace
validation and both independent Runtime SDK consumer modes passed. The earlier
sandbox run failed on process-observation permission and is superseded by this
approved host-permission run, not treated as a source failure. Fresh fetch shows
no main-only commits. Native Windows confirmation is still required for the four
candidate corrections; receiver shell support and daemon timeout remain open.
Production Windows/Pull remain disabled. T7 and the later handoff queue remain
incomplete; this checkpoint supplies stage validation, not completion evidence.


2026-10-20 daemon diagnostic checkpoint: the real network Push fixture now
retains verbose daemon stderr in a temporary file and includes daemon exit state
and receiver-ref presence in its failure assertion. This distinguishes receiver
progress from client timeout without changing production behavior or deadlines.
Host regression with --features git passed 1/1
(/private/tmp/i300-daemon-diagnostics-git-test.log). The preceding default-feature
command ran zero relevant tests and is not validation evidence. Windows native
timeout remains unresolved. Published e532e7a0 CI 39399935245 is still running;
this diagnostic is a later local test-only change, not part of that CI head.


2026-10-20 shell selection hardening checkpoint: local Push now queries the
bound Git's shell selection after final receiver storage inspection, compares
its spelling before capturing/executing a newly selected shell, then revalidates
retained sender/receiver executable identities and the existing shell baseline.
Changed selection refuses without reading new shell contents or rebinding.
Independent review found no new authorization bypass and requested this later
placement; the query-to-spawn race remains, so no OS-level confinement is claimed.
The real Git selection regression passes 1/1
(/private/tmp/i300-shell-selection-regression.log). Earlier local Push execution
regression and MSVC xwin checking passed before the final placement adjustment;
final-position regression passed 1/1 (session 72624,
/private/tmp/i300-shell-final-position.log). Windows shell baseline
and production Windows/Pull gates remain closed. This is an uncommitted stage,
not T7 completion. Final independent static review found no blocking issue;
final MSVC test-target xwin checking passed
(/private/tmp/i300-shell-final-xwin.log), with seven existing warnings. Both
governance validators passed with zero warnings. Ready for stage commit.


2026-10-20 Windows receiver continuation checkpoint: native CI 39401797353
at af87f7be is terminal failure, not a live wait. Windows tools reports 914
passed / 4 failed; Agent/MCP/Runtime production-gate failures also remain.
Daemon diagnostics show a receive-pack request reached the daemon, but no
receiver ref was written before the controlled foreground deadline. This
narrows investigation to the service/receiver exchange; it does not establish
a root cause or justify raising the timeout. Evidence:
/private/tmp/i300-af87-windows.log. The local receiver executable spelling
candidate converts only identity-checked local-drive git.exe paths to a
slash-separated conventional spelling, rejecting unsupported forms. MSVC
cargo xwin test-target check passed with seven existing reduced-feature
warnings; formatting and whitespace checks pass. Native path behavior and
security review remain required before publication. Production Windows/Pull
remain disabled; T7 and the full handoff objective remain incomplete.


Windows receiver path review follow-up (2026-10-20): independent review
identified local-drive junction resolution to UNC as a gap. The converted
executable spelling now independently requires an absolute local Disk prefix;
review confirmed this removes the static blocker. MSVC xwin test-target check
passes after correction. Native shell special-character execution and
junction-to-UNC runtime coverage remain outstanding. Runtime override guard
is independently committed as db56d70f. Fixture-only GIT_TRACE=1 now records
receive-pack child dispatch in captured daemon stderr; the real host daemon
Push regression passed 1/1 (/private/tmp/i300-daemon-dispatch-trace-test.log).
Neither diagnostic nor cross-compilation proves Windows runtime correctness.


2026-10-20 stage commits: db56d70f records the runtime environment guard;
006812ae records identity-preserving Windows receiver executable spelling;
159f98b4 records fixture-only receive-pack dispatch trace. Staged diffs were
reviewed separately. Both governance validators pass with zero warnings.
Native special-path coverage, Windows shell admission and daemon root cause
remain open; no production gate or completion state changes.


2026-10-20 receiver candidate validation: standard release_preflight.sh exited
0 (session 97754), including workspace tests and both external Runtime SDK
consumer modes; /private/tmp/i300-receiver-candidate-preflight.log. A new
Windows-only regression compares direct native fixture execution with the
fixed receiver command through Git's selected shell at a path containing
spaces, apostrophe, shell metacharacters and Unicode. It tests executable
selection and injection refusal, not complete argument fidelity. Review found
and corrected a cross-volume hard-link assumption by placing the fixture next
to the test executable; commands share frozen environment/cwd and discovery
has a deadline. These Windows-only follow-ups pass MSVC xwin checking but
remain unexecuted on Windows. Fresh fetch has no main-only commits. Production
Windows/Pull, remaining Checkout behavior and complete T7 delivery stay open.


2026-10-20 Windows shell baseline local candidate: same-installation canonical
path checks now support the observed ucrt64 Git layout in the gated unit path.
Unknown layouts, other shells/roots/drives and invocation-spelling substitution
refuse. Review caught and corrected use of rebound self.program; the baseline
now consumes sender_program_target() retained canonical evidence without new
reads. Independent re-review has no blocker; MSVC xwin test-target check passes
(seven existing reduced-feature warnings), host Unix shell regression passes
1/1 (/private/tmp/i300-shell-baseline-host.log). Candidate remains local while
7036ddce CI 39444015640 runs. DLL/MSYS effects, native prepared Push and full
Windows/Pull activation remain unproven; no completion claim.

2026-10-20 Windows HOME startup regression: added a native-only shell fixture
that places four startup sentinels in a temporary HOME, verifies all four are
visible through that same shell's HOME expansion, and checks the fixed
noninteractive command completes without executing the sentinels. Discovery and
execution have ten-second deadlines and isolated child environments. Independent
static review found no remaining blocker after adding the HOME visibility check.
MSVC cargo xwin test-target checking passes (seven existing reduced-feature
warnings); this is not native execution evidence or proof that startup files
were never read. CI 39444015640 remains in progress at 7036ddce and does not
contain this test or the local shell baseline. Production gates remain closed.

2026-10-20 Pull attribute execution containment: native merge fixtures reproduce
execution of a custom merge driver and of a smudge filter selected only by new
target attributes. Integration discovery now refuses configured executable
filters and Merge/Rebase drivers before publishing the mutation scope. The
prepared fixtures verify no helper execution or HEAD mutation after refusal.
Independent static review found no blocker. Locked Pull-focused tests pass
154/154 (/private/tmp/i300-pull-attribute-guards.log). Test-only local overrides
isolate inherited global filters without changing global configuration.
This is conservative containment, not finished capability: unused global LFS
configuration is also refused. Precise current/target/intermediate attribute
selection and separate selected-program authority remain required; default
multihead strategy execution and production activation gates also remain open.
Validation follow-up: MSVC test-target xwin check and all-target/all-feature
Clippy -D warnings pass; both governance validators pass. CI 39444015640 is
terminal failure: macOS main job passes; Windows tools 921 passed / 3 failed
(Checkout exact hook tokens, receiver layout shell baseline, daemon Push).
Agent/MCP/Runtime Windows production-gate failures remain. Native special-path
shell dispatch and cancelled-ancestor include tests pass. Daemon trace now
proves the builtin receive-pack child starts, but no receiver ref is written
before timeout; root cause remains unresolved. Log:
/private/tmp/i300-7036-windows.log. The CI head excludes the local baseline and
HOME sentinel commits and this containment change.


2026-10-21 auxiliary capture review checkpoint: the uncommitted Windows
ParentDir capture candidate passes MSVC cargo xwin test-target checking
(/private/tmp/i300-auxiliary-capture-xwin.log; seven existing warnings).
Host config-state tests pass 31/31 with --no-default-features --features git
--locked (/private/tmp/i300-auxiliary-config-tests.log). An earlier filter
selected zero tests and is not validation evidence. Independent static review
identified an unresolved identity-binding concern: retaining original spelling
and equal bytes does not by itself prove that cancelled ancestors and the
resolved target stayed identical. Existing Sources/identity coverage must be
verified and reused or extended before committing this candidate. No complete
call-chain bypass has been reproduced. Native Windows runtime confirmation,
production activation and all T7/handoff closure requirements remain open.


2026-10-21 rebase checkpoint: runtime stage 710582ec and documentation stage
d0fe3164 were saved before rebasing all 327 local commits onto fetched
origin/main 032cce61. Rebase completed without conflicts; their rewritten
commits are 467e6475 (runtime) and 845d5017 (documentation); capability plumbing
91155adf is now 3ac126ee. Earlier SHAs above describe pre-rebase history.
Full standard preflight session 84127 exited 0 with incremental compilation
and dev/test debug information disabled; durable log:
/private/tmp/i300-p1-rebased-preflight-oct21.log. The log ends with `release preflight: passed` and both external runtime
fixtures passed. This validates the rebased runtime candidate, preceding the
subsequent daemon baseline test-only change. P1–P5 remain open and production Windows/Pull
gates remain closed. Next after the gate: native Windows candidate validation
and causal daemon Push diagnosis, then remaining Checkout/Pull acceptance.

2026-10-21 native daemon baseline candidate: the existing real Push fixture now
first runs a ten-second native Git push dry-run against the same receiver, with
hooks bypassed only for that diagnostic and packet tracing confined to the fixed
fixture. It verifies no receiver refs or fixture commit were transferred before
continuing the original structured transfer. This separates a receiver/native
protocol failure from a later supervised-dispatch failure without extending any
production deadline or authorization. Host locked git-only selection passes 1/1
(/private/tmp/i300-daemon-native-baseline-host.log); Windows MSVC cargo xwin check
exited 0 in session 73497 with seven existing reduced-feature warnings; log
/private/tmp/i300-daemon-native-baseline-xwin.log. Native
Windows execution and the actual timeout cause remain unproven.

2026-10-21 receiver absence audit: expanded the existing prepared receiver
skiplist regression to capture an initially absent external file, admit the
receiver auxiliary stage, then create that file before final revalidation.
The receiver rejects the appearance, alongside its existing missing-Read and
same-content replacement checks. Locked host git-only selection passes 1/1
(/private/tmp/i300-receiver-absence-regression.log). MSVC cargo xwin test-target
checking also passes with seven existing reduced-feature warnings
(/private/tmp/i300-receiver-absence-xwin.log), and both governance validators
pass with zero warnings. This test-only follow-up
remains local while exact-head CI 39570564817 tests published 9f4f623a; macOS
full CI and Linux Desktop passed, Windows workspace tests remain in progress
at the latest confirmed observation. PR #735 description now reflects P1–P5
and current evidence. Native Windows closure and all remaining packages stay open.

### 2026-10-21 Windows native failure triage and Checkout prefix repair

Published candidate 9f4f623a CI run 39570564817 completed with Windows workspace
failure: talos-tools 945 passed / 19 failed. macOS full validation, Linux Desktop,
and Issue reconciliation passed. Native daemon dry-run succeeds; structured Push
sends its reference update but no pack header before timeout. Sender pack/child
supervision remains under investigation, not established as repaired. Other
failures cluster around Checkout source protection/path resolution, discovery
frontier fixture assumptions, and Pull configured-filter execution admission.

Local Checkout overlap repair handles ordinary versus verbatim Windows disk and
UNC prefixes. Previously component byte comparison could treat a canonicalized
worktree and Git-reported source as disjoint. This is conservative effect overlap
checking, not a new path grant or canonical identity mechanism. A separate
Windows unit regression covers matching prefixes and distinct drives, shares and
directory names. Final host effective-configuration Checkout regression passes
1/1 (/private/tmp/i300-checkout-prefix-host-final.log); cargo xwin MSVC test-target
check exits 0 (/private/tmp/i300-checkout-prefix-xwin.log), with the existing seven
reduced-feature warnings. Native Windows regression and security review remain
required; this stage is not proof that the native CI failure is resolved. P1–P5 remain open; production Windows/Pull gates stay closed.

### 2026-10-21 Windows fixture corrections and owned pack diagnostic

Discovery auxiliary-read denial now follows every separately admitted sender
executable frontier instead of assuming one discovery stage. Receiver skiplist
configuration is written with Git's quoting rather than unescaped Windows paths.
Pull integration fixtures isolate inherited executable filters from the effective
system/global/include chain, preserving local values; the production filter
execution refusal is unchanged and does not satisfy P4's full capability target.

The daemon transfer test now computes a real pack on stdout through the same
bound sender executable, frozen environment and owned runner before actual Push.
This read-only ten-second probe separates standalone packing failures from the
send-pack child/pipe chain; it does not replace the actual transfer assertion.
Host native daemon/owned pack test passes 1/1; discovery and skiplist selections
pass 1/1 each; Pull integration selection passes 12/12. Final host git_write suite
passes 419/419 (/private/tmp/i300-windows-triage-git-write-host.log). Its first
sandboxed attempt passed 418 and failed only because process observation was
forbidden; the authorized unsandboxed rerun exits 0. Final MSVC cargo xwin
test-target checking exits 0 (/private/tmp/i300-windows-fixture-pack-xwin.log),
with seven existing reduced-feature warnings. Native Windows confirmation,
Push root cause, scoped security review and P1–P5 closure remain outstanding.

### 2026-10-21 P3 Windows initialized-child coverage candidate

Removed the unconditional Windows return from
checkout_new_gitlink_preserves_initialized_child. The same native/prepared,
external-worktree, linked-worktree, filter-denial, ancestor-refusal and raw-startup
Read-denial matrix now executes on Windows instead of silently passing without
coverage. Its override config uses Git's path quoting. Host selection passes
1/1 across all nine modes (/private/tmp/i300-checkout-child-native-matrix-host.log)
and MSVC cargo xwin test-target check exits 0
(/private/tmp/i300-checkout-child-matrix-xwin.log), with existing reduced-feature
warnings. Windows runtime execution is still unproven; this does not complete P3.
Published cb424257 is being validated by CI 39578256961; this later coverage
candidate stays local while that exact-head run is live.

### 2026-10-21 Checkout ancestor rejection expectation

The prior native Windows target-hook failure for ancestor drift already refused
execution: the invocation-owned resolver rejected the replaced traversal directory
before target-config byte evidence checking. The regression now accepts that
earlier identity rejection only for ancestor drift, retaining success/error and
unchanged HEAD/worktree assertions and the existing config/hook checks. Host
selection passes 1/1 (/private/tmp/i300-checkout-ancestor-frontier-host.log).
MSVC cargo xwin test-target checking exits 0 with the seven existing warnings
(/private/tmp/i300-checkout-ancestor-frontier-xwin.log); its initial sandboxed
attempt could not update the external cargo-xwin cache symlink, so validation
was rerun with authorization. Native confirmation remains pending in the next
stable candidate. P1–P5 and the full handoff remain open.

### 2026-10-21 Exact-head macOS metadata refusal triage

CI 39578256961's macOS job 119149158117 failed Release preflight: talos-tools
1020 passed / 1 failed. The Commit replacement-directory success case was refused
because RepositoryState changed after preparation (commit.rs:560). This is not
a completed full-CI pass. The failed job log is retained at
/private/tmp/i300-cb424257-macos.log. The same host test passes individually;
the changed metadata field and cause are not yet established.

Added test-build-only reporting of changed RepositoryState field names, without
configuration values, file bodies or path output. Production rejection remains
unchanged. Full host git_write tests pass 419/419 with this diagnostic
(/private/tmp/i300-cb424257-host-diagnostic.log), after rerunning outside the
sandbox because the first run's process observation was denied (418/419).
MSVC xwin test-target checking passes with seven existing warnings
(/private/tmp/i300-checkout-ancestor-frontier-xwin.log). The native Windows job
remains live; no replacement candidate was pushed. The macOS intermittent
metadata refusal and Windows pack/child-chain diagnosis remain open in P1/P2.

### 2026-10-21 Current-head full local preflight

Pinned-toolchain locked release preflight at b2d7bcfa exits 0
(/private/tmp/i300-b2d7bcfa-preflight.log), including workspace tests and both
runtime external fixtures. The Commit replacement-directory regression passes
in this full run; the earlier macOS CI refusal remains unexplained, not repaired
by the diagnostic. Fetch completed; origin/main has no commits absent from HEAD.
Native Windows CI job 119149158085 passed structured Git identity guard tests
and remains live in workspace tests. No production activation or package exit
is claimed; later local commits remain unpublished while this run is live.

### 2026-10-21 Bounded metadata-refusal reproduction

At 62f89dbc the Commit replacement-directory regression passes another locked
Cargo selection, 30 sequential isolated executions, and 32 isolated executions
with concurrency eight. Logs: /private/tmp/i300-commit-drift-repeat.log,
/private/tmp/i300-commit-drift-30-runs.log, and
/private/tmp/i300-commit-drift-32-concurrent.log. This does not reproduce the
macOS CI refusal or establish a repair; cross-test interaction and native runner
state remain untested hypotheses. Do not weaken RepositoryState comparison or
retry refused mutations. The next stable CI candidate includes the existing
field-name-only diagnostic to identify the changed evidence if it recurs.

### 2026-10-21 Native candidate terminal evidence

CI 39578256961 at cb424257 is terminal failure. Windows job 119149158085
finished at 10:19:24 UTC; retained log:
/private/tmp/i300-cb424257-windows.log. Tools improved to 961 passed / four
failed. Remaining failures are effective-configuration Checkout overlap,
target-hook ancestor diagnostic expectation (corrected locally), auxiliary
Read-frontier path spelling, and real daemon Push. Agent (three), MCP (five)
and Runtime (two) failures still reflect the deliberately closed production
Windows gate; do not skip them or claim production activation.

The real Push passed the owned `pack-objects --stdout --all` probe and its PACK
assertion before timing out at the actual transfer assertion. Thus standalone
pack generation with the frozen environment works. The receiver saw the ref
update and flush but no pack header until sender cancellation after 300 seconds.
Push child dispatch/pipe behavior remains unproven; this is not evidence to
increase deadlines. The workspace step also hit its 20-minute limit.

Checkout still switched branches while rewriting an effective include file.
Treat this as an unresolved protected-boundary failure, not a stale assertion.
Investigate retained path spellings versus repository canonical paths (including
Windows short/long-name aliases); prefix-form handling alone did not close it.
The auxiliary-frontier assertion compares canonicalize output with a resolver
that preserves filesystem traversal spelling; verify the actual granted target
before changing that assertion. P1–P5 remain open.

The bounded correction conservatively treats Windows components containing `~`
as potentially equal in effect-overlap checks, covering 8.3 aliases without
additional pre-admission metadata reads. This may refuse unrelated ambiguous
paths; it never grants authority. Added long/short-name and disjoint-suffix
characterizations. The auxiliary frontier fixture now canonicalizes only its
test-observed facet target to prove it identifies the external directory instead
of requiring a particular spelling. Native confirmation and security review
remain required; do not claim the Checkout failure repaired from compilation.
Host git_write passes 419/419 after authorized process observation
(/private/tmp/i300-short-alias-git-write.log); focused overlap and frontier tests
pass, all-target Git Clippy passes with warnings denied, and MSVC cargo xwin
test-target checking passes with seven existing reduced-feature warnings
(/private/tmp/i300-short-alias-{host,frontier-host,clippy,xwin}.log).
Fetch completed and origin/main has zero commits absent from this branch.

### 2026-10-21 Native full-transfer diagnostic baseline

The real daemon Push regression now performs a native full transfer to a separate
nested bare receiver on the same daemon before exercising Talos. It checks the
expected destination ref and verifies that the actual Talos receiver still has no
refs. The existing owned pack-generation and mandatory structured object-transfer
assertions remain. Unlike the dry-run baseline, this exercises native child/pipe
packing; it does not establish behavior with the frozen environment or owned Job.
Fixture-only tracing remains confined to fixed payloads. No production permissions,
process ownership, deadlines or activation gates changed.

Locked host selection passes 1/1 in 1.65 seconds; Git all-target Clippy with warnings
denied, formatting and MSVC cargo xwin test-target checking pass. xwin reports the
same seven existing reduced-feature warnings. Logs:
/private/tmp/i300-native-full-push-baseline-{host,clippy,fmt,xwin}.log.
The initial sandbox xwin attempt could not update its external clang-cl cache;
the authorized rerun exited 0. Native Windows comparison remains the next causal
experiment; do not infer the sender child-chain cause from host success.

### 2026-10-21 Published diagnostic candidate validation

Candidate `3b00e98e1849e122d7657090f43c98063245e025` is pushed to PR #735.
Its complete local standard release preflight exited 0, including workspace tests,
doctests and both external Runtime SDK fixture modes; retained log:
`/private/tmp/i300-3b00e98e-preflight.log` (`release preflight: passed`).
Exact-head CI run [39587933025](https://github.com/wjhuang88/talos/actions/runs/39587933025)
is still running at this checkpoint. Windows workspace Check and Clippy passed;
Desktop validation is running and native workspace tests remain pending. The macOS
release-preflight step passed and CLI/TUI locale acceptance is running. Linux
Desktop, Windows installer and remote Issue/owner reconciliation passed.
These observations do not prove native Push success or close P1–P5. Continue
observing this run rather than restarting it; production Windows and Pull gates
remain disabled.

### 2026-10-21 Native full-Push comparison result

Run 39587933025 is terminal: macOS and the non-workspace platform jobs passed;
Windows workspace testing timed out after 20 minutes. Retained full native log:
`/private/tmp/i300-3b00e98e-windows.log`. The native full-Push baseline exceeded its
10-second deadline before Talos dispatch. The receiver accepted its ref update
and flush but no subsequent pack was observed. This reproduces the stall without
Talos's frozen environment or owned Job; neither is established as the cause.
The fixture then remained live past 60 seconds despite its panic, preventing a
terminal tools summary. A local test-only follow-up uses file-backed native sender
diagnostics instead of captured pipes: logs remain readable on timeout and native
descendants cannot retain a Tokio output-reader pipe. Host selection passes 1/1
in 1.70 seconds (`/private/tmp/i300-native-sender-file-host.log`). Native confirmation
and the complete real Talos transfer assertions remain mandatory.

The follow-up also passes formatting, Git all-target Clippy with warnings denied,
and locked MSVC cargo xwin test-target checking (seven existing reduced-feature
warnings). Logs: `/private/tmp/i300-native-sender-file-{clippy,xwin}.log`.

The Checkout protected target-configuration fixture passed on Windows. A separate
initialized-child Checkout fixture failed with `stage limit`; preserve the fixed
eight-stage budget and investigate repeated admission stages instead of widening
it. Agent (3), MCP (5) and Runtime (2) failures remain consistent with the closed
production gate. There is no full tools pass or Windows activation evidence.

Checkout diagnostic follow-up: the shared test dispatcher now retains each
permission profile and prints the sequence only on stage-budget refusal. The
initialized-child matrix reports its failing mode. Its nine-mode host selection
passes 1/1 (3.68 seconds); the real daemon Push selection passes 1/1 (1.99 seconds).
Formatting and whitespace checks pass. These fixture-only diagnostics preserve
the eight-stage bound and production admission semantics; they do not repair or
explain the Windows failure. Committed diagnostic stage: `57e11597`. Locked
all-target Git Clippy passes with warnings denied; MSVC cargo xwin test-target
checking passes with the seven existing reduced-feature warnings. Both governance
validators are rerun for this documentation follow-up before publication.
Checkout stage consolidation candidate (2026-10-21): the admitted child-worktree
stage now explicitly includes child HEAD/index/object Reads and discovers
index-selected attribute paths there, publishing actual-target Reads before
attribute bodies are captured. The eight-stage budget is unchanged. The fixture
reserves the Windows executable-binding stage on hosts and covers eleven modes,
including index-Read and attribute-content denial. A malformed denial fixture
initially failed earlier metadata identity checks; corrected it to use a consistent
worktree so the intended content boundary is actually exercised. Locked host
checkout matrix passes 1/1; Git write regressions pass 419/419 with authorized
process observation outside the sandbox; all-target Git Clippy passes with
warnings denied; formatting and whitespace checks pass. The earlier dirty-tree
full preflight is not exact-head evidence and failed a sandbox-denied ps call.
Locked MSVC cargo xwin test-target checking also passes with the seven existing
reduced-feature warnings. Native Windows confirmation, independent boundary review and final full preflight
remain required. This candidate does not activate Windows or Pull or close P1–P5.
