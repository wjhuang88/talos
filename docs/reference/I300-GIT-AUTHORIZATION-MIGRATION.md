# I300 Git Authorization Migration Inventory

Current candidate boundary (2026-10-23; unpublished): Unix prepared Add, Commit,
nonrecursive Checkout, limited Push and native Pull have executable paths. Pull
uses approved bounded configuration discovery followed by one frozen native
command authorization, including transitive network, integration, recursion and
configured programs. It does not promise per-path or endpoint confinement.
Prepared-command Auto requires trusted user intent and explicit full-effect
acknowledgement; explicit Deny and human-only policy remain authoritative.
Unknown or failed automatic reviews ask the human. Conflicts and failures can
leave partial effects; there is no automatic retry/reset/repair.

The local candidate has real MCP fetch and Runtime -> Agent -> provider Auto ->
Git integration evidence. Windows production preparation is enabled in the
candidate, and the affected MSVC test targets cross-compile with `cargo xwin`;
native Windows execution and exact-head CI remain unverified. Direct calls without
prepared authorization remain refused. Final security acceptance is open. T7 is
Active, not delivered or complete.
Earlier checkpoints below are historical and do not override this boundary.

## Native Pull acceptance evidence — 2026-10-24

This maps the current candidate, not completed acceptance. Local test results
do not establish native Windows execution or live-model assessment quality.

| Requirement | Implemented evidence | Remaining validation / coverage |
| --- | --- | --- |
| Real Auto allow, explicit Deny, human-only, assessor failure | `crates/talos-runtime/tests/i300_native_pull.rs` runs all four plus an Auto-approved merge conflict and approval-time configuration drift through Runtime, Agent and supervised Git, checking repository effects, retained conflict state and review/approval counts | Native Windows and exact-head CI |
| MCP production admission and effects | `crates/talos-mcp/tests/git_pull_prepared.rs` checks real fetch, Deny without fetch, duplicate request-ID refusal and real merge conflict with retained FETCH_HEAD/MERGE_HEAD, unmerged index and conflict markers | Request-ID refusal is not proof replay; full failure matrix is not established at MCP |
| Missing/stale/incomplete authority and frozen invocation | Tools `prepared_add_tests/native_pull.rs` and `prepared_add_tests/pull_command.rs` reject missing proof, plan/facet mismatch and argv/environment/cwd/program substitution before fetch | These Tools checks do not establish every cross-entry stale/replay case |
| Configuration drift, configured hook, conflict partial effects | Tools `prepared_add_tests/pull_command.rs` checks drift refusal, approved hook execution, cancellation and retained conflict state | Unix hook cancellation does not prove Windows descendant cleanup |
| Initialized/deinitialized recursive Pull | Tools `prepared_add_tests/native_pull_recursion.rs` exercises the command-authority path | Helper runner baselines alone do not establish prepared authorization; cross-entry recursive/helper coverage remains open |
| One-use admission and conservative fallback | Agent prepared-command and permission-session tests cover exact-plan binding, admission consumption and legacy Auto downgrade | Synthetic assessor scenarios do not substitute for actual Git effects |
| Windows production execution | Production gate removed; affected MSVC test targets compile using cargo xwin and CI includes production Add/Pull tests | Native Windows run, including existing Push daemon regression, remains required |

The original same-family Add/Commit/Checkout/Push, two-file index-preservation,
diff fidelity, cancellation, documentation and final review/merge requirements
remain in the I300 owner. This table neither closes them nor waives missing
cross-entry failure coverage.

Checkout new gitlink update (2026-10-15; unpublished): existing ordinary
empty directories, and on Unix bounded nonempty directories without Git metadata
names or protected HFS/NTFS aliases, can become uninitialized gitlinks. A separate
exact Read precedes top-level name enumeration; directory identity and names
are retained and rechecked without reading descendants. Limits are 4,096 entries
and 1 MiB of names per directory per capture. Initialized children and Windows
nonempty directories remain refused. Conservative alias rejection also applies
on Unix (for example `git~1`). Removed-gitlink deletion retains its separate
empty-directory or exact per-entry deletion rules.
Checkout retains its approved repository-wide Write semantics.

Checkout child deletion update (2026-10-14; unpublished): converting a nonempty
gitlink directory to an ordinary target requires fresh exact Read admission for
each discovered child, then exact Write admission for every deletion entry.
Approval explicitly includes untracked content; directory Write is not inferred
to authorize descendants. Missing Write, added children, changed contents or
identities refuse before mutation. The shared limits are 4,096 entries, 16 MiB
of file evidence and the existing eight-stage runner budget. External gitdir
metadata stays outside the deletion tree. Effective configuration/auxiliary
sources, embedded `.git/config`, symlinks/reparse points and devices
remain protected/refused. Unix FIFO and socket entries are supported with exact Read and
Write admission and identity/type revalidation, without opening or reading the
pipe or connecting to the socket. Native checkout failures can leave partial effects;
inspect before retrying. This supersedes blanket nonempty-directory refusal below.

Unix tracked symlink update (2026-10-14; unpublished): nonrecursive checkout can
replace a tracked symlink with an uninitialized gitlink. The current index must
identify that same path as a stage-zero symlink. The external referent is not a
source permission; clean conversion preserves it, dirty or approval-time changed
links refuse, and configuration/auxiliary overlap protection remains in effect.
This does not enable untracked links, Windows reparse points or recursive checkout.

Checkout file-to-gitlink update (2026-10-14; unpublished): nonrecursive checkout
can replace an already tracked regular file with an uninitialized gitlink
directory. Dirty source files retain native rejection with branch, bytes and
index preserved. Effective configuration/auxiliary sources remain protected
against this replacement; a real include.path fixture verifies refusal and
unchanged state. Existing directories and symlinks still require further child
discovery for forward changes. Missing or empty uninitialized gitlinks can restore
ordinary target files or directory trees; retained absence or empty-directory
identity evidence is checked before execution. A child added after preparation
causes refusal with that child and the current branch preserved. Nonempty or
initialized child paths and recursive checkout remain open.
Revalidation checks parent paths and retained identity before directory enumeration.
The replacement-directory regression verifies Unix inode identity; Windows retains
resolved-path identity only and does not establish same-path object replacement
detection. Filesystem TOCTOU residuals remain.

Pull preparation update (2026-10-14; unpublished): sparse tree discovery waits
for complete, exactly admitted objects and refs in both metadata roots, including
symbolic replacement-ref targets. Newly discovered worktree paths require fresh
admission before content capture. Tree-list output is limited to 1 MiB per pipe;
scalar probes retain 16 KiB, and the shared 128-command/64 MiB reservation budget
is unchanged. A real 300-file sparse fixture passes the six-preparation-stage
reservation and index-preservation checks; this is not arbitrary-size support.
Git all-target Clippy passes. Production Pull fetch/integration remains disabled,
and the proposed Pull metadata capability remains unaccepted. These preparation
results do not establish mutation, cross-platform or end-to-end delivery.

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

Status (2026-10-13): local candidate, not published. Add, Commit and Checkout use
independently admitted configuration discovery and fresh one-use mutation plans.
Push uses the same route and executes only direct `git://` transport; local,
SSH and HTTP receivers still refuse pending effect binding. Pull now refuses its
legacy direct execution path and must migrate before I300 delivery. Cross-platform, full
workspace validation, independent security review and publication gates remain
open. No release or complete authorization boundary is claimed.

Current push evidence: bound executable/environment/configuration ordering and
repository refs; resolved destination facets; refusal of recursive child push,
signing, negotiation, promisor, proxy and custom helper effects; admitted hook and
attribute inspection with final revalidation. Real daemon tests prove transfer,
remote ref/payload content and refusal on absent/stale proof or executable hooks.
MCP tests prove network denial causes no connection and rejects request replay;
MCP success/replay now also uses a real daemon: after the first push, the sender
advances HEAD; a repeated request ID is rejected and receiver HEAD stays at the
first commit. A controlled protocol peer also observes `PACK` bytes for an 8 MiB
payload before cancellation; cleanup confirms and the peer sees EOF/reset.
Per-directory attribute evidence currently rejects worktree symlinks and nested
repositories. These compatibility limits are not a completed push contract.

Latest verification (2026-10-13): Git tools 191/191, MCP `git_prepared` 5/5
and Git-enabled all-target Clippy pass with the pinned toolchain and `--locked`.
Real transport coverage also includes receiver-policy refusal, multi-destination
partial success, and cancellation while connected and awaiting advertisement
(confirmed cleanup receipt and closed socket). The later pack-started fixture
extends this evidence to an active pack transfer; remote rollback is not promised.
Independent review found and rechecked the late legacy-remote repair
and trace2 startup output suppression. The maintainer explicitly accepted Trace2
suppression on 2026-10-13; ADR-089 records that contract. Standard workspace
preflight passed (session 84462,
`/private/tmp/i300-preflight-retry-20261013.log`): locked workspace check, Clippy,
tests and external Runtime SDK fixture all passed. Subsequent implementation
changes still require validation before publication.

The dated checkpoints below preserve implementation history. Later activation
and lifecycle checkpoints supersede their temporary disabled/unwired states.

### Remaining transport implementation evidence (2026-10-13)

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

Git 2.54 source inspection establishes the following next implementation work;
none is authorized by a remote Network facet alone:

| Transport | Confirmed local integration | Remaining plan/execution binding |
|---|---|---|
| SSH | `connect.c::get_ssh_command` selects `core.sshCommand` or SSH; variant detection may execute a probe. | Executable identity, SSH configuration/includes, keys/known-host writes, agent socket, ProxyCommand/ProxyJump and destination effects. A probe itself needs admission. |
| HTTP(S) | `transport-helper.c` starts `git remote-<scheme>`; `http.c` consumes TLS file paths and proxy options. | Helper identity, credential/askpass programs, cookie reads/writes, TLS material, proxy environment and configuration, redirects and resulting destinations. |
| Local/file | Receiver is a local Git process operating another repository. | Receiver configuration-chain discovery, repository/object/ref paths, receiver hooks/maintenance, descendant cleanup and receiver metadata revalidation. |

Sources: Git v2.54.0 [connect.c](https://github.com/git/git/blob/v2.54.0/connect.c),
[transport-helper.c](https://github.com/git/git/blob/v2.54.0/transport-helper.c),
[http.c](https://github.com/git/git/blob/v2.54.0/http.c). SSH downstream behavior
must additionally be checked against the selected SSH implementation; Git source
alone does not establish those effects. Current unsupported-transport errors now
identify these missing effects and the separately authorized Git fallback.

Local receiver implementation sequence (independent read-only review,
2026-10-13): sender discovery resolves all ordered destinations; a fresh receiver
Read admission permits bounded layout discovery only. Do not call automatic
repository discovery here: a `.git` file, `commondir` or linked-worktree pointer
can lead outside the admitted directory. Each newly found metadata root requires
another admission before its contents are read. Bare and non-bare repositories
must both be accounted for; bare-only support would remain partial delivery.
Once layout paths are admitted, the fixed configuration-chain command may run.
Its capability does not authorize HEAD/refs/objects/index/shallow, layout pointer
contents, hooks, attributes, skiplist files or child executable reads. Discover
those paths, obtain fresh Read admission, and bind their evidence before building
the final mutation plan. All destinations must finish admission and validation
before the first receiver is mutated; runtime remote failures may still leave
partial updates.

Receiver effect inventory from Git v2.54.0
[builtin/receive-pack.c](https://github.com/git/git/blob/v2.54.0/builtin/receive-pack.c):
`receive.fsck.skipList`, quarantine and temporary shallow files, `unpack-objects`
or `index-pack`, shared/linked worktree HEADs, and automatic maintenance.
Hooks include pre-receive, proc-receive, update, reference-transaction,
push-to-checkout, post-receive and post-update. `receive.denyCurrentBranch`
`updateInstead` can run update-index/diff-files/diff-index/read-tree and mutate
the checked-out receiver worktree, including a sibling linked worktree; bind
its real Read/Write paths and filter/fsmonitor effects or refuse that configured
mode explicitly. Verify receiver child executable identity and inheritance of
foreground maintenance settings. This is reviewed implementation guidance,
not an enabled local transport or a completed security review of its code.

Integration checkpoint (2026-10-13): Agent and MCP now allow eight unique stages,
with cumulative admission/execution deadlines and profile budgets (65,536 facets,
16 MiB resource/description bytes). Git commands reserve 128 direct spawns and
64 MiB stdout/stderr capacity across continuations. Both production runners have
passing first-stage oversize and cumulative profile-exhaustion regression cases,
including execution-count assertions. MCP proves sticky request-ID replay
refusal; Agent proves no new effects on replay, but first-stage oversize is
rejected before its ledger reservation. Admission-before-execution ordering is
confirmed by code review, not by a hook-call counter. The MCP target passes
6/6; the Agent prepared target passes 5/5. These tests supplement core arithmetic
checks; they do not prove all cancellation interleavings. Descendant/helper and
evidence-byte accounting remain open, as does the actual receiver flow through
layout, configuration/auxiliary evidence and fresh final mutation admission.

Commit migration checkpoint (2026-10-12): GitCommit now refuses direct execution
and prepares a separately admitted configuration discovery/continuation using
the same bound carrier as Add. Its concrete mutation remains explicitly disabled
pending effect/lifecycle review; this is an incomplete migration, not support for
approved commits. The closed configuration-chain compiler admits `git_commit`
as well as `git_add`; other tool names remain excluded. Commit captures tracked
source entries, refs/reflogs, merge/sequencer and rerere state. It refuses merge
autostash and configured signing, and enumerates commit/reference/maintenance
hooks for inspection only after continuation admission. `-m` does not consume
`commit.template`, and unrelated tag signing does not block discovery.

The maintainer accepted foreground maintenance scheduling in ADR-089:
`maintenance.autoDetach=false` and `gc.autoDetach=false` are visible in the
execution facet and preserve maintenance work. Subprocess resolution, rerere
effects, gitlinks and cancellation/shutdown still require verification before
the temporary commit execution refusal can be removed.

Follow-up: active rerere records are now explicitly refused when rerere is
enabled (including the default from an existing rr-cache). Inert enabled and
explicitly disabled configurations retain discovery support. Unmerged index
entries refuse before discovery. The maintenance path comes from admitted
`--exec-path`; child program reads/execution are declared on the continuation,
which requires canonical identity with the bound Git binary. Broader helper
resolution and typed gitlinks remain incomplete; commit mutation is still closed.
Gitlink preparation now binds only checkout kind, index OID and local .git marker;
it declares external metadata paths without opening their contents. Child HEAD,
index and effective configuration must be captured after admission before any
commit mutation can be enabled. Marker retargeting requires fresh preparation.
For all=true, fixed configuration inspection now runs separately in present
child checkouts after discovery admission, retaining content privately and adding
child origin Read facets to continuation admission. Child fsmonitor execution is
refused. This does not yet cover attributes, filters or nested status effects;
it does not enable commit mutation or replace child metadata authorization.
Contract: [ADR-089](../decisions/089-structured-git-effect-authorization.md).
Owner: [I300](../iterations/I300-structured-git-repair.md).
The earlier unbound prepared-execute defect is repaired with exact invocation
proof consumption. Independent review's auxiliary-read finding is locally fixed;
this does not substitute for final exact-head security review.

## Current Entry Points

Review follow-up (2026-10-12): parent configuration now binds canonical path and
file/ancestor identity before and after capture and revalidation, matching child
configuration checks. Each child independently discovers its system/global
attribute paths; a parent core.attributesFile override cannot substitute for a
child's default path. Commit and child object inspection refuse promisor sources
even when maintenance.auto=false: remote promisor, partialCloneFilter and
extensions.partialClone can enable lazy fetching outside maintenance. Regression
coverage includes identical-byte replacement, ancestor retargeting, parent/child
attribute divergence and all three promisor declarations. Tools tests pass
136/136; Clippy passes. Commit mutation and lifecycle integration remain open.

Commit activation review: on Git 2.54, ordinary `commit -m` avoids the normal
editor status display, but its no-committable-changes failure path calls status,
which enumerates worktree directories and reads per-directory `.gitignore`.
Current tracked-source evidence intentionally omits untracked descendants, so it
cannot authorize that failure path. Before removing the commit refusal, either
bind the actual post-`--all` candidate index and reject no-op commits natively,
or add reviewed directory-name/ignore evidence and facets for the host status
traversal without reading untracked file bodies. `--all` cannot compare only the
current index to HEAD. Git child status has the same requirement.

The execution path now runs parent configuration, hook and filter inspection for
both commit modes before the temporary refusal, retaining child evidence through
the final shared validation. The parent filter regression verifies no helper
execution and unchanged index in both modes. Tools tests pass 137/137 and Clippy
passes; this remains inspection evidence, not a successful commit acceptance.

### Accepted Cleanup Receipt Amendment — Implementation In Progress

Status evidence review follow-up (2026-10-12): ignore capture now rejects
non-regular entries before reading and never recursively snapshots an ignore
directory. Every traversed directory retains resolution/identity evidence,
checked when queued, before/after reads and at scan completion. Independent
review confirmed both reported defects repaired. Focused tools tests pass
143/143; the prior all-target Git-feature Clippy passes. Unix ignore opens now
use O_NOFOLLOW/O_NONBLOCK and validate the opened regular-file device/inode;
independent review accepted this additional boundary. Ancestor substitution and
concurrent content writes remain TOCTOU residuals; Windows lacks equivalent
leaf flags and Unix device/inode identity. Traversal conservatively
includes ignored/build directories, so budget exhaustion or nested repositories
there may refuse a request that host Git accepts. These residuals and real
commit success/cancellation acceptance remain I300 work; mutation stays closed.

The cancellation regression establishes group cleanup while the runtime stays
alive, but the current per-call private receipt cannot be awaited by production
shutdown. This is an open publication gate, not completed lifecycle integration.

Proposed additive contract: `AgentTool::cleanup_receipt()` defaults to `None`
and optionally returns an `Arc<dyn ToolCleanupReceipt>` defined in `talos-core`.
Its asynchronous `wait(timeout)` returns a content-free cleanup failure or
success. It confers no execution authority and does not cancel commands.
Git tool instances retain a provider-local owned-command executor; their receipt
adapts the existing `SandboxCleanupReceipt`, avoiding a core→sandbox dependency
and avoiding global cross-session state. Runtime registers receipts before tools
can execute; Actor cancellation joins the aborted execution before waiting.
Standalone MCP cancellation drops and joins the owned invocation before waiting;
its connection owner also waits during shutdown after stopping new admissions.
An embedder that drops an execution future must retain and wait its tool receipt
while Tokio remains alive; runtime teardown cannot be reported as successful
cleanup. Existing tool implementations retain default behavior.

Required acceptance: cancellation after leader exit through real Agent/Runtime
and MCP transport, no descendant delayed write, confirmed reap, bounded shutdown,
sticky cleanup failure, unchanged request replay reservation, and downstream
default-implementation compatibility. The maintainer accepted this lifecycle contract in the current continuation
("继续处理吧,接受决策"). Public-interface implementation is now authorized;
production integration and the acceptance above remain incomplete.

Independent proposal review additionally requires separate operation-local
cancellation evidence and the tool owner's aggregate shutdown receipt: cancelling
one session must not await unrelated sessions using a shared tool. Prepared plans
must expose the local receipt before dispatch; registration must precede spawn
and span both stages. Shutdown seals admission, including queued calls and stage
continuations, before waiting; `wait` itself is observation, not a seal. Actor
join must encompass the actual execution task and nested stage futures. Registry
replacement/removal retains the former owner's receipt until cleanup completes.
Sticky failures cannot be reset by timeout, and all receipts share one remaining
shutdown deadline. MCP connection closure/Drop cannot asynchronously wait: an
explicit async connection-owner shutdown interface and its caller integration
are required. These are design acceptance constraints, not implemented guarantees.

Source inspection on the local candidate based on `55fef9f9`:

| Surface | Actual role | Required migration evidence |
|---|---|---|
| `talos-core/src/tool/agent_tool.rs` | Public default methods and legacy execution routes | Additive preparation hook; unchanged existing implementors; Git writes refuse legacy/unbound entrypoints. |
| `talos-agent/src/tool_execution.rs` | Production normalized-input validation, permission hooks, admission, replay ledger and dispatch | Prepare once before authorization; authorize exactly that plan; preserve final hooks and replay reservation; consume one bound capability before mutation. |
| `talos-agent/src/permission_pipeline.rs` and `talos-permission/src/session.rs` | Production policy, approval and session admission | Preserve deny/Ask precedence and state generations; capability covers the entire prepared request, not only returned Path grants. |
| `talos-runtime/src/lib.rs` | Production Runtime constructs Agent with shared permission state and resolver | Exercise public Runtime construction with real tools; do not add a competing approval authority. |
| `talos-mcp/src/server/handler.rs` and `server/permission.rs` | Separate production MCP dispatch and Agent-backed permission gate | Prepare and admit through the existing headless gate; no interactive approval invented; legacy gate calls cannot authorize an unbound Git execution. |
| CLI `PermissionAwareTool`, `TuiPermissionAwareTool`; Runtime `RuntimePermissionAwareTool` | All three are `cfg(test)` wrappers in the inspected revision | Keep compatibility/negative tests, but do not count wrapper-only changes as production delivery. |
| `talos-agent/src/prepared_execution.rs` | Browser original-argument protected route | Remains isolated; no conversion of a Git request to a browser ticket or bypass of the ordinary pipeline. |

## Binding And Compatibility Tests Required

The local candidate adds `AgentTool::prepare_normalized_invocation` with a
default `None`, and `PreparedNormalToolInvocation` with complete permission
facets plus a consuming execute method. Agent and standalone MCP now use the
same prepared object's facets for admission and consume it at dispatch. The
Agent route refuses prepared execution without its permission pipeline, blocks
prepared Bash from bypassing sandbox dispatch, and retains its call replay
ledger. Existing implementors retain the default route. A focused Agent test
proves no-pipeline refusal and once-only execution. This API is not itself a
Git capability: Git operations must still implement immutable plan identity,
pre-mutation revalidation and fail-closed legacy entrypoints.
Before migrating Git, replace the public unbound execute entrypoint. Admission
must attest all Path, Command and Remote facets for the same exact plan, and
execution must consume an unforgeable one-use capability. Agent's replay ledger
is not a substitute for this boundary; MCP also needs equivalent replay
protection. Test a direct Rust call with empty/path-only authorization cannot
mutate, not merely that Agent and MCP normally call permission first.

- Real GitAddTool, production Agent pipeline, literal two-file request: approval
  stages exactly the selected files while preserving unrelated index entries;
  refusal and invalid resources leave the index unchanged.
- Prepared inputs, repository/worktree/index identity and reviewed configuration
  cannot be substituted after permission. A consumed capability cannot replay.
- Existing tools and public callers retain their old method behavior. Git write
  tools fail closed through unmigrated direct methods, with an actionable error.
- Runtime and standalone MCP exercise their real constructors/handlers. CLI/TUI
  tests confirm the shared Runtime/Agent path and existing approval presentation.
- Standalone MCP must bind prepared mutation replay protection to the MCP
  connection and JSON-RPC request ID, plus the exact normalized invocation.
  The local handler reserves `RequestContext::id` before admission and retains
  it after cancellation or indeterminate execution. A repeated in-flight or indeterminate
  write must not execute again automatically. A distinct request ID may be an
  intentional retry and must not be rejected merely because its tool and
  arguments match. Cover duplicate delivery, changed arguments under the same
  ID, cancellation and uncertain host-Git completion in standalone MCP tests.
  Sequential replay and cancellation-after-execution-start are now covered by
  real stdio transport tests; the cancellation fixture observes future drop,
  a live connection, and refusal to reuse the ID. Actual Git mutation, write-policy denial, same-ID replay and new-ID retry
  are now covered by a real standalone transport fixture. Uncertain host-Git
  completion remains uncovered. Transport-level overlapping
  same-ID response behavior is not proven by the passing fixture.
- Browser raw-argument evidence and authorization remain unaffected.
- Inventory public reexports, external SDK fixtures and examples when actual API
  names are selected; this document intentionally does not claim those edits exist.

## Outstanding Boundary

The local prepared route now exposes `PreparedPlanIdentity` and
`AgentTool::prepare_normalized_with_provenance`. Agent supplies registered-tool
provenance; standalone MCP supplies its own remote provenance. Implementations
retain that source and expose their identity through `plan_identity`, then consume
the proof using `AdmittedInvocation::consume_for_prepared`. Legacy prepared
admission APIs reject missing identities; ordinary non-prepared tools keep their
existing dispatch. Identity handles are transient and cannot replace resource
snapshots or execution-time validation. The production roots compile, but Git
plans and the full integration acceptance remain incomplete.

Configured hooks, filters, signing and transports must have truthful execution,
path and network effects. The inventory above does not establish their enforceability.
Do not substitute Write(Path) on source files for Git index or helper authority.
The maintainer accepted pre-mutation refusal for a configuration whose helper
effects cannot be represented and enforced, with an actionable explanation and
separately authorized shell Git alternative. Do not invoke shell automatically
or disable normal Git behavior wholesale. Implement and test the detection;
no persistent blanket grant is added.

Implementation checkpoint: core now separates `ToolCleanupReceipt` (wait only)
from `ToolCleanupOwner` (aggregate admission seal). AgentTool and prepared-normal
invocations expose optional default hooks. Unix sandbox command ownership has
aggregate and invocation-local receipts, with registration fencing and sticky
failure propagation. These primitives are not yet wired into Git/Agent/Runtime/
MCP; no production cancellation or shutdown guarantee is claimed from them.

Lifecycle integration checkpoint (2026-10-12, supersedes the unwired-primitives
checkpoint above): prepared Git commands retain invocation-local ownership;
Agent records per-turn receipts and Runtime seals retained owners before its
cleanup barrier. MCP's default constructor owns only a connection: shutdown
waits registered prepared invocation receipts without sealing shared owners.
`new_with_owner_lifecycle(..., true)` is reserved for an exclusive registry host
such as CLI stdio and additionally seals/waits aggregate owners. Hosts remain
responsible for non-prepared tools exposing only aggregate cleanup. Drop alone
cannot confirm cleanup; explicit shutdown must run while Tokio is alive.

Latest production checkpoint supersedes the historical disabled-commit and
unmigrated-checkout notes above: Add, Commit and Checkout now use admitted discovery
and fresh one-use mutation plans. Commit execution is enabled with foreground
maintenance and real Agent/Runtime/MCP cleanup acceptance. Checkout binds target
tree/ref/reflog and parent evidence, checks current/index/target filters, and rejects
target-induced hooks or rewrites of effective configuration sources. Sources
include declared include destinations and implicit config entrypoints even when
empty or missing. Symlink traversal and possible case/normalization aliases are
treated conservatively. Active `hook.<name>.command` entries are refused during
parent configuration capture because Git can execute them without a hook file.
Parent Commit also refuses `gc.recentObjectsHook` whenever automatic maintenance
is enabled: foreground GC can execute that command through pack-objects. When
`maintenance.auto=false`, the inactive setting is preserved and Commit proceeds.
Branch-conditional includes are supported for explicit
existing local branches, including `-` when native resolution identifies a local
branch, with retained initial configuration and bound target refs/reflogs;
creating/tracking and other target forms still require further discovery and refuse
before mutation. Same-path gitlinks, missing-path
gitlink additions and nonrecursive removals retain dirty initialized child state.
Unpublished initialized-new-child Checkout support additionally discovers the
marker, metadata, raw startup configuration and effective worktree in separate
admitted stages. Raw startup targets follow native no-include layout selection;
ordinary setup waits for exact target Read and retained identity evidence.
Discovery directory and effective status/source roots remain distinct, including
external core.worktree; filter checks explicitly bind gitdir/worktree and root cwd.
These changes have focused native/prepared preservation and filter-refusal
evidence, but linked/ancestor layouts, public transport combinations and final
current-source validation remain open. The eight-stage limit is unchanged and
combined requests that exceed it refuse before mutation.
Push and Pull still require migration; remaining checkout compatibility, final
workspace validation and publication gates remain open.
