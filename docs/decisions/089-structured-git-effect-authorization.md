# ADR-089: Structured Git Effect Authorization

## Status

Accepted by maintainer on 2026-10-11: “接受”. I300 still requires an effective
target-branch claim before implementation. The accompanying all-built-in Auto
request is separately tracked as PERM-007-H, not silently folded into this contract.

## Context And Evidence

The session handoff T7 reports `permission facet 0 has no safe typed resource`
for a two-file `git_add`. Current GitAddTool, GitCommitTool and GitCheckoutTool
inherit an untyped Write profile; grant compilation rejects that profile when
grant scope is required. This is source evidence matching the error, not yet
proof of the originating executable or production admission reproduction.

Git add also interprets caller paths from the discovered repository root and
passes them as Git arguments/pathspecs. A configured subdirectory workspace,
option-looking filename or pathspec can therefore disagree with intended scope.
Adding a workspace-wide Write facet alone would conceal this mismatch.

ADR-010 already requires bounded host bridges and a Rust-native read-only
primary path. ADR-026 already requires all risk facets. This decision defines
their concrete Git effect and compatibility contract, without weakening either.

## Proposed Decision

1. **Literal paths.** `git_add.paths` denotes literal workspace-relative file or
   directory paths, not options or Git pathspec expressions. A directory includes
   its bounded subtree. Option-looking and wildcard-containing real filenames
   remain usable. Reject empty, outside-workspace and escaping ancestor paths
   before mutation. Correct subworkspace resolution without staging sibling paths.
   Workspace-contained absolute paths remain supported after the same checks.
   Subtree bounds describe authorization scope; any resource-limit rejection is
   explicit and whole-request, never silent partial staging.
2. **Actual effects.** Bind invocation approval to canonical repository/worktree
   identity, selected paths and actual index/metadata locations. Source content
   is read; staging writes Git metadata. Preserve index entries outside the
   selected scope. Linked worktrees use their actual index, not an assumed
   `<workspace>/.git/index`. External metadata never becomes an implicit grant.
3. **Existing facets first.** Use existing Path/Command/Remote facets and existing
   explicit external-path authorization where sufficient. Do not add a Git enum,
   persistent grant format or public API merely for convenience. If existing
   carriers cannot faithfully bind these effects, return with a concrete API
   amendment and migration plan before implementing a broader authority surface.
   Existing facets express admission policy; they do not by themselves provide an
   immutable execution plan. Today `permission_profile` is separate from execution
   and `ToolExecutionAuthorization` carries Path authorization, not a complete
   Git command/configuration identity. Prove the binding through the production
   chain before claiming that existing carriers suffice.
4. **Repository-wide operations.** Commit, checkout and pull declare their actual
   repository effects, not a subworkspace-only write. `commit all=true` includes
   tracked worktree staging; ordinary commit consumes the existing index. Push
   and pull retain network facets and safe structured argument boundaries.
   Repair same-family scope defects found by T7 without adding new Git operations.
5. **Configured execution.** Host Git and repository-configured hooks, filters,
   signing and transport helpers require execution authorization; file approval
   alone cannot authorize arbitrary configured programs. Preserve their normal
   semantics by first reusing existing admission, process and authorization
   mechanisms, after verifying they can bind the reviewed execution/configuration
   plan. If they cannot, follow item 3's amendment gate. If a configuration
   cannot be safely represented/enforced, refuse before mutation with a clear
   reason and explicit separately authorized fallback. Never silently disable
   hooks or filters, silently stage unfiltered bytes, or automatically invoke shell.
   Execution admission is not OS containment or a guarantee that a hook has only
   filesystem effects. Review the actual executor capability and any network or
   external-path effects; do not infer confinement from a Command facet.
6. **Environment and lifetime.** Bind Git environment/configuration affecting
   repository/index selection; inherited redirection cannot substitute a different
   target after approval. Host calls have bounded time/output and owned cancellation
   using existing process boundaries. Missing host Git and errors degrade safely.
   Cancellation bounds process lifetime, not a transaction rollback. Report known
   partial effects truthfully and never automatically retry a Git mutation after
   an ambiguous timeout/cancellation.
7. **Filesystem caveats.** Distinguish staging a final symlink entry from following
   an ancestor symlink outside the workspace. Cover deleted and broken-link entries.
   Revalidate identity before mutation and fail closed on relevant changes; do not
   claim that path rechecks alone provide race-free or OS-enforced confinement.
   Ordinary file canonicalization must not turn authorization of a symlink entry
   into authorization of its target. Bind the entry/parent and metadata effects
   consistently; use item 3's amendment gate if the carrier cannot represent this.
8. **Diff fidelity.** Staged means HEAD→index; unstaged means index→worktree.
   Untracked files are excluded from ordinary unstaged diff. Preserve Rust-native
   gix as primary and existing bounded ref-to-ref bridge semantics. Represent new,
   deleted, binary, mode and symlink changes distinctly; errors are not empty diffs.
   Preserve valid ref comparisons while rejecting option/argument injection;
   compatibility does not preserve an unsafe argument boundary.

## Compatibility And Boundaries

Literal paths deliberately remove implicit option/pathspec interpretation from
`git_add`; document migration to explicit paths/directories. More complete facets
may request approval where prior profiles understated effects. No automatic grant,
force push, hook bypass, credential probe, network test or release is authorized.
No new dependencies, native exception or unsafe code is authorized by this proposal,
except the narrowly accepted Windows identity dependency amendment below.

If public Rust APIs must change, record exact inventory and migration before
implementation; source-breaking changes are next pre-1.0 minor only. This contract
does not permit dropping ordinary Git functionality simply to make tests pass.
Unsupported configurations must have an explicit explanation and tracked residual.

## Accepted Windows Path Identity Dependency (2026-10-17)

The maintainer accepted a Windows-only, Git-feature-only direct dependency on
`same-file = 1.0.6`, already present transitively in Cargo.lock. Stable Rust 1.99
does not expose the required Windows file identity through stable metadata APIs.
`snapshot::PathIdentity` retains safe owned handles for the canonical leaf and
ancestors and compares volume/file-index identities while both handles remain
open. Calls into this dependency are panic-guarded; I/O errors fail preparation
or revalidation. No Talos-authored unsafe or new general file-read authority is
introduced. No host utility is used for this identity check.

The dependency uses winapi-util's read-open directory-capable handles, rather
than metadata-only access: restrictive ancestor ACLs, sharing conflicts or
resource exhaustion can conservatively reject the plan. It does not read file
contents or enumerate directories. Handles are released when evidence drops;
there is no separate aggregate plan handle budget yet. This exception is pinned
to the accepted version and does not authorize arbitrary dependency upgrades.

Limitations remain explicit: same-file 1.0.6 compares 64-bit file indices and
does not fully support ReFS 128-bit identities; this is not proof of confinement
or all-filesystem Windows replacement protection. `RegularEvidence`'s separate
Windows leaf identity now also retains a panic-guarded owned handle while
regular-file evidence lives; identical-byte file replacement therefore changes
the compared identity. The existing replacement regression is selected explicitly
in Windows CI. Native Windows runtime validation,
including rename and sharing behavior with retained handles, remains a delivery
gate. Replace this bridge when stable Rust offers an adequate safe identity API,
or when a separately reviewed solution supplies full filesystem identity support.

## Accepted API Amendment (2026-10-11)

The maintainer accepted this additive API contract on 2026-10-11. Independent
source review found that `ToolExecutionAuthorization` only carries exact Path
authorizations. `PermissionEngine::execution_authorizations` skips Command and
Remote facets; `AgentTool::permission_profile` is recomputed separately from
dispatch. Thus a successful permission decision cannot attest the selected Git
operation, worktree, actual index, configured programs or effective environment.
Representing a source file as a Write(Path) would be false: `git add` reads that
file and writes Git metadata. The browser-only `PreparedToolInvocation` uses a
separate resolver and cannot be reused unchanged without bypassing normal policy.

Additive, pre-1.0 API contract:

1. Add an optional normal-tool preparation hook to `AgentTool`, defaulting to the
   existing execution path. A Git tool prepares one immutable, non-serializable
   semantic plan before permission evaluation: exact operation and normalized
   input, selected literal entries/subtrees, canonical worktree/gitdir/index,
   effective Git environment/configuration and programs that may execute. Its
   reviewable facets distinguish source reads, Git metadata writes, command
   execution and any remote/network effects. Unknown effects fail preparation.
2. Pass that same plan's facets and invocation identity through the existing
   `PermissionPipeline` and `PermissionSessionState` decision/admission path.
   Deny, explicit-Ask, configured policy, approval lifetime and Auto rules keep
   their existing precedence. Admission produces a non-clone, one-use capability
   bound to tool, normalized input and plan identity; no persistent grant schema
   or generic workspace authority is added. All facets must be admitted together.
3. Immediately before dispatch, consume the capability and revalidate repository,
   worktree, actual index, effective config/environment and selected path-entry
   identities against the reviewed plan. A mismatch fails before mutation. This
   is an application-level TOCTOU reduction, **not** OS-enforced containment or
   a claim that external Git helpers cannot have additional effects. Unsupported
   configured behavior must fail with an actionable reason rather than silently
   disabling hooks/filters or running them under file-only approval.
4. Keep `AgentTool::execute` and `execute_authorized_with_output` source-compatible
   for existing tools and callers. Git mutation tools require the new one-shot
   route and fail closed through direct/unmigrated wrappers. Migrate Agent,
   Runtime, CLI/TUI and MCP permission-aware composition roots before enabling
   Git writes; browser's raw-argument prepared route remains isolated. Update
   the public API migration reference, tests and downstream examples. A future
   breaking removal of legacy methods requires its own decision and version.

Before implementation, verify the exact list of wrappers and whether Git's
configured helpers can be represented and enforced without a broader process
authority. If not, narrow the supported configuration explicitly and return to
the maintainer; acceptance is not a security waiver or permission to run an
unrepresentable helper.

## Configured-Helper Boundary Accepted (2026-10-11)

The maintainer accepted a bounded fallback for configurations whose hooks,
filters, signing programs or transport helpers cannot be represented and
enforced by the reviewed plan: the built-in Git mutation refuses **before**
mutation and explains the unsupported configured effect. It may point to a
separately authorized shell Git invocation; it must not execute that fallback
itself or treat model approval as authority to bypass the refusal. Ordinary
configurations remain in scope and must not be disabled wholesale. Tests must
cover both the supported path and pre-mutation refusal.

## Separately Admitted Discovery Accepted (2026-10-12)

The maintainer accepted a separately authorized read-only host Git configuration
discovery stage, followed by preparation and normal admission of the mutation.
Native gix omits installation configuration by default; enabling its installation
discovery would execute host Git before admission. An arbitrary installation's
compiled configuration prefix cannot reliably be inferred from the executable.

Discovery must bind the normalized request, provenance, repository, environment
and executable to its own one-use identity and Execute/Read facets. It runs only
bounded read-only inspection commands after normal permission and hook gates.
Its result remains internal and must not expose credential-bearing configuration.
Configuration origins and selected effects discovered there form a new immutable
mutation plan with a fresh identity and complete facets. Every continuation must
pass the normal permission pipeline and final hook again; a discovery proof is
never mutation authority. Denial, cancellation, stale evidence and replay fail
closed. A bounded stage count prevents unbounded approval or execution loops.
Direct and unmigrated wrappers remain denied. Agent, Runtime and standalone MCP
must implement the same sequence before Git writes can be published.

The maintainer additionally accepted the closed configuration-chain discovery
capability on 2026-10-12: it covers the system/global/include files Git normally
parses, including origins not known until inspection finishes. This is represented
as Read(Remote `git:configuration-chain`) together with the bound executable's
Execute facet, never as repository Read or a filesystem-root grant. `Remote` here
is the existing opaque named-resource carrier, not a network destination. Only
the immutable discovery stage may consume that capability for the fixed config
inspection command and fixed `git var GIT_ATTR_SYSTEM` / `GIT_ATTR_GLOBAL`
path-only queries; generic file tools cannot consume it as Path authority.
The latter report installation/global attribute paths without reading their contents.
Checkout discovery also uses fixed `git var GIT_CONFIG_SYSTEM` and
`GIT_CONFIG_GLOBAL` path-only queries under this configuration-chain capability.
They identify configuration entrypoints even when files are empty or absent and
therefore contribute no `--show-origin` values. Their contents still require fresh
continuation Path Read admission. Include declarations likewise contribute their
origin-relative destination paths, including empty/missing destinations. This
prevents checkout from introducing configuration that a later Git child reloads
without that source appearing in the reviewed plan. Branch-conditional includes
permit explicit existing-local-branch switches and `-` resolving to an existing
local branch under retained initial configuration and ref/commit/reflog binding.
Creating/tracking or other target forms still refuse
pending target-chain discovery; no new configuration-read authority is inferred.
Discovered paths become concrete continuation Read facets; discovery does not
authorize attribute-file contents. An installation lacking these queries refuses
with an explicit separately authorized fallback rather than guessing its prefix.
Configuration values and host inspection diagnostics remain internal. The fresh
mutation plan enumerates discovered origin files as individual Read facets.

## Unix Process Ownership Implementation

### Target-branch configuration through ordinary Path admission (2026-10-14)

Creating/tracking branches can clear Git's repository configuration cache before
HEAD switches; Git 2.54 `format_tracking_info` subsequently reloads configuration
through `status.comparebranches`. Source-cached hook selection does not prove
that target-only includes are unread. Native malformed-target fixtures confirm
that `branch.autoSetupMerge=always/inherit` can fail reading the new branch's
include after HEAD has already changed.

Use existing exact Path Read admission for predicted target configuration rather
than expanding `git:configuration-chain`. The initial native stage retains its
accepted current-configuration scope. Target discovery computes a path frontier
without opening, statting or resolving new destinations; each frontier is admitted
through the ordinary pipeline before Rust reads or parses those exact files.
Root files, missing destinations, environment-derived file inputs and hasconfig
lookahead files need the same evidence. Pure in-memory environment entries remain
bound to the frozen invocation environment. Symlink/realpath dependencies require
their own admission; no new filesystem authority is inferred from a path string.

Retain the normalized request, source HEAD, start refs, repository layout,
configuration bytes and identities across all frontiers. Changed evidence
invalidates the plan. Parse in memory, preserving value order, implicit versus
empty values, conditional matching, repeated include occurrences, Git's depth
rules and hasconfig URL lookahead/prohibition. Do not mutate HEAD/config or create
a temporary repository projection to discover the target chain. Do not use gix
automatic include I/O: it has no admission callback or cumulative byte bound and
documents include-order and hasconfig deviations. Reuse only pure parsing where
parity is verified; keep configuration values and parse diagnostics internal.

The completed target chain and actual source/target cache partition must inform
tracking configuration writes and all subsequent read/write/execute effects.
Generate fresh concrete facets, obtain final admission, revalidate, then execute
the original Git arguments without forcing tracking off. The unchanged eight-stage
and existing snapshot budgets still apply; exhaustion refuses before mutation.
This records the reviewed implementation route using existing permission types,
not new authorization or a claim that target-chain execution is delivered.

### Proposed target configuration link metadata capability (2026-10-15)

Status: Proposed; not accepted or enabled. Independent review confirms exact
Path tokens canonicalize aliases and therefore do not independently authorize
reading the alias entry itself. The fixed-command `git:configuration-chain`
capability does not authorize Rust link traversal in the target continuation.

Propose Read(Remote `git:configuration-link-metadata`) for structured Checkout
target configuration discovery only. Its immutable scope is the initial config
entrypoints and include destinations derived from already admitted config bytes,
plus component/link targets discovered from that same chain. It permits only
`symlink_metadata` and `read_link` needed to resolve those paths, not directory
enumeration, file contents, writes or program execution. It is not consumable by
generic file tools. Every referent file and required directory dependency still
requires fresh concrete Path Read admission before content capture.

Retain raw origin spellings (Git resolves relative includes against the spelled
origin), link text and identities, and actual component traversal order including
`..`. Do not reuse `PathIdentity::capture` under this capability: its implicit
`canonicalize/metadata` operations exceed the listed operations. Capture link
and component identity directly from admitted `symlink_metadata` results;
ordinary Path Read evidence remains a separate authorization boundary.
Bound discovery to 64 link hops per complete source resolution, without resetting
the count at component or continuation boundaries. Apply the existing cumulative
4096-path/4-MiB budgets to paths, link text and identity evidence across the whole
discovery; cycles or exhaustion refuse. Revalidate
directories and links before referent bytes, and protect aliases, cancelled
ancestors, link hops and referents from Checkout replacement/deletion. Missing
entries remain bound absence evidence. Identity checks reduce substitution risk
but are not race-free directory-handle confinement. Final mutation needs a fresh
plan and proof; discovery authority cannot authorize writes.

Maintainer acceptance and authorization/denial/drift tests are required before
the compiler or production continuation accepts this capability.

Independent boundary review confirmed the ordinary-directory route (including
`..`, excluding symlink metadata traversal) needs no new named capability:
each file read obtains normal Path Read permission. Required implementation gates
include ordered/parser native parity, nested conditional and hasconfig cases,
denied frontier preserving all repository state, approval-between-stage drift,
budgets, final effects, real prepared Agent/Runtime/MCP execution, and independent
security review. Full support remains outstanding until those gates pass.

### Accepted auxiliary path resolution capability (2026-10-21)

Status: Accepted by the maintainer on 2026-10-21 in the explicit full-scope
confirmation. Implementation and activation gates remain outstanding. This is the
grouped P1 decision in I300, distinct from configuration-chain and Pull resolution.
Independent static review confirms ordinary Path Read first folds `..` lexically
and cannot authorize original alias entries, cancelled ancestors or a different
filesystem-order target. Merely retaining original spellings is not admission.

Accepted Read(Remote `git:auxiliary-path-resolution`) for prepared structured Git
operations only, bound to the current operation, Git executable, repository identity
and frozen auxiliary source set derived from already admitted configuration or the
selected operation. It permits component metadata, link text and metadata-only
identity handles needed to resolve those sources, including external local paths.
It does not permit file contents, directory enumeration, program execution, writes
or network access. UNC/network namespace paths and links to them refuse; ordinary
Path Read is not a substitute network grant. Local paths backed by OS mounts are
not a claim of network isolation; any stronger guarantee requires OS confinement.

Each complete source resolution allows at most 64 link hops; the invocation shares
4096 evidence entries and 4 MiB for path/link text and retained evidence, including
Windows identity handles within the entry limit. Do not reset allowances per source
or continuation. Preserve actual component order, cancelled ancestors, link identity
and text, missing suffix evidence and actual target. Reject unsupported namespace,
reparse type, cycles and exhausted limits. Retained checks reduce substitution risk;
they do not claim race-free filesystem confinement.

After resolution, publish fresh exact Path Read facets for actual targets and required
directory dependencies. Check the complete frontier before capturing any file content;
an old spelling token does not substitute for this new admission. Retain regular-file
identity and bytes through probes. Revalidate the frozen chain before content access
and again after final probes, immediately before mutation. Drift refuses; it never
follows a replacement target under old proof. Discovery cannot grant mutation effects.

Authorization must be checked at the actual resolution call sites, including overlap
inspection, not only by displaying a facet. New auxiliary roots require a new prepared
continuation/proof. Reuse existing resolution/evidence machinery where it meets these
rules, without widening the accepted `git:pull-worktree-resolution` contract.

This proposal does not accept or enable target configuration include-link traversal:
Sources retains its existing exact frontier and link refusal; the separately proposed
configuration-link capability above remains pending. It also does not authorize hooks,
filters or drivers. Their program effects still need separate grants.

Alternative: retain explicit unsupported-path refusal for auxiliary sources requiring
unexpressible traversal. This cannot be counted as the final configured-behavior
acceptance unless the maintainer explicitly amends that scope. No implicit permission
expansion or wholesale configuration suppression is an acceptable implementation.

Activation requires maintainer acceptance, grant compiler scope/denial tests, real
prepared-stage denial before metadata/content, exact resolved-target admission,
replacement and budget tests, native Windows/Unix evidence, and protected review.

### Accepted Pull worktree resolution metadata capability (2026-10-18)

Status: Accepted by the maintainer on 2026-10-18; implementation and activation
gates remain outstanding. Exact Path Read normalizes `..`
lexically before resolving links. Git's relative linked-worktree backlinks
resolve components in filesystem order, so the approved normalized path may
not identify the actual resolution target. A later target grant cannot
retroactively authorize metadata reads performed while discovering it.

Accepted Read(Remote `git:pull-worktree-resolution`) for structured Pull only.
Its immutable inputs are relative `gitdir` backlinks and `commondir` pointers
retained from separately admitted linked-worktree registration files. Inputs include
both absolute and relative commondir values under the existing strict syntax;
relative values use the registration file's parent. Only checkout backlinks
permit a missing suffix. It covers
only component metadata, symbolic-link text, and read-only identity handles
needed to resolve those inputs and retain/revalidate their ancestor identities.
External inspection is restricted to their actual component chains, link
referents, cancelled ancestors and retained identity dependencies; no arbitrary
external path scanning is allowed. Windows handles must use the minimum rights
needed for identity, without reading content; otherwise resolution refuses.
Resolution may reach outside the repository; this must be explicit in the
approval description. Absolute checkout backlinks remain text-only. No file
contents, directory enumeration, process execution, network access or writes
are granted. Generic tools cannot consume this capability.

Implement component-wise resolution with at most 64 link hops per source,
sharing the existing 4096-entry/4-MiB inventory budgets for components, link
text and retained identity evidence; counts persist across continuations.
The 4096 entries are shared across the invocation, with every retained Windows
handle counted separately. Temporary handles are released promptly and count
toward that same peak bound. The byte bound measures user-space serialized
path/link/identity evidence, not kernel memory. Revalidation may revisit retained
records without charging them again, but cannot add records or reset link-hop
counts; new discovery must consume the remaining cumulative budget.
Unbounded `canonicalize` is not an implementation of this budget. Windows
read-only identity handles use the already accepted safe dependency and must
count against the same retained-entry bound. Unsupported link kinds refuse.
Retain raw spellings, traversal order, cancelled ancestors, missing entries and
link evidence. Drift invalidates the plan; it must not silently rebaseline.

After metadata discovery, publish each resolved target as a fresh exact Path
Read facet. Require both the scoped discovery authorization and all target
authorizations before binding or consuming target references. Reference file
contents and descendants retain their existing separate admissions. Discovery
cannot authorize a Git probe or mutation; normal final effect authorization
and the eight-stage limit remain. Identity revalidation is not atomic TOCTOU
confinement. Denial, redirection, budget exhaustion and real permission-pipeline
tests plus independent security review are required before enabling this path.

This is separate from the unaccepted Checkout configuration-link proposal.
Maintainer acceptance authorizes implementation within this scope; it does not
waive the denial, drift, cumulative-budget, target-admission or independent
security-review gates above. Production Pull remains disabled until its full
delivery gates pass.

### Accepted native Pull startup worktree resolution extension (2026-10-22)

Status: Accepted by explicit maintainer reply “接受扩展，保留原生启动行为”.
Implementation and production activation remain subject to the gates below. A local Git 2.54.0
experiment confirms that `git --git-dir <child> config --show-origin --null
--list --includes`, invoked from the child gitdir, attempts to enter the startup
`core.worktree` directory before returning configuration. A missing directory
causes `cannot chdir`, even though the requested command is read-only. Overriding
`--work-tree` avoids this startup behavior and is not native startup evidence.

Extend only `git:pull-worktree-resolution` to a child startup `core.worktree`
declaration frozen from separately admitted fixed config/config.worktree bytes.
Relative values are anchored to that child's gitdir; absolute local values remain
absolute. Resolve in filesystem component order, retain the original spelling and
ancestor/link identities, and require an existing directory for this startup
target. Do not apply the checkout-backlink missing-suffix exception. Preserve the
existing shared invocation 4096-entry/4-MiB and per-source 64-link limits; this
extension grants no additional allowance.

Publish the actual directory as a fresh exact Read frontier before a configuration
probe can enter it. Later worktree effects require their own explicit operation
scope and Write admission. The extension grants no content reads, enumeration,
execution, network access or writes, and does not expand generic Path permissions.
Changed startup bytes or path-chain identities invalidate the plan. Denial,
filesystem-order versus lexical-order targets, target replacement, shared budget
and real prepared-stage evidence are required before production activation, with
independent security review. This acceptance extends the formerly
gitdir/commondir-only capability solely within the scope above.

### Foreground maintenance accepted (2026-10-12)

The maintainer accepted explicit `maintenance.autoDetach=false` and
`gc.autoDetach=false` overrides in structured Git invocations. Maintenance work
remains enabled according to its normal configuration and thresholds, but runs
in the foreground under the invocation's lifetime and deadline. The execution
facet and user-facing documentation must disclose this scheduling difference.
This does not authorize disabling hooks, signing, filters or maintenance work.
The admitted discovery stage also uses the fixed path-only `git --exec-path`
query for commit maintenance. It does not read child executable contents under
configuration-chain authority: the reported child path becomes concrete Read
and Execute facets for fresh continuation admission. The current candidate
requires that path to resolve to the already-bound Git binary; other layouts
need separately reviewed evidence rather than a guessed PATH fallback.

Independent review of Git 2.54.0 found that ordinary commit invokes automatic
maintenance by default, and its detached path can escape the owned process
group. The overrides prevent that specific detach path; they do not by themselves
prove ownership of arbitrary configured helpers. Helper effects, executable
resolution, cancellation and shutdown still need their required verification.
Source evidence: `run-command.c:prepare_auto_maintenance` and
`builtin/gc.c:maintenance_run_tasks` at Git tag v2.54.0.

Reuse ADR-082's existing sandbox supervisor for Git's admitted host execution.
Its standard-library Child remains unreaped until the last group signal, avoiding
stale PGID reuse. Add a bounded-output entrypoint rather than introducing another
supervisor or dependency. The helper executes only an already-admitted command;
it supplies lifecycle ownership, not permission admission or OS containment.
Cleanup failure must remain an error, including during cancellation or runtime teardown.

## Verification

Production Agent admission/Runtime composition must reproduce the reported failure
on an identified build and then demonstrate the exact two-file request succeeds
after approval, preserves unrelated staged entries, and cannot mutate on denial.
Cover invalid resources, literal special filenames, directories, deletions,
subworkspaces, linked worktrees, symlinks, environment redirection, configured
programs, timeout/cancellation and same-family permission profiles.

Diff tests independently cover staged-only, unstaged-only, three versions of one
file, additions/deletions, unborn HEAD, binary/mode/symlink changes, literal path
component boundaries, truncation and untracked exclusion. Existing tests that add
an unstaged edit to make a staged file visible do not prove staged-only behavior.
Independent permission/security/API review, standard preflight and exact-head CI
remain required. Revert the bounded repair if approval/effect binding regresses.

## Related

- [I300](../iterations/I300-structured-git-repair.md)
- [ADR-010](010-git-search-tool-dependency-boundary.md)
- [ADR-026](026-multi-resource-tool-permissions.md)
- [ADR-047](047-external-path-tool-authorization.md)
- [Handoff T7](../tasks/2026-10-10-session-todo-handoff.md)

## Accepted Cleanup Lifecycle Amendment (2026-10-12 continuation)

The maintainer accepted the reviewed cleanup receipt contract after the detailed
decision explanation. This authorizes additive core interfaces and integration
through Git, Agent, Runtime and MCP; it grants no execution/file/network authority.
Owner admission sealing is separate from invocation-local waiting. Shutdown must
retain replaced owners, stop new admissions and continuations, cancel/join actual
execution, and await cleanup within one shared deadline. Single-call cancellation
must not wait for unrelated calls. Failure remains sticky; timeout/cancellation
never implies transaction rollback or automatic mutation retry.

Migration defaults preserve existing tool implementors. Embedders using managed
Git commands must retain receipts and keep the async runtime alive through bounded
shutdown. Non-Unix parity is unproven and must not be inferred. Full acceptance and
remaining implementation are owned by the
[migration inventory](../reference/I300-GIT-AUTHORIZATION-MIGRATION.md).

## Early Trace Output Contract (2026-10-13)

Independent review reproduced system/global `trace2.*Target` creating output
before `git config` discovery returns. The local candidate sets `GIT_TRACE2=0`,
`GIT_TRACE2_EVENT=0` and `GIT_TRACE2_PERF=0` in structured child environments,
inherited by descendants. Parent environment and configuration files are unchanged;
configuration values remain discoverable internally. Caller-provided GIT_*
overrides continue to refuse before this injection. Trace-required execution
needs a separately authorized invocation with its output effects represented.

The maintainer explicitly accepted this diagnostic-output exception on
2026-10-13. Publication still requires fresh candidate validation and independent
review. A production-runner fixture exercises isolated global config
with all three file targets and a Unix stream socket target; all remain untouched.
The command facet discloses suppression. 189 tools tests and Git-enabled Clippy
passed; independent read-only review accepted both this code boundary and the
legacy-remote revalidation correction. Acceptance does not establish I300 completion.

## Receiver Reference Read Scope (accepted contract, 2026-10-13)

Receiver configuration can depend on the current branch, so retaining only the
HEAD file is insufficient when HEAD follows a symbolic reference. Before the
receiver common stage reads reference content, its new permission profile names
the shared and per-worktree `refs` directories as distinct Path Read facets and
names `packed-refs` as an exact file Read facet. Independent review found that
those exact directory authorizations alone cannot authorize recursive reads.
The revised candidate therefore adds the closed Read(Remote
`git:receiver-reference-tree`) facet, compilable only for a `git_push` request
with an absolute `refs` Path Read facet. The compiler does not itself prove that
the request is prepared or that the named root belongs to a receiver; the
one-use prepared proof and each stage's exact receiver-root checks do that. Its
one-use invocation proof, together with the separately checked exact roots,
covers bounded descendant inspection (65,536 entries and 16 MiB of reference
file content combined across receiver and sibling roots per evidence pass).
This is not a total process-memory or cumulative revalidation-I/O limit;
path names and tree bookkeeping are outside the file-content byte count.
It is not a generic subpath capability:
`ToolExecutionAuthorization::authorizes_path` still matches only exact paths;
other tool names cannot compile this closed facet. The maintainer explicitly
accepted this bounded, dedicated scope on 2026-10-13. Independent implementation
review and complete receiver effect binding remain required before mutation is enabled.
Stable directory symlink redirection and nonregular entries refuse; path checks are not a race-free
filesystem handle. The initial candidate refused final receiver mutation;
the execution checkpoint below records subsequent local implementation progress.

## Pull Repository Metadata Effects (accepted contract, 2026-10-13)

Status: accepted by the maintainer in the handoff continuation; authorized as a
dedicated contract, but not enabled until the implementation, review and real
Agent/MCP evidence below are complete. Owner: I300 Pull continuation.
The accepted receiver capabilities below remain git_push-only and cannot be
reused by git_pull. Exact Path permissions cannot represent object names,
temporary files and maintenance outputs that fetch creates dynamically.

The proposed closed Remote resource `git:pull-repository-metadata` supports
Read and Write only for prepared git_pull stages. Compilation requires exact
absolute Path roots of matching nature and the absolute bound Git Command.
The executor additionally verifies that these are this invocation's retained,
identity-bound gitdir/common roots, checks each root's exact authorization and
consumes the immutable stage proof before using the closed capability. No
generic authorizes_path behavior or other tool's grants change.

Read covers the Git metadata used by this Pull's graph/ref/reflog inspection and
fetch/integration, including objects, refs, packed-refs, HEAD, FETCH_HEAD,
shallow, grafts and commit-graph metadata under the individually approved roots.
Write covers this Pull's objects, refs, reflogs, FETCH_HEAD, shallow metadata,
locks, temporary/quarantine files and accepted foreground maintenance outputs
under those roots. A separately admitted integration Write stage also covers
index/index.lock, ORIG_HEAD, MERGE_HEAD/MERGE_MSG/MERGE_MODE/AUTO_MERGE,
REBASE_HEAD, rebase-merge/rebase-apply state, autostash objects and refs/stash
with their logs, and repository-local branch-tracking configuration updates
when required by the actual operation. External configuration files remain
excluded. Fetch admission cannot authorize this later integration stage: its
new immutable plan binds the actual merge/rebase/autostash effects and targets.
Read and Write are separately admitted; discovery Read never
implies fetch Write. This is directory-scoped metadata authority, not an exact
file list or a total I/O limit. Existing invocation deadlines, process/output
budgets and bounded explicit evidence remain applicable.

The capability excludes worktree files, external alternate object stores,
redirected metadata, additional sibling worktree roots, hooks, filters, remote
helpers, credentials and arbitrary programs. Such paths/executables/effects
require separate fresh facets and admission. Configuration-chain discovery
remains separately authorized; this capability cannot grant general system or
global configuration reads. Stable symlink and nonregular metadata redirection
must refuse; path identity/evidence checks reduce TOCTOU and are not confinement.

The proposed lifecycle is configuration discovery, newly admitted metadata
graph Read, prefetch evidence/transport-effect discovery, separately admitted
fetch Write, then a fresh integration plan binding post-fetch FETCH_HEAD and
target worktree effects. Any further discovered scope requires ordinary fresh
admission within the unchanged core stage budget. No mutation precedes its
effect plan; partial fetch/integration failure is terminal and reports possible
effects, never automatic retry or rollback. Cancellation retains the same owned
execution/cleanup lifecycle across every continuation.

Before activation: maintainer acceptance, independent security/API review,
compiler rejection tests for wrong tools/missing roots/missing command/nature
mismatch, real Agent/MCP approved and denied metadata stages, stale-root and
escape tests, native Pull parity, failure/cancellation checks and all I300 gates.
Acceptance alone does not enable fetch or claim Pull delivery. The implementation
must replace the layer-by-layer production inventory with admitted bounded
evidence under this dedicated scope rather than retain two contradictory modes.

### Native whole-operation Pull amendment (accepted 2026-10-22)

Maintainer clarification (2026-10-23): proceed with the simplified flow of
bounded discovery followed by one complete native Pull execution approval.
Discovery steps inside an admitted scope are internal work, not additional
approval stages for each path or directory. Batch known discovery resources;
only resources outside the admitted scope require additional authorization.
The ordinary-case implementation target is one discovery plan and one execution
plan. This accepts the control-flow direction, not an unrestricted filesystem
read grant: existing dedicated capabilities retain their documented boundaries,
and any expanded discovery capability must state its exact scope and limits.
Preserve explicit Deny, one-use proof binding, identity checks, process ownership
and truthful partial-failure reporting. Reducing eleven stages to eight alone is
not the acceptance target for this simplification.

Concrete discovery scope (accepted by the maintainer on 2026-10-23): a closed
git_pull-only Read capability binds individually displayed absolute worktree,
gitdir and common-dir roots and the fixed Git inspection commands. Within those
roots it may enumerate names and read regular Git configuration, layout pointers,
refs, index/tree evidence, .gitmodules and attribute/hook evidence needed solely
to build the operation plan, including stored child repositories under modules.
It never executes hooks/helpers, writes, fetches or connects to a remote.
Reuse the invocation-wide 4096-entry/4-MiB evidence allowance and existing process
deadlines/output limits; refuse exhaustion. Do not follow a link out of scope:
existing dedicated path-resolution grants may resolve it, but its actual target
needs fresh admission before content reads. Normal system/global/include parsing
uses the separately accepted configuration-chain capability; this proposal does
not turn that capability into general external file access. Configuration values
remain internal. Final execution still requires a new immutable complete-effects
approval, including each child mutation root and destination. Generic Path Read
semantics and other tools remain unchanged. This replaces the narrower pending
identity-only module-inventory proposal. The maintainer's explicit acceptance
authorizes implementation of `git:pull-bounded-discovery`, not production
activation or execution. Permission compilation binds Read, git_pull, an absolute
Command and individually displayed absolute Read roots; the prepared consumer
must additionally bind root identities, fixed inspection argv and the shared
budget before using this capability. Independent security review and the
existing production validation gates remain required.

The maintainer subsequently required the experience of invoking Git in Bash:
review the concrete native Pull as a whole, allow the model to make an actual
invocation-local approval under ADR-064, and let Git perform its own fetch,
integration and configured recursion. This amendment takes precedence over the
separate-fetch/integration sequencing and recursive-child scheduler below.
Those sections remain historical evidence of the earlier accepted design;
they are not additional prerequisites to implement that scheduler.

The prepared request must bind the native executable, exact arguments, working
directory, retained environment/configuration evidence and repository identity
to the consumed approval proof. Display the operation's network and read/write/
execute effects and the complete authorized worktree/metadata scopes. Review
and authorization happen before the native mutation, rather than approving
fetch and predicting every subsequent conflict, rename or replay result.
Post-fetch commit identities are not available before native Pull starts: do
not invent them or require a second integration plan inside the same command.
Retain the pre-execution identity evidence and report the actual result.

Native recursion does not implicitly grant access to arbitrary child roots or
remotes. The operation-level request must represent its authorized recursive
scope; effects outside that scope retain independent authorization or cause an
explicit refusal. Do not recreate Git's recursive scheduler to claim native
parity. Hooks, filters, helpers and repository-external effects retain their
existing authorization boundaries. Model approval cannot override explicit
Deny or human-only policy, and uncertainty or assessor failure falls back to
the human approval path. No persistent grant results from model AllowOnce.

Reuse the immutable prepared proof and existing supervised process executor;
retain foreground maintenance, Trace2 suppression, output/deadline bounds,
cancellation cleanup and truthful partial-failure reporting. Do not retry,
reset, abort or resolve conflicts automatically. Remove obsolete Pull-only
staging/prediction code after its replacement is verified; preserve helpers
still required by other Git operations. T7 owns native execution and identity;
PERM-007-H owns the shared effectful Auto assessor, without a Git-only reviewer.

Production activation still requires approval-to-execution and denial tests,
native parity including configured recursion and conflicts, identity mutation,
partial failure and cancellation coverage, independent security review, and
the applicable Agent/MCP and native Windows delivery gates. This amendment
accepts the implementation direction; it does not assert those gates pass or
enable the currently disabled production Pull path.

### Pull integration worktree scope (accepted 2026-10-19; sequencing superseded)

The maintainer accepted a dedicated one-Pull-integration worktree scope instead
of requiring an exhaustive prediction of conflict files, renames and interactive
replay outputs before execution. This is a wider single-stage authorization,
not permission to execute integration after fetch admission alone.

The closed capability `git:pull-integration-worktree` is reserved for a fresh
prepared `git_pull` integration stage. Its plan must display the concrete
integration operation, retained FETCH_HEAD evidence and selected commit OIDs,
and the full absolute, identity-bound worktree root. Read and Write scopes must
be explicit; Git metadata retains its separately approved roots and capability.
Compilation must require the matching absolute Path facet and bound Git Command;
the executor must verify the retained worktree identity and immutable operation
proof before consuming the capability. Generic recursive Path authorization is
unchanged. The scope expires with this integration stage and cannot authorize a
later Pull, sibling worktree or recursive child integration.

The admitted worktree scope covers dynamic files produced by this operation,
including conflicts, renames and replay results. Repository-external paths,
redirected metadata, hooks, filters, merge drivers, editors and other programs
still require separate effect discovery and authorization. The scope does not
permit following a worktree symlink to write outside the approved root.
Identity revalidation reduces stale-path risk; it is not OS-level confinement.
Existing process supervision, invocation deadlines, stage limits, cancellation
cleanup and terminal partial-failure reporting remain mandatory. No automatic
rollback or retry is introduced.

Before enabling production integration, implement and independently review the
compiler and runtime consumers, reject wrong-tool/missing-root/missing-command
and nature-mismatch grants, and verify approval/denial, stale identity, external
path effects, conflicts, failure and cancellation through the applicable I300
runtime gates. Acceptance records the design decision only. Remove prediction
logic made obsolete by this scope while retaining analysis needed to discover
external effects; do not maintain competing authorization modes.

The integration request is one operation-level decision point. Auto may assess
and approve that exact invocation once under ADR-064's 2026-10-22 amendment;
otherwise the existing human approval path decides it. Present the frozen
operation and fetched identities before execution; do not require advance
enumeration of conflict paths, renames or replay results. If Git returns
conflicts or partial effects, report them and leave resolution to the user.
Explicitly denied or separately constrained effects remain denied or require
their own approval; model approval cannot widen the displayed invocation scope.

### Recursive child fetch extension (accepted 2026-10-16; superseded)

Native Git 2.54 source inspection and an isolated fixture confirmed that the
default `on-demand` parent fetch starts child-repository Git processes and may
import child objects before the parent fetch command returns. A parent remote
approval and parent metadata roots do not authorize those effects. The
maintainer accepted staged equivalent execution: every structured parent fetch
must pass `--no-recurse-submodules`; after it completes, a fresh parent metadata
Read stage discovers fetched ref/tree changes and `.gitmodules`, then each
eligible initialized child repository receives its own identity-bound metadata
Read/Write and remote Network plan before its child fetch. Child fetches also
disable Git-internal recursion; newly found descendants repeat this cycle.

This preserves recursive-fetch intent and separately authorizes each child,
but changes native process ordering, parallelism and failure output. Uninitialized
submodules are not initialized by this feature. Child config/include reads,
helpers, credentials, hooks, external object stores and arbitrary child paths
remain separately authorized or unsupported. The existing 8-stage lifecycle
and 128-command/64-MiB cumulative output bounds remain fixed; if the discovered
tree cannot be fully planned within them, the operation stops with any already
completed parent/child fetch effects reported and never retries automatically.

Acceptance authorizes implementation, not execution. Child discovery, exact
identity binding, per-child approval, nested-fetch iteration, partial-effect
reporting, budget exhaustion, cancellation, Agent/MCP evidence and independent
review remain prerequisites before enabling Pull fetch.

## Receiver Metadata Effects (accepted contract, 2026-10-13)

The maintainer accepted a dedicated git_push-only receiver metadata Read/Write
capability. The closed Remote resource `git:receiver-metadata` accompanies
separately admitted, identity-bound receiver gitdir/common Path roots and the
fixed executable plan. It covers objects, refs, reflogs, locks, quarantine and
maintenance metadata under those roots for this invocation and accepted
foreground maintenance. It does not authorize worktree mutation, external
alternate stores, hooks, arbitrary programs or effects outside admitted roots.
External reads/writes and program execution remain independently admitted.

The compiler requires git_push, a bound absolute Command facet and an absolute
Path facet of the matching Read/Write nature. This syntactic eligibility is not
proof of receiver ownership: the one-use prepared identity and runtime receiver
root/evidence checks must establish that. Exact Path authorization remains exact
for other consumers. The prior reference-tree discovery budget remains in force;
this capability does not retroactively widen discovery or its limits. No new
public resource enum or generic recursive file permission is introduced.

Implementation and review must bind executable/environment/maintenance effects,
refuse unrepresented configurations before mutation, and test denial, replay,
layout changes, actual receiver updates and cancellation before enabling writes.
Contract acceptance alone does not complete I300 or authorize an incomplete
receiver execution plan.

Local implementation checkpoint (2026-10-13): eight admitted stages now reach
actual local/file receiver mutation in the candidate. Retained receiver storage
and configuration evidence is checked again immediately before spawn. Supported
shell spelling is `/bin/sh`, with macOS `/bin/sh` or Linux dash/bash canonical
paths; retained executable evidence and system installation trust underpin this
baseline, not a cryptographic assertion of shell implementation. Other shells
and startup overrides refuse pending their own represented effects. Storage
inspection includes rr-cache, MERGE_RR and its lock for foreground rerere cleanup.
Real prepared tests cover four receiver layouts; MCP covers success, denial and
replay. Independent namespace/shell review found no new definite omission, but
cancellation, partial effects and complete security/publication gates remain
open. No cross-platform or complete I300 delivery claim follows from these tests.

Windows receiver preparation checkpoint (2026-10-20): inherited `MSYS`,
`CYGWIN`, `CYGWIN_DEBUG`, `MSYS2_ARG_CONV_EXCL`, and `MSYS2_ENV_CONV_EXCL`
now require separately authorized invocation rather than silent removal. The
MSYS runtime's option table includes `error_start`, which configures external
error handling; argument/environment conversion overrides also undermine the
fixed receiver command's representation. Source basis:
[MSYS2 runtime environ.cc, msys2-3.6.5](https://github.com/msys2/msys2-runtime/blob/msys2-3.6.5/winsup/cygwin/environ.cc).
This is a conservative preparation guard, not evidence that the runner uses
this runtime revision or that every MSYS effect has been inventoried. Independent
static review found no blocker. Windows-only tests cover case-insensitive
refusal and unchanged environment snapshots; locked MSVC xwin checking passes,
but native execution and the complete shell implementation remain outstanding.
The production Windows structured-Git gate remains closed.


Windows shell baseline candidate (2026-10-20): under the unchanged production
Windows gate, prepared local Push accepts only the observed ucrt64 installation
layout. The retained canonical Git target must end in
ucrt64/libexec/git-core/git.exe on a local drive; the separately admitted
canonical shell must equal that installation root's usr/bin/sh.exe. Comparison
performs no new filesystem access and refuses unknown layouts and escaped
canonical shell targets. Invocation spelling is never substituted for retained
canonical evidence. Existing Read/Execute admission, identity retention and
final shell rediscovery remain mandatory. Independent static review found no
remaining blocker after correcting the rebound invocation/canonical distinction.
MSVC xwin checking and the host Unix shell regression pass. This candidate does
not prove DLL loading, MSYS startup effects or real Windows prepared Push;
these runtime/trust limitations remain activation prerequisites.
