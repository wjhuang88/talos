Linked-worktree reference closure checkpoint (2026-10-17): after separately
admitted `commondir` targets are bound, external `refs` and `packed-refs` become
fresh exact-path frontiers. Their descendants and contents use the existing
4096-entry/4 MiB shared budgets. The main `common/config.worktree` and the
registration's fallback `packed-refs` now retain content evidence as well.
Regression coverage checks denial without external admission and in-place
reference/config changes. Locked Pull tests pass 90/90 and all-target/all-feature
Clippy passes with warnings denied. Independent review identified and confirmed
the fallback packed-refs fix, then found no new blocker. Missing-relative target resolution, object probes, child transport,
integration, runner stage capacity and final delivery remain open; production
Pull stays disabled and T7 remains Active.

Linked-worktree target admission checkpoint (2026-10-17): retained pointer
bytes now produce raw target Read facets, and configuration discovery returns
for fresh admission before target identity capture. Binding first checks all
target grants and compares actual canonical resolution with each approved
normalized path. Target identities remain in revalidation. Absolute gitdir
backlinks remain text-only, matching Git 2.54; only relative backlinks trigger
target resolution. New regressions cover old-frontier denial, wrong mappings,
same-path replacement, and absolute backlinks to pruned checkouts. Pull-focused
tests pass 88/88; all-target/all-feature Clippy passes and independent review
found no new blocker. External common
reference descendants, missing relative targets/forgiving resolution, and
remaining native-probe reads still require implementation before activation.
Production Pull remains disabled; this is not T7 or handoff completion.

Linked-worktree pointer evidence checkpoint (2026-10-17): the admitted
registration files `common/worktrees/<id>/gitdir` and `commondir` now retain
bounded contents in the existing shared 4096-entry/4 MiB reference evidence
budget. These are pointer bytes, not symbolic refs. In-place edits invalidate
the plan. Inventory tests pass 13/13 and all Pull-focused tests pass 86/86;
all-target/all-feature Clippy passes. Independent incremental review found no
new blocker. Git 2.54 worktree.c/setup.c confirm distinct trimming/resolution
rules; target resolution/read admission remains open and native child probing
must not be activated until that authorization is implemented. No production
Pull activation or T7 completion is claimed.

Linked-worktree graph inventory checkpoint (2026-10-17): child graph discovery
now includes `worktrees` under the existing exact-path frontier and 4096-entry
limit. Registered worktree HEAD and refs contents join the shared 4 MiB reference
evidence budget. Regression coverage requires fresh descendant admission and
rejects in-place HEAD/private-ref changes; inventory tests pass 13/13. A real
Git 2.54 fixture confirms `rev-list --all` includes another worktree's detached
HEAD while `--single-worktree --all` excludes it. This closes a prerequisite
read-inventory gap only; no child object probe or production Pull is enabled.
The full Pull-focused suite passes 86/86; all-target/all-feature Clippy and both
governance validators pass. Fetch succeeded with no new origin/main commits
requiring rebase (HEAD is 106 commits ahead before this stage's commit).
Independent review found no new blocker in this inert frontier increment, but
identified an explicit native-probe activation blocker: linked registration
`gitdir` and `commondir` contents must be bound and their resolved targets
restricted/admitted before invoking Git's worktree/ref backend. HEAD/refs
evidence alone does not authorize those pointer resolutions. Do not activate
native child probing on the strength of this checkpoint.

Windows path identity implementation checkpoint (2026-10-17): maintainer accepted
the Windows/Git-only same-file 1.0.6 direct dependency; ADR-089 records its exact
scope and limitations. PathIdentity now retains live handles for leaf/ancestor
volume+file-index comparisons with panic-guarded capture. The configured child
worktree binding is implemented, and replacement regressions are explicitly
selected in Windows CI. Locked all-feature tools tests pass 790/790 outside the
sandbox (`/tmp/talos-t7-identity-all-tests.log`). Windows GNU library cross-check
passes (`/tmp/talos-t7-windows-identity-check.log`), with an existing unused
run_host_command warning; this is compile evidence, not Windows runtime proof.
Independent incremental review found no new blocker. ReFS 128-bit identities,
RegularEvidence's distinct Windows leaf protection, aggregate retained handle
resources, native Windows runtime/sharing behavior and original T7 delivery
gates remain residuals. The earlier pending-dependency decision is superseded;
no all-filesystem guarantee or production Pull activation is claimed.

Final stage validation: governance and collaboration validators both pass with
zero warnings. The configured-worktree fixture now writes its path through
`git config` so Windows backslashes are quoted correctly; its focused locked
test passes after that adjustment. Windows GNU test cross-check could not finish
because this host lacks `x86_64-w64-mingw32-gcc`
(`/tmp/talos-t7-windows-identity-tests-check.log`); native Windows CI remains
required, and library cross-compilation alone does not validate test execution.

Pull graph boundary evidence checkpoint (2026-10-17): the admitted child
inventory now retains shallow and info/grafts contents alongside reference
evidence, sharing the existing per-inventory 4096-entry/4 MiB budget. Missing
paths remain bound by absence or parent listings without opening unadmitted
descendants. A regression covers present/absent variants of both paths and
in-place modifications. Locked inventory tests pass 12/12; the configured
worktree identity regression and all-target/all-feature Clippy also pass.
Independent incremental authorization review found no new blocker. These checks
include the still-uncommitted configured-worktree identity change; its Unix-only
replacement assertion is now explicitly platform-scoped. The Windows identity
decision, object probing, recursive execution, integration, complete runner
budget evidence and final delivery gates remain open. Production Pull remains
disabled; T7 and all subsequent tasks remain incomplete. Fetch confirmed no
new origin/main commits requiring rebase at this checkpoint.

Pull child remote discovery checkpoint (2026-10-16, `2edba07e`): child
configuration retains declaration order and reuses the shared fetch destination
resolver (first URL, insteadOf longest-prefix/tie semantics). Local destinations
request Read path facets; network destinations request Network facets. No source
is opened or contacted by this inventory. Configuration tests pass 25/25 and
all-target all-feature Clippy passes. Selecting the child's default remote is
still hardcoded to origin; branch remote/refspec binding, source/transport
effects, object admission, recursive fetch and integration remain open.

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

Network continuation checkpoint (2026-10-16, `05668643`): a real loopback Git
daemon now exercises the implemented parent fetch through execute_stage. All
three fresh-read outcomes are covered: approval reaches integration selection;
absent proof and replayed fetch proof refuse. The continuation has no Write or
Network facets; fetched metadata is retained and no index is created. The
fixture explicitly configures upstream tracking so FETCH_HEAD contains a merge
candidate (its initial missing-upstream run correctly refused selection).
Locked all-feature Pull tests pass 56/56 and all-target all-feature tools Clippy
passes with incremental compilation disabled. These are tools-layer unit
fixtures, not real Agent/MCP evidence; production activation remains disabled.
Recursive child planning/execution, integration writes, cancellation/budget and
runtime matrices, independent review, full preflight and publication remain open.

Activation correction (2026-10-16, `240f322f`): ADR-089 explicitly requires
recursive execution, runtime evidence and independent review before production
fetch activation. The earlier parent-fetch stage enabled that path prematurely;
production now refuses before spawning fetch while unit fixtures retain the
real execution path. Locked all-feature tools check passes. This gate must only
be removed with the outstanding activation evidence, not merely passing unit
tests. No production Pull delivery or remote publication is claimed.

Post-fetch selection checkpoint (2026-10-16, `fb69814e`): the fresh admitted
Read stage now rebuilds the bounded metadata inventory and FETCH_HEAD evidence
under the retained location guard, then selects merge/rebase, unborn or moved
HEAD reconciliation using production read-only probes. It creates a new plan
identity but does not yet grant integration Write or execute that selection.
A real local-fetch fixture proves obsolete evidence fails, admitted recapture
succeeds, and later FETCH_HEAD tampering invalidates the new evidence. Locked
all-feature Pull tests pass 55/55. All-target all-feature tools Clippy passes
with CARGO_INCREMENTAL=0 after the first attempt failed writing its cache with
ENOSPC; previously authorized incremental cache cleanup enabled the retry.
The first fixture attempt lacked an initial commit and was corrected before
the passing run. Successful network-fetch continuation coverage, recursive
child discovery/approval, integration effects/execution, independent review,
full current-source preflight and publication remain open. I300/T7 stays Active.

Parent fetch execution checkpoint (2026-10-16, `747d0a84`): PullFetch now invokes
the owned command runner after retained evidence, metadata Write, transport and
hook checks. Nonzero exits report possible partial effects; successful fetch
creates a new plan identity with Read/Execute facets only. PullPostFetch still
refuses integration explicitly, retaining fetched metadata. A local fake-peer
regression exercises actual fetch failure and passes; locked all-feature tools
check passes; locked all-feature Pull regression passes 54/54. Successful-fetch continuation evidence,
post-fetch inventory capture, child recursion, integration, independent review
and final publication remain open. This is a local stage, not T7 completion.

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

Pull legacy-remote checkpoint (2026-10-16, `4fee02d5`): Pull rejects
legacy `.git/remotes` and `.git/branches` destination files before configuration
discovery and after fetch planning. Locked all-feature Pull tests pass 49/49;
tools all-target all-feature Clippy, formatting and both governance validators
pass. Sandboxed standard preflight failed with 749 tools tests passing and one
process-observation `PermissionDenied`; log
`/private/tmp/i300-rebased-legacy-preflight.log`. Approved outside-sandbox standard
`release_preflight.sh` passed against source commit `4fee02d5`: 750 tools tests
passed, the external runtime fixture passed, and the final log reports
`release preflight: passed` at
`/private/tmp/i300-rebased-legacy-preflight-approved.log`. The sandbox-only
process-observation failure is environmental. Separate commit `87340529` updates
gix release tracking only. Production Pull fetch/integration, independent review,
Agent/MCP evidence and publication remain open. I300/T7 remains Active; later
tasks remain queued. Keep independently verifiable implementation and
documentation stages in separate commits, as requested by the maintainer.

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

Current Pull helper-admission checkpoint (2026-10-15, `093edd1a`): selected
remote upload-pack/VCS helpers and active automatic-maintenance external
effects now refuse before fetch planning. The locked all-feature
`talos-tools` Pull library selection passes 43/43. `origin/main` is
`0ef5a532`; the requested rebase reported up to date. Production fetch and
integration still refuse, and current-head full preflight/review/publication
remain open. T7 stays Active; later serial tasks remain unactivated.

Current initialized-child validation (2026-10-15; unpublished): the approved
standard `release_preflight.sh` completed outside the sandbox with exit 0
(session 8083). It passed governance and claims validation, the full workspace
test suite including the 699-test `talos-tools` batch, MCP Git transport tests,
doctests, and both external runtime SDK fixture runs; the process-observation
test `prepared_local_push_runs_all_eight_admitted_stages` also passed when run
outside the sandbox with all features (session 52128). The earlier sandbox-only
`PermissionDenied` result is environmental evidence, not a product failure.
T7 remains Active; linked/ancestor compatibility, Pull/Push residuals,
platform evidence, independent review, and publication are still open.

Current Pull contract note (2026-10-15): historical entries below that describe
`git:pull-repository-metadata` as proposed or unanswered are preserved session
history and are superseded by the maintainer's acceptance recorded in ADR-089
and the current I300 checkpoints above. Acceptance does not claim fetch or
integration delivery; those production stages and their evidence remain open.

Current Pull transport-admission checkpoint (2026-10-15; unpublished): resolved
credential-free built-in `git://` fetch destinations now enter PullFetch as
independent Network(Remote) facets, derived from retained Git configuration.
Unsupported destination schemes remain fail-closed at dispatch. The focused
Pull preparation selection passes 19/19 and locked talos-tools all-feature check
passes. This binds the network approval target but does not spawn fetch or
authorize local sources, helpers, credentials, redirects, or integration.
Production fetch/integration, MCP Pull coverage, failure/cancellation evidence,
independent review, current full preflight and publication remain outstanding;
T7 remains Active.

Current Pull hook-boundary checkpoint (2026-10-15; unpublished): Pull now
derives `reference-transaction` and `pre-auto-gc` hook paths from the retained
configuration and requires exact Read admission plus regular-file evidence at
the PullFetch boundary. Executable hooks and configured fetch helpers fail
closed; no hook or fetch process is started. Pull focused tests and locked
tools checks remain required after this source change. This does not deliver
production fetch/integration or close T7.

Current Pull transport refusal regression (2026-10-15; unpublished): unsupported
HTTPS destinations remain visible as exact Network(Remote) facets, but PullFetch
refuses before command construction or FETCH_HEAD mutation. Executable-hook and
unsupported-transport regressions bring the focused Pull selection to 21 passing
tests. This improves approval visibility without authorizing helpers, credentials,
redirects, production fetch or integration.

GIT-001 release-crate recheck (2026-10-15): published gix 0.89.0 source has
network-gated fetch, merge primitives, linked-worktree operations, and a
clone-only `fetch_then_checkout`; these APIs do not provide Talos-equivalent
pull reconciliation or branch-switch orchestration. Talos stays locked to
0.87.1. The GIT-001 owner records exact source paths, feature/MSRV and breaking
API constraints. No dependency upgrade or Git-removal date is claimed.

Current initialized-child validation (2026-10-15; unpublished): real MCP Checkout
matrix passes 40 Unix scenarios (7972, `/private/tmp/i300-initialized-mcp.log`),
including initialized-child allow/deny/replay and target-configuration cases.
Initialized child plus target configuration plus old-child deletion exceeds the
unchanged eight-stage bound; both policy modes refuse before mutation and retain
branch/child state. Production check-attr command construction now has a real Git
ancestor-prefix regression, passed 37219,
`/private/tmp/i300-child-filter-prefix.log`; independent focused re-review finds
no new blocker for the fix. Latest tools all-target Clippy passes (59178), and
both governance validators pass with zero warnings (26115/44453). Sandboxed full
tools library result was 343 pass / 1 process-observation PermissionDenied; this
did not establish a product failure. The intermediate outside-sandbox preflight
28347 (`/private/tmp/i300-initialized-current-preflight.log`) is superseded by
the passing session 8083 recorded above.
Linked/ancestor full behavior, remaining Checkout/Push/Pull and final publication
remain open. T7 remains Active.

Startup-selection and external-worktree checkpoint (2026-10-15; unpublished):
the temporary-borrow compilation error is fixed. Raw startup selection now
matches native Git for stale private configuration, commondir presence,
worktreeConfig override, bare and missing repositoryFormatVersion cases; two
focused tests pass in `/private/tmp/i300-child-layout-native-rules.log`.
Child discovery roots and effective worktree sources retain separate evidence;
real native/prepared Checkout preserves a tracked file and both sentinel roots
with external core.worktree. A configured external clean filter is refused without
execution, and raw-target Read denial still preserves the parent branch and child.
The strengthened fixture passes in `/private/tmp/i300-child-external-parity.log`
(69259). Earlier child suite passes 25/25 and Checkout suite 47/47; those broader
runs predate the final check-attr correction, which explicitly binds gitdir,
effective worktree and cwd to prevent discovery-prefix path confusion found by
independent review. Fresh broad validation, final review, linked/ancestor layouts,
MCP denial/replay combinations and the full current-source gate remain required.
This supersedes the conservative both-target compatibility residual below; T7
remains Active and all later tasks remain unactivated. No completion SHA exists.

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

# Next Task: Session Todo Handoff

## Proposed Pull Auto prerequisite sequencing — 2026-10-23

The I300 governance amendment moves only shared prepared-command Auto assessment
and native Pull admission ahead of T7 activation, resolving the dependency cycle
with T8. It takes effect only after the claim amendment merges into main.
Preserve all T1–T8 requirements and historical ordering below; the remaining
PERM-007-H full built-in coverage follows I300 and is not activated or complete.
I300 records scope, tests, API adaptation, independent review and native Windows
gates. No production activation or successful Auto execution is claimed here.


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

Next concrete T7 gap (2026-10-15): native nonrecursive Checkout succeeds when a
new gitlink meets an existing nonempty directory or initialized child `.git`,
preserving sentinel/metadata. Both isolated fixtures exit 0; current structured
implementation refuses them. Continue with admitted new-child discovery and
effect binding, not a blanket refusal removal. Empty-directory coverage below
does not establish full new-gitlink parity.

Latest verification (2026-10-15): empty-directory Checkout public MCP matrix
expanded to 26 cases and passes (89457), including exact Read denial with Write
allowed, onbranch target config and simultaneous old-gitlink deletion. Unix
late symlink replacement refuses with referent preserved (71549); MCP all-target
Clippy passes (73082), and independent fixture review finds no blocking issue.
Only tests/docs changed since full production preflight 33389. T7 remains Active;
configuration-link and Pull proposals still lack acceptance in ADR-089, and all
other recorded transport/platform/publication residuals remain open.

Current final local validation (2026-10-15): empty-directory approved full
preflight session 33389 exited 0; its log
`/private/tmp/i300-empty-directory-preflight-approved.log` ends
`release preflight: passed`. It covers the new exact-Read stage, workspace
checks/Clippy/tests, doctests and both external Runtime SDK modes. Independent
review has no blocking finding; governance/claim validators pass with zero
warnings. Historical running entries below are superseded. T7 remains Active;
no implementation publication/completion SHA or later-task activation is claimed.

Latest production checkpoint (2026-10-15; unpublished): existing ordinary empty
directories may become new target gitlinks after a separate exact-Read discovery
stage. Directory identity/emptiness and final Read retention are revalidated;
old discovery proof cannot authorize mutation. Checkout 46/46 (71610) and
locked all-target Git Clippy (82271) pass; strengthened focused assertions pass
(29074), and final independent review finds no remaining blocking finding.
Full standard preflight is running as approved session 33389, log
`/private/tmp/i300-empty-directory-preflight-approved.log`; poll this handle
before restarting. This production edit postdates full preflight 1885;
that earlier gate is not current-final validation. T7 remains Active and the
serial queue remains unactivated. See I300 for residuals and next verification.

Authoritative current validation result (2026-10-15): session 1885 exited 0;
`/private/tmp/i300-socket-preflight-approved.log` ends `release preflight: passed`.
Current socket/FIFO production changes pass full locked workspace validation,
doctests and both external runtime SDK modes. Historical live-handle entries
below are superseded. T7 remains Active; original functional residuals and final
publication gates remain incomplete, and later serial tasks remain unactivated.

Worktree-space audit (2026-10-15): `git worktree list --porcelain` reports only
the current `feat/i300-git-repair` checkout at `55fef9f9`. There is no registered
unused worktree to remove. Filesystem free space is approximately 5.6 GiB; no
branch, source directory or build cache was deleted by this audit.

Resume current-source gate (2026-10-15): approved socket-version standard
preflight is live as session 1885, log
`/private/tmp/i300-socket-preflight-approved.log`. Dedicated socket/FIFO/link
type-replacement test passes (29377); MCP Clippy passes (94769); governance and
claims pass zero warnings (84557). Poll 1885 rather than rerunning the gate.
Config-link metadata and Pull metadata contracts remain explicitly proposed,
unaccepted and disabled; prior Push-specific acceptance does not cover them.

Latest continuation (2026-10-15): FIFO standard preflight 9204 passed completely,
including SDK fixtures. Subsequent Unix socket deletion support passes Checkout
45/45, dedicated no-connection/drift test and Clippy; independent security review
finds no new blocking issue. Exact per-entry Read/Write remains mandatory.
MCP socket matrix passed as 93230, log `/private/tmp/i300-socket-mcp.log`.
The full preflight predates socket production changes, so final validation is
still required. All original residuals and the full serial queue remain open.

Current standard-gate handle (2026-10-15): 13206 ended 101 solely at sandbox
process observation EPERM (tools 691/692). The identical approved outside-sandbox
rerun is session 9204, `/private/tmp/i300-fifo-preflight-approved.log`.
Resume 9204; 13206 is terminal. Current-source full validation remains unproven.

FIFO public-path follow-up (2026-10-15): the real MCP Checkout matrix passes
with admitted FIFO deletion, denied FIFO preservation and replay refusal (23059),
`/private/tmp/i300-fifo-mcp.log`. Standard preflight 13206 is still running at
`/private/tmp/i300-fifo-preflight.log`; resume that handle rather than restarting
on an observation timeout. GIT-001 also records a live 2026-10-15 upstream recheck:
gix remains 0.89.0 and full host-Git replacement blockers remain.

Latest FIFO checkpoint (2026-10-15): I300 now supports Unix FIFO entries in
nonrecursive Checkout child deletion through exact Read/Write admission without
opening the pipe. Checkout passes 44/44 (78586); inode/type replacement regression
passes (30551); Clippy passes (25346); independent security review reports no new
blocking issue. Earlier full preflight predates this production change. Links,
Pull, remaining transports/platforms and publication gates remain outstanding;
T7 stays Active and no later queue item is activated.

Current T7 standard validation checkpoint (2026-10-15): complete standard
`release_preflight.sh` passes outside the sandbox (29306), including tools
691/691 and both SDK consumer modes; log
`/private/tmp/i300-current-preflight-approved.log`. The prior sandbox run failed
only at process observation EPERM. Current code includes ordinary-directory
`..` target includes and real target resource-limit admission regressions.
Symlink configuration metadata authorization remains proposed, not accepted;
Pull and other original residuals still prevent T7 completion. I300 remains
Active, later queue items unactivated, with no implementation publication SHA.

Missing hook-directory checkpoint (2026-10-15): optional directory-absence
evidence now removes the functional residual recorded below, while required
gitdir contexts still reject missing paths, including optional→required upgrades.
Prepared tests exercise fresh exact admission and absent-hook capture through
the production discovery stage, ending at the retained final effect-binding
refusal. Checkout passes 40/40, final target-config passes 25/25 and Git all-target
Clippy passes; see I300 owner for logs and review details. Ancestor-checkpoint
governance and claims passed zero warnings. Final mutation, Pull execution and
all original T7 delivery gates remain open; no queue advancement is claimed.

Late-hook ancestor follow-up (2026-10-15): the owner I300 now records closure of
the static parent-symlink recapture gap. Parent directories receive exact Read
frontiers and identity guards before hook leaf capture/revalidation. Checkout
40/40 and Git all-target Clippy pass, logs
/private/tmp/i300-late-hook-guard-{checkout,clippy}.log; independent review confirms
the fix. Missing target hook directory support remains a functional residual,
alongside final effect binding, Pull execution and all original T7 delivery gates.
No final mutation, publication, completion or serial advancement is claimed.

Late checkout hook frontier checkpoint (2026-10-15): see the owner I300 checkpoint.
Target discovery now emits fresh exact Read facets for late traditional-hook
candidates; all tokens are checked before capture. Source non-null hooksPath
cache semantics are preserved. Captured hook evidence joins revalidation and
replacement/deletion protection, while final effect binding still refuses.
Checkout tests pass 40/40 and Git all-target Clippy passes, logs
/private/tmp/i300-late-hook-final-{checkout,clippy}.log. Ordinary-chain frontier
coverage includes the shared stage budget and final-stage reservation; missing
Read and later hook creation refuse. Final target mutation, Pull execution and
all original T7 delivery gates remain open; the serial queue is unchanged.

Previous reload-validator checkpoint governance and claims validation both
completed successfully with zero warnings, logs
/private/tmp/i300-reload-{governance,claims}.log.

Checkout reload validator checkpoint (2026-10-15): fixed the qualified
ConfigState call and verified the new ordered-record validator. Target promisor
true/implicit/invalid values, partial-clone filters and partial-clone extensions
refuse; false promisor is accepted and status.compareBranches uses the final
value, rejecting a final implicit value. This check preserves source-cached
effects and does not enable the final target mutation continuation.
Focused test passes; checkout-filtered tests pass 37/37 and Git all-target
Clippy passes, logs /private/tmp/i300-reload-{validator,checkout,clippy}.log.
Next complete cache-specific target auxiliary effects and fresh final admission;
Pull execution and original T7 delivery gates remain open. No completion or
publication evidence is claimed and the serial queue is unchanged.

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

Remote-guess source checkpoint (2026-10-15): checkout now binds selected remote
and mapped reference as well as OID; final execution compares that source then
revalidates. A same-OID two-remote fixture verifies distinct retained sources
without creating a branch. It does not directly test stale-source execution.
Checkout 30/30 (33878) and Git all-target Clippy (98472) pass, logs
/private/tmp/i300-tracking-source-{checkout,clippy}.log; diff check passes.
Independent read-only review found no new flaw or authority expansion. Actual
tracking writes, source/target cache partition and final target effects remain
unimplemented; target mutation refusal remains. Next: predict tracking behavior
from native Git 2.54 branch.c using retained remote/reference and normalized new
name; preserve original argv and exact target Read admission. Pull and original
publication gates remain open. Standing gix tracking in GIT-001 was refreshed:
published latest remains 0.89.0 versus locked 0.87.1, no Git-removal date.
Earlier status saying research was unfinished was inaccurate: previous tracking
records already existed. No completion or publication is claimed.
Governance and Collaboration Claim validators exit 0 (55213/39722), logs
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

Next T7 capability: complete ordered target-config evaluation and full effect
binding on the newly admitted source evidence, including conditional/hasconfig
lookahead, depth/cycles, source/target cache partition and tracking config writes.
Extend component evidence for parent traversal/symlink roots without normalizing
away dependencies or following an unadmitted referent. Preserve the original
creation scope and eight-stage bound; do not force tracking off. The target Read
phase is implemented, but potential-source discovery alone cannot enable mutation.
No new named capability decision is needed for ordinary exact Path Read.

Commit maintenance correction checkpoint (2026-10-14): independent review
confirmed that native automatic maintenance can execute `gc.recentObjectsHook`
via pack-objects. Parent Commit now refuses the configured command while
maintenance is enabled, preserving inactive configuration when disabled.
Focused regression and Commit suite 14/14 pass; Git all-target Clippy passes;
independent review found no concrete gap. Logs:
/private/tmp/i300-commit-recent-hook-enabled.log,
/private/tmp/i300-commit-recent-hook-suite.log,
/private/tmp/i300-commit-recent-hook-clippy.log. Native hook reachability is based
on Git 2.54 source, not an auto-GC trigger in this regression.
Previous-local-branch standard preflight session 56638 exited 0 and its log ends
with release preflight: passed, but predates this correction; a final-source gate
remains required.
Post-correction standard preflight 70906 exited 0, and
/private/tmp/i300-commit-recent-hook-standard-preflight.log ends with
release preflight: passed, including both SDK consumer modes. Separate governance
and Claim checks pass with zero warnings (logs under the same commit-recent-hook
prefix). This proves the local correction checkpoint, not full T7 completion.
Governance/claims session 93144 exited 0. T7 and the full serial queue remain open.

Current deletion update (2026-10-14): I300 now supports exactly admitted child
deletion inventories for nonempty gitlink→ordinary checkout, including external
gitdir layouts. Checkout 26/26 and MCP 13-case allow/deny/replay/depth matrix pass;
missing descendant Write and late-entry refusal regressions pass. Effective
configuration deletion and symbolic/special entries still refuse. Owner Current
Delivery State contains logs and remaining gates; historical blanket refusals
below are superseded. Standard preflight session 49959 exited 0, with
`release preflight: passed` in `/private/tmp/i300-deletion-standard-preflight.log`.
T7 and the original serial queue remain incomplete.

Current Checkout update (2026-10-14): the I300 Current Delivery State table
supersedes historical present-directory refusals below. Empty uninitialized
gitlinks can restore ordinary targets with parent/identity/emptiness revalidation;
Unix tracked symlinks can become uninitialized gitlinks without modifying their
referents. Checkout tests pass 24/24 and all-target Git Clippy passes; independent
review confirmed the read-order correction and symlink effect binding. Nonempty
or initialized child deletion, recursive behavior, production Pull and final T7
delivery gates remain open. See I300 for current evidence and platform limits.

Initialized-child deletion residual (2026-10-14): native Git can remove untracked or initialized child contents when converting a present gitlink directory to an ordinary file, even with nonrecursive settings. Exact-path Read/Write evidence currently retained by Talos does not safely model that recursive deletion set, so prepared Checkout continues to refuse present child directories and metadata. The refusal regression passed (59465); no permission widening or silent deletion was introduced.

Initialized-child type-transition boundary (2026-10-14): native Git experiments show gitlink→ordinary-file checkout can delete untracked files and initialized dirty child repositories under nonrecursive settings. Prepared Checkout therefore continues to refuse present directories/child metadata; a regression now verifies refusal preserves parent branch/index and child HEAD/dirty bytes (59465), /private/tmp/i300-initialized-kind-refusal.log. This documents the required deletion-effect boundary rather than widening authorization.

MCP prepared Git regression complete (2026-10-14): full `git_prepared` integration suite passed 6/6, covering checkout, commit, local push and network push approval/denial/replay; test log /private/tmp/i300-mcp-git-prepared-full.log. Test target Clippy passed, /private/tmp/i300-mcp-git-prepared-clippy.log. This strengthens real admission-chain evidence only; production Pull, unsupported transports, Windows execution and final publication remain open.

MCP Checkout integration evidence (2026-10-14): `real_checkout_transport_denial_success_and_replay` now covers ordinary-file and gitlink target switches under allow/deny permissions, plus replay refusal; the four-case real MCP transport test passed (69642), /private/tmp/i300-mcp-gitlink.log. This confirms the new type-transition path through the shared permission gate, not only direct tool tests. Pull mutation and final T7 delivery gates remain open.

Current audit checkpoint (2026-10-14): working tree passes `git diff --check` and `cargo fmt --all -- --check`. The authoritative owner table remains T7 Active, with T8/T2/T3/T4/T5/T1/T6 unactivated. ADR-089 Pull repository metadata is still explicitly proposed and disabled; `Phase::PullFetch` still rejects execution. No completion status is inferred from preparation tests or preflight alone.

Current-source standard preflight passed (2026-10-14): 69747 exited 0, log /private/tmp/i300-checkout-current-preflight.log. This includes latest Checkout source changes, workspace tests and SDK consumers. Windows guard CI step has local YAML/PowerShell parse evidence only, not Windows execution. T7 remains Active with production Pull, remaining transport/submodule behavior and exact-head publication gates open. Current owner table takes precedence over historical running-state entries below.

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

Status: Serial execution active; I299/C0 implementation merged and owner closure recorded; all seven original deliverables remain incomplete.
Responsible recipient: Codex under maintainer direction; product claims remain separate.
Source: Maintainer requested a durable handoff and explicitly stopped further implementation in this session.
Baseline: local `main`, HEAD `63c3ab6` (merge PR #697). Recheck HEAD and both staged/unstaged diffs before resuming.
Completion evidence: None for this long task. This document is a handoff, not an effective Collaboration Claim or implementation authorization.

## 2026-10-11 Receiving Checkpoint

Current standard preflight (2026-10-14): sandbox-external `release_preflight.sh` running in session 48514 with CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0; log /private/tmp/i300-current-standard-preflight.log. Installer/governance phases progressed into workspace compilation; no final pass claimed. Poll this live handle, do not restart from missing output. Existing git_plan_review reviewer requested to audit current Pull prefetch boundaries read-only. Pending Pull metadata decision and production fetch/integration remain open; no task completion or publication evidence.

Order-file runtime evidence (2026-10-14): shared config suite passes 17/17 and Git Clippy passes (84571). Real prepared cleanliness fixture now configures an external diff.orderFile, verifies its exact Read facet and retained file evidence, then changes its contents and confirms revalidation fails; focused matrix passes (96819). Index preservation assertions remain. These results close the order-file-specific regression, not pending metadata authorization or production fetch/integration.

Order-file path parser test (2026-10-14): direct config-state fixture covers workdir-relative and absolute diff.orderFile paths. NUL case exposed missing explicit rejection in shared configured_path; added rejection before path conversion. Focused retry 60003 at /private/tmp/i300-orderfile-test.log; initial fixture setup and NUL assertion failures are superseded only after inspecting that result. Full shared-config regression/Clippy remain required for this parser change. No pending permission decision inferred.

Pager/audit claim correction (2026-10-14): Pull probe helper and clean diff command now explicitly pass --no-pager. Previous text describing a frozen '--no-pager environment' was inaccurate; a command flag is not an environment property. Removed the unsupported assertion that orderFile was the only remaining configuration-backed read. Pull tests pass 36/36 (14198); exhaustive effect review and original production/delivery gates remain outstanding.

Diff external-effect audit (2026-10-14): Pull clean probes explicitly use `--no-ext-diff`, `--no-textconv`, `--no-renames`, `diff.autoRefreshIndex=false`, `GIT_OPTIONAL_LOCKS=0`, and a frozen environment. Configured `diff.orderFile` is exact-admitted and evidence-bound; this is not an exhaustive proof of all Git configuration effects. Explicit pager suppression was added in the subsequent correction. Git-enabled Clippy and governance/claim validators pass (47575). Full tools Git suite previously passed unsandboxed (98071); final production Pull/fetch/integration and security review remain open.

Pull diff.orderFile admission correction (2026-10-14): native `git diff --numstat` can read configured `diff.orderFile`; Pull policy discovery now resolves this path from admitted config, adds it to the fixed attribute/evidence set, and requires fresh exact Read before clean comparison. Pull suite remains 36/36 (73774), diff check passes. Added path is bounded by existing config path rules; content/filter and production mutation requirements remain open.

Permission regression validation (2026-10-14): full locked talos-permission tests pass (75464): 145 unit tests, three integration tests and one doctest. Proposed Pull metadata remains rejected while accepted capabilities retain their tests. Fixed EVOLUTION lesson placement/index and named the concrete regression test. Pending maintainer decision is unchanged; no production Pull mutation or overall completion claimed.

Symbolic-policy admission evidence (2026-10-14): strengthened Alias->Topic fixture verifies the newly discovered tracked source is absent from old exact tokens and no source/attribute content is captured at that continuation; fresh admission then completes preparation. Focused matrix passes (81809); production-fix Clippy passed (78388) before final assertion additions. This closes the specific late-policy admission test gap, not production fetch/integration or full delivery gates.

Late policy attribute correction (2026-10-14): direct HEAD can name an alias whose policy differs from the fully resolved branch. Final prefetch policy now discovers fixed common/system/global attribute paths as well as source ancestors, requesting fresh exact admission before capture when newly required. Real Alias->Topic fixture with Alias rebase=false and Topic rebase=true passes in the cleanliness matrix (30039); captured attribute count matches discovered paths. Pull suite after production fix passed (69028); latest test addition still needs final Clippy. Pending Pull metadata decision and production fetch/integration remain unchanged.

Decision-receipt correction (2026-10-14): an asynchronous input tool returned `accepted: true` as submission status, not the maintainer's answer. A provisional Pull metadata compiler change was reverted. Added `proposed_pull_metadata_capability_remains_rejected` (permission test passed) and recorded the reusable lesson in EVOLUTION.md. Governance and Collaboration Claim validators passed (32219). ADR-089 remains proposed; no Pull metadata Read/Write or fetch/integration mutation is enabled.

Policy-scoped source discovery (2026-10-14): early source/attribute discovery now checks exact HEAD Read and obtains non-recursive symbolic HEAD, then applies admitted configuration policy. Only rebase without effective autostash requests prefetch clean-source/attribute facets. Later full graph/checkout/index checks remain. Pull suite 36/36 and Git Clippy passed (74753); sparse budget remained passing (65162). Added explicit autostash assertions for no tracked-file Read facet, no captured sources/attributes; focused run 48578 at /private/tmp/i300-policy-facets.log. Symbolic branch chains are conservatively handled by direct HEAD policy discovery; broader chain semantics need review. No mutation or Pull metadata capability enabled.

Autostash precedence runtime validation (2026-10-14): prepared cleanliness matrix now covers pull.autostash=true overriding rebase.autostash=false and pull.autostash=false overriding rebase.autostash=true; all eleven scenarios pass (17145) with index preservation. Early source-name admission is still conservative and requests paths even for policies that skip cleanliness; narrowing it requires a branch-bound, exact-admitted policy discovery without adding stages. This remains an explicit residual, not implemented narrowing. No production mutation capability enabled.

Full Git-enabled tools validation (2026-10-14): cargo test -p talos-tools --features git --locked passed outside the sandbox (98071; /private/tmp/i300-tools-git-full-unsandboxed.log), using compact debug/incremental settings. The sandbox run had 283 passes and one EPERM process-observation failure in prepared_local_push_runs_all_eight_admitted_stages (22239); unsandboxed retry exited 0. This is package/feature validation, not full workspace preflight or publication evidence. Production PullFetch still explicitly refuses because fetch/integration are unimplemented; metadata contract acceptance and original T7 delivery gates remain open.

Staged gitlink prefetch fixture (2026-10-14): added a staged gitlink using update-index --cacheinfo to the real prepared cleanliness matrix. All nine cases pass (36897); gitlink-only staging reaches prepared PullFetch without changing original index bytes. This validates the implemented ignore-submodules=all prefetch behavior, consistent with the earlier Git 2.54 source audit; it is not native end-to-end rebase/integration parity. Pull metadata decision, production fetch/integration, security review and all original task gates remain open.

Sparse real-history reservation evidence (2026-10-14): replacement-ref guard suite passed 36/36 and Git all-target Clippy passed (77756). Strengthened sparse fixture now records configuration and every discovery into actual PreparedStageHistory, then reserves fetch and a fresh integration identity under the retained profile; focused test passes (71056). This verifies stage/profile capacity for that fixture, not actual mutation-stage profiles or execution. Production fetch/integration, metadata contract acceptance, independent review and original delivery requirements remain open.

Early sparse replacement-ref guard (2026-10-14): inspection found ls-tree may consult replacement refs before full graph readiness. Early expansion now additionally requires refs/replace subtree completion (or absence proved by its admitted parent's listing) and packed-refs completion, with exact current tokens for every existing replacement/packed path. Sparse tracking six-stage fixture still passes (44328). Added replacement-descendant/absent-subtree drift test (23407). Latest governance and claims logs passed before this source/test change. Full latest suite/Clippy and independent security review remain required; production fetch/integration remain disabled.

Sparse listing validation (2026-10-14): parser now validates nonzero SHA-1/SHA-256-shaped hexadecimal IDs, bounded input and joined lexical paths; malformed mode/type/OID, unterminated listing, parent traversal and oversized output tests pass (66928). Path validation remains filesystem-free. ADR-089 Pull Repository Metadata Effects still explicitly reads proposed/not authorized; I300 and this handoff retain an unanswered earlier maintainer request. Do not confuse accepted Push receiver metadata with Pull authorization or re-request an already pending decision. Production fetch remains disabled pending authoritative acceptance and implementation/review. Full parser-change suite/Clippy/governance remain to run.

Sparse stage coalescing repair (2026-10-14): object-subtree readiness is now separate from full reference inventory completion. Only fully visited object roots with no pending descendants and exact current tokens for every object path permit early ls-tree discovery; descendant source/attribute facets coalesce with remaining reference frontiers. Real sparse tracking fixture passes the unchanged six-stage reservation (30162). Pull suite 34/34 and Git all-target Clippy passed (45623). Added frontier assertions ensure pending object children cannot report ready; focused test running in /private/tmp/i300-subtree-frontier.log. Security review, stress/config/submodule cases, final lifecycle/production fetch/integration and all original delivery gates remain open.

Sparse stage trace (2026-10-14): failing reservation test now records (stage, graph-ready, inventory-complete, facet-count): [(2,false,false,43),(3,false,false,56),(4,false,false,59),(5,false,false,60),(6,true,true,62),(7,true,true,62)] (21180). Next fix: allow tree-name discovery once object storage is fully inventoried and admitted, coalescing descendant facets with remaining deeper reference frontiers; never equate object-root Read with descendant Read. Six-stage reservation assertion remains intact. A diagnostic run with an eight-stage-only assertion passed (83528), but that weaker assertion was reverted and is NOT delivery evidence. No stage-budget repair claimed.

Sparse tracking stage-budget regression (2026-10-14): strengthened sparse fixture now includes origin/Topic tracking and asserts preparation reserves both fetch/integration stages. It FAILS (46281): preparation uses seven stages, above the six-stage reservation budget. Preserve the assertion and fix discovery coalescing; do not widen lifecycle bounds or claim sparse delivery. Fixture branch setup now uses branch -m Topic rather than assuming master. Prior sparse implementation Clippy passed (41153). Governance/claims validation launched in 53498 with logs /private/tmp/i300-sparse-governance.log and /private/tmp/i300-sparse-claims.log. All production fetch/integration and original closure gates remain open.

Sparse descendant preparation implemented (2026-10-14): after exact graph admission, sparse index tree OIDs are enumerated with bounded ls-tree -r -z; regular/symlink blob names are lexically validated, gitlinks omitted consistently with native clean checks. New descendants and ancestor attributes generate fresh exact Read facets before source capture. Real sparse fixture proves the old tokens lack the new descendant, the continuation has no captured sources, fresh admission reaches prepared PullFetch, and index bytes remain unchanged. Pull suite passed 34/34 (85523). This supersedes the earlier sparse refusal-only checkpoints. Still required: malformed/deep/large tree fixtures, shared lifecycle stage-budget coverage for sparse tracking repositories, object substitution/security review, latest Clippy/governance, and full fetch/integration implementation. The existing 16 KiB probe output bound remains and oversize errors refuse; no arbitrary-size support claimed.

Sparse control-file evidence checkpoint (2026-10-14): Pull fixed evidence now includes per-worktree `info/sparse-checkout`, including its absent state. Creating, changing, deleting or replacing it with a symlink invalidates the retained Pull evidence; new Pull suite passed 34/34 (47281). This closes the control-file admission gap but does not implement sparse tree expansion: mode-040000 index entries still require admitted object descendants and fresh source/attribute facets. No fetch/integration capability enabled.

Pull numstat edge validation (2026-10-14): extended the real prepared-flow cleanliness fixture with binary changes, tracked-file deletion and Unix executable-mode changes. All eight scenarios pass (2151), including original clean/touch/staged/dirty/autostash cases; original index bytes remain unchanged and no index.lock/FETCH_HEAD appears. Sparse expansion still requires an implementation that consumes admitted object storage, preserves bounded tree OIDs/names and returns new descendant source/attribute facets before reads. Existing checkout traversal cannot be adopted without auditing its discovery/read behavior. No sparse support or overall completion is claimed.

Sparse-index boundary fixture (2026-10-14): real cone sparse index with kept and excluded directories retains a mode-040000 entry in the pure decoder, and the prepared rebase discovery route refuses with the explicit sparse-directory object-discovery requirement (43727 passed). Fixture setup initially selected all available subdirectories and was corrected; those failed setup attempts do not count as boundary evidence. This tests the existing refusal only, not sparse support. Next implementation must expand tree OIDs from admitted object bytes and request each descendant source/attribute Read before content capture. Prior clean suite passed 32/32 and all-target Git Clippy passed (67029). T7 and all production fetch/integration requirements remain open.

Pull touch-only repair (2026-10-14): replaced quiet/exit-code probes with bounded NUL numstat, no-renames, no-ext-diff/no-textconv and diff.autoRefreshIndex=false plus GIT_OPTIONAL_LOCKS=0. Native experiments show numstat compares content without index writes, while quiet reports stale stat data as changed; mode-only changes produce nonempty 0/0 records. Real prepared-flow clean/touch/dirty/staged/autostash fixture now passes (88270), checks index bytes unchanged and no index.lock/FETCH_HEAD. Full Pull suite and all-target Git Clippy are running sequentially in 67029; logs /private/tmp/i300-clean-suite.log and /private/tmp/i300-clean-clippy.log. This supersedes the failing touch-only checkpoint but does not close sparse expansion, staged-gitlink/config variation coverage, independent review, production fetch/integration or original delivery gates.

Pull clean-probe regression (2026-10-14): new HEAD/index and index/worktree probes are provisional and NOT publication-ready. Real clean/touch/dirty/staged/autostash fixture fails on touch-only: ordinary git diff --quiet rewrites index despite GIT_OPTIONAL_LOCKS=0 (also reproduced independently); disabling diff.autoRefreshIndex prevents the write but reports touch-only as dirty. Preserve the failing test pull_rebase_clean_probes_preserve_index_and_touch_semantics and repair semantic equivalence without unauthorized repository writes. A temporary-index experiment was removed; production must not create ad-hoc files inside gitdir under Read admission. Native Git 2.54 require_clean_work_tree ignores submodule changes in both comparisons, so probes now use ignore-submodules=all. Earlier assumption that staged gitlink changes must be rejected here was incorrect. Clippy found chunks_exact_to_as_chunks; corrected, fresh Clippy still required. No fetch/integration enabled; T7 remains Active.

Pull filter refusal runtime evidence (2026-10-14): production check-attr change passed Pull suite (85325). Real Git fixture pull_rebase_filter_refuses_without_running_helper passed (15321): configured filter.probe.clean is refused before PullFetch; neither helper-ran nor FETCH_HEAD exists. This proves refusal without helper execution, not native clean-worktree comparisons or production fetch. Git all-target locked Clippy is running with compact build settings, log /private/tmp/i300-source-attribute-clippy.log. Remaining: native HEAD/index and index/worktree comparisons with optional locks disabled, touch-only and staged gitlink semantics, sparse expansion and independent security review. No original acceptance requirement is closed by this fixture alone.

Pull attribute capture checkpoint (2026-10-14): source snapshot retry 65412 passed; governance and claims session 76456 exited 0. Exact-admitted attribute paths now retain bounded RegularEvidence and parent identities, checked before/after content reads and revalidation. Added missing-file creation, content drift, leaf-link and parent-link tests; Pull suite passed 30/30 (64983). Subsequent production check-attr filter refusal is now added and under test in session 85325, log /private/tmp/i300-pull-filter.log. Native clean comparisons, sparse expansion, real filter-helper negative fixture and independent review remain required before publication; fetch/integration remain disabled. Root-to-leaf parent checks conservatively refuse symlink ancestors, including externally configured attribute paths. No claim of race-free containment.

Pull source snapshot checkpoint (2026-10-14): prepared prefetch now captures tracked source bytes and parent identities after exact-path admission for rebase clean-worktree inspection, and retains them for revalidation. New tests cover content drift, creation of a missing source and parent symlink replacement. This does not implement attribute capture, filter refusal, native clean comparisons, sparse expansion or fetch/integration. Initial focused test builds exited 101 on ENOSPC. After the already-authorized target/debug cleanup, free space was 8.4 GiB; retry uses CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 with cargo test -p talos-tools --features git --locked pull_. Session 65412 is running; poll it rather than restart, with output in /private/tmp/i300-pull-source-evidence-compact.log. No passing result or completion is claimed.

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

Pull prototype checkpoint (2026-10-13): test-only pull_state.rs now parses bounded
FETCH_HEAD merge targets; three tests pass (18597), including real multi-branch
fetch selecting configured upstream and excluding not-for-merge entries. It is
not connected to production; retained pre-fetch state/config policy and admitted
fetch/target-effect/merge-rebase stages remain implementation work. Git-enabled
all-target Clippy passes (90761); diff whitespace check passes. No build is live.

Latest validation/Pull checkpoint (2026-10-13): complete tools library passes
247/247 (6272) including corrected real maintenance cancellation. No build is
live. Pull implementation source review found pre-fetch fork-point/original HEAD
must be retained; fetch may update current HEAD before worktree reconciliation.
Preserve configured ff/rebase/autostash precedence, FETCH_HEAD merge selection,
unborn/multiple-head behavior and recursive child effects. See owner checkpoint
and upstream Git v2.54 builtin/pull.c (/private/tmp/i300-pull-2.54.c).
Next implementation is Pull pre-fetch evidence and admitted fetch continuation,
then fresh target-effect admission. Pull remains unimplemented, not Complete.
Old I299 governance worktree was removed after confirming clean state and no
open-file users; its branch and be35835a commit remain, releasing about 27 MiB.

Maintenance cancellation implemented (2026-10-13; supersedes below): the ninth
real local push scenario positively observes automatic pack-refs under the
receiver maintenance/receive-pack chain, cancels after ref update, confirms
cleanup and child exit, preserves the fixture lock and checks no late packed-refs
write after lock release. Test 85061 passes unsandboxed; 48356 was blocked by
sandbox ps restrictions. Independent review found a post-cancel observation
ambiguity: failed ps with empty stdout could pass. The assertion now accepts only
exit 1 with both streams empty (absent), or success with empty stderr and zombie
state; corrected test 72880 passes. Clippy 31460 passed before that assertion
correction. All builds are terminal; remaining scope is unchanged.

Latest cancellation checkpoint (2026-10-13): real local receiver cancellation
now passes after objects transfer while reference update waits on a fixture lock.
Cleanup is confirmed, the fixture lock is preserved, and releasing it causes no
late ref update. Receiver automatic loose-object maintenance also demonstrably
finishes before push returns (12219). This covers receiver cancellation and
maintenance completion separately; maintenance-phase cancellation remains open.
Independent reviewer git_plan_review confirmed this evidence boundary and proposed
blocking automatic pack-refs with packed-refs.lock after refs update. Git 2.54
source inspection confirms pack-refs auto triggers at 16 loose refs when no packed
refs exist; use at least 17 fixture refs, core.packedRefsTimeout=30000, and prove
the actual maintenance/pack-refs descendant starts before cancellation. Source
copies: /private/tmp/i300-gc-2.54.c and /private/tmp/i300-files-backend.c.
Current Clippy passed (67597); no build remains live. All accepted contracts
remain in force. Maintenance cancellation fixture is not yet implemented.

Local execution continuation (2026-10-13; supersedes disabled-transport notes
below): real eight-stage admitted local/file push passes for bare, normal,
linked and file URL receivers (29677). MCP success, Write denial and duplicate
request replay protection pass (4893); that build is terminal. Independent
review confirmed shell restrictions and rr-cache/MERGE_RR coverage, with no new
definite maintenance namespace omission. This remains partial candidate evidence:
cancellation, multi-destination partial effects, Pull, other transports, Windows
and full publication gates remain open. Full tools validation passed 247/247
(92388); all-target Clippy passed (47702). The expanded eight-stage test then
passed for two local pushurl destinations (5084), with neither written before
final admission. All these builds are terminal.
No additional maintainer decision is pending for the accepted metadata scope.
Partial local multi-destination failure is now verified (26449): first receiver
updates, divergent second receiver stays unchanged, and execution reports partial
effects. Governance/claims pass with zero warnings (22133/30900). Next resume item
is cancellation/descendant cleanup, followed by remaining transport and Pull work;
do not treat these local tests as complete security review or T7 closure.

Final-plan continuation (2026-10-13): local receiver discovery now produces a
fresh Mutation plan preserving sender facets and adding receiver Write/Execute.
Storage evidence is retained and checked at execution entry and immediately before
spawn; consumed proof tokens are retained for exact Write checks. Fixed receiver
command is wired into production push arguments, but local transport remains
disabled until the shell baseline and execution validation are closed. Tests
cover stale proof, missing Write and changed object storage for bare/normal/linked
layouts; full Git-enabled tools library passes 245/245 (7378). T7 remains Active.
Independent review confirms final-spawn evidence recheck and command wiring;
Clippy passes (29970), governance/claims pass (43655/94418). No build remains live.
Next is shell-baseline closure, then replacing the local transport rejection
with complete destination validation and exercising actual admitted push.

Latest storage continuation (2026-10-13): dedicated metadata Read and exact roots
are tested through real bare/normal/linked discovery chains; missing capability
or exact root refuses. Full Git-enabled tools library passes 244/244 (37939).
Storage now rechecks missing leaves and parent mapping; independent review
confirms the fix. Unix hardlinked regular metadata refuses (3/3 storage tests,
87272); final inode/nlink recheck subsequently passes all-target Clippy (33737)
and all three storage tests (41154). Independent review confirms five remaining local
push items: final sender/receiver Write plan, retained and revalidated storage,
production fixed receiver command/Execute facets, shell baseline, and actual
admitted mutation/failure/cancellation tests. No new authorization is needed for
the accepted metadata scope. Local mutation stays disabled; I300/T7 stays Active.

Receiver metadata contract checkpoint (2026-10-13): maintainer accepted the
dedicated git_push-only `git:receiver-metadata` Read/Write scope. Grant compiler
and permission regression now require matching absolute Path roots, a bound
absolute Git Command and push tool; ADR-089 records exclusions. Child evidence
now binds receive-pack and maintenance Git to the sender executable. Git source
review showed local transport removes configuration parameter environment, so a
fixed, shell-quoted receive-pack command carrying foreground maintenance settings
was added and tested. Mutation remains disabled until dynamic receiver writes,
quarantine/locks/reflogs, environment/startup effects and cancellation ownership
are fully bound and independently reviewed.

Latest verified continuation (2026-10-13): clean rebuild 15092 completed with
239/239 Git-enabled tools library tests passing. This includes the subsequent
main-worktree HEAD/config.worktree binding and sibling marker/common/backlink
verification, with relative backlinks deferred to a new Read wave before target
resolution. Independent reviews found no new definite blocker in those additions.
Governance/claim validators pass with zero warnings (87877/64556); diff whitespace
check passes. Git-enabled all-target Clippy passes with warnings denied (45407);
no build remains live at this checkpoint. Local receiver child/maintenance/write effects,
object reads for detached rebase occupancy, prepared Pull, remaining transports,
Windows lifecycle and final delivery gates remain open. No task completion or
implementation publication is claimed. This supersedes the rebuild-in-progress
and pointer-binding-pending notes immediately below.

Current continuation (2026-10-13): maintainer explicitly accepted the dedicated
receiver-reference-tree scope; ADR-089 records it. Rebase directory/fixed-file
evidence and sibling private refs now have fresh exact Read admission, retained
evidence and later revalidation. Common and sibling reference trees share the
65,536-entry/16 MiB reference-file-content budget across destinations. Independent
read-only reviews found and verified fixes for admitted rebase mapping and root
set mismatch, then found no new definite sibling-reference/budget blocker.
Full Git-enabled tools library passes 238/238 (53679), including multi-destination
bisect grouping, rebase state and shared-budget regressions. Main/common HEAD and
config.worktree evidence was subsequently added; its latest compilation (98377)
failed from disk exhaustion before test execution. No build remained live;
previously authorized target/debug cleanup completed (67285), restoring 2.7 GiB.
Fresh full tools validation is rebuilding from clean outputs. Do not duplicate
or restart a live build. Local push mutation, sibling pointer targets, child and
maintenance effects, prepared Pull, remaining transports, Windows lifecycle,
full preflight and publication remain incomplete. I300/T7 stays Active; all
accepted decisions remain required and no original task is declared complete.

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

Production-chain checkpoint (2026-10-13): accepted ADR/I300 decisions remain
required in full, per maintainer clarification; no ordinary operation is removed
from acceptance to close T7. Real MCP transport integration passes 5/5 (78360),
covering add/commit/checkout and direct Git-daemon push approval/denial/replay.
Production Agent two-file add passes 3/3 (71337), including unrelated staged
entry preservation and discovery-approved/mutation-denied behavior. Local push,
prepared Pull, other transports, Windows lifecycle and full delivery gates remain
open. A Windows-only retained-executable-guard Git startup test was added but
cannot be validated on this macOS host. This is partial progress, not completion.

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

Explicit phase checkpoint (2026-10-13): AddPlan dispatch uses SenderConfiguration
and Mutation phases rather than config.is_some(). Direct mutation dispatch with
a valid discovery proof now refuses before repository access. The new regression
and corrected stale-discovery entrypoint tests pass; Git-enabled tools 197/197.
Next add actual admitted receiver layout waves, then configuration/auxiliary
evidence and final all-destination mutation admission. The receiver implementation
is still missing; this checkpoint must not be treated as transport delivery.

Receiver path prerequisite (2026-10-13): local destination classification now
rejects parent traversal following an ordinary component, preventing permission
lexical normalization from disagreeing with Git's symlink/parent traversal.
Leading relative parents remain allowed. The classifier regression passes;
this is not receiver execution support. Next replace config.is_some() stage
dispatch with explicit phases, defer all mutation-only facets, and bind each
receiver layout wave to fresh returned path authorizations before reading it.
Independent review established the integration sequence in the migration inventory.

Command-budget diagnostic checkpoint (2026-10-13): configuration and attribute
path discovery preserve the explicit cumulative-budget error rather than
misdiagnosing exhaustion as an installation failure. The targeted case and
all 196 Git-enabled tools tests pass with the pinned, locked toolchain; governance
and Collaboration Claim validators report zero warnings. Receiver stages and
later handoff tasks remain open.

Runner budget regression checkpoint (2026-10-13): Agent and MCP now test
first-stage oversize refusal with zero effects and cumulative profile exhaustion
after two admitted stages. MCP proves sticky request replay refusal; Agent
proves no additional effects but initial oversize precedes ledger reservation.
Agent 5/5 and MCP
6/6 pass after fixture input binding was updated to preserve exact proof input.
No receiver transport was enabled; continue its staged layout/config/evidence
implementation and the outstanding shared evidence/helper budgets. All original
handoff deliverables and publication requirements remain in scope.

Trace2 contract accepted (2026-10-13): the maintainer explicitly authorized
fixed suppression of GIT_TRACE2/GIT_TRACE2_EVENT/GIT_TRACE2_PERF for structured
Git children. This supersedes historical pending-acceptance notes below.
ADR-089 records the accepted diagnostic scheduling/output difference; receiver
support, complete validation and publication remain unfinished.

Cumulative budget checkpoint (2026-10-13): profile reservation is bounded to
65,536 facets and 16 MiB of resource/description UTF-8 bytes before cloning;
Git discovery/mutation share 128 Talos-spawned commands and 64 MiB of reserved
stdout/stderr capacity. Core budget tests and 195 tools tests pass. The initial
Agent/MCP clone bypass found in review is fixed and independently rechecked;
Agent tests pass 5/5 after the fix. Descendant/helper and evidence-byte accounting
remain unresolved; these bounds do not authorize receiver transports.

Current runner continuation checkpoint (2026-10-13): the candidate now has an
eight-stage bound, invocation-wide identity history and separate cumulative
permission/execution deadlines. This supersedes the historical two-stage
constraint below. Focused Agent tests pass 5/5 and MCP transport tests 6/6;
no receiver transport was enabled. Continue with cumulative resource accounting
and the remaining concurrency tests, then admitted receiver discovery. The I300
owner and bounded-stage proposal retain the unfinished scope and review findings.

MCP continuation verification (2026-10-13): session 67957 exited 0 for
`cargo test --locked -p talos-mcp --test prepared_replay` with the pinned
toolchain and reduced-debug, single-job, non-incremental settings. All five
transport tests passed, including fresh two-stage progression, third-stage
refusal, repeated identity/request refusal, cancellation, transport error and
EOF cleanup. This verifies the shared two-stage history refactor; it does not
implement the proposed larger stage budget or receiver support.

Full preflight retry passed (2026-10-13): session 84462 exited 0; log
`/private/tmp/i300-preflight-retry-20261013.log` ends with
`talos-runtime external fixture passed` and `release preflight: passed`.
The standard script completed governance/claim checks, formatting, locked
workspace check, Clippy, workspace tests and the external Runtime SDK fixture
using pinned Rust 1.99.0 and reduced-debug, single-job, non-incremental builds.
This supersedes the failed run below. The local candidate is still unpublished;
remaining push transports, pull migration, final independent review and pending
trace2 contract acceptance prevent I300 completion. Next implement receiver
discovery and effect binding for local/file push, including bare repositories.

Local/file push design review (2026-10-13): independent review defined the
required receiver sequence: sender destination discovery; bounded receiver
layout discovery only after Path Read admission; separate admissions for every
external `.git`, `commondir`, linked-worktree or metadata root; fixed config-chain
discovery after layout closure; auxiliary Read evidence for receiver refs,
objects, hooks, attributes, fsck skiplist and child executables; then one final
mutation admission covering all destinations before any receiver write. Bare and
non-bare receivers both remain in scope. `receive.denyCurrentBranch=updateInstead`
can mutate a sibling worktree and is not yet represented. This review produced
implementation guidance and documentation only; local/file execution remains
disabled pending code, tests and fresh security review.

Pull legacy-route containment (2026-10-13): `GitPullTool` previously invoked
`git pull` directly after its ordinary permission profile, bypassing prepared
discovery and fresh mutation admission. Direct execution now refuses with an
actionable message until fetch, merge/rebase, hooks, worktree and remote effects
are represented by a prepared plan. A regression test covers this refusal. This
is containment evidence, not pull support; the prepared pull migration remains
open.

Pull refusal verification (2026-10-13): with the `git` feature enabled, the
focused tools test passed 1/1 (191 other tests filtered). Formatting passed.
The tool description now states that pull is temporarily unavailable until
prepared fetch/merge authorization exists, so Agent/MCP callers are not invited
to retry an operation that is deliberately fail-closed. Independent review
found no safety issue in the containment change.

Git tools regression verification (2026-10-13): `cargo test --locked -p
talos-tools --features git` passed **192/192** unit tests plus integration and
doc-test targets. This covers the pull containment change and the existing
prepared add/commit/checkout/direct-git transport regressions after removing the
legacy pull helper path. It does not expand transport support or complete I300.

Governance verification (2026-10-13): project governance and Collaboration
Claim validators both passed with zero warnings after the pull containment and
documentation updates. SQLite consumer and delivery workflow fixture checks
also passed as part of the governance validator run. I300 remains the effective
claimed work slice; no T8 activation or completion transition is made.

Preflight resource failure (2026-10-13): session 70897 completed the main
workspace tests and began the external Runtime SDK fixture, but its second
fixture build failed while compiling `ring`: the compiler could not create
object files because the disk was full (`No space left on device`). No Rust
test failure was reported. The authorized rebuildable `target/debug` directory
was then removed, restoring about 8 GiB. A fresh full preflight is deferred
until enough disk headroom exists; this is verification infrastructure evidence,
not a passing preflight.

SDK recovery verification (2026-10-13): after freeing the authorized build
artifacts, session 10528 ran `python3 scripts/validate_runtime_sdk_fixture.py`
with pinned-toolchain, reduced-debug, single-job, non-incremental settings.
It exited 0: both default and `coding` external consumer modes passed using
their independent lockfile and `--locked`. Log:
`/private/tmp/i300-sdk-isolated-20261013.log`. Disk remained at approximately
5.8 GiB free. This completes the previously failed fixture checks separately;
it does not retroactively change session 70897 into a successful preflight.
No Rust source changed between that preflight and the isolated fixture run.

Post-review source check (2026-10-13): after withdrawing the unconnected
receiver-layout prototype, `cargo check --locked -p talos-tools --features git`
passed with the pinned toolchain. No speculative receiver authorization code is
left in the tree; local/file push remains disabled until a complete evidence
model is implemented.

Prepared-stage architecture constraint (2026-10-13): Agent and MCP runners cap
prepared invocations at two stages. The reviewed local/file receiver sequence
needs at least four stages, so Git cannot safely add it within the current
runner contract. Any stage-budget increase requires a separate security/API
review covering proof consumption, replay, cancellation and bounded work. This
is an implementation dependency, not a reason to weaken the existing cap.

Prepared-stage extension proposal (2026-10-13): documented the bounded multi-stage
runner design and required security/API evidence in
`docs/proposals/i300-bounded-prepared-stage-extension.md`. It preserves fresh
proofs, plan identities, replay reservations, cancellation and cleanup receipts;
it does not change the current two-stage implementation or activate local/file
push.

Independent proposal review (2026-10-13): required explicit initial-plus-final
stage counting, invocation-wide identity/replay tracking, atomic proof
consumption, per-stage scope binding, cumulative deadline/output/helper/request
budgets, cancellation-vs-admission serialization, single cleanup-owner handoff,
seal/cancel/join/wait ordering, and bounded replay-reservation lifetime. These
requirements are now stated in the proposal; no runner cap or security contract
has changed.

Follow-up proposal review (2026-10-13): the proposal now requires the Agent/MCP
runner-owned stage ledger, unforgeable admission context, cleanup-owner transfer
on Continue/Complete/error/drop, no external effect before Continue, and a
concurrency matrix for cancellation, disconnect, replay, stage jumps, parallel
proof consumption, budget expiry and cleanup timeout precedence.

Proposal wording correction (2026-10-13): clarified that an admitted read-only
discovery stage may execute bounded Git inspection commands before returning a
continuation, consistent with ADR-089. Such children must finish with confirmed
cleanup; only unaccounted or uncertain mutation is forbidden from becoming a
retryable continuation.

Stage-history refactor (2026-10-13): introduced the shared core
`PreparedStageHistory` ledger and replaced duplicate Agent/MCP two-stage identity
tracking with it. The bound remains two; this additive type does not grant
authority, consume proofs or enable multi-stage Git. Its core boundary test
passes 1/1, and formatting passes. Full Agent/MCP validation remains required
before publication.

Stage-history validation (2026-10-13): locked `cargo check -p talos-agent -p
talos-mcp` exited 0 (session 1050). Independent review found the replacements
equivalent and no safety blocker: the shared history still reserves before
admission, refuses duplicate identities or a third stage without mutation, and
does not move proof consumption. The additive public type is not approval of
the broader multi-stage proposal. Runtime tests remain required before release.

Full validation checkpoint (2026-10-13): preflight in session 76242 exited 101
at the permission unit suite after workspace check and Clippy passed. The
configuration-chain regression still expected `git_push` to be refused despite
its implemented prepared discovery route. Updated that assertion to allow push
and retained a negative assertion for the still-unmigrated `git_pull`; generic
read and missing-executable rejection checks remain. The permission library
suite now passes 142/142 with the pinned toolchain and `--locked`. Full preflight
must be rerun; no workspace-pass or completion claim follows from this result.

Pack-started cancellation acceptance (2026-10-13): the cancellation fixture now
also advertises an empty receiver, observes actual `PACK` bytes from an 8 MiB
pseudorandom payload and confirms the invocation remains active before abort.
Joined cancellation yields a confirmed cleanup receipt, socket EOF/reset and
unchanged local index. Targeted test passes for both connection-wait and
pack-started cases. This supersedes the earlier pack-transfer evidence gap; it
does not promise rollback of remote effects or cover every transport protocol.

MCP push success acceptance (2026-10-13): real JSON-RPC calls through the
registered GitPushTool now push to a loopback Git daemon. After sender HEAD
advances, replay of the same request ID refuses and receiver HEAD remains the
first approved commit. Explicit handler cleanup and daemon shutdown complete.
MCP `git_prepared` passes 5/5. This closes the MCP success/replay evidence gap for
direct Git transport only; remaining transports and the serial queue stay open.

Push connected cancellation acceptance (2026-10-13): a real TCP listener accepts
the prepared Git connection and intentionally withholds protocol response. Abort
and join then produce a confirmed operation cleanup receipt, the peer observes
EOF/reset, and the local index remains unchanged. Targeted regression passes.
The first fixture timed out because no valid explicit source ref was present;
it now creates an empty commit and supplies an explicit refspec. This covers
connection-wait cancellation, not cancellation during pack transfer or rollback
of remote updates already accepted.

Multi-destination partial push acceptance (2026-10-13): real daemon regression
configures two push URLs, verifies both resolved destinations in the plan, then
observes the first receiver update while the second missing receiver fails.
The result is an error with explicit partial-update guidance; the successful
first update is verified by OID. Targeted test and diff check pass. This proves
the need for inspection before retry, not cross-destination atomicity.

Push partial-failure output (2026-10-13): errors from the actual push command
now explicitly state that remote references may be partially updated and require
inspection of every destination before retrying. Discovery/admission failures
retain their pre-mutation diagnostics. The real receive-policy rejection test
asserts the new guidance; this does not imply atomic push or automatic retry.

Remote push rejection acceptance (2026-10-13): the real loopback daemon fixture
now enables receiver `receive.denyDeletes`, attempts an admitted deletion refspec,
and confirms an error plus unchanged remote ref. The targeted daemon test and
diff check pass. This proves graceful receive-policy rejection for the direct
transport, not atomic rollback for arbitrary multi-ref partial failures.

Empty push alias correction (2026-10-13): upstream `remote.c::add_pushurl`
clears generated push destinations when a pushInsteadOf rewrite yields an empty
string. Native discovery now mirrors that reset and falls back to fetch URLs
when the resulting push list is empty. A real host-Git comparison regression
passes. Trace2 diagnostic suppression remains a pending maintainer contract
decision; automatic goal continuation is not recorded as acceptance.

Independent push review (2026-10-13): reviewer identified late legacy remote
insertion and early trace2 configuration output as blocking findings. Push now
rechecks legacy remotes/branches on every revalidation; regression inserts each
after discovery and verifies mutation refusal. The trace2 finding remains open:
system/global trace targets may write during the first Git configuration command,
before values can be inspected. Resolve the pre-discovery logging boundary and
re-review before publication; current tests do not prove read-only discovery.

Reference evidence consolidation (2026-10-13): review confirmed RepositoryState
already binds shared/per-worktree refs, HEAD and packed-refs. Removed the later
redundant SenderRefs snapshot, retaining push loose/packed/HEAD-change tests
against the shared revalidation path. Earlier claims that this was a missing
binding were incorrect; the added tests strengthen evidence, while the duplicate
implementation added no required coverage. No reference guard was removed from
RepositoryState and no transport scope was expanded.

Ordered host evidence (2026-10-13): configuration capture now retains the exact
ordered origin/key/value output internally, and host reinspection compares it
in addition to grouped values and origin paths. A regression proves that
cross-key reordering with identical grouped values is rejected. This protects
ordered recursion and URL rewrite semantics during discovery/reinspection;
configuration values remain internal and are not printed to the model.

Push recursion parser correction (2026-10-13): source inspection of Git 2.54
`submodule-config.c::parse_push_recurse` confirms boolean true is invalid for
`push.recurseSubmodules`, unlike `submodule.recurse`. The ordered parser now
rejects invalid early values even when a later setting disables recursion;
tests cover true/yes/1/ON and invalid text. Named modes remain supported for
effect classification, with enabled child execution still requiring its own
authorization. This preserves Git failure semantics before network activity.

Ordered recursion checkpoint (2026-10-13): push discovery now preserves the
host configuration record order when applying `submodule.recurse` and
`push.recurseSubmodules` to their shared effective setting. Later explicit
disable values are accepted; later enabling values remain refused before child
execution. This supersedes the mixed-setting compatibility residual below.
Existing 184 tools tests passed after implementation; added cross-key override
regression covers both orders and malformed values.

Push recursive-config correction (2026-10-13): Git 2.54 `builtin/push.c`
also maps `submodule.recurse=true` to on-demand child push. Discovery now refuses
this unbound child transport effect as well as `push.recurseSubmodules`; explicit
false remains accepted. Config regressions cover both values. Mixed settings
are conservatively refused if either setting enables recursion, even when Git's
declaration order could ultimately disable it; exact ordered override support
remains a compatibility residual, not silently altered execution semantics.

Push HEAD checkpoint (2026-10-13): current RepositoryState already retains
HEAD content. A new prepared-execution regression verifies both symbolic HEAD
retargeting and replacement with a detached OID invalidate the admitted push
before transport. Focused test and diff check pass; this is additional evidence
for existing binding, not a new transport capability or completion claim.

MCP push denial checkpoint (2026-10-13): a real JSON-RPC transport fixture
allows configuration discovery/local facets but denies Network. The continuation
returns a permission error, duplicate request IDs refuse replay, a bound loopback
listener receives no connection, and no index is created. Explicit handler
shutdown joins cleanup. MCP `git_prepared` passes 4/4 with `--locked`; successful
push evidence remains the tools daemon fixture, not yet MCP success acceptance.

Push attribute traversal (2026-10-13): admitted execution now captures and
rechecks per-directory `.gitattributes`, including untracked directories that
may correspond to historical object paths, without reading ordinary file
bodies. The worktree Read facet explains directory/attribute traversal. Worktree
symlinks and nested repositories conservatively refuse rather than traverse
outside admitted paths; support for those layouts remains incomplete. Bounded
traversal reuses directory identity checks. Tests cover changed historical-path
rules, unchanged ordinary bodies and external directory links; tools 183/183
pass. This supersedes the per-directory evidence gap below, not the outstanding
transport, pull or full publication gates.

Attribute-read checkpoint (2026-10-13): Git 2.54 `pack-objects` consults the
`delta` attribute through the index while packing. Push discovery now additionally retains
the root worktree attribute path alongside the existing common and global
attribute evidence, and opens/revalidates it after continuation admission.
per-directory attribute stacks and other configured helper effects remain
conservative residuals. Tools tests pass 181/181.

Push authorization acceptance (2026-10-13): loopback daemon tests cover missing
proof, replaying a discovery proof, and a pre-push hook introduced after
discovery; each refuses before remote refs change. Git 2.54 source inspection
also confirms send-pack starts built-in pack-objects, while pack-objects reads
configuration and attributes for delta decisions. Promisor/negotiation effects
remain refused, and executable hooks are checked before mutation. Historical
tree attribute-read coverage still needs review before publication; these tests
do not prove complete process/read-effect coverage. Tools tests remain 181/181.

Direct push checkpoint (2026-10-12): prepared mutation now admits only built-in
`git://` transport, after sender configuration, hook, executable and reference
revalidation. A loopback Git daemon regression proves object transfer, receiver
ref update and payload contents. File/SSH/HTTP/helper transport effects remain
unimplemented and explicitly refuse; this is an incremental implementation, not
full push delivery or T7 closure. Pull and all later serial work remain open.

Push sender checkpoint (2026-10-12): discovery now retains bounded loose and
packed sender-reference evidence, including non-HEAD branches and tags, and
revalidation refuses changes before mutation admission. Linked reference entries
are refused rather than read through. Loose/packed-ref change regressions pass;
Git tools tests pass 180/180. Transport execution is still closed; receiver and
network helper binding remain next, followed by pull and the serial queue.

Push destination checkpoint (2026-10-12): local/file receivers now produce Path
Read/Write facets instead of Network authority, without opening the receiver
during configuration discovery. Custom remote helpers refuse pending executable
effect binding. URL classification and actual prepared-discovery tests pass;
Git-enabled tools tests pass 179/179. Push mutation remains closed until receiver
configuration/hooks/process and transport effects are bound; pull migration and
the remaining serial deliverables remain open. gix tracking was refreshed in
GIT-001 (Talos 0.87.1, published 0.89.0); no dependency upgrade or full Git removal
is claimed. Earlier pending-decision notes below are historical checkpoints;
the maintainer has since accepted configuration discovery, cleanup lifecycle and
foreground maintenance decisions.

Real prepared-plan process cancellation test passes: abort/join plus local receipt
prevents a delayed descendant write. MCP sealing experiment was reverted; restored
11 unit/3 replay-EOF tests pass. Shared registry/connection ownership remains the
next lifecycle correction; full Agent/MCP process acceptance still required.

MCP EOF transport regression passes: disconnect during active invocation drops
execution, awaits its receipt, and completes explicit shutdown after release.
Shared registry/owner isolation, arbitrary transport failure and real descendant
cleanup remain unfinished. No Commit activation or whole-T7 completion claimed.

MCP review correction: each continuation receipt is retained and waited under a
shared deadline; request cleanup failure remains sticky at shutdown. 11 unit and
3 transport tests pass. Cross-connection owner isolation and EOF/transport-error
cancellation acceptance remain mandatory; Commit remains disabled.

Latest MCP checkpoint: explicit handler shutdown and CLI post-transport caller
integration exist. MCP/CLI checks and three transport tests pass; cancellation
fixture now proves Drop-before-wait and blocked-until-released cleanup. Next:
review connection shutdown races/sticky failures/shared owners, complete real
process lifecycle acceptance, then enable Commit only after required evidence.

MCP request cleanup now waits after its execution future leaves select scope,
with bounded timeout/panic containment and retained replay reservation. MCP check
and three existing real transport tests pass. Connection-level owner shutdown,
active-handler joining and dedicated cleanup transport evidence remain open.

Latest Agent correction: invocation cleanup is now keyed by internal TurnId,
with confirmed receipts released and failed/pending receipts retained. Check and
two focused cleanup tests pass, covering cross-turn isolation. MCP async owner
shutdown and real cancellation acceptance remain next; no completion claim.

Agent cleanup checkpoint: prepared operation receipts are registered before each
stage and awaited after Agent cancellation join or normal/error completion, with
one bounded remaining budget and panic-to-Unconfirmed conversion. Agent tests
pass 467/467. MCP explicit owner shutdown and final end-to-end acceptance remain.

Git lifecycle checkpoint: Add/Commit owner and per-plan operation scopes are
wired through all discovery/inspection/mutation commands. Sealing regression
rejects both existing continuation and new discovery without index creation;
tools tests pass 147/147. Next required implementation is Agent invocation-local
receipt collection before dispatch, cancellation join/wait, then MCP shutdown.
Commit remains disabled and the full serial task remains incomplete.

Latest Runtime checkpoint: tool owner finalizer registration and pre-cancel
sealing are wired. Owner seal panics become failure; abnormal barriers cannot
claim completed cleanup. Runtime check/61 tests pass; final corrected candidate
still needs revalidation. Git operation scopes, Agent receipt integration and
MCP explicit async owner shutdown remain unfinished. No T7 completion claim.

Latest lifecycle checkpoint: typed sandbox-to-core cleanup evidence and registry
owner retention now exist; core 112/112, sandbox 45/45 and targeted Clippy pass.
Next: Runtime owner finalizer registration with seal before cancellation, Git
operation-local scopes shared across stages, Agent cancellation joining, MCP
explicit async shutdown. These integrations and real acceptance remain open.

Latest decision: maintainer accepted cleanup lifecycle amendment after the
explanation and resumed the goal. Earlier pending-decision notes are superseded.
Core contracts/default hooks and Unix aggregate/local receipt primitives now
exist locally; integration into Git/Agent/Runtime/MCP remains open. Core tests
pass 111/111. Commit remains disabled until actual lifecycle/behavior acceptance.

Stable validation (2026-10-12): session 21275 exited 0; full standard preflight
including workspace tests, validators and both SDK fixtures passed without code
edits during the run. Log: `/private/tmp/i300-preflight-checkout-boundary.log`.
No build remains live. This supersedes the pending checkout preflight notes.
Cleanup receipt API decision remains pending; Commit mutation and full family
migration/publication remain unfinished. All later serial tasks remain pending.

Latest T7 correction (2026-10-12): checkout branch/path ambiguity reproduced
and fixed with a trailing argument separator. Missing branch matching a tracked
file now fails without restoring edits; branch creation, previous-branch and
unique remote tracking remain supported. Tools tests pass 146/146 and independent
review accepted the local boundary. Final preflight is required after this code
change; 8828 validated its predecessor. Cleanup API decision remains pending,
Commit disabled, and the full serial queue is not complete.

Latest verified continuation (2026-10-12): status-reader stable preflight 8828
exited 0, including workspace tests and SDK fixtures; log is
`/private/tmp/i300-preflight-status-reviewed.log`. A test-only symlink diff
host-comparison added during that run passed separately, as did subsequent
Git-feature all-target Clippy and diff checks. Include this new test in final
candidate preflight. No build remains live. Cleanup receipt API remains pending;
Commit is still disabled and no T7 completion/publication is claimed.

Status review continuation (2026-10-12): actual workspace execution succeeded.
The original preflight session `5976` exited 0, including the SDK fixture,
with log `/private/tmp/i300-preflight-stable.log`. Source changed after it
started, so a stable-candidate rerun is required. Status ignore reads are now non-recursive and per-directory
identity is retained and rechecked throughout capture. Independent review
confirmed both reported defects repaired. Tools tests pass 143/143; prior Clippy
passes. Unix ignore leaf opens now reject links/FIFO substitution and compare
opened identity, with independent review. Ancestor/content races, Windows
limitations and conservative traversal remain in the migration inventory.
Commit mutation stays disabled; cleanup receipt API approval remains pending.

Latest continuation (2026-10-12): I300 remains Active on `feat/i300-git-repair`,
based on effective claim `55fef9f9`, with uncommitted implementation preserved.
Latest commit activation checkpoint: parent config/hook/filter checks now run
before final child evidence revalidation; both commit modes refuse configured
filters without helper execution or index mutation. Tools tests 137/137 and
Clippy pass. No-change host commit invokes status with untracked directory/ignore
reads not yet bound, so mutation remains closed. Cleanup API decision is pending.
Full preflight session `8197` exited 1 after workspace tests/doctests passed:
the independent Runtime SDK lock omitted talos-tools' new talos-permission edge.
The fixture lock is now synchronized (one dependency line; no version upgrades).
Retry `34549` exited 1 at formatting while status-evidence edits were in progress;
formatting is now applied. Neither process remains live. Full preflight must be
rerun on the stable candidate. Latest local tools tests pass 140/140: commit now
declares worktree Read for status enumeration, captures directory names/types and
ignore rules without untracked file bodies, and revalidates parent/child evidence.
Child external excludes are declared from frozen config/environment. Independent
review remains open and commit mutation is still disabled.
Latest review corrections: parent config identity now rejects equal-content
replacement/ancestor retarget; child attribute paths are discovered in each
child cwd, including defaults when the parent overrides core.attributesFile.
Promisor/partialCloneFilter/extensions.partialClone refusal now covers commit
and child object inspection even with maintenance disabled. Tools tests 136/136
and Clippy pass. Commit mutation remains disabled and cleanup API is pending.
Latest promisor checkpoint: Git pack-objects may fetch missing promisor objects;
automatic maintenance now refuses enabled promisor remotes before mutation.
Tools tests 134/134 and Clippy pass. Network/helper authority and lifecycle
remain incomplete; GitCommit stays disabled.
Latest Git child-resolution audit: Git 2.54 source and isolated trap binaries
show maintenance/status/repack/pack-objects built-ins dispatch internally;
`git-<subcommand>` PATH helpers were not invoked. This narrows executable
identity scope but does not complete configured internal helper effects or
lifecycle integration. Commit mutation remains disabled.
Latest maintenance validation: isolated host auto-maintenance reproduced external
pack/index/reverse-index writes through gc.repackFilterTo. Maintenance entry is
rechecked after inspection; late-retarget regression passes. Tools tests 133/133
and Clippy pass. Full helper/lifecycle binding and GitCommit enablement stay open.
Latest maintenance checkpoint: nonempty gc.repackFilterTo is refused when
automatic maintenance is enabled because the destination write is not represented;
explicit maintenance.auto=false preserves discovery. Tools tests 132/132 and
Clippy pass. This is a conservative configured fallback, not complete maintenance
effect binding; GitCommit stays disabled and lifecycle decision remains pending.
Latest evidence retention: child filter inspection returns retained attributes/
tracked-content evidence for later revalidation; late-edit regression passes,
tools tests 131/131. Mutation remains disabled. Cleanup receipt public-interface
decision has been explicitly presented and remains awaiting maintainer response;
other authorized implementation work may continue independently.
Latest tracked-source checkpoint: child filter inspection compares tracked
membership/content before and after, with directory replacements captured by
kind rather than recursively reading untracked descendants. Tools tests 130/130
and Clippy pass. Final mutation/source binding and lifecycle remain unfinished;
GitCommit execution stays disabled and the full queue remains incomplete.
Latest attribute snapshot correction: admitted child filter inspection now
captures local .gitattributes from every tracked source ancestor as well as
external attribute sources, and rechecks them after inspection. Tools tests
129/129 and Clippy pass. Untracked-child trace confirms status invocation only;
it does not prove reads of untracked file bodies. Commit remains disabled.
Latest ordinary-commit correction: no-staged-changes host trace exits 1 and calls
child `status --porcelain=2`. Both commit modes now receive child discovery/filter
checks; earlier success-only traces do not establish absence of child effects.
Both modes pass helper-refusal regression; tools tests 129/129 and Clippy pass.
Recursive/untracked child status and remaining lifecycle work stay open.
Latest child status checkpoint: commit-all now checks admitted child attributes
before status and refuses configured filters; isolated same-length edit regression
proves helper execution does not occur. Nested gitlinks remain refused pending
discovery. Tools tests 129/129 and Clippy pass; commit mutation remains disabled.
Latest admitted metadata checkpoint: continuation now captures and rechecks child
RepositoryState against discovered checkout/gitdir/common-dir after proof
consumption. External separate-git-dir regression passes; tools tests 128/128
and Clippy pass. Index evidence is byte-level only. Child status source/filter/
helper effects and lifecycle still prevent enabling commit mutation.
Latest location checkpoint: admitted child discovery binds gitdir/common-dir
identity and commondir bytes, rechecks around config discovery, and declares
new shared metadata Read facets for continuation. Pointer-retarget/directory-
replacement/link regressions cover the new boundary. Child HEAD/index/source
and execution lifecycle work remain incomplete; commit mutation stays disabled.
Latest identity checkpoint: child config origins bind canonical paths and Unix
device/inode identities through canonical ancestors; pre/post-read checks reject
equal-content ancestor retarget and identical-byte file replacement. Windows
identity remains limited to canonical paths. Child repository/common-dir identity
and the remaining commit effects are still unfinished; mutation remains closed.
Latest child configuration checkpoint: admitted commit all=true discovery now
inspects child configuration chains, binds values/origins/source bytes internally
and declares child origin Read facets on the continuation. A real external-include
fsmonitor regression verifies refusal before helper execution and support for
explicit false. Tools tests 123/123 and Clippy pass. Child metadata and broader
status/filter/helper/lifecycle integration remain open; commit stays disabled.
Latest child-read correction: GitCommit preparation no longer reads child
RepositoryState before admission. It binds checkout kind/local .git marker and
declares the destination Read path. External invalid-config and marker-retarget
regressions pass; tools tests 122/122 and Clippy pass. Admitted child state/config
capture remains unfinished; mutation stays disabled. Earlier claims of child
HEAD binding apply only to the superseded pre-admission implementation.
Latest gitlink checkpoint: typed commit sources bind index OID, checkout kind
and child metadata without recursive child worktree snapshots; 121 tools tests
pass. Host trace confirms commit -a starts child `status --porcelain=2 -uno` in
the dirty-submodule fixture, unlike ordinary commit. Child configuration/helper
effects and premature external child metadata reads remain review findings to
resolve before enabling mutation. Existing temporary commit refusal remains.
Latest metadata checkpoint: scalar commit state cannot be directories, and
cache/sequencer roots cannot be ordinary files. Types are checked before snapshot
recursion; wrong-type and MERGE_RR symlink regressions pass. Locked tools tests
pass 119/119; mutation remains disabled with gitlink/lifecycle work outstanding.
Latest commit evidence: tools git-feature tests pass 117/117. Active enabled
rerere records and unmerged index entries explicitly refuse, while inert/disabled
rerere discovery remains supported. Admitted `--exec-path` discovery now produces
concrete child-program Read/Execute facets for continuation identity validation.
Commit mutation remains closed pending typed gitlinks, remaining maintenance
helper resolution and lifecycle/source boundary completion.
Current commit checkpoint supersedes earlier "legacy route" notes: GitCommit
now prepares two-stage admission and refuses direct execution, but its mutation
stage is deliberately disabled while effect/lifecycle review completes. Git
tools library tests pass 115/115 and the closed permission discovery regression
passes. Remaining concrete commit work: maintenance helper executable binding,
active rerere worktree effects, typed gitlinks, source/stat scope, final metadata
ancestor checks and lifecycle integration. The maintainer accepted foreground
maintenance overrides (ADR-089), and isolated Git 2.54 trace2 confirmed
`maintenance run --auto --no-quiet --no-detach`; no complete migration is claimed.
The maintainer additionally authorized rebuildable `target/debug` cleanup;
it recovered about 11 GiB from a 116 MiB free-space floor. Tests rebuilt after
cleanup. Source/Git data remained intact; no publication occurred.
Latest reference checkpoint: bounded loose-reference snapshots now detect branch
movement without object creation. Real Git regressions also cover shared packed
references and isolated linked-worktree private references; reference symlinks
are refused. The complete tools git-feature library suite passes 112/112 and
all-target Clippy passes with warnings denied. This snapshot is currently shared
with Add, so unrelated reference changes conservatively invalidate that plan;
operation-specific scope still needs review during commit migration. Commit has
not migrated, and full preflight/security review/publication remain open.
Execution tools were directly called successfully; no evidence establishes
subagent context leakage as the cause of earlier false unavailable-tool reports.
Latest boundary checkpoint: unstaged diff previously exposed outside file content
through a replaced ancestor symlink. The regression now passes after ancestor
validation in worktree content/mode reads; removed parent directories still
produce deletions. Tools git-feature suite 108/108 and Clippy passed. This is
not race-free containment; independent boundary review remains outstanding.
Follow-up: pre-existing special worktree files are refused before opening;
Unix FIFO prompt-rejection regression passes. Current tools git-feature library
suite passes 109/109 and all-target Clippy passes. Broader gates remain open.
Latest local checkpoint: checkout previous-branch shorthand regression reproduced
and repaired by normalizing non-creating `-` to `@{-1}`. Real branch-switch and
creating-checkout refusal tests pass; git-feature tools library suite 105/105.
Latest local checkpoint update: Git `core.fsmonitor` false spellings now match
host behavior (`false`, `0`, `off`, `no`, empty and numeric zero); true/nonzero
and command-valued configurations remain refused before effects. gix integer
suffix parsing also covers host-compatible zero forms such as `0k`; malformed
values still fail closed. Real-host coverage and the complete git-feature suite
pass 107/107; tools Clippy passes. The closed `git:configuration-chain`
permission pair remains intentionally restricted to `git_add`; it is not
evidence that commit/checkout/push/pull have been migrated.
Previous checkpoint: execution tools restored; positional option-injection
input checks verified. Current git-feature tools library tests pass 104/104 and
all-target tools Clippy passes with warnings denied, using the pinned locked
toolchain and incremental compilation disabled. Approximately 731 MiB remains;
no additional cleanup performed. Cleanup receipt API acceptance, remaining write
migration, full gates and publication are still outstanding.
Previous checkpoint: absent/empty submodule checkout and ordinary-file
replacement host-comparison tests passed; focused Git tests now 23/23.
Previous checkpoint: native unstaged submodule OID/dirty/ignore handling added;
real host comparison and stale .gitmodules removal regression pass, focused Git
21/21. Missing/deinitialized checkout and type replacement remain unverified.
Previous checkpoint: core.fileMode=false worktree fidelity repaired after
a failing real-Git comparison; owner executable-bit handling also corrected
after independent review and real-Git reproduction. Git focused tests now 20/20. GIT-001 corrective
candidate documentation synchronized. Staged gitlink add/change/delete fidelity repaired;
Git tests 18/18 and complete tools library with git feature 97/97 passed.
Unstaged submodule state remains unverified. Real Runtime Git approval/discovery-denial/mutation-denial
focused test passed; real MCP Git transport policy-denial/replay/retry focused
test passed including final response-ID assertions. User-facing candidate
boundaries are recorded in both READMEs. Git 30/30, real Agent Git scenarios 3/3 and sandbox
42/42 passed with locked builds; affected-crate all-targets Clippy passed.
System/global attribute paths now come from fixed admitted Git path-only queries,
with content reads deferred to concrete continuation admission. Object namespace
links/special entries, nonempty alternates and split-index layouts refuse before
mutation with explicit fallback. Selected directory rules are enumerated even
without ordinary file leaves. The current Unix implementation reuses ADR-082's
supervisor; the earlier experimental bare-PGID cleanup remains rejected.
Production cancellation/shutdown cannot yet wait the private per-call receipt.
A concrete additive tool-receipt proposal is recorded in the I300 migration
inventory and awaits a maintainer architectural decision; this is a publication
gate. No implementation commit/PR, full preflight or final approval is claimed.
Prepared-stage tests also passed 3/3: bounded continuation, reused identity,
forged proof and request replay remain denied as applicable. Both governance
validators passed with zero warnings; subsequent Agent all-targets Clippy passed.
Earlier checkpoints below retain their historical results, not current gate closure.
Independent incremental review found and verified a fix for blocking proof
consumption; all 141 permission unit tests, three public API integration tests
and one doctest passed. GitAdd now uses a one-use prepared admission with repository,
source, program and configuration evidence. T7 and the full queue remain incomplete.
The host installation configuration gap is now addressed by maintainer-accepted
separately admitted configuration discovery followed by fresh mutation admission.
The focused Git suite passed 23/23; real Agent approval, discovery denial and
mutation denial passed 3/3. An experimental Unix cleanup was rejected by review
for stale-PGID/runtime-lifetime risks and removed; process-tree ownership remains
open. Independent review found premature auxiliary reads; these were
moved behind second-stage Path Read admission with a passing regression test.
See I300's latest checkpoint for remaining cross-platform and same-family work.
Maintainer-authorized incremental cache cleanup recovered approximately 1.8 GiB.
No implementation publication or release occurred.

I299's original C0 claim became effective when #698 merged at `50781eab`. The
maintainer accepted ADR-088's producer-owned, transient Todo display metadata contract;
its cross-crate scope amendment #700 merged at `2f9420f95cd2db3654de463c928debe592a89845`
after exact-head CI, independent Agent review and merge-time CAS. Preserved local C0 implementation stays on
`feat/i299-presentation`; no implementation merge or native acceptance is claimed.
All seven numbered deliverables below remain incomplete. The receiving session must
still follow their full requirements, not treat this governance checkpoint as closure.

### Current C0 checkpoint (supersedes earlier implementation-pending notes)

Completion Commit: 690926759229818fcec92b1487d0cffef1bf5fcb (C0/I299 only;
not completion evidence for this whole task or any of its seven original deliverables).

PR #701 is merged as `690926759229818fcec92b1487d0cffef1bf5fcb`; I299 is
Complete / Closed with that existing implementation SHA as evidence. Final head
`58e74613` passed all six CI jobs in run `38115866825`, independent Agent review
`6105965862`, and merge-time CAS `6106128866`. All four native observations passed.
This is the current delivery checkpoint; earlier pending statements below are
historical. No release occurred. Next is T7, with a new effective claim required
before implementation. This does not close any of the seven numbered requirements.

Latest acceptance: the maintainer completed all four native observations on `b19fc58e`,
ending with “符合预期，已退出”. I299 is Review / Claimed, ready for stable publication;
the previous manual gate is satisfied. Remaining: final-head review, remote CI/CAS/merge
and owner-first closure. The seven original deliverables remain incomplete. Earlier
pending-human statements below are preserved checkpoint history, not current blockers.

The local ADR-088 chain is now implemented from native Todo formatting through permission
wrappers, Runtime/Agent events and ConversationEngine to TUI rendering. The migration record
is `docs/reference/I299-TODO-PRESENTATION-MIGRATION.md`. Independent incremental dirty-tree
review found the two previous defects resolved (orphan truncated-item highlight and missing
real execution/export regression); this is not exact-head merge approval.

`cargo test -p talos-cli --test todo_presentation_chain --locked` passed (1 test), proving
real native Todo execution through Runtime's permission wrapper, transient UI metadata,
unchanged durable text and both export modes against literal replay. Both governance
validators passed with zero warnings. Native drawing/selection acceptance is still outstanding;
the test ends at UI output. A dedicated offline fixture is being prepared at
`crates/talos-cli/examples/i299_presentation_acceptance.rs` for that observation.
Remaining C0 gates: final complete preflight, stable commit and independent exact-head review,
native acceptance, remote CI/CAS/merge and owner-first closeout. No implementation has been
published and none of the seven original deliverables is complete.

Subsequent local validation: full standardized preflight exited 0 (`release preflight: passed`),
including workspace tests and external SDK fixtures. The final I299 acceptance example's
`--check` and targeted Clippy passed; four native human observations remain unchecked in I299.
Stable local candidate `e96438eb84066ef178a830ea4b42edb452c78d69` subsequently received independent
Agent API/privacy APPROVE against base `2f9420f95cd2db3654de463c928debe592a89845`.
The full task remains active. Next: guide the four native observations before remote publication;
stage 1 instructions have been sent, but no human result has been received. No T7 or later
product slice is activated here. Later documentation edits do not broaden that exact-head approval.

## Receiving Checkpoint And Revised Plan - 2026-10-10

Current execution: I299 claim PR #698 merged at
`50781eab529e424894d11e7b1c502fb1d0cb50f5`; implementation branch
`feat/i299-presentation` starts from that merge. Preserved code has been reapplied locally.
The rebuilt TUI suite passed 603 unit tests, 2 integration tests and 2 doctests. Todo
multiline spoofing now has regression coverage; full-list decoration still needs trusted
provenance. On 2026-10-11 the maintainer accepted ADR-088's transient, producer-owned,
exact-text/invocation-bound metadata and API migration boundary. ADR and I299 owner record
the acceptance; #700 establishes the expanded claim, while cross-crate implementation remains pending.
The implementation branch is now based on `main@2f9420f9`, including #590 Auto locale work.
Preserved TUI/process code was compared with recovery stash `4de7707f` after restoration:
no code difference. Documentation conflicts were reconciled while retaining the full queue.
The recovery stash retains the original staged/unstaged snapshot until candidate convergence.
Both governance validators passed with zero warnings for #700.
Native acceptance, full preflight and implementation
publication remain pending. All seven original deliverables remain incomplete.

This section governs the receiving session's task order. The original seven deliverables and
earlier checkpoints below remain preserved requirements and historical evidence, not current
claims that the checkout is dirty or the handoff is unpublished.

### Goal Execution Ledger - 2026-10-10

Requested outcome: close this entire handoff, including all seven original requirements and
the preserved TUI/process integration candidate. Documentation refinement is not completion.
Work mode: Standard. Deferred validation tracker: None; required manual acceptance remains a
gate until the maintainer explicitly changes the work mode.

Artifacts: this coordinator, each selected Story/iteration owner, implementation and tests,
user-facing documentation, and applicable ADR/migration records. Preserve all historical
checkpoints, raw session history, other-session branches and published acceptance.
Synchronize product owners before derived Backlog, Board and iteration indexes.

| Item | Current execution state | Required closure evidence |
|---|---|---|
| C0 preserved TUI/process candidate | Complete; I299 owner closed | #701 merge `69092675`; full preflight, all native observations, exact-head CI/review and CAS recorded in I299 |
| T7 structured Git resources/diff | [I300](../iterations/I300-structured-git-repair.md) Active / Claimed after #704 merged as `55fef9f9`; ADR-089 and API amendment accepted | Real-wrapper reproduction, cause, approved/denied/escape tests, diff fidelity, protected-boundary review and delivered repair |
| T8 all-built-in Auto approval (accepted 2026-10-11) | [PERM-007-H](../backlog/active/PERM-007-H-all-builtin-auto-review.md) Refinement / Unclaimed; immediately after T7 | Complete registered native-tool inventory, shared effect-aware model review, real admission/execution tests, policy reconciliation, visible decisions, independent security review and merged evidence |
| T2/T3 MODEL-014 | Unactivated; consolidated implementation/test slice | Accepted authority/schema decisions, inference/override/variant/adapter tests, walkthrough, docs and merged evidence |
| T4 compaction policy/controls | Unactivated | Whole-request policy, safe summary replacement, failure/cancellation tests, observable status, runnable manual command and integrated evidence |
| T5 Actor compaction/recovery | Unactivated | Safe-boundary model-callable operation, bounded authoritative history recovery, discovery/concurrency/privacy tests and merged evidence |
| T1 validate/agile tools | Unactivated | Shared services, supported read-only agile summary, gated bounded validation execution, discovery/failure tests and docs |
| T6 memory parameter | Unactivated | Retained default, explicit units/range/inheritance, foreground/background/composition coverage, platform truth and security review |

Execution environment: pinned Rust 1.99.0; receiving workspace has no `target/` cache and
approximately 14 GiB available at validation start. Standard preflight started with
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
These resource settings do not omit any preflight phase or weaken process permissions.
Site and installer checks passed; remaining phases have no result yet.

External authorization: the maintainer accepted the consolidated contract on 2026-10-10:
serial implementation, stable-candidate commits/pushes, necessary claim/implementation PRs,
independent subagent review and gated merges. Human acceptance and new ADR decisions return
to the maintainer. Release, deletion of other-session branches, paid probing and security
waivers are excluded. No new product implementation is committed before an effective claim.

Branch/checkpoint plan: preserve the existing four commits unchanged; publish them through one
reviewable recovery candidate rather than directly pushing local main. New governed slices start
from their effective claim merge. Use this checkout serially; no extra worktree is required.
Append actual commands, heads, reviews and closure evidence before moving to each next phase.
Resource policy: single-concurrency, debug-free local builds; monitor free space, stop before
disk exhaustion and seek approval for destructive cache cleanup. No paid or sensitive live
provider test. Retry a failed gate only after inspecting its cause; never waive it for progress.
Fallback: keep the affected item non-terminal with its owner and diagnosis; independent work may
proceed only when its dependencies, overlap checks and explicit scheduling authorization hold.

Initial T7 source finding: `GitAddTool` uses the default permission profile, which constructs an
untyped Write facet; `compile_proposal` requires a supported typed resource for consequential
facets. This explains a plausible rejection path but is not yet a verified real-wrapper
reproduction or a complete assessment of related Git tools/pathspec behavior.

Read-only T7 follow-up on `e96438eb` (2026-10-11; no activation or code change):

- `crates/talos-tools/src/git_write.rs` confirms `GitAddTool` has no permission-profile
  override; the default in `talos-core` supplies Write without resource kind/value.
  `talos-permission/src/grant.rs` rejects consequential untyped facets when compiling a grant
  proposal. This matches the recorded error, but active-binary identity and real-wrapper
  reproduction remain mandatory before claiming the production root cause verified.
- The same Git add implementation appends caller path strings directly after `git add`,
  without an option terminator or literal-pathspec boundary. A repair must bind the actual
  staging effect to approved resources, including option-looking names, Git pathspec syntax,
  repository/workspace root differences, symlinks and linked worktree index locations.
  Do not merely mark every request as a workspace-wide write.
- `GitDiffTool` enumerates index/worktree status for both modes, while reading HEAD as the
  old side for both. The existing `git_diff_staged_shows_head_vs_index` regression explicitly
  adds a second unstaged edit so the iterator sees the file. It therefore does not prove the
  staged-only case. Add independent staged-only, unstaged-only, mixed, new/deleted and binary
  fixtures; compare staged HEAD/index and default index/worktree semantics separately.
- These are source facts and test-coverage findings, not a successful execution reproduction
  or a delivered fix. Retain the original scope, permission gates and independent security
  review; do not mix this implementation into I299. I299 still awaits four human observations.

Recovery: refresh Git/PR state, inspect the live preflight process before restarting any build,
record its actual exit result, then resolve C0 ownership and publication gates. Residuals stay
in their original requirement row; none may be silently dropped to close this coordinator.

### Integration And Validation Truth

- Maintainer requested integrating the preservation branch and reorganizing the handoff.
  Local `main` fast-forwarded from `e58e2678` to `385a2d6a`; no remote-main push or
  validated remote merge is claimed. The four preserved commits remain unchanged.
- Existing TUI/process-tool code is an integration candidate, not completed delivery.
  Prior focused tests and Clippy passed, but prior full workspace validation failed during
  CLI compilation. Exact-head remote CI/review and applicable ownership reconciliation remain
  required before publishing the integrated code to remote main.
- The receiving shell reports unlimited virtual-address-space limit and approximately 19 GiB
  free disk. This does not establish the earlier allocation failure's cause. Do not weaken
  process hardening or require the memory-parameter feature merely to run validation.
- Receiving preflight passed site, installer, SQLite-consumer, workflow and project-governance
  checks before being deliberately stopped for this documentation-only refinement. It is NOT
  a completed workspace preflight or code-validation result.
- The structured Git permission defect and staged-diff discrepancy remain unresolved. Earlier
  host-Git publication succeeded; publication recovery is no longer a pending deliverable.
  Repair the actual Talos wrapper, not this receiving session's unrelated development tools.

### Research Findings That Change The Plan

1. **LLM compaction already runs in production paths.** `session.rs` calls
   `Compactor::compact(..., self.agent.provider())` in both runtime-managed and compatibility
   submission paths. MEM-003's statement that layers 4/5 are unwired is stale. Preserve its
   original acceptance, append corrected current facts, and verify missing integration evidence
   rather than implementing a second provider wiring path or declaring MEM-003 Complete.
2. **Current trigger is not whole-request pressure.** `should_compact` estimates history against
   80% of model context; prompt/tool definitions and request output reservation belong to the
   separate request-admission calculation. I298 fixes request estimates and bounded overflow
   recovery, not semantic checkpoint policy. Reuse its admission authority; do not add another
   token-budget authority or reopen completed I298 scope.
3. **Summary failure handling needs verification before expansion.** The current summary reader
   collects text deltas, does not explicitly fail on streamed error events, and substitutes
   `[No summary generated]` when empty. Test error-only, partial-text-then-error, cancellation,
   oversized output and failed terminal outcome. An invalid summary must not replace usable
   history; provider failures must remain observable. This is a source-inspection risk, not a
   reproduced production data-loss incident.
4. **Manual API is not a user command.** `manual_compact` exists with unit tests, but inspection
   found no registered `/compact` command or Actor-owned model-callable compaction operation.
5. **Existing summaries lack the target semantic contract.** Layers 4/5 use an unstructured
   summary projected as `Message::User`. Issue #390 proposes structured model-directed
   checkpoints and bounded current-session history recovery. Coordinate provenance/summary
   representation with SESSION-011/#612 rather than silently changing transcript semantics.
6. **Pre-entry compression is separate work.** MEM-007 has default-off deterministic Bash
   compression with raw UI/export evidence. Other tool strategies, cross-turn dedup and
   representative measurements remain open; #143 owns related Bash/PowerShell filtering.
   Do not bundle those expansions into compaction-policy completion.
7. **Chat thinking transport is proven for one test.** A non-sensitive arithmetic request to
   the currently configured Codex custom endpoint using `gpt-6-astra`, Chat Completions and
   `reasoning_effort=low` returned HTTP 200 with nonempty `delta.reasoning_content` (20 characters)
   and reported 13 reasoning tokens. Talos parses this field. Lack of Responses support alone
   does not explain absent thinking. Reproduce through actual Talos request/config/UI paths;
   this observation neither verifies every gateway capability nor authorizes active probing.

### Consolidated Serial Queue

| Order | Deliverable / existing ownership | Original tasks | Exit evidence |
|---|---|---|---|
| 0 | Close out the preserved TUI/process integration candidate | Existing local work | Reconcile owner/authorization, review diff, reproduce focused tests, complete locked preflight and obtain exact-head CI/review before remote-main delivery; retain a separate rollback path |
| 1 | Repair structured Git permission resources and inspect diff fidelity | 7 | Actual wrapper reproduction, root cause, approved/denied/escape regression tests and independent security review; do not repeat already-completed handoff publication |
| 2 | Custom-model metadata, variant and thinking-path consistency / MODEL-014 | 2 + 3 | Accepted capability precedence; explicit false overrides preserved; unique/ambiguous/unknown matching tests; protocol-compatible output/reasoning/image behavior, provenance display and real Talos walkthrough |
| 3 | Reliable compaction policy and controls / MEM-005, MEM-003, #390 | 4 | Whole-request budget integration; valid-summary-only atomic replacement; cancellation/fallback; observable before/after and skip/failure; typed `/compact`; long-session integration evidence with recoverable raw history |
| 4 | Actor-owned model-directed checkpoint and history recovery / #390 | 5 | Bounded schema, session/branch references, safe-boundary application, pressure eligibility, tool discovery, cancellation/concurrency tests and bounded private-history recovery; no second transcript store |
| 5 | Model-callable validation and read-only agile summary | 1 | Shared business service, typed schemas, permission-gated host execution with deadlines/cancellation/output limits, failure tests and tool documentation; do not imply unsupported agile mutations |
| 6 | Explicit subprocess memory-limit parameter | 6 | Units/range/default/inheritance specified; 2 GiB default retained; all supported foreground/background paths tested; honest platform enforcement and independent process-security review |

Tasks 2/3 from the original handoff form one implementation-and-regression slice, not two PR
loops. Compaction stages 3/4 share the MEM-005/#390 roadmap; separate candidates only when
they are independently runnable and claims/acceptance warrant it. Local corrections, tests and
documentation updates stay within their candidate. Do not create an Issue per checklist item.

### Compaction Acceptance And Scope Boundaries

- Runtime owns hard admission and safe application; the model owns semantic retention.
  Checkpoint fields must retain goal, user constraints, decisions, completed/pending work,
  unresolved failures and next steps with bounded source references.
- Keep recent complete tool exchanges and instructions; never replay tools as a compression
  side effect. In-flight batches cannot mutate Actor history directly.
- Checkpoint provenance must identify covered raw history across repeated checkpoints;
  summary-of-summary must not become the only recoverable evidence.
- Preserve raw durable history and stable prompt prefix. Exclude hidden reasoning, private
  arguments and UI-only output from model-visible recovery unless separately authorized.
- A TLOG archive marker is storage metadata, not proof of a usable semantic checkpoint.
- Trigger/target/reservation and observability require initial, continuation, manual,
  irreducible-budget and missing-usage tests. A successful request after truncation alone does
  not satisfy semantic preservation acceptance.
- MEM-007 broader compression, #143 filters, cross-session memory and complete SESSION-011
  rearchitecture are dependencies or residual owners, not implicit scope additions.

### Existing Iteration Disposition

Receiving inventory of explicit non-terminal iteration headers and open PRs:

| Owner | Observed state | Disposition |
|---|---|---|
| I291 / #682 | Review; open Auto locale PR | Other-session work; retain, do not modify or merge under this handoff |
| I293 / #696 | Active; open Search router PR | Other-session work; retain and recheck overlap before activation |
| I294 | Active; native Search evidence | Retain external ownership; no takeover; historical #693 wording may need its owner's refresh |
| I290 | Review; locale baseline | Retain published evidence and follow-up relation to I291; no duplicate locale implementation |
| I277 | Review; remaining human acceptance Deferred | Keep deferred device/accessibility acceptance with its existing owner |
| I249 | Planned / Unclaimed | Keep dependency pilot deferred; no dependency upgrade in this queue |
| I296 / I297 / I298 | Terminal owners; claims closed | Preserve completion evidence; never reuse closed claims for these expanded tasks |

No new iteration or product Collaboration Claim is activated by this plan. Before each selected
slice, refresh main, all owner statuses and open PRs; record any additional non-terminal owner
found by full activation inventory and establish the required effective claim. This task document
coordinates the queue; Story owners remain authoritative for product scope and completion.

### Next Action And Documentation Closure

First finish the preserved code candidate's validation/ownership path. Then prepare the bounded
Git defect scope, followed by MODEL-014 and the unified MEM-005/#390 sequence above. Decisions
about capability authority, checkpoint representation and public API changes must be recorded
before implementation. No release, tag, Issue closure or remote-main direct push is authorized
by a planning checkpoint.

This refinement changes only the coordination document. Product owner statuses, historical
acceptance and completion evidence remain unchanged. Run documentation/governance validators
and diff checks for this edit, not a Rust rebuild. Remaining code-validation and claim gates
are registered above rather than hidden behind a Complete status.

## Latest Preservation Checkpoint

The maintainer subsequently requested committing and pushing all remaining worktree changes. The receiving agent should use branch `handoff/session-worktree-20261010`, not only `main`, to recover the complete work. The earlier dirty-worktree statements below are historical snapshots.

- `e58e2678`: handoff and recovery documents; successfully pushed to `origin/main`.
- `0a0622c0`: existing TUI activity, queue and result-rendering changes plus regressions.
- `3af78f04`: process-tool summary fields and regression assertion.
- `1bda5181`: MODEL-014 intake and related owner/index references.

The last three commits are preserved on the handoff branch; this checkpoint accompanies their push. Verify remote branch state before resuming. No merge into main, feature activation, claim repair, or completion of tasks 1-7 is implied. Full workspace validation remains blocked by the previously recorded CLI compilation allocation failure. Both governance validators were rerun successfully, and the process-tool filtered agent suite passed 5 tests. The structured Git mutation defect remains unresolved; staging/commit/push used the maintainer-authorized host Git fallback. Read-only structured staged-diff output also disagreed with host Git during this operation (reporting the untracked MODEL-014 file instead of the staged code); inspect that separately when investigating task 7.

## Start Here

1. Preserve the dirty worktree listed below. Review staged and unstaged changes separately; do not reset, clean, overwrite, or assume that every change belongs to this agent.
2. Read AGENTS.md, docs/sop/AGENT-COLLABORATION.md, docs/sop/START-ITERATION.md, and the scope owners below. Inventory every Active, Review, Planned and Blocked iteration from its owner before selecting work. The Board is derived, not authoritative.
3. Establish appropriate effective target-branch claims for the new deliverables. MODEL-014 is Refinement / Unclaimed and MEM-005 is Planned. Completed I212 and I298 claims do not authorize their expanded scopes. Do not invent PRs, authorization, completion SHAs or remote evidence.
4. Investigate task 7's structured Git permission resource defect before relying on those tools for subsequent work. The maintainer explicitly authorized a one-time bash Git fallback for publishing this handoff; it does not repair or authorize bypassing the tool permission gate for future work. Resolve the validation execution environment, then finish one deliverable at a time. Suggested subsequent order: validation environment and command memory parameter; validate/agile tools; custom-model resolution and regressions; compaction policy; Actor-owned compaction tool; full verification and local-change closeout. This is a recommendation, not an activated iteration plan.
5. Run focused tests, the locked workspace test suite and standard preflight. Complete owner status only with already-existing implementation SHA evidence; synchronize indexes/Board and remote Issues through the required workflow.

## Outstanding Tasks: Seven Independent Deliverables

This document is the durable work queue for the receiving agent. Tasks 1-6 below preserve the six earlier unfinished feature/test tasks; task 7 records the subsequently discovered Git permission defect and blocked publication. The recipient does not need access to the previous session's todo list or conversation. None of these seven tasks is completed.

Session todo IDs are navigation aids only; a new session may not inherit the todo store. This document preserves the requirements independently.

### 1. Model-callable validate and agile

Todo: `f8cdca5b-53c5-4e59-ab7e-a68e5340b09b`; Blocked. No implementation yet.

Expose the existing command capabilities as tools the model can discover and select. Reuse business logic rather than duplicating command behavior. Verify exact supported operations, inputs, outputs and side effects before defining schema. Preserve the permission pipeline and add discovery, execution, invalid-input and failure tests plus user-facing tool documentation.

Verified entry points:
- `crates/talos-conversation/src/engine/commands.rs`: `handle_agile_command` currently ignores its argument and reads the bound workspace through `governance_summary::format_governance_summary`; do not imply arbitrary agile mutation support.
- The same file: `handle_validate_command` supports only governance, builds a shared validation plan and executes it.
- `crates/talos-cli/src/validation.rs`: CLI validate exposes plan/run and governance/i076/workspace profiles.
- `crates/talos-conversation/src/validation.rs`: shared service; internal governance checks are distinct from host-tool checks. Host checks currently use synchronous `std::process::Command::output` without a timeout at that call site. An allowlisted command is not permission authorization. Do not expose this runner as unrestricted/read-only execution; review timeout, cancellation, output bounds, panic/native boundary and permission facets.
- `crates/talos-runtime/src/composition.rs`, `crates/talos-cli/src/registry.rs`, and `crates/talos-cli/src/mode_interactive.rs`: tool contribution/registration surfaces. Determine intended host coverage and crate dependency direction before adding a shared adapter.

### 2. Custom-provider variant selection and context matching

Todo: `655b3c1b-2b30-43fd-91e4-3b8b4b58e263`; Pending. Investigation completed earlier, implementation not completed.

Correct variant-stage triggering and automatic named-model metadata matching through the existing architecture. The maintainer also requested inference of image-input capability, reasoning capability and output limit, not just context size. This expanded scope is recorded in `docs/backlog/active/MODEL-014-catalog-capability-inference.md`; do not lose it or describe it as delivered.

Owners and boundaries:
- MODEL-013 / I212: completed context-only baseline. Exact opaque model ID first; only supported single leading slash/colon prefix normalization; unique candidate required; ambiguity/unknown/missing metadata remain unknown. No fuzzy matching or stripping opaque version/@ suffixes. Explicit context values win.
- MODEL-014: new capability/output/variant expansion, Refinement / Unclaimed. Resolve explicit-negative overrides, inferred/unknown representation, protocol-compatible reasoning invocation, image eligibility and adapter output caps. Inference is not proof of gateway/endpoint support and must not be labeled successful probing.
- MODEL-011 / Issue #124: separate active-probe/evidence authority path; do not claim it complete or silently expand it through catalog inference.
- MODEL-007 / ADR-048: existing declared variant selection and identity semantics. No declared variants means skip variant picker; do not fabricate endpoint support from a preset.

Entry points: `crates/talos-config/src/config.rs`, `model.rs`, `variant.rs`, `tests.rs`; `crates/talos-cli/src/model_lifecycle.rs`, `mode_runtime.rs`, `mode_runners_tests.rs`. Existing tests explicitly enforce that context-only catalog projection does not inherit output limits; expanded behavior requires new scope/authority decisions rather than rewriting historical acceptance silently.

### 3. Custom-model regression coverage and verification

Todo: `f88d7365-eaba-49a0-96cc-b0fa551bcdd0`; Pending; depends on task 2.

Cover custom providers, conditional variant selection, exact/normalized/ambiguous/unknown model identity, context size, explicit overrides (including false capability overrides), image/reasoning/output inference, adapter requests and visible provenance. Focused existing catalog tests passing is a baseline, not evidence that the requested fixes exist. Run the custom-provider user walkthrough and update setup/model-selection docs after implementation.

### 4. Context compaction trigger and observability

Todo: `5c3baae4-1de6-4985-88d2-76458b54c560`; Pending. User reported budget near 100% without visible summary compaction.

Owner: `docs/backlog/active/MEM-005-context-compaction-policy.md`. I298 / MEM-005-A is Complete through #691, Completion Commit `8d4ad50e4f8e9bc5c620db2641056b1c1118183d`, but only delivered request-projection trimming and budget recovery. It explicitly does not complete manual compact or new summary strategy. Establish a new effective claim.

Unify trigger and target with the complete outbound request budget, including stable prompt/tool schemas, reserved output and applicable reasoning budget, rather than history-only estimates. Make success, skipped/no-op and failure reasons observable; cover initial and continuation requests plus manual triggering. Preserve raw durable history, complete tool-call/result pairing, hidden-output boundaries, safe fallback and cancellation. Do not summarize hidden tool output into visible scrollback.

Entry points to inspect: `crates/talos-agent/src/compaction.rs`, `compaction/engine.rs`, `compaction/policy.rs`, `compaction/tests.rs`, `request_plan.rs`, `session.rs`, `session/`, and conversation/CLI/TUI typed-command/status adapters. Existing manual_compact APIs do not by themselves prove runtime wiring or whole-request triggering.

### 5. Model-callable context compaction

Todo: `b0bc5602-72b5-4a17-a67b-45425acd2fa4`; Pending; depends on task 4.

Model must be able to choose active compaction of its session. Route through the session Actor at a safe execution boundary; do not directly rewrite history while executing an in-flight tool batch. Return before/after estimates and explicit success/skipped/failure reasons. Preserve persistence, pairing, permissions and hidden-output boundaries. Add discovery, execution, active-turn ordering, concurrent cancellation and failure tests. Independent effective ownership is required; do not reuse closed I298 authorization. Exact tool name/schema and deferred-operation protocol remain to be designed.

### 6. Model-selectable command memory limit

Todo: `84e1268a-1091-458b-ad3d-546fcf4c8264`; Pending. Maintainer authorized exposing a subprocess memory-limit parameter; no implementation yet.

Cover bash and exec, foreground/background, sequential/parallel/pipeline and per-step input as applicable. Preserve default 2 GiB. Specify unit, accepted range, default/inheritance and invalid-value handling. Explain RLIMIT_AS is virtual address space, not RSS or a total process-tree quota. Do not silently imply enforcement on unsupported platforms.

Verified boundary: `crates/talos-tools/src/process_boundary.rs::apply_process_hardening` configures RLIMIT_CORE=0, RLIMIT_CPU=300 and RLIMIT_AS=2 GiB in Unix pre_exec. setrlimit errors are currently best-effort ignored; this is existing behavior, not a verified portable guarantee. `bash_tool.rs`, `exec_tool.rs` and UnixBackgroundLauncher call the shared boundary. Inspect all paths before changing signatures. Review escape vectors and ADR-007 authorization; do not add unaudited unsafe code. Include permission-boundary/security review and schema, propagation, invalid-value, background/foreground and platform tests plus docs.

### 7. Git permission resource defect and blocked handoff publication

Status: Blocked; newly added maintainer-requested recovery task. This is independent of tasks 1-6 and must not be omitted when transferring ownership.

The maintainer explicitly requested committing and pushing the handoff, then requested a retry. Both structured `git_add` attempts failed before staging succeeded. Identical arguments were used on both attempts:

```json
{
  "paths": [
    "docs/tasks/2026-10-10-session-todo-handoff.md",
    "docs/tasks/2026-10-10-session-todo-recovery.md"
  ]
}
```

Both returned exactly:

```text
Permission denied: permission state error: permission facet 0 has no safe typed resource
```

Publication update: after the two failures, the maintainer explicitly authorized temporary bash Git operations. Host `git status` confirmed an empty index and untracked handoff files; host `git add` of exactly the two documents and `git diff --cached --check` succeeded. Commit/push verification is still required at this writing; use Git history and remote state for the resulting publication SHA. This fallback does not fix the structured-tool defect, which remains outstanding. Historical statements below describe the checkpoint before this authorization.

Observed impact and state:
- No successful git_add from these attempts; git_commit and git_push were not attempted after either denial.
- No commit SHA or remote publication exists for the handoff from this session. The documents remain local. Read-only git_status/git_log/git_diff worked; file read/write/edit also worked. This does not prove other Git mutation tools work or share this defect.
- Intended commit scope was ONLY these two handoff/recovery documents, not the unrelated uncommitted code or MODEL intake changes. The maintainer requested commit and push; preserve that pending publication requirement after resolving the blocker.
- Do not treat the error as Git rejecting a file, a remote authentication failure, or an ordinary user denial: the reported failure is permission-state resource validation. Its implementation-level cause has not been traced.

Investigation and implementation checklist:
1. Reproduce against the actual active Git tool implementation and capture input-to-permission-facet resolution. Find the `git_add` implementation and the permission-state error literal; inspect tool permission_profile, resource kind/path construction, workspace containment, normalization and wrapper registration. Start in `crates/talos-tools` and `crates/talos-permission`, then trace runtime/CLI composition as needed. Paths are investigation leads, not verified defect locations.
2. Determine why facet 0 lacks a safe typed resource. Check whether this is missing facet resource metadata, a mismatch between Git staging resources and permission resource kinds, or validation/registration loss. These are hypotheses only. Record the actual cause and verify the active executable/build matches the source before claiming a fix.
3. Review related Git mutation tools for the same construction pattern, without expanding into unrelated refactors. Retain staging write permissions and fail-closed checks. Do not bypass the permission pipeline, disable validation, grant global permission, reset the worktree, or silently use host-shell Git. A specific shell fallback requires explicit user approval under the repository instructions.
4. Establish the appropriate governed owner/claim for the repair if needed. Changes to talos-permission or protected execution/permission boundaries require security review against escape vectors.
5. Add regression coverage for a valid workspace-relative file and this two-file staging request through the real permission wrapper. Cover approved success, explicit denial, missing/invalid typed resource, and outside-workspace/path-escape rejection as applicable to the discovered contract. Confirm errors degrade gracefully rather than panic or mutate on denial.
6. Validate that authorized git_add succeeds without relaxing unauthorized behavior. Then inspect staged and unstaged diffs separately and ensure the handoff commit contains only the two intended documents; preserve any preexisting staged changes. Follow conventional commit/model attribution rules, record the real commit SHA, push without force, and verify remote success. If commit/push encounters a separate failure, record it independently rather than assuming this fix covers it.

Acceptance: root cause is documented; focused permission/Git regression tests pass; security review is obtained where applicable; the exact authorized two-file staging operation succeeds; staged diff is reviewed; an actual handoff commit and successful push are recorded. Until then the task remains Blocked, and a receiving agent in another checkout must obtain the local documents and existing local changes separately.

## Existing Local Work: Preserve And Close Out Separately

Earlier session todo items are marked completed locally, not committed/merged delivery:
- Tool activity animation and queued message body visibility.
- Body prefix changed to the small bullet.
- Process tool human-readable summary fields: action/job_id/cursor/max_bytes/wait_ms, plus unit assertion.
- Todo status glyphs and in-progress highlighting (display-only).
- Edit diff text uses normal text with soft red/green backgrounds. All normal/head/tail output paths use an optional space fill. Projection pads each visual row for space fills (including CJK wrapping); decorative non-space fills remain last-row-only. Original logical text is unchanged.
- Model intake/evidence-scope correction: MODEL-014 introduced and MODEL-011/MODEL-013/index/Board adjusted; no capability implementation.

Important tests: `tool_display.rs::todo_tool_status_glyphs_and_highlight_are_display_only`, `edit_tints_survive_wrapping_and_head_tail_but_not_errors`; `history_projection.rs::edit_background_fills_every_projected_row` (2/40 lines; widths 7/20/80); `app/app_tests.rs::queued_body_remains_visible_beside_pending_tool_animation`.

Git status snapshot before writing this handoff:
- Added: `docs/backlog/active/MODEL-014-catalog-capability-inference.md`, `docs/tasks/2026-10-10-session-todo-recovery.md`.
- Modified: `crates/talos-agent/src/process_tool.rs`.
- Modified TUI: `src/app.rs`, `src/app/app_tests.rs`, `src/app/frame.rs`, `src/app/tool_activity.rs`, `src/history_projection.rs`, `src/scrollback.rs`, `src/state_tests.rs`, `src/tests.rs`, `src/theme.rs`, `src/tool_display.rs` under `crates/talos-tui/`.
- Modified docs: `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/backlog/active/MODEL-011-custom-model-capability-probe.md`, `docs/backlog/active/MODEL-013-catalog-context-window-inference.md`.

Structured staged diff inspection reported the two added docs as binary or unreadable; do not interpret that as proof of staging or content corruption. Reinspect staged/unstaged state with repository-supported tools before committing. No commit, branch creation, push, remote Issue update or release was performed in the visible continuation. These local files must travel with the handoff; HEAD alone does not contain them.

## Validation Evidence And Build Blocker

Pinned toolchain: Rust 1.99.0 from rust-toolchain.toml; use --locked and preserve Cargo.lock.

Passed:
- `cargo fmt --all`.
- `cargo test -p talos-tui --locked -j 1 -- --quiet`: 602 unit tests, 2 integration tests, 2 doctests.
- `cargo test --locked -p talos-config -j 1 catalog`: 19 tests, 207 filtered out.
- `cargo clippy --workspace --all-targets --locked -j 1 -- -D warnings`.

Build environment for the recorded Cargo runs:
```sh
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
RUSTFLAGS='-C link-arg=-Wl,--threads=1'
```

Failed:
- Standard `./scripts/release_preflight.sh`, job `job_40bd63ef-76cd-433a-b83a-c1351cf53a0a`, exit 101. Full logs recovered at handoff: site/installers/governance/collaboration/text/classifier/format/check and default workspace Clippy passed. Failure is in cargo test compiling CLI binary and CLI test binary, with `memory allocation of 5832720 bytes failed` / `2097152 bytes failed`, rustc SIGABRT. SDK fixture phase was not reached.
- `cargo test --workspace --locked -j 1 -- --quiet`, job `job_79e8c938-ac63-4751-8337-71d73eb5ae29`, exit 101. CLI test compilation fails with `memory allocation of 2097152 bytes failed` and SIGABRT; no workspace tests executed in this attempt.

Correction to earlier checkpoint: preflight failure stage was initially unknown because only part of the output was read. Continued cursor reads established the above test-compilation failure; use this document as the latest evidence. The jobs are terminated naturally; no live build remains.

Known fact: the source hardening default is 2 GiB and the current model-visible bash/exec schemas expose no override. Inference only: inherited address-space restriction may explain the allocation failure. Physical memory pressure versus RLIMIT/cgroup/other limits was not measured. Verify the actual execution process limits and memory pressure; do not report 2 GiB as proven root cause or weaken sandbox/permissions to get a green build. A trusted environment with sufficient address space may unblock validation before the memory-parameter feature is delivered.

## Resumed Verification Checkpoint

- Full standardized preflight passed on local candidate
  `0b396c48865b74e4cf3e7d0f711ccb208f264afd` (2026-10-10):
  `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 ./scripts/release_preflight.sh`.
  This includes workspace check, Clippy, tests/doctests, both governance validators
  with zero warnings, and the independent Runtime SDK fixture in default and coding configurations.
  The process exited 0; no build remains running. Earlier allocation failures did
  not recur, but their root cause remains unproven.
- Independent preserved-work review found a display-only Todo defect:
  `tool_display.rs` recognizes status rows by textual prefixes, so a multiline
  description containing `  [~] literal checklist` can be incorrectly rewritten
  and highlighted. C0 must repair contextual recognition and cover multiline
  titles/descriptions, including head/tail truncation, before delivery.
  This review is not merge approval; native acceptance remains outstanding.
- Follow-up source inspection confirms both `item.title` and `description` are
  interpolated without newline escaping in `talos-session/src/todo/formatting.rs`.
  A suffix/UUID regex alone cannot establish row provenance: user text can mimic
  that suffix or even a list header. The fix must preserve the raw transcript and
  avoid interpreting ambiguous user-authored continuation text as status metadata.
  Regression coverage must include forged list headers and UUID-shaped suffixes,
  not only ordinary multiline descriptions; evaluate producer-provided display
  provenance versus conservative rendering before choosing the implementation.
- Autonomous serial implementation and gated publication are authorized. On
  2026-10-10 the maintainer explicitly confirmed non-overlapping parallel work
  with other sessions' Active I293/I294. This queue remains internally serial;
  other-session ownership and branches remain untouched. Each new slice still
  requires its effective target-branch claim before committed implementation.
- This checkpoint records verification only. It does not complete C0 or any of
  the seven requirements, establish an effective claim, or authorize a release.

## Delivery And Documentation Gates

- This handoff does not activate a product owner or complete any of the seven tasks. The twice-reproduced structured git_add error remains task 7; the maintainer authorized temporary bash Git publication of only the two handoff documents. Verify publication in Git history/remote state; fallback success is not defect resolution.
- Verify independent claims/overlap against current target branch, not just this local snapshot. Owner documents retain authority.
- Tool work must update relevant user-facing tool/command reference; custom-model work must update setup/provenance/variant docs; compaction must document status/manual/model controls and persistence; memory parameter must document units/platform limitations. Choose existing targets through docs/sop/DOC-CHECK.md.
- Run both governance validators after governance edits and standardized locked preflight before merge. Require appropriate exact-head CI and API/privacy/permission/security review for affected slices.
- Review staged diff, secrets and conventional commit requirements. Completion status needs existing implementation SHAs, never a status-only self-reference. No publication, tag or release is authorized.
- Related prior checkpoint: `docs/tasks/2026-10-10-session-todo-recovery.md`. This handoff supersedes its incomplete failure diagnosis, not the product scope owners.

## 2026-10-12 MCP Cleanup Verification Checkpoint

T7/I300 remains Active. Shared registry connections now retain local cleanup
receipts without closing other connections' owners; CLI stdio explicitly owns
aggregate shutdown. Verified MCP EOF and ConnectionReset cancellation plus a
real descendant delayed-write cancellation test with shared-owner isolation.
Locked MCP tests: 13 library, 4 replay/transport, 1 real process test passed;
CLI check and MCP/CLI all-target Clippy passed. See the I300 owner for remaining
Agent/Runtime, Git commit/--all, mutation migration and full delivery gates.
This checkpoint does not complete T7 or advance the serial queue.

## 2026-10-12 Commit Execution Checkpoint

Previous-branch conditional Checkout checkpoint (2026-10-14): `checkout -` now
permits conditional configuration only when bounded native symbolic resolution
selects an existing local branch whose captured commit matches native resolution.
Full refs/reflog revalidation is retained, including a same-OID reflog-name drift
test under active conditional config. Bidirectional marker switches, complete
Checkout suite 28/28 and Clippy pass (49062), logs
`/private/tmp/i300-previous-conditional-suite.log` and
`/private/tmp/i300-previous-conditional-clippy.log`. Independent review found no
concrete defect in the limited extension. New production-source preflight remains
required; detached previous targets are not
enabled by this change.

Conditional-source protection evidence (2026-10-14): target rewrites of empty and
populated inactive `includeIf.onbranch` destinations now have direct prepared
regressions, preserving branch/index/source/marker on refusal and allowing an
unchanged existing branch. Prior unconditional-source creation coverage remains.
Focused regression and all-target Clippy pass (88038), logs
`/private/tmp/i300-inactive-include-protection.log` and
`/private/tmp/i300-inactive-include-clippy.log`. Production code is unchanged from
the passing preflight checkpoint; all original T7 residuals remain open.

Latest real MCP Checkout evidence (2026-10-14): the 15-case matrix adds approved
and denied existing-local-branch switches using external conditional config
sources. It checks current marker, retained external bytes, denial preservation
and replay refusal. Matrix and test-target Clippy pass
(`/private/tmp/i300-onbranch-mcp.log`, `/private/tmp/i300-onbranch-mcp-clippy.log`).
This is additional end-to-end evidence, not T7 completion.

Latest Checkout conditional-config checkpoint (2026-10-14): explicit existing local
branch switches now support `includeIf.onbranch` with initial-config retention,
native/captured target commit agreement and full revalidation. Real regression
covers benign bidirectional switches, stale active config refusal preserving
branch/index, and inactive target hooks that are refused on the following switch
when active. Checkout suite passes 28/28 (`/private/tmp/i300-onbranch-checkout-suite.log`).
Creating/tracking and other target forms remain refused. Git-enabled tools
all-target Clippy passes (`/private/tmp/i300-onbranch-clippy.log`). Follow-up
review and post-correction preflight results are recorded below; preflight 23429
failed with sandbox EPERM (`/private/tmp/i300-onbranch-standard-preflight.log`).
This supersedes the
historical blanket branch-conditional refusal, not T7's original scope or gates.

Follow-up security correction (2026-10-14): Git 2.54 active
`hook.<name>.command` configuration can execute without a hook file. Parent
configuration capture now refuses all such entries before mutation, including
entries supplied by an active `includeIf.onbranch` source. Independent follow-up
found no remaining concrete defect in this limited slice. Admission-before-drift
ordering is now exercised directly. The Checkout suite still passes 28/28
(`/private/tmp/i300-onbranch-hook-checkout.log`) and all-target Clippy passes
(`/private/tmp/i300-onbranch-hook-clippy.log`); the prior standard preflight failed at the existing
process-observation assertion with sandbox `EPERM`, so a post-correction
unsandboxed preflight 72934 exited 0 with `release preflight: passed`, including
tools 648/648 and both external SDK modes
(`/private/tmp/i300-onbranch-hook-standard-preflight.log`). Governance and claims
also pass with zero warnings (52109). Final delivery gates and original T7
residuals remain open; no implementation completion SHA exists.

T7 remains Active. Agent and Runtime real descendant cancellation tests passed;
Runtime additionally proves owner admission is closed after shutdown. Following
independent review and source-capture/revalidation fixes, prepared commit's local
temporary refusal was removed. Real ordinary/index-only, --all/untracked,
no-change, and configured automatic-maintenance tests pass. These are partial
acceptance only; push/pull/checkout migration and full delivery gates remain.

## Checkout Production Continuation Checkpoint

Supersedes earlier notes calling checkout absent or prototype-only: checkout now
runs through the production prepared permission route. Current/index/target filter
checks, target ref/reflog drift checks and linked-worktree metadata authorization
are integrated. Same-path nonrecursive gitlinks preserve dirty child worktrees,
declare child index refresh writes, and refuse configured child fsmonitor before
mutation. Git library tests passed 164/164 and Git-enabled all-target Clippy
passed. Added/removed/type-changing gitlinks and recursive checkout remain
explicit unsupported cases; push/pull still use their old route and require
migration. Full final-candidate preflight, review and publication remain pending.
No T7 completion or advancement of the remaining serial queue is claimed.

Latest continuation supersedes the blanket gitlink-add/remove limitation above:
missing-path additions and nonrecursive removals now pass real acceptance while
preserving dirty child contents. File/directory transitions and conflicting edits
are covered; real MCP checkout denial/success/replay/cleanup acceptance was added.
Independent checkout review found target-created hooks and target-rewritten
configuration sources. These are now checked before mutation, including aliases,
empty/missing include destinations and system/global/local/worktree config paths.
Branch-conditional includes remain explicit unsupported configuration. Gitlink
existing-path additions/type transitions, recursive checkout, push/pull migration,
final validation and publication remain pending. See the latest I300 owner entry;
this is concrete partial implementation, not T7 or whole-queue completion.

Push continuation: production preparation now uses independently admitted config
discovery and resolved-destination continuation facets with owned cleanup. Direct
execution and stale discovery proofs refuse. Mutation remains explicitly closed
while transport effect implementation/review is pending; restore supported push
before delivery. Pull now fails closed before the old route and remains without
prepared support. Latest tools library verification passes 173 tests. This does
not complete T7 or any subsequent serial deliverable.

Push review continuation: corrected URL rewrite tie order against real Git and
added pre-push/reference hook paths plus signing/recursive/custom-transport helper
checks. Latest tools library tests pass 176/176; all-target Clippy passes. Push
transport remains disabled pending complete effects implementation and review;
Pull migration and all later serial deliverables remain open.
Pull child configuration continuation repair checkpoint (2026-10-16, `88246a89`):
the Pull child configuration stage now records an explicit discovery state and
the complete Git-reported source frontier. Correction (`b11c91ee`): the next
continuation verifies exact Read admission, retains and revalidates the original
snapshots, and compares values/origins before accepting discovery; changed
includes refuse before new source capture. Shared preparation tests pass 129/129
and configuration identity/origin tests pass 3/3. `8a312b3e` restores the distinct
Commit/Checkout gitdir interpretation in shared revalidation.
Locked Pull tests pass 76/76, `cargo check --locked -p talos-tools --all-features`
passes, and all-target all-feature Clippy with `-D warnings` passes. Child
remote semantics, object admission, recursive fetch, integration, and delivery
gates remain open; Production Pull remains disabled.
Pull child gitdir binding correction checkpoint (2026-10-16, `afc61b1c`):
Pull child identity admission now resolves the `.git` marker before capturing
`ChildLocation`, so external gitdirs and linked common directories are bound to
the actual metadata roots. This only establishes helper-level resolution:
the helper-only implementation is superseded by `98dff9b9`: marker, gitdir/fixed
pointer, and common-root discovery now use separate fresh admissions, retaining
Root/Metadata evidence and refusing marker/commondir redirection. The new test
advances real prepared plans with permission-engine path proofs, including
targets absent until their own wave. All 779 tools library tests pass. This does
not prove end-to-end runner stage capacity or recursive fetch delivery.
The stage also requires the dedicated configuration-
chain capability before invoking Git with includes. Locked Pull tests pass
76/76, cargo check and all-target all-feature Clippy with `-D warnings` pass.
Dedicated external-gitdir/include fixtures, object admission, recursive fetch,
integration and delivery gates remain open; Production Pull remains disabled.

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

Child fetch external-effect checkpoint (2026-10-17): parent and child transport
preparation now share the existing fetch bundle/proxy/external-maintenance
checks. Final empty bundle URI and disabled automatic maintenance retain their
existing semantics; invalid maintenance booleans refuse. Configuration tests
pass 27/27, Pull-focused tests pass 100/100, and all-target/all-feature Clippy
passes with warnings denied. Independent incremental security review found no
blocker. The initial new fixture lacked its configuration source file and was
corrected before these runs. This closes a child preparation gap only; hooks,
credentials, transport planning and actual child fetch remain open, as do all
original T7 and handoff delivery requirements. Production Pull stays disabled.

Child fetch hook checkpoint (2026-10-17): configuration discovery now publishes
exact child fetch hook Read paths before inspecting contents. After effective
worktree/root binding and path admission, all children share one deduplicated
4 MiB hook content budget; retained evidence is rechecked on continuation.
Executable hooks refuse instead of being silently disabled. Independent review
found the inherited linked-worktree default-path bug: default hooks belong to
common_dir/hooks, not private gitdir/hooks. Both parent and child now use that
default; explicit relative hooksPath remains relative to the effective worktree.
Real linked-child coverage refuses an executable common hook, and regressions
cover missing exact hook authorization, later content changes, and path-only
planning without reading targets. The Pull-focused suite passes 101/101 and
independent follow-up review found no remaining blocker for this increment.
Child fetch execution/transport credentials, recursive retries, parent integration,
Windows input and all original runtime/publication/closure gates remain open.
T7 remains Active and production Pull remains disabled.

Child fetch execution work-in-progress checkpoint (2026-10-17): the uncommitted
stage now executes at most one child fetch before rebuilding every child graph
inventory observing the changed metadata roots. Per-child completion replaces
an all-children flag, and pending children regain their own Network/Write facets.
Parent-graph overlap currently refuses before mutation and still requires a
coordinated refresh implementation. Stage errors disclose possible retained
partial fetch effects. The attributes-only fixture correctly selects integration
when no children exist. Pull-focused tests passed 102/102 before strengthening
the skip fixture; the strengthened real-daemon test subsequently passed with a
nonempty reachable commit set freshly probed after write approval. This is not
completion evidence: shared linked-child regression, independent security
re-review, full tools validation, parent overlap handling and the existing
transport/recursive/integration/runtime gates remain outstanding. Production
Pull remains disabled. Resume from the uncommitted prepared_add.rs changes;
validation logs are /tmp/talos-child-fetch-refresh.log and
/tmp/talos-child-fetch-nonempty.log.

Child fetch shared-graph follow-up (2026-10-17): real Git daemon coverage now
runs two linked worktrees sharing a common store, refreshes graph evidence after
the first fetch, and successfully fetches the second. A separate branch mutates
a reference between fetches and verifies refusal. Independent incremental review
confirmed the two prior graph-evidence blockers are resolved; its remaining
post-fetch configuration error-context finding is fixed for resumed discovery.
Full locked all-feature tools library tests passed 809/809 (outside the sandbox
for process-supervision observation). Final focused tests/Clippy are being
rerun after the error-context correction. The parent overlap refusal and all
original activation/closure residuals remain, without any completion claim.
Final error-context correction validation: locked Pull tests 102/102 and
all-target/all-feature warnings-denied Clippy passed. This stage remains a
production-disabled implementation increment, not T7 completion.

Child module-name scheduling checkpoint (2026-10-17, in progress): selected
initialized current children now suppress same-name historical paths after
successful identity capture; names remain case sensitive. Commit probes and
historical unconditional recursion policy follow the selected module name
across path changes. Independent review caught an initially path-only policy
comparison; the correction and real-daemon renamed-path/On/reachable-object
regression pass with all 103 locked Pull tests. Missing/uninitialized/legacy
repository fallback and final security follow-up remain open. Origin/main was
fetched and advanced 21 commits to 891f6c49; rebase must follow this local stage's
validation and commit. Production Pull remains disabled and T7 remains Active.
Module-name follow-up: independent re-review found the policy correction closes
its blocker. Path/name matching now compares joined paths to avoid Windows
separator differences. Locked Pull tests pass 103/103 after that adjustment;
Clippy and both governance validators pass for this increment.

Rebased validation checkpoint (2026-10-18): all 128 local implementation commits
were rebased onto origin/main 891f6c49; current stage commits are 24cff179
(child fetch/shared graph refresh) and 208ad52e (module-name scheduling).
Only EVOLUTION.md conflicted; both branches' lessons were retained. The first
post-rebase standard preflight passed workspace check and Clippy, but test
compilation terminated with ENOSPC, not a test assertion failure. Under the
existing maintainer authorization, rebuildable target/debug artifacts (17 GiB)
were removed. Standard preflight is now running again with CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0 and CARGO_BUILD_JOBS=2;
log: /tmp/talos-i300-rebased-preflight-compact.log. No full-suite success is
claimed. Native Git 2.54 repo_submodule_init confirms an unpopulated module may
fall back to the parent's modules/name gitdir without a worktree; the current
Root-based child admission does not yet implement this behavior. That fallback
requires separately admitted metadata identity rather than a fabricated
worktree binding. All original T7 activation and handoff closure gates remain.

Relative backlink resolver work-in-progress (2026-10-18): the uncommitted
pull_inventory.rs change supports ParentDir in a missing checkout suffix.
After each missing-component cancellation it immediately resolves the remaining
path again, preserving existing symlink semantics and retaining every ancestor
identity. An iterative loop avoids path-controlled recursion. Independent
review caught an earlier interleaved symlink/ParentDir error; the latest
correction was re-reviewed without a remaining blocker. A native Git worktree
list fixture confirms the relative missing/../link/../checkout backlink maps to
actual/checkout when link points to actual/nested; absolute backlinks retain
their separate existing behavior. Added tests cover simple cancellation,
existing and newly exposed symlinks, exact admission and directory replacement.
The first focused Cargo command omitted the Git feature and ran zero tests;
it is not validation evidence. All-feature focused tests (session 73693) and
the complete inventory module tests (session 17590) are queued behind standard
preflight (session 93128). Logs respectively:
/tmp/talos-i300-forgiving-target-all-features.log,
/tmp/talos-i300-forgiving-inventory.log,
/tmp/talos-i300-rebased-preflight-compact.log. Do not commit the resolver as
validated until these tests and affected Clippy checks have actually passed.

Resolver validation follow-up (2026-10-18): all-feature focused tests passed
5/5, but the complete inventory suite exposed the inherited receiver syntax
guard rejecting the new relative checkout backlink before resolution. The
correction exempts only checkout gitdir backlinks from that receiver-specific
guard; commondir retains the strict rule and every checkout target still needs
exact normalized-path admission. The complete inventory suite now passes 27/27,
including the interleaved backlink admission/replacement regression; formatting
passes. Latest log: /tmp/talos-i300-forgiving-inventory-final.log. Affected
all-feature/all-target Clippy and final security follow-up remain in progress;
standard workspace preflight is still running. This supersedes the earlier
queued-test state without claiming T7 completion.

Real authorization follow-up (2026-10-18): affected all-target/all-feature
Clippy passed. Inspection of talos-core normalize_authorized_path confirms it
normalizes ParentDir lexically before filesystem canonicalization; interleaved
symlink backlinks may therefore disagree with Git resolution. The resolver
continues to refuse such mismatched authorization, and a regression now uses
the real normalizer to assert that refusal. This is a remaining compatibility
gap, not a completed ordinary runtime path. The full workspace preflight ended
with three tools failures: the earlier pre-correction backlink test binary and
two native_inventory baseline checks. The four protected source files were
compared with origin/main 891f6c49; only process_boundary.rs differs, by shell
feature cfg gates. Independent protected-source review is requested before any
fingerprint update. Latest focused log:
/tmp/talos-i300-forgiving-real-normalizer.log. Full preflight must be rerun
after corrections; no full-suite success is claimed.

Protected-source review follow-up (2026-10-18): independent reviewer
/root/review_prepared confirmed only shell feature gates changed in
process_boundary.rs relative to 891f6c49; native operation bodies, arguments,
resource transfers and cleanup are unchanged. Refreshed that file's fingerprint
only. All three native_inventory tests pass with --locked and --all-features.
This does not establish Windows runtime evidence. Corrected standard preflight
is running as session 76149; log:
/tmp/talos-i300-rebased-preflight-corrected.log. The real-normalizer compatibility
gap remains open; production Pull remains disabled.

Resolution authorization design review (2026-10-18): resolver increment committed
as 98de1988; native inventory baseline committed as 92b4ca8d. Both governance
validators pass. Independent review rejected the proposed raw-approval followed
by resolved-approval shortcut: WorktreeTarget::capture itself canonicalizes and
captures ancestor metadata/Windows handles before a mismatched resolved target
has admission. Refusing the mismatch is therefore not proof that all discovery
reads were authorized. Production Pull must remain disabled. A component-wise
admitted resolver or explicitly accepted bounded link-metadata capability is
required before activation; the proposed configuration-only capability in
ADR-089 is not accepted and does not cover Pull. Do not implement a second-stage
resolved token as a retroactive grant for the first resolution. Existing
27/27 resolver tests establish parsing/identity behavior, not this permission
boundary. Full preflight session 76149 remains live at this checkpoint.

Corrected preflight result (2026-10-18): session 76149 exited 101. Workspace
check and Clippy passed; tools had 824 passing tests and one failure:
prepared_local_push_runs_all_eight_admitted_stages could not launch ps under
the sandbox (EPERM at prepared_add.rs:7464). The exact all-feature locked test
was rerun outside the sandbox as session 97898 and passed (1/1). This isolated
pass is not a full-preflight pass. A fresh complete standard preflight outside
the sandbox is running as session 59099, log:
/tmp/talos-i300-preflight-outside-sandbox.log. ADR-089 now contains a concrete,
disabled Pull worktree-resolution metadata proposal; maintainer decision has
been requested. No new capability is accepted by the implementation yet.

Outside-sandbox preflight follow-up (2026-10-18): session 59099 exited 1 only
at the final external SDK fixture check. All workspace tests and doctests,
check, Clippy and preceding preflight phases passed. The independent fixture
Cargo.lock omitted talos-tools' accepted same-file direct dependency. A minimal
one-line lock correction is under validation as session 52464, log:
/tmp/talos-i300-sdk-lock-validation.log; unrelated offline resolver downgrades
were discarded before this run. Full preflight success is still not claimed.
Fetch found origin/main 8fa2a500 (#732); rebase waits for the fixture validation
and checkpoint commits so no test runs across a moving source tree. Independent
review refined the disabled Pull resolution proposal's commondir syntax,
external chain scope, invocation-wide budgets and Windows handle accounting.
Maintainer decision remains pending; no capability is enabled.

Rebase and native fallback evidence (2026-10-18): SDK fixture session 52464
passed in both default and coding modes after the one-line lock fix. All 133
local commits rebased cleanly onto origin/main 8fa2a500. Rewritten stage SHAs:
a172c725 (native inventory), 48de42ad (backlink resolver), 0a4f33d1 (SDK lock),
40ea5b83 (pending authority proposal). Post-rebase preflight session 99760 is
running; log /tmp/talos-i300-post-rebase-preflight.log. A new test-only native
fallback fixture was added while that run was active; do not claim the run
validates that new test unless its output selects it. Separate all-feature
focused validation is session 17853, log:
/tmp/talos-i300-unpopulated-native-test.log. A real Git fixture confirms an active
unpopulated child with no checkout/.git can fetch via modules/named-child when
the fetched parent advances its gitlink. The first manual fixture omitted an
upstream parent advance and activity restoration, so it was inconclusive and
is not positive evidence. Prepared Pull does not yet implement this fallback:
its bootstrap config command currently requires a worktree and must gain an
explicit metadata-only mode with fresh admission, not a fabricated worktree.
The Pull resolution metadata proposal remains unaccepted and disabled.
Focused native fallback regression session 17853 passed 1/1 with --locked and
--all-features. It is native behavior evidence, not prepared fallback delivery.
Affected all-feature/all-target Clippy session 40706 is running.

Post-rebase full validation result (2026-10-18): standard preflight session
99760 exited 0 with `release preflight: passed`. The tools binary selected
830 tests and did not contain the later native fallback test; its separate
locked all-feature run passed 1/1 and affected all-target/all-feature Clippy
session 40706 exited 0. SDK default/coding execution passed in the full run.
Native fallback evidence is committed as acd572e8. This closes the current
local preflight failures, not T7 activation or delivery: metadata-only child
execution, recursive retry/integration, resolution authority and original
platform/runtime/publication gates remain. No fresh full preflight is needed
solely to repeat the separately verified test addition; substantive upcoming
code changes require fresh appropriate validation and final exact-head gates.

Legacy child admission frontier (2026-10-18): selected modules now publish a
name-based `common/modules/<name>` exact Read candidate in the existing identity
wave. Construction performs no filesystem inspection; fresh approval is required,
and malformed names omit this candidate without blocking populated discovery.
Pull tests pass 109/109, the separate lexical escape regression passes 1/1,
and all-feature/all-target Clippy passes with warnings denied. Independent
review found no blocker for this inert frontier. This does not implement fallback
execution: prefix-repository/alias validation, configured gitdir mappings and
explicit populated/metadata-only layout remain required. Native source review
also confirms scheduling and layout are independent: index tasks preserve
configured worktree setup, while changed tasks always override `--work-tree=.`
with cwd=gitdir. Do not apply that override to every metadata-only child.
Fetch advanced origin/main from 8fa2a500 to dea3edcc; all 136 local commits
rebased cleanly. Frontier implementation is now 565fc093 and preceding validation
documentation is 7aa96710. Post-rebase locked Pull tests again pass 109/109 and
all-feature/all-target Clippy passes with warnings denied (logs:
`/tmp/talos-i300-frontier-post-rebase-{tests,clippy}.log`). Both governance
validators passed before rebase; final delivery still requires exact-head gates.

Pull recursion precedence correction (2026-10-18): native source review found
that the previous OR of current/historical unconditional modes was incorrect.
The selected current-index task now owns its recursion mode; a historical On
cannot upgrade current OnDemand. The real fetch fixture now expects renamed-on
to skip reachable objects and separately verifies historical-only On still
fetches. Its scheduling state is established before discovery rather than changed
after approval. Locked Pull tests pass 109/109 and all-target/all-feature Clippy
passes; independent review found no new blocker. These fixtures manually seed
the selected task state, so they do not prove native changed-list calculation or
full runner delivery. Metadata-only fallback, execution layout, resolution
authority and original T7 closure gates remain open. The reusable test-model
lesson is recorded in EVOLUTION.md.

Test module separation (2026-10-18): moved the complete prepared Git test module
to `git_write/prepared_add_tests.rs` without changing its module path or fixture
visibility. `prepared_add.rs` is now 7,258 lines rather than 17,742; production
logic is unchanged by this move. All 137 prepared-plan tests pass outside the
sandbox, and all-target/all-feature Clippy passes. The sandbox run passed 136
and failed only the existing process-observation EPERM case; the full outside
rerun closes that environment failure. Independent review compared the extracted
module and confirmed scope/content preservation. Production orchestration remains
large and T7 remains Active; this is test separation, not feature completion.

Legacy child device-name boundary (2026-10-18): the lexical fallback candidate
now omits Windows DOS device names, including extensions, superscript COM/LPT
numbers and spaces before extensions. The latter alias was found during
independent review and added to regression coverage. Both candidate tests pass
with --locked; all-feature/all-target Clippy passes with warnings denied.
This is portable candidate filtering only, not native Windows runtime evidence
or metadata-only fallback execution. Filesystem prefix/alias validation,
execution layout, resolution authority and original T7 delivery gates remain.

Child fetch execution-base correction (2026-10-18): real Git 2.54 probes show
relative core.hooksPath and remote paths resolve under cwd=gitdir, including
configured/stale worktrees and explicit worktree overrides. Both hook admission
waves and local-source candidates now use the retained gitdir, not the logical
checkout or configured worktree. A Unix native regression checks actual hook cwd
and relative remote success in five layouts; the prepared fetch matrix checks
executable relative-hook refusal without fetch and exact relative-local facets.
Focused child tests pass 5/5; locked Pull tests pass 110/110; all-feature/all-target
Clippy passes. Stale core.worktree did not fail in these fetch probes, so native
source commentary is not evidence that this case must fail. This correction does
not implement missing-checkout fallback, enable local transports, remove startup
admission, or activate production Pull. Full T7 and handoff closure remain open.

Child fetch command separation (2026-10-18): fetch now has a dedicated command
builder instead of sharing the explicitly bound object-probe worktree override.
Current-index tasks, including OnDemand, keep native worktree setup; historical
ones use --work-tree=. with cwd=gitdir. Object probes retain their separately
bound layout. The real fetch matrix verifies arguments and execution for current,
OnDemand, renamed/current-wins and historical-only cases. Scheduling inputs are
still fixture-seeded, not end-to-end parent discovery evidence. Locked Pull
suite passes 110/110; all-feature/all-target Clippy passes; independent review
found no blocker. Existing startup target admission/revalidation is retained;
metadata-only fallback and removal of unnecessary historical target admission
remain open. Production Pull stays disabled and original closure gates remain.

Historical child probe layout (2026-10-18): historical selected tasks now bind
object probes to their admitted gitdir and omit admission of the core.worktree
target overridden by fetch. Startup configuration bytes remain retained and
revalidated; current-index tasks retain configured target admission. The real
historical fetch regression succeeds with a nonexistent configured worktree.
Locked Pull tests pass 110/110 and all-feature/all-target Clippy passes with
warnings denied (logs: /tmp/talos-i300-historical-layout-{tests,clippy}.log).
Independent review found no blocker. This does not implement missing-checkout
metadata fallback; production Pull and all original T7 closure gates remain open.

Prepared test responsibility split (2026-10-18): the extracted 10,582-line test
file is now a 110-line shared-fixture entrypoint plus six include fragments for
authorization, checkout, commit, pull, pull children, and push. Includes retain
the original module path, private access and external execute_approved helper.
All 140 conditional test declarations and extracted function contents are
preserved. No production behavior changes. All-feature/all-target locked Clippy
and formatting checks pass. The sandbox run passed 137/138, with the existing
process-observation EPERM case requiring an outside-sandbox rerun; final rerun
evidence is recorded below. T7 and the full handoff remain Active/incomplete.
Outside-sandbox locked rerun passes 138/138 prepared tests
(/tmp/talos-i300-test-fragments-unsandboxed.log); the environment failure is
closed for this slice. This is focused test evidence, not the final workspace
preflight, Windows runtime validation, or production Pull activation.

Pull worktree-resolution decision accepted (2026-10-18): maintainer explicitly
accepted the dedicated git:pull-worktree-resolution metadata-only capability
with 64 link hops/source and invocation-wide 4096-entry/4-MiB limits, followed
by separate exact target Read approval. ADR-089 records acceptance without
enabling the capability or waiving denial/drift/budget/security-review gates.
The existing unbounded resolver must be replaced before consuming this scope;
production Pull remains disabled and T7 remains Active.

Legacy metadata prefix approval frontier (2026-10-18): selected legacy
candidates now publish the modules container and each enclosing module-name
prefix, plus its HEAD/objects/refs, before any future collision validation.
Candidate construction remains filesystem-free, with invalid lexical names
producing no prefix resources. The real permission-pipeline regression checks
stale approval and individual token removal for every prefix resource while
preserving populated-child discovery. Locked focused test passes 1/1, Pull
tests pass 110/110, all-feature/all-target Clippy and formatting pass. Logs:
/tmp/talos-i300-prefix-frontier-{tests,pull-tests,clippy,governance}.log.
This only publishes the read frontier; identity/alias binding, collision checks,
configured mapping and metadata-only fallback execution remain open. No new
filesystem reads or production Pull activation are introduced.
Independent review found no blocker in this frontier-only change; it explicitly
retains prefix identity, alias and enclosing-repository validation as residuals.

Bounded Pull resolver characterization (2026-10-18): added a test-only,
component-wise resolver and a separate test file. It retains cancelled ancestors,
absence, link text and directory identity; refuses file components, dangling
links, unsupported entries and Windows drive-relative link targets; bounds link
expansion to 64 hops and charges an explicit cumulative budget before metadata
inspection. Revalidation replays retained evidence without resetting counters.
Five focused tests pass, covering the hop boundary/loop, symlink-before-parent
semantics, redirection, missing suffix/ancestor drift, file-parent rejection and
atomic cumulative budget exhaustion. The hop fixture canonicalizes its root so
macOS temporary-directory aliases do not consume its intended 64-hop boundary.
All-feature/all-target locked Clippy passes. This module is compiled only for
tests: dedicated capability admission, invocation-wide inventory accounting,
replacement of the old resolver, Windows runtime evidence and independent
security review are still required before production use. The accepted decision
and T7 are not closed; production Pull remains disabled.

Windows foreground supervision checkpoint (2026-10-18): independent review of
49cc215ff710aae078716366e171ab4b485d3b69 found a merge blocker: the non-Unix
structured Git runner used direct-child cleanup, so Commit maintenance and Push
receiver descendants could survive cancellation. The maintainer accepted extending
ADR-068 by reusing the existing Job launcher, without a new dependency or duplicate
Job FFI. Acceptance is now recorded in ADR-068. As immediate containment, all
non-Unix structured Git preparation refuses before configuration discovery, and
the command runner independently refuses instead of invoking the unsafe fallback.
This is not the Job implementation or cross-platform completion. Remaining work:
lossless command/environment transfer, owned foreground Job operation and cleanup
receipt, bounded stdin/output/deadline/cancellation/shutdown, real Windows tests,
and fresh independent exact-head review. Existing Windows success tests require
explicit reconciliation with containment; do not hide failures with test bypasses.
The full preflight started on 49cc215f is still running and cannot certify these
subsequent changes. No stage has been merged. T7 remains Active.

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

Current Windows checkpoint (2026-10-18): cleanup confirmation is committed as
02f5136c (ActiveProcesses must reach zero within five seconds), explicit NUL EOF
as f358673f, and exclusive Pipe/Null ownership as 1d7fe170. Source review found no
blocker in these deltas; Git-only Windows cross-check and macOS all-feature locked
Clippy pass. Windows runtime tests have not run. Native input fault/release proof,
foreground operation/receipt wiring and the complete Pull implementation remain
open. Both governance validators pass with zero warnings. Fresh origin/main fetch
shows no upstream-only commits; no rebase needed. No T7 stage has been merged.

Windows foreground execution checkpoint (2026-10-18): the shared
native Job launcher now backs the foreground executor, with retained IO workers,
finite stdin flush, cancellation, output/deadline bounds and sticky unconfirmed
cleanup on launch/worker failure. Native launch unwinds follow the same raw-owner
cleanup path; attribute-list pointers enter cleanup only after initialization.
Independent review found the latter failure-path defect during this integration.
Separate Windows-only tests cover IO panic versus ordinary IO failure and operation
receipt isolation; these tests have not been compiled or executed on Windows.
Evidence correction: earlier Git-only Windows checks excluded `process_boundary`
through its module cfg and therefore did not validate native Job code. The current
module cfg includes Git on Windows; its locked library cross-check passes (with
unused background/fallback warnings). macOS all-feature/all-target locked Clippy
passes for the foreground integration. Production non-Unix preparation remains
closed; real Windows runtime validation, final review and current-head full gates
remain outstanding. T7 and the overall handoff remain Active, not delivered.
Foreground runtime regression checkpoint: six explicit test names are now required
by Windows CI, including binary finite stdin/EOF, timeout cleanup, cancellation by
dropping the caller, and lifecycle/worker-failure checks. Latest fetch confirms
zero upstream-only main commits (local branch ahead 155 before this stage).

Stage validation: standard `./scripts/release_preflight.sh` exited 0 for this
source candidate, including locked workspace check/Clippy/tests and external SDK
fixture (`/tmp/talos-i300-foreground-preflight.log`). The earlier sandbox EPERM
push regression passed outside the sandbox; the refreshed protected native-source
inventory tests pass 3/3 after independent review. No Windows runtime or merge
completion is inferred from these macOS results.

Stage publication (2026-10-18): pushed implementation candidate f16c2d09 and
opened draft PR #735 (https://github.com/wjhuang88/talos/pull/735), based on the
effective #704 claim. The candidate includes the accumulated T7 implementation;
it is not a Windows-only change and does not close T7. Local standard preflight
passed as recorded above. Real Windows CI and exact-head final review remain
required. Production Pull and non-Unix preparation remain disabled. Resume with
PR #735 check results and invocation-wide Pull budget/resolution authorization;
preserve the full original handoff queue.

CI follow-up (2026-10-18): PR #735 run 39114660162 failed Windows Clippy on
three unused legacy host test helpers and Linux preflight on the post-fetch
metadata evidence fixture. Local fixes restrict the helpers/imports to tests
and apply the accepted foreground maintenance settings to that fixture, retaining
all stale-content/location assertions. Background maintenance is a suspected,
not proven unique, cause of the Linux failure. Targeted independent review found
no new blocker. Host-boundary tests pass 8/8; post-fetch fixture passes 1/1;
Windows Git-only library cross-check passes (7 unrelated feature-only dead-code
warnings remain). Full-feature Windows cross-Clippy cannot run locally because
x86_64-w64-mingw32-gcc is absent; real Windows CI remains mandatory.
Local preflight session 65235, /tmp/talos-i300-ci-correction-preflight.log, is
still running; its source snapshot predates the fixture-only adjustment, whose
focused test passed separately. Do not label this a final-head full pass.
Pull retained-byte discovery was separately committed as 4c9fec62 after 28/28
inventory tests, Git library Clippy, both governance validators, and independent
review. Shared budgets/raw-target authorization and the full queue remain open.
Preflight continuation: session 65235 exited 101; the sole tools failure was
prepared_local_push_runs_all_eight_admitted_stages, whose process observation
returned sandbox EPERM (857 tools tests passed). An approved sandbox-external
standard preflight is now running in session 78771, log
/tmp/talos-i300-ci-correction-preflight-unsandboxed.log, against cd9755be source.
Windows test-target cross-check also requires the absent MinGW C compiler and
therefore did not compile/validate the Windows tests. Do not infer success.
Local commits 83a0521f and cd9755be contain the independently reviewed CI fixes;
keep these local until the stable-candidate validation checkpoint completes.
Standing gix tracking recheck (2026-10-18): GitHub releases API still lists
gix-v0.89.0 (published 2026-10-08) as the newest gix crate release in the most
recent 100 releases; gix-utils/gix-error patch releases on 2026-10-11 are separate
crates. Release notes confirm linked-worktree add/checkout/removal APIs and
submodule configuration trust/provenance fixes, not an announced complete Talos
push/pull/add/commit replacement. This corroborates, rather than supersedes, the
2026-10-15 source audit. No dependency update or host-Git removal date is inferred.
Source: https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.89.0 .

CI correction preflight result (2026-10-18): sandbox-external session 78771
exited 0; its log ends `release preflight: passed`. This validates cd9755be
source through the standard locked workspace gates and external SDK fixtures.
The owner I300 records the result. Publish the reviewed local correction batch
to PR #735 and obtain fresh remote CI; Windows runtime verification, shared Pull
budget/resolution admission, full Pull execution and the original remaining queue
are still open. No T7 completion or merge is claimed.


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

Windows metadata-handle correction (2026-10-18, `f4c3ba0d`): PathIdentity now
uses metadata-only handles with explicit read/write/delete sharing instead of
same-file's content-read path constructor. Independent review found no blocker;
all original replacement assertions remain, local snapshot tests pass 9/9 and
Windows GNU Git-only library check passes (seven existing feature-only warnings).
Native Windows runtime success remains unproven. Standard full preflight is
running in session 48484, log `/tmp/talos-i300-metadata-handle-preflight.log`.
Resume by checking that session, then publish a validated stable candidate for
fresh Windows CI. Shared full resource accounting, resolution authorization,
complete Pull execution and the original remaining queue are still open.

Shared alternate evidence checkpoint (2026-10-18, `d830597a`): fixed alternate
files join the shared reference-file allowance; repeated binding reuses retained
evidence after drift checks. Nonempty alternate stores still refuse. Independent
review found no blocker; inventory tests pass 33/33 and all-feature library
Clippy passes. Session 48484 failed on sandbox PermissionDenied executing CLI
fixtures. Approved standard preflight session 96720 is live with log
`/tmp/talos-i300-shared-alternates-preflight-unsandboxed.log`; inspect that handle
before restarting any validation. Native Windows CI, unified non-file resource
accounting, dedicated resolution authorization, complete Pull and the remaining
original queue are still required.

Stage synchronization (2026-10-18): fetch succeeds, current implementation
`d830597a` is ahead 170 / behind 0 origin/main; no rebase needed. Both governance
validators pass without warnings. CI 39122382667 is terminal with Linux workspace
success and the previously recorded Windows identity runtime failure; it predates
the corrective handle change. Check live preflight 96720 before publishing.

Stable-candidate gate result (2026-10-18): session 96720 exited 0 and standard
preflight passed, including tools 865/865 and external runtime SDK fixtures,
for implementation `d830597a`. Log:
`/tmp/talos-i300-shared-alternates-preflight-unsandboxed.log`. Publish this
reviewed stage to #735; native Windows verification and the full remaining
handoff are still required. No T7 completion is claimed.


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

2026-10-19 resumed execution: `bc7fd24a` repairs the Windows checkout reference
movement fixture to reach the actual mutation stage through fresh approvals,
rather than assuming one discovery continuation. Local focused test passes;
native Windows rerun remains required. CI 39225369123 Windows job 117970282428
passed Desktop then failed the old fixture with incomplete-discovery rejection;
overall run was still live when checked. Log:
`/tmp/talos-i300-windows-39225369123.log`.
`6587407c` adds native merge-editor evidence: Git 2.54.0 with null stdin executes
GIT_EDITOR when GIT_MERGE_AUTOEDIT=yes, but not when unset or no. Focused locked
all-feature test and all-target/all-feature Clippy pass. This narrows effect
handling requirements without changing inherited configuration or enabling Pull.
Fresh fetch requires no rebase (ahead 240 / behind 0 before these commits).
Standard preflight session 51544 is live; log
`/tmp/talos-i300-discovery-editor-preflight.log`. Inspect before rerunning.
T7 and the original full task queue remain incomplete; no completion is claimed.
Editor follow-up: existing `validate_environment` already rejects inherited
GIT_MERGE_AUTOEDIT/GIT_EDITOR overrides; branch merge-option validation handles
editor options. No extra environment guard or configuration suppression is needed
for these known routes. Both governance validators pass with zero warnings.
Gate follow-up: 51544 exited 101 with tools 944/945, solely sandbox process
observation PermissionDenied in the existing eight-stage Push test. Approved
external preflight session 36855 is live; log
`/tmp/talos-i300-discovery-editor-preflight-unsandboxed.log`. Inspect before
rerunning. CI 39225369123 is terminal failure (Linux workspace/Desktop passed;
Windows failed the old fixture corrected in bc7fd24a). Native validation of the
correction and full local gate remain pending. Governance validators both pass.
Superseding gate result: approved external session 36855 exited zero; log
`/tmp/talos-i300-discovery-editor-preflight-unsandboxed.log` ends
`release preflight: passed`. Pinned locked workspace gate, tools 945/945,
doctests and both external Runtime SDK modes pass for bc7fd24a/6587407c.
Publish the stable stage for fresh native Windows CI. T7 remains incomplete.

2026-10-19 integration continuation: native Git v2.54.0
`builtin/merge.c:add_strategies` confirms direct `merge FETCH_HEAD` selects
`pull.twohead` / `pull.octopus` and may discover custom `git-merge-*` programs.
The existing merge-options module now conservatively validates both strategy lists
(parent reduction can change the selected key), preserving only internal
recursive/ort/subtree strategies and refusing external helper execution without
printing configuration values. Focused locked all-feature tests (2/2), all-target
all-feature Clippy and diff whitespace checks pass. Initial independent review
caught parent-reduction selection and built-in helper execution gaps; both were
corrected and static re-review found no new blocker within this guard. Default
octopus helper effects still need final execution validation; execution remains
disabled. CI 39228852503 at a04fe659 remains
live (Linux Desktop passed; Windows Desktop and Linux preflight running).
Resume actual integration effect validation and supervised execution; do not
mistake this guard for Pull completion or omit the original T1–T8 queue.

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

2026-10-20 T7 checkpoint (`0cecd803`): Windows network Push mutation fixtures
now pass through production internal-Git-program admission. Focused Push tests
pass 15/15; locked standard release preflight passes, including tools 953/953,
workspace checks/Clippy/tests, doctests and both external SDK fixture modes.
Pinned `cargo xwin check --locked` passes for the Windows MSVC test target;
format and whitespace checks pass. This is cross-compilation, not native Windows
execution evidence. PR #735 remains at `3128117a` with CI run 39352700931 still
running on that older head; `0cecd803` is not yet published and needs fresh
exact-head CI. T7 still requires native Windows execution/failure closure,
initialized-child Checkout, authorized recursive Pull/fetch/integration,
independent delivery/security/API review, synchronized docs, and final merge and
completion evidence. Windows production preparation and Pull remain disabled;
T7 and subsequent serial tasks remain incomplete.

Windows CI failure follow-up (2026-10-20): predecessor run 39352700931 for
`3128117a` failed the Windows workspace test step and reached its 20-minute
limit. Results included Agent 471/3, MCP 0/5, Runtime 70/2, and tools 858/32
passed/failed. Cross-layer Agent/MCP/Runtime Git fixtures hit the explicit
Windows process-tree supervision refusal; tools failures cover Windows
path/configuration expectations, Checkout/Push admission fixtures, and Pull
resolution/integration evidence. The Push fixture admission issue is addressed
by `0cecd803`; exact-head CI run 39355259779 for `77a8cf84` is now running.
This predecessor failure is diagnostic only: native Windows execution and the
remaining T7 delivery gates are still open.


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
