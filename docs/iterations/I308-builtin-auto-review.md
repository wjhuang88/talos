# Iteration I308: Shared Auto Review For All Built-In Tools

## Local MCP injected command Auto stage — 2026-10-28

The MCP gate now accepts an explicitly injected permission pipeline and current-turn
embedder intent; the default constructor remains Headless without a resolver. Registry
factory identity determines native prepared provenance consistently at preparation and
admission. Matching names and external replacement instances cannot carry command Auto
evidence. A new real rmcp client/server test runs the production Auto resolver and Git Add,
asserts the staged blob, and denies final commands through human fallback so success cannot
masquerade as model approval. Configuration discovery is separately host-approved. Negative
cases cover same-name replacement, absent intent, Headless context, explicit Ask and Deny;
all refuse model assessment and leave the index absent. A separate injected ordinary approval
test proves actual file write/denial, not file Auto.

Locked `git_prepared` (48/48) and `server_handshake` (3/3) pass with low-debug,
nonincremental settings (`/private/tmp/i308-mcp-hosted-native-tests.log`). The first test
attempts exposed missing interactive fixture context, separately approved discovery and
inconsistent prepared provenance; all were corrected without weakening the Auto resolver.
The earlier host preflight (session 80805) exited 0, including external Runtime SDK fixtures,
but compiled MCP before these edits, so it does not certify this candidate. Fresh Clippy,
independent permission/security/API review, final coherent preflight, exact-head CI (including
Windows), merge and completion evidence remain required. MCP file/network/ordinary invocation
Auto routing and the remaining I308 acceptance are still unfinished; this is not I308 closure.

## Sender-binding rebuild verification — 2026-10-28

Implementation correction is local commit
`6696d880c9cbb36d7d3d300d46d9410244a56045`. After the maintainer explicitly
authorized cleaning this active worktree's regenerable target, `cargo clean
--locked` removed 54,761 files (27.1 GiB). Both worktrees and their sources were
preserved; the main worktree has no target directory. Free space is now about
24 GiB after rebuilding. Earlier disk-exhaustion checkpoints below are historical.

With `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0`,
locked all-feature sender-binding tests pass 3/3
(`/private/tmp/i308-sender-binding-rebuild.log`). Full all-feature Tools library
tests first reported 1009 passed / 1 failed under sandbox process-observation
EPERM (`/private/tmp/i308-sender-binding-tools-full.log`). The unchanged authorized
host retry exits 0 with 1010/1010 passing
(`/private/tmp/i308-sender-binding-tools-host.log`). Warnings-denied all-feature
Tools lib/tests Clippy passed before cleanup
(`/private/tmp/i308-sender-binding-clippy.log`). Format, whitespace and both
governance validators pass. Standard host `release_preflight.sh` is running with
the same low-debug/nonincremental settings plus `RUST_TEST_THREADS=1`; inspect
`/private/tmp/i308-sender-binding-preflight-host.log` for its terminal outcome.
No complete preflight result is claimed yet.

Fresh fetch leaves main at `4bf3488b2f4344b054754851b588686126c436f4`.
#797 remains open at old head `1993432dccd3571abac7849a32c5b4640007f622`.
Its CI `40659215369` passed offline dependency parity; Linux Desktop validation
was still running, and Windows workspace remains failed. New-source native
Windows execution, exact-head independent review/CI and merge CAS remain required.
MCP default disposition is not model Auto reachability; all remaining I308
acceptance and MODEL-014 → T4 → T5 → T1 → T6 remain unfinished.

## Local CLI decision visibility follow-up — 2026-10-28

Git stage #797 is published at
`1993432dccd3571abac7849a32c5b4640007f622`, based on
`4bf3488b2f4344b054754851b588686126c436f4`. Independent final-head static
permission/security/API review approved with shared-identity limits; the receipt
is posted on the PR. Exact-head CI `40659215369` is running, not passed or merged.
The preceding local-only checkpoint below describes its pre-publication state.

The next local branch makes CLI/TUI effectful AllowOnce notices explicitly say
that the model allowed the request once, with a safe Git/file/network/invocation
category rather than generic completion. Regression covers those categories and
unknown future reasons without exposing provider identity or request digest.
Locked focused CLI tests pass 8/8
(`/private/tmp/i308-visible-auto-cli-tests.log`); warnings-denied CLI/test Clippy
passed (`/private/tmp/i308-visible-auto-cli-clippy.log`).
This follow-up has not been published. Full stage checks, review and publication remain
pending. MCP hosted-native model reachability, complete profile/recovery/platform
coverage and all later handoff tasks remain unfinished; I308 stays Active / Partial.

### Standalone MCP fail-closed disposition regression — 2026-10-28

Added a real MCP client/server integration test using the production file-tool
contribution factory and permission gate. With the default standalone headless
composition, native `read` returns the existing file contents, while `write` to a
new path is rejected, creates no file and leaves the original unchanged. Locked
handshake integration suite passed 2/2
(`cargo test --locked -p talos-mcp --test server_handshake`,
`/private/tmp/i308-mcp-native-default.log`). This proves the default profile's
read-Allow/write-deny disposition; it does not establish model Auto reachability.
ADR-064 requires an embedder to inject a resolver and effective policy, and CLI
configuration is not implicitly loaded by MCP. MCP hosted-native Auto remains open.
The focused compile consumed disk; available space was about 671 MiB afterward, so
the full preflight was not run. This local test has not received final review or CI.

### Git-stage Windows CI finding — 2026-10-28

Exact-head run `40659215369` at `1993432dccd3571abac7849a32c5b4640007f622`
has a failed Windows workspace job `122717780697`; the main workspace job was
still running at inspection. Windows format/check/Clippy passed, but structured
Git guard tests and workspace tests failed. Tools reports 923 passed / 40 failed;
MCP Checkout and Runtime Checkout also fail. Multiple failures report retained
Add/Commit/Checkout command drift after preparation. Job log:
`/private/tmp/i308-git-windows-job.log`. Sender-program discovery/rebinding after
initial command capture is the current investigation lead, not a proven complete
root cause. Do not merge this candidate or claim native Windows coverage. No live
workflow was restarted. The local MCP disposition test above does not fix this.

Inspection confirmed that `FrozenCommand` captured the entry Git executable before
the separately admitted sender-program frontier, while Commit and Checkout can
rebind to Git for Windows' internal `git.exe`; Add did not take that path. The
local correction validates the old command before binding, then changes only its
retained executable path at that exact admission boundary. Arguments, workdir and
environment remain frozen and revalidation still rejects changes. A regression
exercises Add/Commit/Checkout frozen-field checks after binding. This is a focused
source-level hypothesis and regression, not confirmed Windows runtime evidence.

Low-debug locked focused validation was attempted twice. The first invocation
omitted the `git` feature and therefore selected zero sender tests; it compiled
but is not test evidence. The correct `--all-features --lib sender_program_binding`
invocation failed while linking `talos-tools` with `ld: write() failed, errno=28`
at about 215 MiB free. The test did not execute. No target cleanup was performed;
another active worktree's target remains untouched. Re-run focused tests and native
Windows CI when disk capacity permits, then obtain fresh exact-head review and CI.

Subsequent low-debug `cargo check --locked -p talos-tools --all-features --tests`
passed (`/private/tmp/i308-sender-binding-check.log`), including the new regression's
compilation. This does not execute tests. Independent local Agent design review
found no blocking permission/security finding: old invocation validation precedes
exact admitted executable capture, only program changes, and subsequent identity
checks remain. The reviewer ran no builds/tests and explicitly did not approve a
committed final head. Both governance validators passed with zero warnings after
the owner/handoff checkpoint. Native Windows and repair runtime validation remain
required. Latest fetched main remains `4bf3488b2f4344b054754851b588686126c436f4`.

## Local operation-specific Git Auto checkpoint — 2026-10-28

Git stage delivery state: Review (local candidate; remote gates pending).
Iteration delivery remains Active / Partial; this stage does not close I308.

Implementation is committed locally as
`a1490da3479f6da752c98d2ccca8cefeed4b17b5`; it is not pushed or merged.
Independent static permission/security/API review approved that exact implementation
against `4bf3488b2f4344b054754851b588686126c436f4`, without running tests.
Approved-host Windows Tools all-feature cross-check passed
(`/private/tmp/i308-git-command-xwin-host.log`); this is compilation evidence,
not native Windows execution. The unchanged focused Push eight-stage test also
passed in a subsequent sandbox run, without establishing why earlier workspace
runs received EPERM from process observation. Final approved-host standard
preflight with default debug/incremental settings failed at workspace test linking
with ENOSPC (exit 101; `/private/tmp/i308-git-command-final-preflight-host.log`).
After explicitly authorized deletion of only this worktree's regenerable
`target/debug/incremental` cache, the standard script passed completely (exit 0) with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0
RUST_TEST_THREADS=1` in approved host execution
(`/private/tmp/i308-git-command-final-preflight-host-lowdebug.log`), including
workspace tests and both external Runtime SDK consumer modes. This certifies the
unchanged executable implementation; the owner evidence update requires its own
governance checks. No remote gate or native Windows execution is claimed.

The four-operation Agent matrix now also covers same-name external registration
and replacement of a trusted builtin instance: both invoke zero assessors and
produce no Git mutation (`/private/tmp/i308-git-instance-trust.log`). Its denied
command fallback checks structured capability scope rather than relying only on
explanation text. The shared five-operation resolver matrix also rejects assessor
timeout and oversized output (`/private/tmp/i308-git-resolver-bounds.log`).
Standard preflight completed successfully, including both external SDK modes
(`/private/tmp/i308-git-command-preflight.log`), but that run overlapped subsequent
fixture changes; final frozen-tree validation is still required.

Runtime shared-composition integration now exercises Add, Commit, Checkout and
Push through RuntimeBuilder, provider Auto review, admission and real local Git.
Each operation checks AllowOnce against human-required with command fallback
denied, including actual index, commit, branch and receiver-ref effects. The
complete integration file passes 4/4, preserving existing Pull policy, recursion
and cancellation coverage (`/private/tmp/i308-git-runtime-integration.log`).
Runtime integration warnings-denied Clippy passes after replacing two fixture
`unwrap()` calls with contextual `expect()` calls
(`/private/tmp/i308-git-runtime-clippy.log`). Both governance validators pass
(`/private/tmp/i308-git-auto-governance.log`,
`/private/tmp/i308-git-auto-claims.log`). Fresh fetch and GitHub main-ref query
both confirm `4bf3488b2f4344b054754851b588686126c436f4`.
This is local host evidence, not Windows runtime or merged delivery. The earlier
overlapping preflight is superseded by the final low-debug run recorded above.

The current locally committed continuation produces separate versioned Add, Commit,
Checkout and Push review evidence only at their final retained-command frontier.
The closed capability compiler checks each tool/capability pair and downgrades
approval to Once. Unknown transitive effects remain explicitly disclosed;
active Deny and incompatible scoped human checkpoints are checked before Allow.
This does not extend Pull authorization merely by relabeling its proof.

Real registered Agent dispatch passes four operations across nine scenarios
(Allow, human-required, unsupported model `deny` output, assessor error, session expiry, policy Deny,
scoped Ask, external registration, trusted-instance replacement), with actual index/commit/branch/receiver-ref assertions and zero
persistent grants. Policy restrictions prevent model assessment. The fixture
denies command-review fallback so it cannot mask failed Auto approval. Existing
Pull's ten-scenario matrix also passes. Log:
`/private/tmp/i308-git-real-auto-policy-matrix.log`.

Agent library tests passed 513/513 before the final added negative scenarios;
the expanded resolver operation-mismatch matrix subsequently passed. Permission
library tests passed 167/167. Git-feature Tools passed 341/341 in approved host
execution (`/private/tmp/i308-git-command-tools-full-host.log`); sandbox execution
passed 340 and failed the unchanged process-observation assertion with EPERM.
Agent/Permission library/test warnings-denied Clippy passed before the final
policy fixture expansion (`/private/tmp/i308-git-agent-permission-clippy.log`).

Initial actual Agent testing found a request-fingerprint mismatch from a redundant
transitive-effects marker. The marker was removed: operation capability guards
already enforce the restriction, including manually registered Git instances.
Full Agent rerun then passed. No new permission-pipeline marker remains.
Fetched main remains `4bf3488b2f4344b054754851b588686126c436f4`.
Remaining: final owner-evidence checks, production/profile reconciliation and native
Windows evidence, final exact-head permission/security/API review,
exact-head CI/CAS/merge and remaining original
I308 acceptance. This is local implementation evidence, not merged delivery.
I308/T8 and the full handoff remain Active / Partial; MODEL-014 → T4 → T5 → T1 →
T6 retain their complete scope.

## Local retained Push checkpoint — 2026-10-28

Implementation `dc78bab16` retains Push's executable, cwd, argv and environment
at the first final mutation frontier, after receiver discovery and Windows
internal-program rebinding, before that frontier receives admission. Subsequent
checks reject a changed command rather than replacing it. The final dispatcher
requires and executes the retained command. A real Git/local-listener regression
rejects remote, branch, force, environment, cwd, program and missing-command
changes without contacting the receiver or changing HEAD/worktree status.

Full locked Git-feature Tools library tests passed 340/340 in approved host
execution (`/private/tmp/i308-retained-push-full-host.log`). The initial focused
run exposed fixture entrypoints bypassing the production retention seam; those
now reuse the same retention check. The corrected sandbox focused run passed
16/17, with only the existing process-observation EPERM failure. The host full
run preserves that assertion and passes. Warnings-denied Git-feature library/test
Clippy passed (`/private/tmp/i308-retained-push-clippy.log`); formatting and diff
checks passed. Clippy's collapsible conditional was corrected after the full
suite; the change is syntactic and Clippy compiled both library and tests.

Fetched `origin/main` remains `4bf3488b2f4344b054754851b588686126c436f4`.
Current rebased retention implementation mapping: Add `b6764dd48`, Commit
`0cb83e527`, Checkout `48515fc40`, Push `dc78bab16`. Earlier SHA mappings below
are historical. These local commits have not been pushed or merged. They do not
implement operation-specific Add/Commit/Push/Checkout Auto assessment/admission.
That implementation, higher-risk recovery/profile/platform coverage, full locked
preflight, independent permission/security/API review, exact-head CI and merge
remain required. I308/T8 and the full handoff goal remain Active / Partial;
T2/T3 → T4 → T5 → T1 → T6 remain queued with unchanged acceptance.

## Local retained Checkout checkpoint — 2026-10-28

Checkout now retains its exact executable, cwd, argv and environment, rejects
changed branch/create/environment/cwd/program or missing retained command before
mutation, and executes the retained invocation after equality checks. Real Git
regression verifies unchanged HEAD and clean status in all six refusal cases.
Focused regression passed; full Git-feature Tools tests passed 339/339 in
approved host execution (`/private/tmp/i308-retained-checkout-full-host.log`),
and warnings-denied library/test Clippy passed
(`/private/tmp/i308-retained-checkout-clippy.log`). The sandbox full run passed
338/339 and failed the existing Push process-observation fixture with EPERM;
the host rerun preserves that assertion and passes. No Auto admission, full
workspace/platform validation, final security/API review or merged delivery is
claimed. Push command retention and operation-specific Add/Commit/Push/Checkout
assessment/admission remain outstanding, together with original full acceptance.

## Local retained Add checkpoint — 2026-10-28

Rebased onto baseline closure merge `958a391316c82181853d1df176687361b14ff82b`:
Add implementation is now `e6237f668`; retained Commit implementation is
`686a57641`. Commit retains the exact message/mode, executable, cwd and
environment, rejects changed message/`--all` or missing retained command before
mutation, and executes the retained invocation. Current Git-feature Tools tests
passed 338/338 in approved host execution
(`/private/tmp/i308-retained-add-git-full-host-tests.log`), and Git-feature
library/tests warnings-denied Clippy passed
(`/private/tmp/i308-retained-commit-clippy.log`). The new fixture initially assumed
an existing HEAD in an unborn repository; it now checks actual unchanged status.
These remain local retention changes, not Add/Commit Auto admission or merged
delivery. Push/Checkout retention, operation-specific assessment/admission and
original stage gates remain outstanding. Earlier Add-only SHA/results below are
historical pre-rebase evidence.

Implementation `cda61015c` retains Add's exact executable, cwd, literal argv and
environment before assessment and executes that retained command after matching
revalidation. A real Git fixture refuses path, environment, cwd, executable or
missing-command substitutions before staging; existing selective-staging behavior
is preserved. This is command-retention groundwork, not Add model Auto admission.
Commit/Push/Checkout Auto and the full original H/G acceptance remain unfinished.

Git-feature Tools library tests passed 337/337 in approved host execution:
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test --locked -p talos-tools --no-default-features --features git --lib`
(`/private/tmp/i308-retained-add-git-full-host-tests.log`). The sandbox run passed
336/337, failing only the existing Push process-observation assertion with EPERM;
no assertion was skipped or weakened. Git-feature library/tests Clippy with
`-D warnings` passed (`/private/tmp/i308-retained-add-git-clippy.log`), and
format/diff checks passed. Full preflight, independent security/API review,
Windows evidence, remote CI and merge are outstanding for this local stage.
T8 remains Active / Partial; T2/T3 → T4 → T5 → T1 → T6 remain unchanged.

## Current delivery checkpoint — 2026-10-28

Pull #786 merged as `652749f42b7dac0d1ab9a665a4c54e6e48576312`; all eight
exact-head CI checks passed in run 40610739691. Legacy search disposition #789
merged as `e03a2984267eacbd4f65d0085be1d028a8c420ed`; all eight exact-head
checks passed in run 40611467832. Both stages passed full local locked preflight.
The search cases prove refusal of unavailable legacy search, not working search
delivery. I293 retains search implementation ownership.

T8 remains Active / Partial. Add/Commit/Push/Checkout model Auto evidence and
remaining higher-risk recovery/profile/host acceptance are unfinished. The
T2/T3 → T4 → T5 → T1 → T6 queue is unchanged. Earlier checkpoints are historical.

## Current delivery checkpoint — 2026-10-27

I308/T8 remain Active / Partial. Ordinary Shell/Exec/HTTP #783 merged as
`da280c8a77b25798e59df37f3bdc2bf2fafce1c9`. Fetch #784 merged as
`d6f1e6a56c3d337168b587feb92add36223007c4` after all eight jobs passed in
exact-head CI 40504783551, independent static permission/security/API approval,
both governance validators and merge-time CAS. The review receipt is
https://github.com/wjhuang88/talos/pull/784#issuecomment-6443350514.

Recoverable Save Auto #785 merged as
`f0d484b255bf766eafb1573bd0870ef38ae8de77` after exact-head CI
40525328345 passed all eight checks, full locked preflight/SDK, Windows xwin,
independent static permission/security/API review and merge CAS. This closes
only the Save stage; the original complete acceptance below is unchanged.

Pull ownership PR #786 now targets main at candidate
`cee25226e3ab24564d8076c22f3ffdf1de5fd3e0`, based on
`9d38a7365c494ade99a484bdd158476f850f4428`. The rebased five patches
preserve builtin instance ownership and production shared Runtime composition.
Windows CI 40530302518 failed a strict content assertion because host Git
converted LF to CRLF. The candidate explicitly disables autocrlf in source and
clone fixtures; production semantics and strict effect assertions are unchanged.
Fresh full locked standard preflight passed, including workspace tests and
external Runtime SDK execution (log:
`/private/tmp/i308-pull-rebased-preflight.log`). Independent static security/API
review approved this exact head/base without independently running tests.
Native exact-head CI and merge CAS remain required; no Pull merge is claimed.

Concrete legacy WebSearch disposition is now checked locally in six production
Agent registration scenarios (Auto, disabled Auto, Deny, mandatory-human,
policy Allow and headless). All refuse before execution: its current legacy
profile has no safe typed network resource. Neither model assessment nor human
resolution runs, including under explicit Allow. This is an unavailable search
path, not successful manual/search backend coverage. I293 owns the protected
search implementation; I308 must not route it through ordinary HTTP Auto.
Focused locked test passed (1 test / 6 scenarios), log
`/private/tmp/i308-profile-rebased-search.log` after replay onto the current main-based Pull candidate; full stage validation is pending.

### Earlier merged-stage checkpoint — 2026-10-26

Foundation #769 merged as `d45d08f0a82706c2abb26fcb32899f8b24fa64b1`.
Exact head `ffa35973022c18f752cacad1922325e15c648379` passed all six jobs in
CI 40274739232, including native Windows. Independent permission/security/API
review and merge-time CAS are recorded in
[#769 review receipt](https://github.com/wjhuang88/talos/pull/769#issuecomment-6409815251).
The review was by a separate read-only Agent sharing the session/GitHub identity,
not a separate natural person. It did not independently run tests.

I308/H/G/T8 remain Active and incomplete. Mutation #770 merged as
`687b84a93b4a9f40ede1eb600151c0a22a8f2d59`; policy/read coverage #771 merged as
`b21b6146c21894dc42e8fc5bf1d815af71fbc7c8` after all-six-success exact-head
CI 40280848025, independent final-target review and merge CAS. Scheduler/read/
Todo/recovery stage #772 merged as `881a27b890e5310055380e6e3f5b0da78f9fdd3c`
after all-six-success exact-head CI 40296390604, independent review and merge CAS.
Ordinary effectful Execute/HTTP/WebSearch Auto, remaining registered
operation/profile dispositions and final delivery gates remain outstanding.
Earlier checkpoints below preserve historical states and do not override this one.

> Document status: Active (#765 merged; effective at `8f0ce2340d7728ac5b3ee8eba814050a2360e194`)
> Published plan date: 2026-10-26
> Planned objective: complete handoff T8 with model-assisted, invocation-local approval across registered native tools, including suitable Write, Network and Execute operations and higher-risk file changes.
> Baseline rule: preserve this published target; implementation stages do not reduce acceptance.
> MVP deliverable: a production native tool request needing approval can receive one model AllowOnce and actually execute through normal admission; every registered native operation has a tested disposition.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | I308 / PERM-007-H and PERM-007-G: native-tool Auto evidence, shared assessment and invocation-local admission, higher-risk file recovery, production registration coverage, tests and documentation; excludes search transport and provider protocol implementation. |
| Claimed At | 2026-10-26 |
| Source Issue | #563; handoff T8 |
| Governance Claim PR | #765 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized continued full handoff delivery; #765 requires independent permission/security/API review, exact-head CI, both validators and final merge-time CAS before merge. |
| Implementation PR | #769 (foundation, merged); #770 (file mutation, merged); #771 (policy/read tests, merged); #772 (scheduler and native operation evidence, merged); #773 (parent Runtime registration, merged); #779 (Desktop Todo registration, merged); #783 (ordinary Shell/Exec/HTTP, merged); #784 (Fetch URL, merged); #785 (recoverable Save URL, merged); #786 (Pull Auto instance ownership, merged); #789 (legacy search refusal evidence, merged) |
| Last Updated | 2026-10-28 |
| Handoff / Release Condition | Satisfy full H/G acceptance with existing implementation commits, independent permission/security/API review and platform evidence; no release authorized. |

This governance candidate proposes claim and activation atomically in #765. Both are ineffective
until the finalized exact-head record merges to main; implementation starts from that merge or later.
No production implementation belongs in this governance candidate.

## Published Baseline

### Coexistence Decision Checkpoint — 2026-10-27

The maintainer confirmed the I293/I308 boundary as the responsible actor of both owners:
ordinary HTTP may receive whole-invocation review; WebSearch, search compatibility/default
transport and required prepared-network tools retain ADR-090 authority and cannot downgrade.
The proposal and ADR-090 record that decision. This documentation stage preserves the published
baseline and runtime acceptance. Required evidence: independent exact-head security/API review,
both governance validators, exact-head CI and target-branch merge. Remaining implementation and
platform/dispatch evidence stay owned here; no ordinary Auto activation or completion is claimed.

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-H | PERM-007 | Refinement / Unclaimed; acceptance explicit and ready for selection | I300 completed at `ee84889fd`; existing shared evaluator and permission pipeline | All registered native operations have tested, effect-aware dispositions and suitable requests can actually execute after model AllowOnce. |
| PERM-007-G | PERM-007 / #563 | Refinement / Unclaimed; selected together with H | Shared typed evidence and atomic admission | Existing-file edits and destructive/high-impact native file operations have bounded diff/impact evidence, real backup/recovery and audit behavior before Auto allowance. |

### Scope And Policy Reconciliation

- Inventory actual production assembly in CLI interactive/TUI/MCP, Runtime and Desktop, including feature-gated registrations, Todo, scheduler, status, process and conditional skill tools. Distinguish dormant constructors and externally supplied overrides.
- Reuse one shared model assessment path with implementation-owned typed, bounded evidence bound to exact normalized execution arguments, current user intent, resources and freshness. Presentation/redacted arguments are not authority.
- Implement actual one-shot approval for suitable native Write, Network and Execute requests. Category-wide refusal, tool-name allowlists, advisory-only responses and unconditional human fallback do not satisfy H.
- Preserve deterministic policy-allowed read-only fast paths with no additional model call. Multi-facet approval is atomic; partial approval cannot execute.
- Preserve explicit Deny, explicit mandatory-human user policy, execution-unknown replay protection, admission identity checks, cancellation, evaluator isolation, bounded output and secret redaction.
- H and G share this one iteration/claim. G's historical W2/W3 human gate remains until the relevant implementation is reviewed and validated; ADR-064's accepted 2026-10-22 amendment permits suitable effectful model approval. Do not silently remove G's diff, impact, backup/recovery or audit requirements.
- Cover higher-risk operations exposed by existing native schemas: existing-source edits, delete and any existing rename/batch behavior, dependency/config/release/security files. Record inventory evidence where a proposed operation has no native tool; do not invent new tools to satisfy an inventory label.
- Implement bounded backup/recovery for native file mutations, with failure/cancel and stale-target behavior. Describe actual recovery limits; never promise transactional rollback for arbitrary shell, Git, network or scheduled agent work. If a concrete effect cannot be safely assessed/enforced, expose its precise human decision and retain incomplete coverage honestly.
- Unknown/unbound effects, insufficient intent/evidence, uncertainty, assessor failure and unsupported enforcement give actionable human/headless fallback. Model denial cannot execute; cancellation cannot leak an AllowOnce.
- Trusted implementation ownership must survive production composition and wrappers without name spoofing: custom RuntimeBuilder tools and plugins do not become trusted native implementations through matching names, default provenance or diagnostic contribution source.
- Retain I300's reviewed whole-command Pull semantics, one-shot credentials and process supervision; integrate through shared assessment without reopening per-path effect prediction.
- Public API adaptations must preserve embedder compatibility or obtain the required ADR and migration decision before implementation; no silent exhaustive-enum or struct-literal break.

### Non-Goals

- Automatic authorization of external MCP/plugin/embedder tools merely because they resemble native tools.
- New native tool families, search backend activation/transport, provider protocol work, universal rollback or permanent grants.
- Dependency upgrades, new unsafe code, release/tag/publication or waiving any remaining handoff task.

### Acceptance

1. Given each production profile and applicable feature set, registration coverage identifies every native operation and a tested policy/model/human disposition; an undeclared new built-in fails the invariant. External same-name replacements cannot inherit native approval.
2. Given a suitable native request needing permission, the shared assessor receives bound intent/effect evidence and model AllowOnce leads to actual production execution; representative non-shell Write, Network and Execute operations prove this end to end.
3. Given multi-facet effects, all are admitted together or none execute. Existing deterministic read-only allows incur zero model calls.
4. Given explicit Deny/human policy, uncertainty, error, cancellation, changed input/resource/evidence or replay, no unauthorized execution occurs and the user receives a truthful reason/fallback. Verify allow, deny, uncertain, error, cancel and stale outcomes through the production pipeline.
5. Given higher-risk native file changes, bounded diff/impact evidence and actual backup/recovery behavior are tested before Auto execution, including special/config/security paths, destructive operations, partial failure and stale identity. No prose-only rollback claim satisfies G.
6. Given CLI/Runtime/Desktop/MCP-hosted native composition and applicable hosts, intended behavior is reachable with visible model decision/rationale and safe secret handling; missing platform evidence stays outstanding.
7. Auto user docs, API/migration notes and coverage inventory match implemented behavior. Independent permission/security/API review, full locked preflight, exact-head CI and merge CAS pass before completion.

### Execution Stages

1. Inventory and contract: complete operation/profile matrix, trusted evidence ownership and compatible API design; registry invariant and focused tests.
2. Shared admission: unify assessment and atomic single-use admission; implement representative non-shell production allow/deny/error/cancel/stale paths.
3. Full native coverage: finish all registered families and G diff/impact/backup/recovery behavior; preserve existing policy fast paths.
4. Integration and closure: real host/feature coverage, user-visible diagnostics, documentation, independent review and final exact-head gates.

Stages are logical commits/reviewable candidates, not substitutes for the complete acceptance above.
Converge each stage locally before pushing; record remaining acceptance explicitly after any stage merge.
Consult relevant existing implementations and upstream best practices when a design stalls; record the
source and applicable limits instead of extending speculative local machinery.

### Planned Validation

- Repository-pinned Rust 1.99.0; all workspace checks/tests/builds use `--locked`.
- Focused production registration/permission/assessment/execution tests, including external overrides and all decision outcomes.
- Profile/feature coverage checks for native assemblies, deterministic read-only no-model tests and multi-facet denial tests.
- Higher-risk write diff/impact/backup/recovery tests with failure, cancellation and stale targets.
- `./scripts/release_preflight.sh` (full workspace checks, Clippy, tests and SDK consumers).
- Windows compile through `cargo xwin` with locked dependencies; native Windows CI execution is a separate mandatory evidence row, not inferred from cross-compilation. Native Unix tests cover applicable behavior.
- `scripts/validate_project_governance.sh .` and `bash scripts/validate_collaboration_claims.sh .`.
- Independent exact-head permission/security/API review, CI and final merge-time CAS.

### Documentation To Update

- `README.md` Auto behavior/configuration sections and SDK examples affected by the final API.
- Tool behavior/API migration notes, ADR-064/070/089 where affected, and a complete production operation coverage matrix in this owner or a linked reference.
- H/G owners, #563, handoff ledger, backlog, iteration index and Board; completion requires already-existing implementation SHA evidence.

### Risks And Rollback

- Hard: write-capable effects remain permission-gated; explicit Deny/human policy and immutable safety constraints cannot be overridden.
- Soft: accepted operation-level model review replaces historical category-level eligibility limits only after concrete implementation and independent review.
- Assumption to validate: a compatible trusted evidence interface can serve all actual native assemblies without accepting embedder-provided claims as native authority.
- Risks: stale evidence, native-name spoofing, partial multi-facet admission, sensitive diff leakage, unsafe recovery and implicit scheduled work. Each needs focused adversarial tests and independent review.
- Rollback: revert a reviewed implementation stage to prior runnable behavior; do not auto-reset user data or imply external effects can be undone. Runtime backup recovery must be implemented and separately tested, not inferred from Git revert.

## Selection Inventory And Disposition — 2026-10-26

| Owner | Current state / disposition |
|---|---|
| I293 SEARCH-001-C | Active; retain its transport/endpoint-authority ownership. I308 assesses registered native requests without changing search routing/transport contracts. |
| I294 SEARCH-001-D | Active; retain provider research/evidence, no overlapping implementation selected. |
| I306 PROVIDER-008-A | Active, open implementation #749; retain provider protocol ownership, no protocol changes in I308. |
| I302 TEST-003 / #717 | Review; retain unexplained Windows resource residual and #705 diagnostics; not absorbed or waived. |
| I277 Desktop validation | Review, human/device rows deferred and nonblocking; retain original acceptance. |
| I249 dependency pilot | Planned; explicitly defer unchanged behind the selected handoff queue. |
| I164 | Paused; preserve supersession/layout disposition, do not reactivate. |
| I146/I147/I150/I162/I262 | Current owner headers Complete; historical Review lines/stale index do not reopen them. |
| I300 / I307 | Owners are Complete; T7 and its sandbox prerequisite have merged evidence, not activation blockers. |

Existing parallel owners remain separately scoped; the maintainer's continued handoff execution
selects I308 as this session's only new implementation stream. Open PR inspection found #749 and
#705, neither claims H/G. Recheck target owner/PR overlap and dependencies immediately before merge.
Handoff ordering after T8 remains T2/T3 → T4 → T5 → T1 → T6, all incomplete.

## Actual Activation And Execution

- 2026-10-26: governance draft only; source Issue #563 and its comments reviewed. No effective claim, implementation or activation yet.

## Verification Evidence

- Draft baseline: both local governance validators passed with zero warnings. Finalized candidate rerun, independent review, exact-head CI and merge CAS remain required.

## Completion Evidence

No implementation completion evidence exists. Do not mark H, G or T8 Complete from this plan.

## Variance And Residuals

- Full production operation matrix and compatible API design are stage 1 implementation deliverables, not claimed complete by discovery notes.
- G remains human-gated until implemented/validated; no higher-risk policy implementation is claimed here.
- Remaining handoff tasks and gix capability/advisory tracking retain separate owners.

- 2026-10-26: #765 number backfilled; proposed Claimed/Active record and H/G owner synchronization are ineffective until main merge.

### Local execution checkpoint — 2026-10-26

- Activation #765 is merged; implementation branch `feat/i308-builtin-auto-review`
  starts from `8f0ce2340d7728ac5b3ee8eba814050a2360e194`.
- Added a compatible default `ApprovalResolver::resolve_network_invocation` entry point.
  The pipeline now passes the retained transport manifest separately from exact execution
  input and human presentation. Existing adapters delegate to their previous exact-input
  resolution method. The manifest does not establish native implementation ownership.
- Focused validation: `cargo test --locked -p talos-agent --lib
  permission_pipeline::execution_input_tests` passed all four tests, including network
  evidence delivery, denial without grants and legacy adapter compatibility.
- This is unmerged admission plumbing, not model-based network allowance. Shared typed
  assessment, complete native ownership coverage, actual Auto execution and G
  recovery remain unimplemented; full preflight and independent review remain outstanding.

- Ownership foundation: added explicit built-in instance registration, with same-name
  ordinary replacement revoking the marker and duplicate batches remaining atomic.
  Product-owned CLI groups and Runtime coding-preset/skill contributions now use it;
  caller overrides and plugin registration retain ordinary APIs. Direct scheduler,
  process and other registrations still need coverage; no complete inventory claim.
- Core ownership tests passed 3/3. Runtime `shared-composition` locked check passed
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`. An earlier default-debug
  check exhausted disk space and is not a code-validation success. Previously authorized
  `target/debug/deps` and incremental cleanup recovered space; no source/Git data removed.
- CLI production registry ownership assertions are in a separate test module.
  Ownership is not yet consumed for model approval; implementing
  the common assessment/admission route remains the next step.

- CLI production registry ownership test passed (1/1) across print/TUI/MCP assemblies.
  Formatting and both governance validators passed. Ownership foundation is recorded
  in local implementation commit `d2d31b9b6`; it has not been pushed or independently
  reviewed. No stage merge, full preflight, native Windows run or T8 completion claimed.

- Network review implementation checkpoint: dispatch now conveys registered built-in
  ownership with retained network evidence to a dedicated model assessment route.
  The response must match the fresh invocation digest and closed schema; only
  high-confidence AllowOnce can admit the request. Explicit human/Deny policy,
  stale state, replay, provider failure and external instances remain covered by
  focused tests. Serialization stops at 16 KiB, with an input check before cloning.
  The initial human-policy test incorrectly appended its rule after the default
  Ask; it now constructs an explicit-rule engine so that the intended rule matches.
- Validation: locked `talos-agent --lib auto_resolver` tests passed 71/71;
  log `/private/tmp/i308-auto-resolver-tests.log`. This includes encoded-size
  boundary coverage and 15 network-admission scenarios. These are pipeline-level
  results, not production tool execution evidence. The network changes remain
  uncommitted; complete native registration coverage, production end-to-end
  execution, G recovery, full preflight, Windows and independent review remain open.
- Agent-loop dispatch test passed for five scenarios: model-approved built-in
  executes with live network authority, external same-name registration and ordinary
  replacement do not inherit trust, explicit Deny blocks execution, and deterministic
  Allow executes without model assessment. This uses an offline prepared transport
  fixture, not the WebSearch HTTP implementation. Actual product transport evidence
  remains outstanding. Log: `/private/tmp/i308-auto-network-dispatch.log`.
- Independent working-tree review found JSON credential fields could bypass the
  shell-oriented secret heuristic. Added bounded structural key/value inspection
  and consistent text scanning for intent and serialized assessment fields.
  Six credential scenarios (keys, nested objects, arrays, prefixes, intent and
  endpoint paths) assert zero assessor calls. Auto resolver tests remain 71/71.
- Standard preflight initially failed in two Git process integration tests under
  sandboxing (process observation EPERM; configured hook start timeout). Both
  actual target tests passed outside the sandbox with `--features git` (1/1 each).
  An earlier rerun without that feature selected zero tests and is not validation.
  Full standard preflight is rerunning outside the sandbox; current process 64384,
  log `/private/tmp/i308-network-stage-preflight-unsandboxed.log`. No full pass
  or stage readiness is claimed until its terminal result is inspected.
- Terminal validation: the sandbox-external full standard preflight exited 0,
  including workspace check, Clippy, tests, governance and the independent Runtime
  SDK consumer (`release preflight: passed` in the log above). Independent working-tree
  security/API follow-up found the credential fixes resolved its blockers. This
  is local stage evidence, not final exact-head CI/Windows evidence or T8 completion.
  Fetch confirmed origin/main remains `8f0ce2340`; no rebase was needed.
- Production ownership follow-up: interactive mode now explicitly registers its locally
  constructed scheduler instances as built-ins, and MCP registers its concrete `status`
  instance as a built-in. Generic registry builders still treat caller-supplied scheduler
  instances as external; a regression test verifies they cannot inherit built-in ownership.
  Locked CLI `builtin_registration_tests` passed 2/2. The earlier filter
  `registry_builtin_tests` selected zero tests and is not evidence. These changes are
  local and unreviewed; they do not enable default HTTP/WebSearch Auto execution or
  implement G recovery. Complete production inventory, common admission, final platform
  validation and independent review remain open.
- Ordinary invocation evidence plumbing: dispatch now conveys exact arguments, complete
  evaluated facets, implementation description and optional implementation-owned working
  directory for explicitly registered ordinary built-ins. Prepared network, prepared Git
  and shell routes retain precedence. The additive resolver default delegates to the
  existing execution-context/exact-input methods; external adapters cannot construct the
  evidence, and no ordinary-tool model AllowOnce implementation is enabled by this change.
- Locked `talos-agent --lib permission_pipeline::execution_input_tests` passed 4/4,
  including registered evidence delivery and legacy-adapter delegation. The Auto resolver
  suite passed 71/71 before the private context-enum cleanup; focused input tests passed
  again after cleanup. Independent working-tree review found no blocker in ownership,
  routing or API compatibility; this is not final exact-head delivery approval.
- Registration follow-up is committed locally as `06aa93f3e`; no push/merge claimed.
  Both governance validators passed with zero warnings before the ordinary evidence
  extension. Default HTTP/WebSearch model execution and G backup/recovery remain
  unimplemented, so H/G/T8 remain Active and incomplete.

- Latest local checks: locked agent Clippy passed with warnings denied after the context-enum cleanup; registration/input tests and independent incremental review are stage evidence only. Ordinary evidence plumbing is committed as `a44cba863`. Fetch reconfirmed unchanged main `8f0ce2340`; no rebase required.

- HTTP configuration stability prerequisite: per-request timeout now uses the retained
  client's request builder instead of constructing a client that resets redirect policy
  and rereads proxy environment. Separate regression tests verify timeout clamping and
  rejection of a redirect under a zero redirect budget. Locked `talos-tools --features
  network --lib http_request::` passed 26/26. The initial no-feature invocation selected
  zero tests and is not validation evidence; an initial regression assertion incorrectly
  expected a 302 rather than reqwest's zero-budget redirect error and was corrected.
  This does not implement HTTP Auto admission, G recovery or final delivery gates.

- HTTP bounded-response prerequisite: response collection now stops after the configured
  body limit plus one byte, rather than downloading the complete body before truncation.
  Output reports a lower bound instead of claiming an unknown total. Separate transport
  tests cover empty, exact-limit, over-limit and zero-limit bodies plus a chunked server
  that never completes its stream. This bounds retained application body bytes, not
  reqwest's internal per-chunk/decompression buffers. Locked `talos-tools --features
  network --lib http_request::` passed 29/29 after adding the independent reviewer's
  suggested incomplete-body error test. Static incremental review found no blocker;
  both governance validators passed with zero warnings before this test-only follow-up.
  This is not final exact-head delivery approval. HTTP Auto admission remains
  unimplemented. A maintainer question is pending on whole-operation network review
  versus per-destination isolation: ADR-064 does not currently authorize unknown/unbound
  targets, while ADR-090's prepared route requires every destination to be declared.
  No scope expansion is implemented or assumed while that decision is pending.

- File recovery implementation checkpoint: `edit` now accepts opt-in `retain_backup`
  for legacy and anchored edits and returns a real recovery ID. A separate `edit`
  call with path/ID restores original contents through current Write authorization,
  verifies the post-edit digest and consumes the ID on successful restoration.
  The instance-local memory store expires entries after 15 minutes, caps originals at
  2 MiB/file, 16 MiB total and 64 records, and refuses capacity before mutation without
  evicting live entries. README describes actual use and limits. Mutation errors retain
  recovery IDs; post-mutation snapshot errors explicitly report the changed state.
- Locked file-tools tests passed 76/76 and locked file-write library Clippy passed with
  warnings denied before the additional concurrent-restore test. Independent security/API
  review and that final test rerun are pending. This is uncommitted implementation,
  not complete G: no durable/delete/directory recovery, full OS identity/CAS protection
  against external writers, higher-risk Auto admission, complete impact evidence or
  cancellation/platform validation is claimed. H/G/T8 remain Active and incomplete.
- Independent incremental review requested changes: recoverable and ordinary anchored
  edits used separate locks, and a post-rename restore error omitted its potentially
  completed effect. The working implementation now shares the registry path lock for
  anchored edits/backups/restores and reports uncertain restore completion with retained
  ID. Prior tests passed 77/77 including single-consumption concurrent restore; latest
  fixes still require regression coverage for mixed edit modes and injected post-rename
  failure plus reviewer follow-up. Do not treat the recovery stage as review-approved.
- Recovery follow-up: locked file-tools tests passed 80/80, including injected
  post-rename edit and restore failures, concurrent single-consumption recovery and
  mixed recoverable/ordinary atomic writes sharing a revision lock. Independent
  incremental security/API re-review withdrew both blockers and found no new concrete
  blocker; it was static review, not final exact-head approval. Legacy non-backup
  string writes retain their existing non-atomic behavior and do not join this lock.
  Both governance validators passed before this checkpoint-only update. The stage
  remains local; complete H/G acceptance and final platform/preflight/CI gates remain.

### Production inventory checkpoint

`registry_builtin_tests::product_registries_have_no_uninventoried_native_operations`
compares the complete Print/TUI/MCP registries against an explicit inventory and checks
instance ownership for every entry. New registrations fail this test until inventoried.
This is a coverage assertion, never a runtime tool-name allowlist or proof of Auto support.
Runtime shared composition has a separate Product/Runtime inventory-equivalence test.

| Registered operations | Current I308 disposition / remaining evidence |
|---|---|
| `read`, `ls`, `read_image`, `document_extract`, `grep`, `glob`, `diff`, `stat`, `tree`, `find_symbol`, `find_references`, `list_symbols`, `list_imports` | Real Agent dispatch tests cover default Read Allow and explicit Read Deny for all listed operations, with actual results and zero model assessments/grants. Image attachment reaches only the next provider request after an allowed read. Complete profile/platform coverage remains outstanding. MCP omits `read_image`. |
| `write`, `edit`, `delete` | Exact-content atomic creation is merged in #769; complete-image review and opt-in real recovery for string/anchored edit and text deletion are merged in #770. Binary/NUL and secret-like content retain human fallback. Expanded special-path/policy tests are published in #771; full G acceptance remains incomplete. |
| `http_request` | Ordinary whole-invocation Auto merged in #783, with actual proxy/redirect AllowOnce and refusal/stale/external/replacement effects tested and exact-head platform CI. Full profile acceptance remains open. |
| `fetch_url` | Ordinary whole-invocation Auto merged in #784 after full local preflight, review and exact-head platform CI. Reuses HTTP configuration/final-context binding and eight real proxy/redirect scenarios; full profile acceptance remains open. Configuration distinguishes constructor redirects from timeout-override defaults and output truncation from full response buffering. |
| `save_url` | Recoverable capture and complete-text mutation Auto merged in #785 at `f0d484b255bf766eafb1573bd0870ef38ae8de77`; 16 production Agent scenarios cover model outcomes, policy, registration ownership, input changes and binary/default fallback. Full standard preflight, cargo-xwin, independent review and all-eight-success exact-head CI 40525328345 passed. Complete profile/G acceptance remains open. |
| `web_search` | ADR-090 excludes ordinary whole-invocation Auto, including compatibility/default search. Concrete production legacy registration currently refuses before model/human review because its profile has no safe typed network resource, including with explicit policy Allow. Six real-Agent disposition scenarios pass locally; this is unavailable behavior, not a working search path. Guarded instances require their dedicated prepared-network route and cannot downgrade. I293 backend/transport activation remains separately owned; synthetic carrier fixtures do not prove search backend success. |
| `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list` | Real Agent dispatch verifies default Read Allow returns actual temporary-repository status/diff/history/commit/branch evidence and explicit Read Deny rejects all five, with zero assessor calls/grants. Complete profile/platform evidence remains outstanding. |
| `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout` | I300 prepared command route retained; no reopening of accepted whole-command Pull semantics. I308 full coverage evidence outstanding. |
| `bash` (Unix) / `powershell` (Windows), `exec` | Whole-invocation Auto merged in #783 with real Agent allow/refusal/cancel/stale/registration/replay tests. MCP now routes registered instances through cwd-aware one-use invocation admission; local MCP integration proves Exec and Unix Bash Git-config mutation after AllowOnce plus explicit Execute Deny. Windows PowerShell MCP execution and exact-head Windows CI for this route remain outstanding; full profile acceptance remains open. |
| `todo_create`, `todo_create_batch`, `todo_update_batch`, `todo_update_status`, `todo_update`, `todo_delete`, `todo_add_dependency`, `todo_remove_dependency`, `todo_query` | Session-scoped Internal facets retain default deterministic Allow, not model approval. Local real-Agent tests cover all nine operations under default Allow and explicit Internal Deny, actual SQLite effects, unchanged other-session data, zero assessor calls and zero grants. MCP does not add Todo; complete profile/platform evidence remains outstanding. |
| `status` | MCP-only read tool; concrete built-in ownership checked. |
| `delay`, `schedule`, `list_scheduled_tasks`, `cancel_scheduled_task` | Scheduler registration and action review merged in #772. Product-owned instances are built-ins; caller-supplied instances remain external. Exact scheduling/cancellation review does not approve future tool effects. `list_scheduled_tasks` retains Read policy. Complete profile/final acceptance remains open. |

Inventory work found and repaired a production gap: Print/Inline/RPC/TUI previously
constructed scheduler tools locally but passed them through generic external registration.
The new internal factory creates and registers the instances together; it accepts no
caller-supplied tools. Generic registry constructors preserve external ownership semantics.
Desktop-specific extensions, conditional skills, process capabilities and complete feature/host
coverage still require separate verification; the table does not declare acceptance 1 complete.

Validation: locked CLI `builtin_registration_tests` passed 5/5, including rejection
of external scheduler collisions without relabeling. The CLI bin suite passed
375/375 before the final collision test was added; CLI Clippy passed with warnings
denied. Independent incremental ownership/security review found no new concrete
blocker and confirmed registration failure starts no scheduler actor. This is static
incremental review, not final exact-head approval or full T8 validation.

### Exact-input creation checkpoint

Trusted ordinary `write` now uses an additive, default-unsupported
`assess_builtin_invocation` assessor method. The provider receives exact bounded
contents, complete facets, current user intent, the existing capability-bound create
projection and a fresh nonce/digest. High-confidence matching responses return Once;
explicit human/Deny policy and admission checks remain authoritative. Existing
third-party assessors do not automatically acquire this capability. The complete
encoded input is checked before cloning and the complete model payload is capped at
16 KiB; oversized or secret-like evidence falls back to human approval. This narrows
the model-reviewed payload relative to the legacy 64 KiB content-shape classifier.

Real Agent tests use `WriteTool` and `CapStdAtomicCreateCapability`, not a logging
tool: allow creates the exact file, while uncertainty, provider error, Auto cancellation,
session rebinding and oversized input create no file and no persistent grant. Final
locked Agent library tests passed 492/492 and Agent Clippy including tests passed
with warnings denied. Independent incremental static review identified the pre-clone limit correction
and otherwise found no concrete new blocker. This stage does not enable existing-file
edits/deletes, expand network authorization or complete T8. Ordinary Execute and the
remaining H/G coverage remain unimplemented.

### Creation stage validation checkpoint

- Local implementation `9852b65b3` adds exact-content creation assessment;
  `75ffc8f9f` adds real Agent refusal cases for low confidence, mismatched digest,
  wrong schema, extra response fields, malformed JSON and oversized responses.
  All cases prove no file creation and no persistent grant; the focused test passed.
- Independent incremental static re-review confirmed the pre-clone input limit
  finding is resolved and found no new concrete blocker. It is not final whole-stage
  exact-head approval, and the reviewer did not independently rerun tests.
- Standard locked preflight at `75ffc8f9f` passed exit 0, including workspace checks,
  Clippy, tests (Tools 951/951) and both Runtime SDK consumers. Log:
  `/private/tmp/i308-creation-stage-preflight-authorized.log`. The preceding sandboxed
  run failed one process-observation test with EPERM; the authorized rerun passed.
- These commits remain local. Native Windows evidence, complete ordinary
  Write/Network/Execute coverage, full higher-risk recovery, final review, CI and
  merge are outstanding. This checkpoint does not complete T8 or any later task.

### Recoverable deletion local checkpoint — 2026-10-26

- Added optional regular-file `delete` backup/recovery without changing public
  `DeleteInput` struct literals. Backup capture declares and checks Read in addition
  to Write; restoration requires Write and never overwrites a recreated path.
  Instance/path-bound, expiring receipts reuse the existing edit store limits.
- Unix backup opening uses the existing Git integration pattern with libc
  O_NOFOLLOW/O_NONBLOCK; Windows uses the existing windows-sys reparse-point flag
  and rejects reparse metadata. The file-write feature now explicitly enables these
  already-declared system dependencies; no dependency package or unsafe code added.
- Locked file-write file-tool tests passed 86/86 and scoped tests Clippy passed with
  warnings denied. An initial default-feature run selected zero tests and is not
  evidence; a subsequent missing test import was fixed before the passing run.
- Independent incremental review identified the original blocking/FIFO and symlink
  backup-open race; the revised no-follow/nonblocking handle check closed that finding.
  It did not grant whole-stage or merge approval. Ancestor/open-to-delete external
  path races remain a limitation; no atomic rollback or concurrency isolation claimed.
- README documents actual recovery limits. Auto deletion and full G coverage remain
  unimplemented. Final full preflight, xwin/native Windows, exact-head review/CI and
  merge remain outstanding. This local checkpoint does not complete T8.

### Bounded recoverable edit follow-up — 2026-10-26

- Recoverable legacy/anchored edits now declare Read alongside Write and require
  matching resolved targets before backup capture. Original content is read through
  the shared no-follow/nonblocking regular-file handle with a 2 MiB maximum.
- Atomic revision validation uses the same bounded handle read and its permissions,
  eliminating its previous full-file read. Recoverable edits reject either before
  or after content above 2 MiB before backup insertion/mutation, so admitted backups
  remain restorable within the bound. External concurrent path replacement is still
  not isolated; this is not a general transactional rollback guarantee.
- Locked file-write file-tool tests pass 90/90, including external Read/Write
  authorization, oversized originals/replacements and enlarged revision targets.
  Log: `/private/tmp/i308-bounded-edit-tests.log`. Final-source cargo-xwin locked
  MSVC test-target checking passes; log:
  `/private/tmp/i308-file-recovery-xwin-authorized.log`. This is compilation, not
  native Windows runtime evidence. The first sandboxed xwin attempt failed cache
  permission; the authorized rerun supplied compilation evidence.
- Full H/G model review/execution coverage and final platform/review/CI/merge remain
  outstanding. No Auto edit/delete allowance or T8 completion is claimed here.

### Recovery stage full validation — 2026-10-26

- Implementation candidate `c924a3ee6` passed the complete standard preflight:
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 ./scripts/release_preflight.sh`.
  Authorized run exited 0; log:
  `/private/tmp/i308-recovery-stage-preflight-authorized.log`.
  This includes both governance validators, formatting, locked workspace check,
  Clippy with warnings denied, workspace tests and both external Runtime SDK fixtures.
- The preceding sandbox run exited 101 with 959 tools tests passing and one
  process-observation EPERM in `prepared_local_push_runs_all_eight_admitted_stages`.
  The authorized complete rerun passed; the sandbox result is not a product pass
  and was not bypassed by excluding the test.
- This verifies the local recovery implementation stage only. Auto edit/delete
  review and execution, ordinary effectful Execute/Network coverage, remaining
  host evidence, final independent review, exact-head CI and merge are outstanding.
  T8 remains Active; no later handoff task is completed by this evidence.

### Stable foundation stage submission — 2026-10-26

- Stage delivery state: Review; the complete iteration remains Active and partial.
  The candidate contains built-in ownership/registration, retained network review,
  exact-input creation review, bounded edit/delete recovery and HTTP boundary fixes.
  This is a stable implementation stage under the existing #765 claim, not a new
  requirement or completion of H/G.
- Refreshed main to `fef542c0186dc8a3413a66396d2e77d8d8eaa666` and rebased all
  18 local commits without conflict. `git range-diff` reports every patch unchanged
  from the previous `8f0ce2340..d56feeb3a` series. New base adds the dormant I293
  SearchRouter core; the rebased stage requires fresh full validation and review.
- Previous independent security/API review found no concrete blocking issue, but
  it is not the new candidate's exact-head approval. Fresh stage CI/review and
  merge-time CAS remain required; native Windows runtime remains outstanding.
- Remaining implementation: higher-risk edit/delete model evidence and actual Auto
  execution, ordinary effectful Execute/Network admission under resolved contracts,
  complete tool/profile/host dispositions and full I308 acceptance. No later
  handoff item is completed or waived by this submission.

### 2026-10-26 Current-head validation failures and fixture isolation

At `99c828313`, two authorized parallel standard preflights each failed one
unchanged search refusal socket fixture (Tools 970 passed / 1 failed). An isolated
workspace-feature rerun passed. A temporary test-only error-chain diagnostic in
a complete library run captured `ConnectionReset` / errno 54 with
`is_connect=false`; it was removed. No production classifier change or root-cause
fix is claimed. Issue #768 owns this independent validation residual; the earlier
I300 BrokenPipe observation and this reset must not be relabelled as typed
ConnectionRefused or used to broaden failover.

The serial standard preflight then failed earlier at this stage's new CLI
inventory assertion: Print lacked all Todo tools. `default_sessions_dir()` depends
on process-global home environment, and existing init-wizard tests remove HOME.
The inventory fixture now uses the existing explicit Todo-contribution builders
with a temporary sessions directory, preserving the full expected inventory and
instance-ownership assertions without depending on ambient home state. Focused
registry tests pass; full post-fix preflight, renewed exact-head review and remote
platform evidence remain outstanding. No T8 completion or merged stage is claimed.

Local logs: `/private/tmp/i308-foundation-rebased-preflight.log`,
`/private/tmp/i308-foundation-preflight-retry.log`,
`/private/tmp/i308-search-classification-diagnostic.log`,
`/private/tmp/i308-foundation-preflight-serial.log`, and
`/private/tmp/i308-registry-isolation.log`. Serial scheduling is a diagnostic and
validation alternative, not a fix for #768 or evidence of parallel reliability.

### Foundation candidate published — 2026-10-26

- Source Issue #768 is the independent search-fixture validation residual discovered
  while validating this stage. This owner retains the evidence and index mapping;
  fixture implementation remains outside the I308 Work Slice and requires its own
  effective ownership before changes.
- Candidate `aed967c4322f942a239c4b12cd1ab9747203adfb` passed complete standard
  preflight with `RUST_TEST_THREADS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
  CARGO_PROFILE_DEV_DEBUG=0 ./scripts/release_preflight.sh`: exit 0 and final
  `release preflight: passed`. CLI 376/376, Tools 971/971 and both external Runtime
  SDK fixtures passed. Log: `/private/tmp/i308-foundation-preflight-serial-fixed.log`.
  This proves this serial execution, not parallel reliability or repair of #768.
- Independent read-only Agent static permission/security/API review approved that
  exact candidate without blocking findings; it did not rerun the tests and shares
  the session's tool/GitHub identity, not a separate natural-person account.
- Pushed the stable stage and opened Draft implementation PR #769. Remote CI is
  running. The initial remote reconciliation failure was the missing #768 index
  row, now synchronized here; no functional test failure is inferred from it.
- Stage remains Review; I308/T8 remains Active / partial. Native Windows runtime,
  final exact-head CI/review/CAS and all remaining H/G implementation are open.


### File mutation review implementation started — 2026-10-26

- Local continuation branch `feat/i308-file-mutation-auto` starts at the published
  foundation head `ffa35973022c18f752cacad1922325e15c648379`; #769 stays unchanged.
- Added a compatible default prepared-invocation file evidence hook and a separate
  plan-bound before/after text carrier. Each image is bounded to 2 MiB without
  truncation; relative targets are rejected and Debug hides contents. The carrier
  is not authority; dispatch must independently check built-in instance ownership.
- `cargo test --locked -p talos-core` with incremental/debug disabled passed:
  122 unit tests plus all eight integration tests. Existing downstream trait
  defaults compile. The two new tests cover plan rebinding/private Debug and both
  over-limit images. No new dependency or changed existing public struct shape.
- This is local implementation plumbing only: no file producer, dispatcher or
  Auto assessor consumes it yet. Edit/delete Auto execution is still unimplemented.
  Next implement read-admitted content capture, identity-bound mutation review,
  fresh-state checks and mandatory bounded recovery through the normal pipeline,
  then end-to-end outcome tests and independent review before publication.

### File mutation dispatch admission — 2026-10-26

- Prepared normal-tool dispatch now routes file-change evidence through an
  instance-ownership check and verifies its retained plan identity before
  resolution. Simultaneous command and file-change evidence is rejected.
- The additive resolver receives complete private images, normalized arguments,
  permission facets and user intent separately from redacted presentation. Its
  default preserves existing approval behavior and adds no Auto allowance.
- Focused locked agent pipeline tests pass 20/20, including trusted allow/deny,
  same-name external instance rejection and mismatched-plan rejection. Approved
  evidence yields an exact one-use prepared proof without persistent grants.
  `cargo fmt --all -- --check` passes.
- This stage does not implement the file-tool producer or model assessor.
  Actual edit/delete Auto execution, stale-state/recovery integration tests,
  independent security review and final platform/CI gates remain outstanding.

### Recoverable string edit producer — 2026-10-26

- Added a prepared producer for legacy string edits with `retain_backup=true`.
  Preparation performs no file read. An exact read admission captures bounded
  complete text; a fresh plan exposes before/after evidence and requires a new
  Read/Write admission before atomic replacement through the owning backup store.
  The final replacement checks the captured content digest and rejects changed
  content. Anchored edits, restore and non-backup edits retain their existing route.
- Explicit `cargo test --locked -p talos-tools --features file-write --lib
  file_tools::` with incremental/debug disabled passed 93/93. New tests exercise
  real capture, first-match replacement, owning-instance restore, missing proof
  refusal and concurrent-content preservation. Tests live in a separate file.
- The initial default-feature check did not compile this feature-gated module;
  its zero-test run is not evidence. Explicit feature validation exposed and
  fixed a provenance move and incorrect test permission setup before the passing
  run above. No default-feature success is claimed as file-write coverage.
- Changes are local and still require source/security review, final integration
  and platform validation. The Auto assessor is not implemented, and this does
  not establish model-approved edit/delete execution or T8 completion.

- Follow-up standalone file-write Clippy exposed that test dev-dependencies masked
  the production optional permission dependency. `file-write` now enables the
  existing `talos-permission` dependency directly; standalone locked Clippy with
  `--features file-write --lib -- -D warnings` passes. The producer refuses an
  oversized after image before allocating it; the additional test also proves
  the file remains unchanged, and capture/commit identities differ.

### Model-reviewed recoverable string edits — 2026-10-26

- Added a separate complete-image assessor payload and provider adapter. Auto
  dispatch reviews registered native `edit` with `retain_backup=true`; the
  payload includes exact arguments, original/result, facets, user intent and a
  fresh nonce/digest. Strict matching high-confidence `allow_once` is required.
  Permission revision/reset checks remain live, and all outcomes are one-use.
- The shared 16 KiB encoded review budget is enforced without truncation. Larger
  files may use bounded recovery but require human approval; suspected secrets,
  unsupported assessors, invalid responses and uncertain outcomes also fall back.
  Delete and anchored-edit Auto remain unimplemented, not implicitly supported.
- Agent end-to-end tests cover 13 scenarios: real replacement on allow; human
  required, assessor error, disabled/stale context, oversized request, low
  confidence, wrong digest/schema, extra response fields, malformed/oversized
  response, and a concurrent writer during review. That writer's data survives
  even when the assessor returns allow. No persistent grant is created.
- Locked agent library tests pass 494/494; locked agent library Clippy with
  `-D warnings` passes. These are local checks, not full workspace, native Windows
  or independent review evidence. Foundation PR #769 remains pending remotely
  (run 40274739232 has no jobs); the continuation is not pushed or merged.

### Captured recoverable deletion — 2026-10-26

- Added read-admitted deletion capture with a fresh final plan identity. Text files
  expose complete-original evidence to the same invocation-local model review;
  binary files retain human-approved backup/deletion without text-model evidence.
  Preparation itself performs no filesystem read. Recovery and other deletion
  modes keep their existing routes.
- Final execution consumes exact-plan Read/Write evidence, re-resolves the target,
  and checks the captured bytes under the recovery/path lock before creating a
  backup and removing the file. The returned receipt restores through the owning
  tool instance. This lock serializes participating Talos mutations; it does not
  claim exclusion of arbitrary external writers after the final check.
- Separate producer tests cover real text/binary deletion and restoration, changed
  content preservation, missing admission and distinct capture/commit identities.
  Explicit standalone file-write file-tool tests pass 94/94. Agent end-to-end
  review tests pass for edit and delete, with 13 scenarios each. These remain local
  tests; security review, final workspace/platform checks and merge are outstanding.
- String-edit assessor stage committed as `81df1fc45`. Anchored-edit Auto, ordinary
  effectful Execute and ordinary HTTP/WebSearch Auto remain unimplemented. T8 and
  all later handoff items remain incomplete.
- Independent static review found a binary-boundary omission: UTF-8 with NUL was
  treated as text. The continuation now excludes captured NUL files from deletion
  text evidence and rejects NUL in either edit image before invoking the assessor.
  Added Agent refusal cases for NUL original/replacement and a producer case proving
  NUL deletion remains human-admitted and recoverable. Review is not final approval;
  the review finding and current full-preflight outcome still require verification.
- Follow-up independent static inspection confirmed the NUL fix and found no new
  concrete blocker in the staged continuation. Focused Agent tests pass with 15 edit
  scenarios and 14 delete scenarios, including the zero-assessor NUL cases. The
  complete agent library suite passed 495/495 before the final NUL fixture addition;
  do not treat that earlier suite as exact-final evidence. Full serial locked
  preflight is running with log `/private/tmp/i308-mutation-preflight.log`; no final
  outcome, native Windows evidence or publication approval is claimed.

### Retained anchored-edit review — 2026-10-26

- Extended the existing prepared edit path to snapshot-anchored operations with
  `retain_backup=true`. Both edit schemas share read-admitted capture, complete
  before/after evidence, one-use approval and real recovery-backed execution.
  Final anchored execution also rechecks the snapshot record and captured revision;
  an invalidated snapshot does not execute even when file bytes are unchanged.
- Extracted one shared anchored-edit calculation used by ordinary and prepared
  execution. It preserves anchor/revision/overlap checks, checks the complete output
  length before allocation for recoverable edits and builds the output in one pass.
  This is implemented behavior, not a separate future refactor promise.
- Explicit standalone file-write file-tool tests pass 95/95. A separate prepared
  anchored test covers actual edit/restoration, changed content before/after capture,
  snapshot invalidation, oversize, bad anchor and overlapping operations. Agent
  production dispatch tests pass for string edit, delete and anchored edit; anchored
  review uses the same allow/uncertain/error/cancel/stale/malformed/oversize/NUL matrix.
- Full preflight started before this anchored continuation is still running at
  `/private/tmp/i308-mutation-preflight.log`. Source changed during that run, so even
  a passing result will not establish final exact-head validation for this stage.
  Final workspace/platform validation, independent review and merge remain open.
  Ordinary effectful Execute and ordinary HTTP/WebSearch Auto remain unimplemented;
  T8 and the original later handoff queue are incomplete.
- Follow-up validation: Agent library tests pass 496/496, Agent and standalone
  file-write library Clippy pass with `-D warnings`, formatting and both governance
  validators pass. Incremental independent static review found no concrete blocker;
  this is not final exact-head publication approval.
- The earlier full preflight terminated with Tools 974 passed / 1 failed:
  `prepared_local_push_runs_all_eight_admitted_stages` could not launch `ps`
  (`Operation not permitted`) to observe maintenance descendants under the sandbox.
  A full serial preflight is now running outside that sandbox with stable source;
  log: `/private/tmp/i308-mutation-preflight-final.log`. Its outcome is pending.
- Anchored continuation committed as `037bf5e55`. Locked cargo-xwin checking of
  `talos-tools --no-default-features --features file-write --tests` for
  `x86_64-pc-windows-msvc` passes; log: `/private/tmp/i308-anchored-xwin.log`.
  This proves test-target compilation only, not native Windows execution. The
  sandbox attempt could not update the cargo-xwin cache link; the authorized
  outside-sandbox rerun succeeded. Fetch of `origin/main` found no upstream-only
  commits at this checkpoint; no rebase was necessary.
- Locked Agent test-target cargo-xwin checking also passes for
  `x86_64-pc-windows-msvc`; log: `/private/tmp/i308-agent-mutation-xwin.log`.
- Independent read-only Agent permission/security/API static review APPROVE:
  base `ffa35973022c18f752cacad1922325e15c648379`, head
  `099e6bd4754e79e0eca7e8cbb4a86c09c5d15a46`. No blocking findings remain;
  the earlier NUL boundary issue is closed. Reviewer shares this session's tool
  and GitHub identity, is not an independent human, and did not independently run
  tests. Approval covers this file-mutation stage only; full preflight, native
  Windows, remote CI and merge-time CAS remain pending. No external-writer
  isolation or universal secret detection is claimed.
- Current full preflight has passed Tools 976/976, including the formerly
  sandbox-blocked eight-stage local Push test, and workspace doctests. Runtime
  SDK consumer checks are still running; no complete preflight pass is claimed.
  Both governance validators pass after the evidence update.
- Full serial locked preflight subsequently exited 0 (`release preflight: passed`),
  including both independent Runtime SDK consumer fixtures. Log:
  `/private/tmp/i308-mutation-preflight-final.log`. Production source remained
  stable from `037bf5e55`; subsequent commits only record validation evidence.
  Native Windows execution, remote exact-head CI and merge CAS remain required.

### File mutation policy coverage continuation — 2026-10-26

- File-mutation candidate published as stacked draft #770, head `400b76a67`,
  based on foundation #769. Both have live remote CI; neither is merged.
- The next local test-only continuation extends real Agent dispatch for string
  edits, anchored edits and deletions to configuration, dependency, release and
  security paths. Complete captured images and the exact target are asserted at
  the assessor boundary. These path fixtures prove routing, not semantic safety
  of arbitrary configuration contents or a real provider's judgment.
- Explicit priority Write Deny and Ask rules prevent both model review and
  mutation, leaving file bytes intact. Secret-like original/replacement fixtures
  also fall back before any assessor call. No persistent grants are created.
  An initial fixture incorrectly appended Ask after the default rule; it was
  corrected to install an effective first-match explicit rule, without changing
  production policy ordering.
- Locked Agent library tests pass 496/496 with the expanded scenarios, and Agent
  library/test Clippy passes with `-D warnings`. Logs:
  `/private/tmp/i308-mutation-policy-agent.log` and
  `/private/tmp/i308-mutation-policy-clippy.log`. This continuation has no new
  production code; publication review/platform gates remain outstanding. Full T8,
  ordinary Execute/HTTP/WebSearch Auto and all later handoff items remain open.

### Read policy execution continuation — 2026-10-26

- Separate `tests/read_auto.rs` exercises the real Read, Ls, Stat and Tree tools
  through production Agent dispatch with Auto enabled. Workspace-policy Allow
  returns actual file content/metadata/listings with zero assessor calls and zero
  persistent grants. Explicit Read Deny returns an error, also with zero assessor
  calls. Each invocation must produce exactly one tool result.
- This verifies four read operations, not the complete production/profile inventory.
  The file-mutation policy tests are committed independently as `82667ca54`.
- Locked Agent library tests pass 497/497; library/test Clippy passes with
  `-D warnings`. Logs: `/private/tmp/i308-read-policy-agent.log` and
  `/private/tmp/i308-read-policy-clippy.log`. No new production behavior or full
  workspace/native Windows validation is claimed by this test-only continuation.

### Todo deterministic admission continuation — 2026-10-26

- Separate `tests/todo_auto.rs` runs all nine actual Todo tools through Agent
  dispatch against initialized SQLite storage. Default Internal Allow performs
  actual create/batch/update/status/delete/dependency/query operations; explicit
  Internal Deny leaves rows and dependency edges unchanged. Both paths require
  exactly one result, zero assessor calls and zero persistent grants. Other-session
  data remains unchanged and absent from results.
- This corrects the earlier inventory assumption that Todo mutation needed model
  assessment: the existing session-scoped Internal policy already admits it. No
  production permission rule or authority is changed. Cross-session malicious-ID
  attempts and complete profile/platform coverage are not established by this test.
- Locked Agent library tests pass 498/498; library/test Clippy passes with
  `-D warnings`. Logs: `/private/tmp/i308-todo-agent.log` and
  `/private/tmp/i308-todo-clippy.log`. Full T8 remains incomplete.
- Issue #563 was incorrectly completed when foundation #769 merged, despite its
  remaining acceptance. It is reopened; #769's body now states the issue remains
  incomplete without a closing-keyword phrase. #771's failed reconciliation must
  be rerun after its current CI run is terminal; no product failure is inferred
  from that metadata mismatch.

### Search and diff read admission continuation — 2026-10-26

- Existing `tests/read_auto.rs` now also dispatches real grep, glob and two-file
  diff tools under default Read Allow and explicit Read Deny. Allowed results
  contain the fixture path; denied calls return errors. All seven operations
  retain zero assessor calls, zero grants and exactly one result per invocation.
- Agent's dev-only talos-tools dependency enables its existing `search` feature
  so these tests also compile in isolated package checks. No production feature
  default, dependency version or permission behavior changes.
- Locked Agent library tests pass 498/498; library/test Clippy passes with
  `-D warnings`. Logs: `/private/tmp/i308-search-read-agent.log` and
  `/private/tmp/i308-search-read-clippy.log`. Complete read-family/profile/platform
  evidence remains outstanding; these fixtures do not prove arbitrary search or
  diff content correctness.
- Todo commit `3626be38682369d3f490d285db5c86ece6f3a4b4` received independent
  read-only Agent APPROVE against base `e8e8dccdb5a7d8de281cf54206036778e45e5ac2`.
  No blocking findings; reviewer did not independently execute tests and shares
  this session's tooling/GitHub identity. Full T8 is not complete.

### Retained mutation stage merged — 2026-10-26

- #770 merged as `687b84a93b4a9f40ede1eb600151c0a22a8f2d59` after terminal
  all-success CI 40279276903, including native Windows. Independent read-only
  Agent renewed APPROVE for exact main base `d45d08f0a82706c2abb26fcb32899f8b24fa64b1`
  and head `400b76a678381421ee6a265364ae952a4744776a`: the foundation merge tree
  equals the earlier reviewed base; subsequent mutation changes only record
  evidence. No blocking findings; no independent test rerun or natural-person review.
- Merge CAS and review receipt:
  https://github.com/wjhuang88/talos/pull/770#issuecomment-6410153470
  Issue #563 remains Open and has the partial-stage update. Full T8 and later
  handoff tasks remain incomplete. #771 is still unmerged.
- Search/diff test commit `29212de48809b0147fb9f03f46df6c0e8dd18762` received
  independent read-only Agent APPROVE against `3626be38682369d3f490d285db5c86ece6f3a4b4`;
  no blockers, no independent test execution. Complete local serial locked preflight
  for this continuation is running; `/private/tmp/i308-dispositions-preflight.log`
  is not yet final success evidence.

### Serial workspace validation result — 2026-10-26

- The continuation's authorized full preflight completed successfully with
  `RUST_TEST_THREADS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 ./scripts/release_preflight.sh`.
  The script uses locked dependency checks; both external SDK fixture runs passed
  and the terminal result is `release preflight: passed`. Evidence log:
  `/private/tmp/i308-dispositions-preflight-authorized.log`.
- The earlier sandbox run in `/private/tmp/i308-dispositions-preflight.log`
  failed while observing the prepared local push subprocess with EPERM. The
  authorized rerun passed; the earlier failure is retained rather than reported
  as success. Production/test content is `29212de48809b0147fb9f03f46df6c0e8dd18762`;
  subsequent changes record evidence only. Native Windows CI for the unpublished
  Todo/search continuation is still outstanding. I308 and T8 remain Partial.

### Todo cross-session input verification — 2026-10-26

- The real-Agent Todo matrix additionally supplies another session's item ID to
  batch update, status update, update, delete, dependency addition and dependency
  removal under default Allow and explicit Deny. Current-session rows/edges and
  the other session's item remain unchanged; results do not disclose its title.
  Missing-item delete and missing-edge removal preserve their existing successful
  no-op response, explicitly asserted as `not found`; the other foreign-target
  mutations error. Assessment and persistent-grant counts remain zero.
- Locked Agent library tests pass 498/498 after this extension; formatting passes.
  This establishes these concrete foreign-ID cases, not arbitrary cross-session
  attack coverage, new production behavior or complete I308 acceptance.

### Git read admission verification — 2026-10-26

- `tests/read_auto.rs` now dispatches all five actual Git read tools under
  default Read Allow and explicit Read Deny, alongside the seven existing file
  read/search/diff operations. Fixture initialization uses a cleared environment,
  an empty template and a local unsigned commit; no user repository is modified.
  Allowed calls must return fixture-specific path, commit message or branch
  evidence. Every call produces one result, zero model assessments and zero
  persistent grants. The focused locked production-dispatch test passes.
- This does not implement new Git behavior or prove the full feature/host/profile
  matrix. T8 remains incomplete; ordinary Execute/HTTP/WebSearch and remaining
  native families retain their outstanding requirements.

### Document and symbol read verification — 2026-10-26

- The real-Agent read matrix additionally exercises document extraction and all
  four symbol operations against text/Rust fixtures. Allowed results contain the
  actual content, function, reference path or imported type; explicit Read Deny
  rejects each operation. Each invocation has one result and no assessor calls
  or grants. The focused locked test passes across 17 operations.
- Agent's dev-only tools dependency enables existing `document` and
  `code-intelligence` features. Production defaults and dependency versions are
  unchanged. `find_symbol` takes a directory search root; its fixture uses `.`.
  Image continuation delivery, complete host/profile evidence and full T8 remain
  outstanding; this test stage does not add effectful Auto approval.
- Independent read-only Agent review approved base
  `29212de48809b0147fb9f03f46df6c0e8dd18762` through published #772 head
  `9514b0152e48f4db73fbabdbdd61f0bc9ca0c7fc`, with no blockers and no independent
  test rerun. This review does not cover the later document/symbol additions.

### Image read continuation verification — 2026-10-26

- Added the actual `read_image` tool to the existing real-Agent policy matrix,
  bringing it to 18 operations. An allowed PNG read reaches only the second
  provider request with the expected basename, MIME and byte count; explicit
  Read Deny produces no image attachment in either request. Both paths retain
  zero model assessments and zero persistent grants.
- Enabled the existing image feature only in Agent's development dependency.
  Locked Agent library tests pass 498/498, Agent library/test Clippy passes with
  warnings denied, and workspace format checking passes. These results do not
  establish native Windows or final workspace acceptance.
- Published #772 CI 40285467815 failed its Runtime conditional Checkout test:
  retained repository metadata changed after preparation. The focused local
  reproduction passes; the cause remains unestablished and is not repaired by
  this image test change. Existing Windows jobs remain running. T8 remains
  Partial, with effectful invocation families and final acceptance outstanding.
- Independent read-only Agent static review approved image commit
  `b3b1eb28cff6788f295462ab0a97e75a8167ccf8` against
  `824a276b748e702ae1bac7ca84401cfb356de701`, without blockers or an independent
  test rerun. This was a same-session Agent sharing tools/GitHub identity, not
  a separate human. Mock provider capture does not establish real provider
  acceptance, pixel decoding or longer-history attachment lifetime.
- Additional local investigation: locked Runtime library tests pass 72/72, and
  the compiled conditional Checkout test passes 30/30 isolated repetitions.
  This narrows reproduction conditions but does not explain or repair the
  published macOS CI failure. No security revalidation was weakened.

### Checkout CI diagnosis prerequisite — 2026-10-26

- Retained-repository refusal now reports only fixed changed category names
  (for example `entries` or `object_layout`), including when Tools is compiled
  as a Runtime dependency. Previously that diagnostic existed only under Tools'
  own test configuration and was absent from the failing Runtime CI output.
  No paths, configuration values or captured contents are disclosed, and the
  comparison and refusal behavior are unchanged. This is diagnostic capability,
  not a root-cause fix or permission relaxation.
- Locked Tools Git repository-state tests pass 14/14, Tools Git library/test
  Clippy passes with warnings denied, and Runtime's focused Checkout test passes.
  The initial Tools command omitted the Git feature and selected zero tests;
  only the subsequent explicit `--features git` run counts as evidence.
- Local Git is 2.54.0 while the failing CI recorded 2.55.0. Version difference
  is an observation, not an established cause. Existing Windows jobs continue;
  no failed CI rerun or production fixture change is used to claim resolution.

### Recovery invocation permission verification — 2026-10-26

- A separate real-Agent dispatch test covers restoration through both native
  edit and delete instances. Explicit Write Allow restores and consumes the
  receipt exactly once. Write Deny and Ask with a denying resolver leave the
  mutation untouched and the receipt usable. A concurrent edit or recreation
  is preserved even when current Write policy allows restoration. Every case
  returns one tool result and creates no persistent grant.
- This tests current permission enforcement on a later recovery invocation;
  it does not implement model-assisted recovery approval. The focused locked
  Agent test and Agent library/test Clippy pass. The first test attempt assumed
  edit's multiline receipt format for delete too; the fixture parser was
  corrected to accept delete's existing inline format.
- Standard serial preflight was started at `a110a12fe` before this new test
  existed. Its eventual result cannot establish final-head coverage of this
  additional test; the separate focused run supplies that evidence.
- Independent static read-only Agent review approved `a110a12fe` against
  `17c6e458f`, without blockers or independent test reruns. It confirmed fixed
  diagnostic categories expose no captured paths/values and preserve refusal.
  This same-session Agent shares tool/GitHub identity and is not a human review.

### Current-head validation checkpoint — 2026-10-26

- Independent static read-only Agent review approved recovery verification
  `c16a4548c7c163f705b6b67023140bb3e4eb4a01` against `a110a12fe`, with no
  blockers and no independent test reruns. The review does not establish
  exhaustive race coverage or model-assisted recovery approval.
- The earlier serial preflight exited 101: the prepared local Push test could
  not observe its process (`EPERM`); Tools reported 975 passed and one failed.
  This is failed validation, not a passing final-head result.
- A newly authorized sandbox-external serial locked preflight now covers
  `c16a4548c7c163f705b6b67023140bb3e4eb4a01`; its log is
  `/private/tmp/i308-read-diagnostic-preflight-authorized.log`. It remains
  running at this checkpoint, so no final result is claimed.
- Remote runs 40280848025 and 40285467815 were verified live with Windows
  workspace jobs still in progress. The latter retains its macOS Runtime
  Checkout failure; added category diagnostics are not a root-cause repair.
  Full T8 and the later handoff queue remain incomplete.
- Both project governance and Collaboration Claim validators passed with zero
  warnings after this checkpoint update.

### Session-local action review implementation — 2026-10-26

- Scope follows the accepted effectful H baseline: review only one session-local
  scheduling or cancellation invocation; later scheduled model work retains normal
  tool permissions. No whole-shell/network authority is inferred from this stage.
- Additive `BuiltinActionAssessmentRequest` and a default-unsupported assessor
  method preserve existing public struct literals and exhaustive enums. The
  provider receives bounded exact arguments, implementation description, complete
  facets, current user intent, session/revisions, nonce and digest. Failures,
  stale context, disabled Auto and invalid responses retain human fallback.
- Real dispatch testing exposed an existing scheduler admission gap: Execute
  facets had no typed resource, so the permission pipeline could not even form
  an approval proposal. Following existing session-owned process cancellation,
  scheduler actions now use named Command resources bound to operation plus the
  complete argument digest. They grant no OS executable authority; the normal
  invocation/session binding still applies. No permission-engine bypass added.
- Tests are separate in `tests/scheduler_auto.rs`: 51 real Agent/Scheduler
  combinations cover delay, recurring schedule and cancellation with Allow,
  explicit Deny/human, external ownership, uncertain/low-confidence, stale,
  disabled, malformed/replayed-digest/schema/extra-field/oversized responses,
  error/panic/timeout and oversized input outcomes. Actual scheduler state and
  zero persistent grants are checked. Final locked Agent library tests pass
  500/500; Agent library/test Clippy passes with warnings denied. Logs:
  `/private/tmp/i308-scheduler-agent-full.log` and
  `/private/tmp/i308-scheduler-auto-clippy.log`. Both governance validators pass.
- Independent static read-only Agent security/API review approved the final
  working diff against `f29b4cbc7`, including the final input/payload size checks;
  no blockers or independent test reruns. Same-session Agent shares tool/GitHub
  identity and is not human review. Final exact-head CI/merge remain outstanding.
- Provider review payloads are bounded to 16 KiB, including the digest. Resource
  hashing still serializes the full already-supplied input; no claim that every
  temporary allocation is 16 KiB-bounded. Cancellation exposes task ID and exact
  invocation, not retained task message/timing; insufficient user intent must
  require human review. Future-fire permission enforcement and exhaustive policy
  revision/race coverage are not established by this test.
- The authorized preflight above terminated during the external Runtime SDK
  fixture after concurrent source edits introduced a digest-format compile error.
  The error was fixed. This mixed-source run cannot prove final-head validation;
  rerun only after this stage stabilizes, with no concurrent source edits.
- Remote results: #772 Windows workspace passed in run 40285467815, while its
  macOS Runtime failure remains. #771 run 40280848025 completed all substantive
  Windows tests/checks/governance/smoke successfully, then exceeded the 45-minute
  job limit in post-job Rust cache saving. GitHub still reports cancelled, so
  this is not a green gate. Failed-job rerun requested after terminal evidence;
  no live job was cancelled or replaced.

### Stable scheduler candidate validation — 2026-10-26

- Serial sandbox-external locked `./scripts/release_preflight.sh` exited 0
  at implementation commit `b5e8bbe98b4a4290552fec40c13273d6b9fcc831`,
  with no source edits during the run. Workspace tests, doctests and the
  independent external Runtime SDK fixture passed. Log:
  `/private/tmp/i308-read-diagnostic-preflight-authorized.log`.
- Independent read-only Agent review approved exact delta
  `f29b4cbc76fdca9a219ae39237357a671c873d5c` to
  `b5e8bbe98b4a4290552fec40c13273d6b9fcc831`, with no blockers.
  The reviewer did not independently rerun tests and is not a human reviewer.
- This local success does not repair or explain the previous macOS CI Checkout
  drift. Fresh exact-head CI and merge gates remain mandatory; T8 remains Partial.

### Recovery caller-cancellation evidence — 2026-10-26

- Real edit and delete restoration dispatch now also waits in the production
  approval pipeline, then aborts the owning caller after observing approval
  entry. Neither file state nor the reusable recovery receipt changes, no
  ToolResult is fabricated and no persistent grant remains. A direct restore
  afterwards verifies the receipt was not consumed. This proves cancellation
  before admission, not rollback of an already-started filesystem mutation.
- Locked focused Agent tests passed 2/2 (including ten recovery dispatch
  combinations); Agent library/test Clippy with warnings denied and formatting
  passed. Logs: `/private/tmp/i308-recovery-cancellation.log` and
  `/private/tmp/i308-recovery-cancel-clippy.log`. This local extension is not
  covered by the already-running CI for `6f56cb939`; fresh review/CI remains due.

### Recovery replacement-directory evidence — 2026-10-26

- Real Agent restoration dispatch for edit and delete now also covers a target
  replaced by a nonempty directory. The permission-allowed invocation fails,
  preserves the directory and sentinel contents, and retains the receipt. After
  explicitly restoring the expected post-mutation state, same-instance recovery
  succeeds. This covers a changed target type, not arbitrary same-content inode
  replacement or rollback after a mutation starts.
- Locked focused Agent tests passed 2/2, with twelve recovery dispatch scenarios.
  No production behavior changed. This test extension is not covered by the
  running remote candidate; independent review and fresh exact-head CI remain due.

### Stage integration and rebase checkpoint — 2026-10-26

- #771 completed all six CI jobs successfully, including native Windows and its
  cache post-job. Final target review approved main `687b84a93` to head `e8e8dccdb`;
  merge CAS and review are recorded in
  [the final receipt](https://github.com/wjhuang88/talos/pull/771#issuecomment-6412063934).
  Merge commit is `b21b6146c21894dc42e8fc5bf1d815af71fbc7c8`.
- #772 published head `6f56cb939` also passed all six jobs in CI 40291895444.
  Retargeting to main exposed a merge conflict after the preceding squash merges.
  Rebase onto `b21b6146c` completed without file conflicts, including the two local
  recovery-test commits; the resulting tree at `525df4172` is identical to the
  pre-rebase tree at `9c57b246d`. No cache-workflow change was needed or made.
- Earlier CI does not cover the additional recovery tests or the new commit
  identities. Fresh aggregate review, exact-head CI and merge CAS remain due.
  T8 and the later handoff tasks remain incomplete.

### Conditional Runtime skill policy evidence — 2026-10-26

- Added real Runtime dispatch coverage for opt-in `skill_read`: body/reference
  reads under default Read Allow and explicit Read Deny. All four cases reach
  the next provider request; Deny returns refusal without either content marker.
  Initial provider requests contain neither marker, and an allowed reference
  read does not disclose the skill body. This proves these read-policy outcomes,
  not Auto assessor/grant behavior or all conditional product tools.
- Locked Runtime `tests::skill_disclosure` passed 5/5 again with the final
  disclosure assertions. Locked Runtime lib/tests
  Clippy passed with warnings denied. Independent read-only incremental review
  approved the final test and checkpoint diff. This was a separate Agent sharing
  the session/identity, not human review or an independent test execution.
- This local test-only extension is outside published #772 head `05d0f4889` and
  its running CI 40296390604. No production implementation changed. T8 remains
  incomplete; ordinary effectful command/network Auto and other acceptance gaps
  are not satisfied by this evidence.

### Ordinary invocation decision draft — 2026-10-26

- The previously pending ordinary command/network authorization question now has
  a concrete [decision proposal](../proposals/i308-native-invocation-authorization.md).
  It requests invocation-level review of transitive effects, not per-target
  confinement, and remains unaccepted/unimplemented. A maintainer question is
  pending; no answer is inferred from the earlier Pull-only acceptance.
- Independent read-only design review requested explicit pre-Allow policy
  compatibility and positive network eligibility/I293 ownership boundaries;
  the proposal now includes both, and distinguishes human review from isolation.
  This is design feedback, not implementation approval or T8 delivery evidence.

### Runtime shared-instance registration repair — 2026-10-26

- `RuntimeBuilder::shared_tools()` previously discarded factory contribution
  identity and registered its tools as caller instances. The local repair retains
  internal contributions, preserves last-registration-wins ordering with `tool()`,
  and never transfers built-in identity to a caller replacement. Presets still
  fill only absent names. All registered cleanup owners, including replaced
  instances, remain retained and pointer-deduplicated for shutdown.
- Three separate registration regressions pass; the full locked Runtime library
  suite with `shared-composition` passes 82/82, including actual shared Read,
  preset override, skill policy and shutdown tests. Locked Runtime library/tests
  Clippy passes with warnings denied. These tests do not establish full Runtime
  Auto-dispatch acceptance or ordinary command/network Auto implementation.
- The default-feature locked Runtime library suite also passes 73/73; both
  governance validators pass with zero warnings. The standard workspace
  preflight is running and is not yet claimed successful.
- Independent same-session read-only Agent security/API review approved the
  production/test diff, with no independent test execution. This local repair
  is outside published #772 head `05d0f4889`; it still needs the next stable
  candidate's full validation, exact-head CI/review and merge. T8 remains Partial.
- The first broad test invocation failed during linking with disk exhaustion,
  before test execution. Previously authorized `target/debug` example and
  incremental artifacts were cleared; subsequent library-only tests above pass.
- The sandboxed standard preflight later failed at the eight-stage local Push
  fixture's process-observation call (`EPERM`), with Tools 975/976 passing. It
  therefore did not complete workspace/SDK validation. The same standard command
  is now running with explicit outside-sandbox authorization; its result remains
  pending (`/private/tmp/i308-runtime-registration-preflight-authorized.log`).
- #772 subsequently merged as `881a27b890e5310055380e6e3f5b0da78f9fdd3c`.
  Four local continuation commits were rebased onto it without conflicts; file
  trees before and after rebase match. The repair is now `75d5811c0`, on
  `feat/i308-runtime-registration`. No new candidate has been published yet.

- Authorized standard preflight completed with exit 0 on 2026-10-26 for
  production tree `57ee6103c3831e9ebaad18339fa0057fc170a653`:
  `RUST_TEST_THREADS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 ./scripts/release_preflight.sh`.
  Workspace checks, Clippy, tests/doctests, governance and external Runtime SDK
  fixture passed; log: `/private/tmp/i308-runtime-registration-preflight-authorized.log`.
  This supersedes the pending result above, preserves the sandbox failure, and
  does not establish remote CI, merge, or full T8 completion.

### Rebased registration candidate — 2026-10-27

- Registration candidate rebased onto main `326e05751ad5d5bead11c7bc7789a82bae265526`; implementation head before this evidence update is `f5d724896`. PR #773 still points to the previous head until local convergence.
- Post-rebase standard locked preflight failed with exit 101 at CLI test linking: `ld: write() failed, errno=28 (No space left on device)`. This is not a passing post-rebase gate and does not establish a product regression. Log: `/private/tmp/i308-rebased-ci-preflight.log`.
- With no Cargo/compiler process active, removed only rebuildable executable files from the previously authorized root `target/debug/examples` directory (33 files, 3,394,336,464 logical bytes); dependency libraries were retained. Available disk rose from 219 MiB to 2.6 GiB.
- Standard locked preflight is being retried with unchanged source, incremental compilation disabled, dev/test debug information disabled and serial tests. Log: `/private/tmp/i308-rebased-ci-preflight-retry.log`. Result remains outstanding until the process exits.
- Full T8 and the original handoff remain incomplete. Ordinary invocation authorization remains a maintainer decision; Desktop Todo factory registration and the remaining operation/profile/platform matrix are not implemented by this checkpoint.

- During retry, reclaimed 96 executable files older than six hours under authorized root `target/debug/deps` (2,485,667,936 logical bytes), excluding shared libraries and current compilation outputs. Available space recovered to 3.0 GiB; workspace tests are now executing. No source or Git data was removed.

- Correction: the retry exited 101 because the age-based executable cleanup removed a still-required cached `talos_config` test binary after Cargo had scheduled it. The earlier assertion that old outputs were safe during an active build was incorrect. This was an agent cleanup error, not a product test failure. Do not remove artifacts during any Cargo run; the next full retry must rebuild missing outputs. Code review APPROVE covers base `326e05751` to `f5d724896`, with no independent builds; final documentation, CI and CAS remain pending.

- 2026-10-27: maintainer explicitly accepted the ordinary Shell/Exec/HTTP whole-invocation authorization proposal. ADR-064 records this scope decision; earlier pending-decision checkpoints are historical. Implementation and its full acceptance remain outstanding; search is excluded under I293/ADR-090.

- Third post-rebase standard locked preflight passed (exit 0) with `release preflight: passed`, including workspace checks, Clippy, tests/doctests and both external Runtime SDK consumers. Log: `/private/tmp/i308-rebased-ci-preflight-retry2.log`. This supersedes the pending retry result, preserves both earlier failures and establishes local validation only; exact-head remote CI and merge CAS remain outstanding.

### Desktop Todo registration continuation — 2026-10-27

- Parent Runtime registration stage #773 merged as `d2ab62dc9b206777913bcd4f721e71e11d357aa7`, after exact-head CI 40440474869 passed all eight checks at `1ccb979421d4c7aaac81ddfc95a1174387c12bbb`. Independent final permission/security/API review and unchanged-base CAS are recorded in [the merge receipt](https://github.com/wjhuang88/talos/pull/773#issuecomment-6433284386). Both local governance validators passed with zero warnings. This closes only the parent registration stage; Desktop Todo and ordinary invocation implementation remain outstanding.

- Follow-up branch `feat/i308-desktop-todo-registration` starts at reviewed PR #773 candidate `1ccb97942`; the submitted parent head remains unchanged while CI 40440474869 runs.
- Added a factory-only RuntimeBuilder session Todo registration method and switched Desktop composition to it. Caller `.tool()` remains untrusted, including caller-supplied instances from the same public Todo factory. Registration-order tests check both override directions. This does not enable ordinary command/HTTP Auto or complete the profile matrix.
- Focused locked shared-composition registration tests are running; full validation, native platform execution and independent review remain outstanding. README documents the new additive builder method.

- Focused registration suite passed 4/4 (`cargo test --locked -p talos-runtime --features shared-composition --lib tests::builtin_registration`); log `/private/tmp/i308-desktop-todo-registration-tests.log`. This verifies factory/override identity and cleanup registration, not Desktop production dispatch or full T8 acceptance.

- Focused locked Runtime shared-composition library/test Clippy passed with warnings denied; log `/private/tmp/i308-desktop-todo-clippy.log`. Independent review and full/Desktop/native acceptance remain outstanding.

- Default-feature Runtime dispatch now has separate session Todo coverage: an allowed create persists the expected item in its bound session, explicit Internal Deny prevents creation, and another session cannot see that item. All nine factory tools are checked against same-name caller overrides in both registration orders without requiring shared-composition. Both tests passed; default-feature Runtime library/test Clippy passed with warnings denied. Logs: `/private/tmp/i308-session-todo-dispatch.log` and `/private/tmp/i308-session-todo-default-clippy.log`. This establishes deterministic policy/store behavior, not model Auto approval or native Desktop execution. Full Runtime shared-composition validation is running; stage-wide preflight, final independent review and exact-head platform CI remain outstanding.

- Full locked Runtime shared-composition library suite subsequently passed 84/84; log `/private/tmp/i308-session-todo-full-runtime.log`. Stage-wide preflight and Desktop/platform gates remain outstanding.

- Desktop Todo candidate `b6fb0bf29bd6d5799ba4c8475f7afa16d6036704` received independent same-session read-only permission/security/API approval against base `d2ab62dc9`; the reviewer did not independently build. The sandboxed standard preflight failed process-observation EPERM. Its authorized unchanged-source retry passed workspace checks, Clippy, tests/doctests (Tools 976/976, including the formerly blocked eight-stage Push), then failed the external SDK coding executable after disk exhaustion. Log: `/private/tmp/i308-desktop-todo-preflight-authorized.log`. This is not a successful full preflight.
- After all Cargo/compiler processes stopped, removed only authorized root `target/debug/examples` artifacts. The first SDK retry reused the damaged executable and failed; cleaning only `talos-runtime-external-fixture` rebuilt it. Both external SDK consumer modes then passed, exit 0: `/private/tmp/i308-desktop-todo-sdk-rebuilt.log`. These component results do not replace an uninterrupted standard-preflight pass. Exact-head CI, native Desktop/platform evidence and merge remain outstanding; T8 and the whole handoff remain incomplete.
- The next standard preflight at `d21cca9aa` also exited 101: rebuilding examples exhausted disk during CLI example linking (`errno=28`), before workspace test execution. Log: `/private/tmp/i308-desktop-todo-preflight-final.log`. With no Cargo/compiler process active, recovery uses the maintainer's existing authorization to clear only root `target/debug` and rebuild consistently with incremental/debug information disabled. Full preflight is still outstanding; independent exact-base/head content review approved `d21cca9aa` without independent builds.
- Clean standard preflight at `423e9236d6cb256da11c6ebad8f1dd172dfa956a` subsequently passed end to end, exit 0, with `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 RUST_TEST_THREADS=1 ./scripts/release_preflight.sh` in the authorized environment. Log: `/private/tmp/i308-desktop-todo-preflight-clean.log`; both SDK external-consumer modes passed. This closes the local preflight gap only; final-head independent review, exact-head CI, applicable native Desktop/platform evidence and merge remain outstanding.

### Ordinary invocation permission foundation — 2026-10-27

- Added an opt-in transitive-effects request marker. It applies incompatible Deny/explicit-human restrictions before deterministic Allow, binds the marker into the one-shot fingerprint, prevents broad authority from reusing or installing Session grants, and preserves it when retained NetworkAuthority reconstructs a request. Existing unmarked requests retain their behavior; the native Pull guard shares the restriction scan.
- Locked permission validation passed: library 164 tests, integration suites 1/1/1/8 tests, and one doctest; warnings-denied all-target Clippy passed. Logs: `/private/tmp/i308-transitive-permission-all.log`, `/private/tmp/i308-transitive-permission-clippy.log`. Tests include 12 lifecycle invalidation combinations, narrow-grant rejection and retained-network reconstruction. Independent same-session read-only security/API review approved the revised foundation after the network reconstruction blocker was fixed; reviewer did not independently run builds.
- This is permission infrastructure only. Ordinary Shell/Exec/HTTP production routing and model review are not connected by this change. End-to-end effects, profile/platform evidence, exact-head CI and final delivery remain outstanding; neither T8 nor the handoff is complete.

### Workspace recovery checkpoint — 2026-10-27

- After pause/resume, all former `/private/tmp/talos-*` worktree directories and local validation logs were absent; Git reports their registrations as prunable. The root checkout and committed foundation `dc1e91c0b3071a65047ddb304ffe8506bb441da6` remain. No cause for deletion has been established. Scanning 3,209 unreachable Git blobs found no ordinary-invocation schema or test sources. The uncommitted Shell/Exec/HTTP routing and its tests cannot currently be recovered or claimed as delivered; their reconstruction and fresh validation remain required.
- Recovery continues in the persistent project-local `target/worktrees/i308-invocation` checkout, branch `feat/i308-invocation-recovery`. Logs are retained under root `target/recovery`; neither directory is within the authorized `target/debug` cleanup scope. Preserve this checkout before any broader target cleanup.
- The preceding ordinary-invocation full preflight failed at linking with disk exhaustion, not a passing workspace result. Historical 504-test local results applied to now-missing sources and do not validate this recovery candidate.
- The sandbox hook fix is being rebuilt as a bounded prerequisite: changed command, directory identity or permission-session state after `BeforeBashSandboxExec` refuses dispatch. Seven scenarios cover unchanged execution plus command/directory/policy/mode/restriction/session mutation. Current test execution: root `target/recovery/sandbox-hook-test.log`; no passing result is claimed yet. Independent review, full preflight, native Windows and final CI remain required. T8 and the handoff remain incomplete.
- Recovery candidate validation: the seven-scenario focused hook regression passed, followed by all 501 Agent library tests (`--locked`, root `target/recovery/sandbox-hook-agent-all.log`). This closes the deterministic hook window only; the existing non-Unix metadata identity fallback is not proof of robust Windows directory replacement detection. No ordinary invocation routing, native Windows result or workspace-preflight success is claimed by this checkpoint.

- Recovery branch rebased without conflicts onto origin/main `33fad1b8d`; rebased Agent library tests passed 501/501 (`target/recovery/rebased-agent-tests.log` in the root checkout). Added an opt-in implementation configuration evidence method with a default of unavailable, Shell/Exec/HTTP implementations and forwarding through CLI/Runtime wrappers. This exposes evidence only; it does not activate ordinary Auto authorization. Locked checks for Tools, Runtime and CLI passed (`target/recovery/invocation-config-check.log`). End-to-end routing, immutable admission binding, security/API review and the full acceptance matrix remain outstanding.

- Connected implementation configuration evidence to registered invocation approval. Registered ordinary calls apply the transitive-effects marker before evaluation. Agent library suite passed 501/501 before the added regression; the five execution-input tests then passed, including eight tool/restriction combinations verifying Deny termination and explicit-Ask human checkpoint precedence over deterministic Allow. Logs: root `target/recovery/invocation-plumbing-tests.log` and `invocation-guard-tests.log`. The initial fixture lacked a typed command resource and was corrected. Explicit Ask currently remains human-resolvable: terminal rejection for incompatible fine restrictions is not implemented by this checkpoint. Shell dispatch still uses its legacy route; the new ordinary model-review path remains unavailable. Do not claim full broad-invocation admission or T8 completion.

- Closed the ordinary scoped-Ask compatibility gap: explicit non-Internal Ask rules with a resource/path scope now produce terminal Deny for transitive-effects requests before deterministic Allow; unscoped Ask remains a human once-only checkpoint. Native Pull without the ordinary marker retains its existing contract. Locked Permission library tests passed 166/166, Agent library tests 502/502, and Permission all-target warnings-denied Clippy passed (root `target/recovery/scoped-ask-{permission-tests,agent-tests,clippy}.log`). Independent same-session read-only permission/security/API review approved `ea9060bd0` plus this bounded diff; reviewer did not build. Updated earlier scoped-Ask test expectation to the accepted contract; this supersedes the preceding checkpoint limitation. Ordinary model routing, actual execution matrix, workspace/platform gates and overall T8 remain incomplete.

- Rebuilt the ordinary invocation assessor contract on the shared bounded action-review engine, with a separate schema/prompt, complete configuration/cwd/directory identity in the digest, one-use freshness fields, secret/size checks and existing deadline/panic/error fallback. A test-only adapter exercises eight admission/assessment cases including stale/digest/error/panic/timeout/missing configuration; no ordinary production route is activated by this checkpoint. Agent library tests passed 503/503 and warnings-denied library Clippy passed (root `target/recovery/ordinary-review-{tests,clippy}.log`). The assessor test proves admission behavior, not real process/HTTP effects or final dispatch freshness. Dispatch binding, search coexistence gate, actual execution matrix, independent final review and full workspace/platform gates remain required.

- Desktop session Todo registration stage #779 merged as `13f4ab2052263bf8f65e46a337d4acfc2f9543df` after exact-head CI 40451455846 passed all eight checks at `4293dc8130546af891dd1cbab168dcb789aabcc8`, both governance validators passed with zero warnings, and independent final target/head review approved against `f86cb225c477b640fe504d33492d97b4e3c31bd8`. This closes the registration stage only. Recovery branch rebased onto that merge, preserving both sides of the sole owner-log conflict; no source conflict. Post-rebase validation remains outstanding. T8 and full handoff remain incomplete.

- Post-rebase dispatch-context stage: configured ordinary Shell/Exec/HTTP calls now retain permission generations, configuration, declared execution directory identity and selected sandbox availability before approval. Final dispatch rejects changes after approval hooks, after asynchronous sandbox-fallback approval and after the sandbox command hook. Prepared/protected network admission remains separate; no ordinary Auto route is activated. Actual normal-pipeline fixture writes a temporary file only in the unchanged case; directory replacement, policy/mode/restriction/session and configuration changes reject without that effect. Locked Agent library tests passed 504/504; the subsequent configuration-only test extension passed both focused tests. Warnings-denied Agent library Clippy passed. Logs: root `target/recovery/dispatch-context-{tests,focused,clippy}.log`. Independent final static permission/security/API review approved the bounded diff atop `d8ffb5cfd`. Fallback-handler mutation still lacks a runtime regression; Windows directory identity retains the existing weak metadata fallback and is not reliable same-file proof. Actual per-input Exec cwd, sandbox-route review disclosure, ordinary routing, search coexistence coordination, execution acceptance matrix and full workspace/platform/remote gates remain incomplete. This supersedes only the preceding post-rebase Agent-validation gap, not full preflight or T8 completion.

- Input-directory metadata stage: additive default `AgentTool::invocation_execution_directories` reports initial directories for the exact input; Exec reuses its real parser/resolver for single commands, steps and pipes, and CLI/Runtime permission wrappers forward it. Frozen review configuration includes directory identities, resolution completeness and sandbox availability/network/read-path settings; Exec reports its actual configured stream bound. Review caught a compatibility regression for steps creating a later cwd: unresolved directories now preserve the original deterministic/human path with explicit incomplete evidence instead of rejecting before execution. The ordinary assessor cannot auto-approve that incomplete evidence; dynamic-directory Auto remains unimplemented, not removed from scope. Locked Exec tests passed 23/23, Agent library tests 504/504, subsequent incomplete-evidence focused assessment test passed, CLI/Runtime check and Agent library warnings-denied Clippy passed (`target/recovery/input-directories-{tools,agent,focused,check,clippy}.log` at repository root). Unix Exec directly verified mkdir followed by a step using the new directory; this is not Agent-integrated dynamic-directory evidence. Independent static security/API review approved after the compatibility repair. Ordinary activation, complete sandbox fallback disclosure, Windows reliable identity, dynamic-directory Auto and remaining execution/platform/preflight/remote gates remain outstanding.

- Sandbox-fallback evidence stage: frozen context and ordinary review configuration now include the actual Deny/Ask/AllowUnsandboxed policy, fallback-handler availability and retained tool configuration. A normal-pipeline Bash fixture with an unavailable sandbox verifies actual temporary-file effects only for unchanged fallback approval; directory/policy/mode/restriction/session/configuration changes during the asynchronous fallback handler reject despite ApproveOnce. This is a custom tool fixture, not host Shell process evidence. Locked Agent library tests passed 505/505, focused context tests 3/3 and warnings-denied Agent library Clippy passed (`target/recovery/fallback-context-{agent,tests,clippy}.log` at repository root); independent bounded static security/API review approved atop `6d3f1d46f`. This closes the previously missing fallback-handler mutation regression and fallback policy disclosure only. Ordinary production routing, real Shell/Exec/HTTP dispatch matrix, I293/ADR-090 coexistence confirmation/amendment, dynamic-directory Auto, Windows identity and full platform/preflight/remote gates remain incomplete.

- Real Exec compatibility evidence: the Unix normal Agent pipeline now executes the actual built-in Exec under an explicit deterministic Allow policy, first creating `nested` then executing `pwd` with that previously nonexistent cwd. It verifies the directory effect and canonical cwd output, closing the earlier custom-fixture/direct-tool-only compatibility evidence gap. Agent dev tests enable the existing Shell feature on all hosts; the redundant Windows-only Shell/Git dev entry is removed without changing Windows feature coverage or dependency versions. Locked Agent library tests passed 506/506 and warnings-denied library/test Clippy passed (`target/recovery/real-exec-{directory,agent,clippy}.log` at repository root). This is Unix deterministic admission evidence, not model Auto, native Windows or full workspace evidence. Dynamic-directory Auto and production ordinary routing remain unimplemented. Coexistence amendment #782 has separately merged as `85ac39b48d7e58f645b1d8b3838c3c73852b0bae` after independent exact-head security/API review, successful documentation CI 40459798936 and merge CAS; this resolves the pending concurrence/amendment prerequisite only.

- Ordinary production routing checkpoint: registered Shell/Exec/HTTP now enter the shared whole-invocation assessor; untrusted same-name Shell retains the legacy route and prepared-network authorization keeps priority. Dynamic future directories are disclosed with complete arguments and bound initial cwd instead of categorically refusing model assessment. The ordinary assessment regression now uses the production resolver directly, without a test adapter. Real Unix Bash and Exec execute temporary directory/file effects only after model AllowOnce; uncertainty, assessor failure and session rebinding produce no such effects (four scenarios per tool). Locked Agent tests passed 507/507 and warnings-denied library/test Clippy passed (`target/recovery/ordinary-routing-{agent,clippy}.log` at repository root). The earlier unchanged `17b723a8b` passed full standard locked preflight outside the sandbox, including both SDK consumers (`target/recovery/invocation-post-coexistence-preflight-unsandboxed.log`); its sandboxed attempt failed only at denied process observation and is not reported as a pass. That preflight does not validate this new routing diff. Independent bounded static security/API routing review approved the local stage, excluding final HTTP/Windows activation; its prompt-accuracy observation was corrected to distinguish future directories from invalid input. The local code does route HTTP and Windows already; missing validation is a merge blocker, not a claim that these routes are disabled. Reliable Windows directory identity, actual HTTP transport effects, the remaining policy/registration/platform matrix and fresh full preflight/remote gates remain outstanding; this local checkpoint is not a final activation candidate or T8 completion.

- Real HTTP execution evidence: an independent test subprocess constructs the production network contributions and actual `HttpRequestTool`, explicitly discloses AdvancedNetwork, and runs the normal Agent/Auto pipeline with inherited local HTTP proxy configuration. Model AllowOnce causes an observed POST body, 302 follow-up GET and successful tool response; uncertainty, assessor failure, session rebinding, explicit Deny and mandatory-human Ask cause no accepted or queued proxy connections. Explicit policy cases make zero assessor calls. Environment changes are confined to the child process; no product SSRF bypass, injected transport API, new dependency version or unsafe code was introduced. The existing network dev feature is now enabled for standalone Agent tests. Locked Agent tests passed 508/508 (`target/recovery/ordinary-http-final-agent.log`); the initial fixture omitted AdvancedNetwork disclosure and was repaired rather than treating the unavailable tool as an authorization result. Independent review identified and prompted repair of a zero-connection assertion scheduling window; final bounded independent static security/API review approved the repaired stage, and warnings-denied library/test Clippy passed (`target/recovery/ordinary-http-final-clippy.log`). Both governance validators passed with zero warnings. Native Windows runtime, reliable Windows identity, remaining invocation policy/registration cases and fresh complete/remote gates remain outstanding.

- Windows directory-token repair checkpoint: replaced size/type/mtime fallback with panic-guarded directory-handle metadata from the existing cap-std 4.0.3 and cap-fs-ext 4.0.3 dependencies; no new dependency or unsafe code. Missing/unopenable directories fail capture. A same-path replacement regression retains the original directory under another name and checks stable identity, replacement detection, missing paths and regular-file refusal. Locked Agent library tests passed 509/509, warnings-denied library/test Clippy passed, and cargo-xwin Windows test-target checking passed (root `target/recovery/ordinary-directory-{agent,clippy,xwin}.log`). The first sandboxed xwin attempt failed updating its external cache symlink; the authorized retry passed. Windows compile is not native runtime evidence. This replaces only the weak metadata fallback: handles are not yet retained across approval and the upstream index is 64-bit, so identifier reuse and full ReFS 128-bit protection remain unimplemented. Native Windows replacement evidence, final independent review, remaining invocation matrix and fresh workspace/remote gates remain outstanding; T8 remains Partial.

- Retained-directory identity checkpoint: the shared private identity value now owns an Arc-backed cap-std directory handle through asynchronous review, final permission hooks and sandbox fallback checks, instead of reducing evidence immediately to a string. Equality and serialized assessment evidence still use only the token; the public assessment request remains Option<String>. Unix and Windows derive device/index from opened-handle metadata inside a panic guard, and canonical paths hash their encoded bytes. Identity code and its regression are split out of the large resolver/test modules. The regression now unlinks the renamed original while its handle stays open, verifies original metadata remains available, and distinguishes the replacement. Locked Agent tests passed 509/509, warnings-denied library/test Clippy passed, and cargo-xwin Windows test-target compilation passed (`target/recovery/retained-directory-{agent,clippy,xwin}.log` at repository root). This closes the missing handle-retention implementation only: native Windows rename/delete/share behavior remains unverified, upstream ReFS 128-bit identity remains unsupported, and freshness checks are not atomic process-cwd confinement. c393c9337 and the retained-handle successor each received independent bounded static security/API approval; the reviewer did not run tests. Remaining execution matrix and fresh workspace/remote gates remain outstanding. T8 and the full handoff are not Complete.

- Real command policy/platform matrix: production Shell and Exec normal-Agent Auto tests now each exercise nine scenarios (AllowOnce, uncertain, error, stale session, explicit Deny, mandatory-human Ask, assessor panic, timeout and mismatched digest). Only AllowOnce creates the temporary directory/file; explicit policies invoke no assessor and no scenario, other cases enter their intended scenario once, and no case persists a grant. Independent review found a false-pass risk from assertions inside the panic-guarded assessor; an outer verified scenario-reached counter repairs it. Final focused Unix execution and warnings-denied library/test Clippy pass (`target/recovery/command-policy-final-{focused,clippy}.log`). Before that counter-only repair, full Agent 509/509 and cargo-xwin test-target checking passed (`target/recovery/command-policy-{agent,xwin}.log`); these are not represented as final-source full/platform proof. The fixture now also selects native Windows PowerShell and cmd.exe-backed Exec, but native Windows execution remains unverified. Independent bounded static review approved the repaired test delta without running tests. Protected-search/no-downgrade, remaining registration/atomicity/profile evidence and fresh final workspace/remote gates still require closure; T8 is Partial.

- Final-baseline and test-reliability checkpoint (2026-10-27): full standard locked release preflight at implementation commit `258fd55d4` passed, including workspace checks, Clippy, tests/doc tests and both independently resolved Runtime SDK consumers (`target/recovery/invocation-258fd55d4-preflight.log` in the root checkout). Subsequent test-only changes add externally asserted scenario-reached counters to ordinary resolver and real HTTP assessment fixtures, so panic containment cannot disguise failed request-evidence assertions as expected refusal. Production Shell/Exec fixtures additionally disable Auto during assessment and verify that a returned AllowOnce does not create either the directory or file. The updated focused `cargo test --locked -p talos-agent ordinary_` passed (`target/recovery/ordinary-reliability-cancel.log`). This is cancellation of Auto authority, not proof of every caller/task cancellation path. Updated-source warnings-denied Agent library/test Clippy passed (`target/recovery/ordinary-reliability-cancel-clippy.log`); independent bounded static review approved the three-test delta without running tests. Registration/replay integration gaps, native Windows execution and final remote gates remain outstanding; the preflight result is baseline evidence and is not claimed for subsequent edits. T8 remains Partial.

- Registration/replay integration checkpoint (2026-10-27): actual production Shell/Exec and HTTP fixtures now test unchecked external registration and replacement of a previously trusted registration with a distinct production instance. Neither enters ordinary-invocation assessment; denied fallback produces no fixture directory/file or HTTP proxy connection. Shell/Exec additionally repeat a successful call ID with identical and changed inputs across model continuations. Ordinary review occurs again before the final execution-ledger reservation: both reviews must satisfy independent precondition counters, the first result succeeds, and the second explicitly reports replay or identity conflict without a second execution. The initial test incorrectly assumed reservation preceded assessment and failed; the corrected fixture verifies the actual contract rather than claiming assessor deduplication. Updated focused `cargo test --locked -p talos-agent ordinary_` passes (`target/recovery/ordinary-registration-replay-all.log`). Full locked Agent library tests passed 509/509 (`target/recovery/ordinary-registration-replay-agent.log`), warnings-denied library/test Clippy passed (`target/recovery/ordinary-registration-replay-clippy.log`), and independent bounded static review approved the two-test delta without running tests. Both governance validators passed with zero warnings, and locked cargo-xwin Windows test-target checking passed (`target/recovery/ordinary-registration-replay-xwin.log`); the sandboxed first attempt could not update its external cache link, and the authorized retry passed. Native Windows execution, final documentation/evidence reconciliation and remote gates remain outstanding. T8 remains Partial.


### Ordinary invocation API and documentation reconciliation — 2026-10-27

- README now describes whole-invocation authority, inherited HTTP proxy/redirect effects,
  dynamic-directory disclosure, explicit policy precedence and lack of transitive confinement
  or rollback. This documents the local candidate, not a merged release.
- Public `AgentTool` additions are defaulted: `invocation_review_configuration()` returns
  unavailable by default, and `invocation_execution_directories(input)` defaults to the existing
  execution working directory. Existing implementers need no new methods. Product wrappers
  forward both methods unchanged. Input-dependent tools must use their real execution parser
  when overriding directory evidence. Metadata never grants trusted registration or authority.
- Custom `AutoPermissionAssessor` implementations must implement
  `assess_ordinary_invocation` for ordinary model approval; its default reports unsupported.
  Existing human/headless fallback remains available. The public assessment directory identity
  remains an optional string; retained handles are private execution evidence.
- Latest owner header and network/command inventory rows now distinguish local implementation
  from merged stages. Full save_url/fetch_url/WebSearch production dispositions and applicable
  host/profile evidence are not inferred from the ordinary HTTP fixture. T8 remains Partial.

- Documentation review approved the bounded diff and identified a substantive inventory distinction:
  default production `save_url`, `fetch_url` and legacy `WebSearchTool` do not opt into
  prepared-network admission. They currently retain deterministic/human paths and have no
  ordinary model AllowOnce implementation. Only I293 guarded compatibility/SearXNG instances
  establish that protected contract. The corrected table does not treat these missing Auto
  capabilities as delivered by fallback documentation. Remaining production fallback tests and
  original Auto acceptance require implementation or an explicit maintainer scope decision.
  Both governance validators passed with zero warnings for this documentation stage; source
  has not changed since the preceding validated test commit.

### Desktop protocol-consumer repair — 2026-10-27

PR #783 first exact-head CI run 40474043084 failed the two enabled Auto Desktop
fixtures on both Linux and Windows at their legacy prompt assertion. The fixture
now validates the actual ordinary-invocation/v1 JSON, frozen input and user intent,
configuration presence, directory identity presence and digest shape, then echoes
the digest with the supported response schema. Existing real Host approval/result,
workspace execution and configured-model report checks remain. A validated counter
asserted outside the assessor ensures caught assertion panics cannot pass fallback
coverage. This is test adaptation, not a change to product permission behavior.
Local pinned locked focused Auto tests pass 3/3 and full desktop-ui tests pass
112/112; desktop-ui all-target Clippy passes with warnings denied. Logs are root
`target/recovery/invocation-desktop-{auto,all,clippy}-fix.log`. Independent bounded
static review approved the test delta without running tests. The fixture checks
configuration presence and digest shape/echo, not independent full hash recomputation.
Native Windows and final exact-head gates remain pending. T8 remains Partial.

### 2026-10-27 ordinary review locale repair

- Remote failure: run 40475830775, workspace job 122093962825, timed out waiting
  for the real REPL's Chinese decision point. The ordinary provider request used
  pure JSON; the old fixture treated it as a conversation. Product inspection
  also found lost session locale and fixed English fallback, so fixture-only
  changes would not repair the behavior.
- Implementation: additive default `assess_ordinary_invocation_with_locale`
  preserves existing assessor implementations. Provider adds a bounded locale
  outside the hashed request; isolated review still has exactly two messages.
  Optional ordinary response summary/decision points use the legacy safe-text
  filters and renderer only after schema/digest/current-context validation.
  Missing or invalid responses use localized content-free failure copy. Existing
  scheduler protocol and fallback behavior are retained.
- Real REPL and TUI scripts now decode ordinary JSON while retaining legacy
  envelopes; they verify request identity/configuration and preserve all original
  locale/history isolation/resume/rotation/manual-denial assertions. The optional
  token-measurement utility accepts both captured encodings without claiming new
  measurement results.
- Local Agent library tests pass 509/509, including a response-presentation
  regression proving stale/digest-mismatched explanation is not shown. Logs:
  `target/recovery/ordinary-locale-{agent-final,repl,tui}.log` in the root worktree.
- Full explicit-feature Desktop tests pass 112/112 and the standard locked release
  preflight passes, including both Runtime SDK consumers. Logs:
  `target/recovery/ordinary-locale-{desktop,preflight}.log` in the root worktree.
  Independent bounded static security/API review approved the locale source and
  fixture delta. Native Windows and fresh exact-head remote evidence remain pending;
  this is not T8 completion.


### 2026-10-27 Windows retained-directory fixture repair

- Native Windows run 40475830775 failed three Agent tests at attempted directory
  rename with sharing violation (OS error 32). cap-std 4.0.3 deliberately opens
  directories without FILE_SHARE_DELETE; its `Dir::from_std_file` contract
  explicitly requires this to prevent races. Upstream source:
  https://github.com/bytecodealliance/cap-std/blob/v4.0.3/cap-std/src/fs/dir.rs
  and `cap-primitives/src/windows/fs/dir_utils.rs` at the same tag.
- Retain production handle semantics. Unix tests still replace/unlink the original
  and require rejection. Windows tests assert rename is blocked, original path
  remains and no destination appears; execution stays in the unchanged directory.
  The identity unit test also checks stable identity and successful rename after
  releasing the handle. Hook/fallback completion flags are asserted outside panic
  containment, so an assertion panic cannot masquerade as successful protection.
- Locked Agent library tests pass 509/509 and cargo-xwin Windows test-target
  compilation passes. Logs: root
  `target/recovery/windows-directory-fixture-{agent,xwin}.log`. The first xwin
  attempt was blocked updating its external cache; the authorized retry passed.
  Independent bounded static security review approved; no product authorization
  predicate, dependency, unsafe code or public API changes in this repair.
- Final full preflight and exact-head remote/native Windows gates remain pending.
  T8 and the whole handoff remain Partial.

- Final unchanged-source standard locked preflight at `cb6e20878` passed end to end (exit 0), including Tools 983/983 and both external Runtime SDK consumer modes. Command: `CARGO_TARGET_DIR=/Users/GHuang/WorkSpace/RustProjects/talos/target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 RUST_TEST_THREADS=1 ./scripts/release_preflight.sh`; log: root `target/recovery/ordinary-final-platform-preflight-serial.log`. Two earlier full-suite attempts failed search connection-refusal classification; focused network/workspace reproductions passed. Serial success does not establish their root cause or a fix; retain the logs and track the unresolved intermittent failure under I293. No search implementation changed. Fresh exact-head remote CI/native Windows and final review/CAS remain outstanding; T8 remains Partial.

### Fetch URL ordinary Auto follow-up — 2026-10-27

The next local stage extends the accepted ordinary network route to positively
registered fetch_url instances. It reuses shared assessment, transitive-policy
compatibility, one-use admission and final retained-context checks. Search and
save_url are unchanged. The existing subprocess proxy fixture now exercises both
production HTTP and fetch instances, with tool-specific input/intent, real redirect
and response effects, all eight allow/refusal/stale/registration scenarios, and
zero accepted or queued proxy connections on refusal. Initial fixture attempts
found a missing Network presentation family and a missing execution-context capture
entry; both were corrected and the wrapped configuration assertions now pass.
Independent static review required accurate disclosure of existing transport:
timeout_secs rebuilds a client with the default redirect limit; maximum_body_bytes
only truncates displayed content after complete buffering. Both facts are explicit
in bound configuration without changing transport or claiming a memory cap.
Focused real proxy tests pass; full locked Agent library tests pass 509/509.
Tools tests, Clippy, final exact-head review, native Windows and remote gates are
still pending. This local stage does not complete T8 or waive save/search acceptance.

### 2026-10-27 additional local validation and remote gate finding

- Fetch stage: `cargo test --locked -p talos-tools --features network` passed (406 library tests plus integration/doc targets); Agent/Tools library-and-test Clippy passed with network enabled and warnings denied. The earlier default-feature Tools invocation ran no fetch tests and is not evidence for network coverage.
- Three HTTP response fixtures now read complete request headers with a 16 KiB bound and five-second timeout, handling partial reads rather than discarding the returned byte count.
- PR #783 exact-head run `40475830775` passed both explicit Desktop feature jobs, but the workspace job failed the real REPL locale acceptance. `scripts/accept_auto_locale.py` still recognizes only the old assessor protocol. Inspection additionally confirmed a product gap: ordinary invocation human fallback uses fixed English and does not propagate the existing session locale/presentation behavior. This finding was unresolved at the stash checkpoint; the ordinary locale repair above now implements the correction and passes local real REPL/TUI acceptance. Changing fixture expectations alone was insufficient. PR #783 must not merge until the behavior and real REPL acceptance are repaired and fresh required gates pass.
- Fetch completed successfully against origin; current implementation HEAD is 20 commits ahead and zero behind origin/main. No rebase was needed at this checkpoint.

### Ordinary invocation stage merged — 2026-10-27

- PR #783 merged as `da280c8a77b25798e59df37f3bdc2bf2fafce1c9`, exact candidate `ae01ebb3920d4e50d8cd2942e81528d1eb66b731`. CI 40482737690 passed all eight checks, including native Windows workspace/Desktop and Linux Desktop. Both local governance validators passed with zero warnings. Independent same-session read-only Agent final permission/security/API review approved exact head/base `85ac39b48`; no independent builds or natural-person approval claimed. Merge receipt: https://github.com/wjhuang88/talos/pull/783#issuecomment-6441484942 .
- This closes the Shell/Exec/ordinary HTTP stage only. Fetch follow-up remains local; Save URL/search and complete operation/profile acceptance remain incomplete. T8 and the full handoff remain Partial.

- Fetch full standard locked preflight passed (exit 0), Tools 983/983 and both external Runtime SDK consumers; root log `target/recovery/ordinary-fetch-preflight.log`, same low-debug/incremental-disabled serial command and shared target as the preceding ordinary stage. Independent static permission/security/API review approved original implementation `0eb2a70c5` against `cb6e20878`, without builds. Rebased onto merged parent `da280c8a7`; `git diff 1dcdf885d HEAD` showed only one owner-document separator change, with all executable sources, lockfile and scripts identical. The conflict retained both parent evidence and Fetch notes. Remote exact-head/native Windows and final review remain pending; T8 remains Partial.

- Fetch stable candidate published as PR #784 at `6e01c5690`; exact-head CI and native Windows evidence are pending. No T8 completion is claimed.


### Save URL implementation checkpoint — 2026-10-27

- Local implementation commit `7590b1df09555d5c0a452ffc6f3fff331a6cca08`
  checks each received response chunk before adding it to the retained download.
  Over-limit responses stop before destination creation or writing. This bounds
  retained body length; it does not claim a bound on reqwest internal allocations
  or bytes already received by the transport. Existing download/write permission
  behavior is unchanged; Save URL model Auto admission and overwrite recovery
  remain unimplemented.
- Save URL tests moved to `crates/talos-tools/src/save_url/tests.rs`.
  Locked Tools tests with `network,file-write` passed 10/10, including exact limit,
  no Content-Length, and oversized unfinished response rejection before EOF.
  An initial `network`-only command selected zero Save URL tests because this tool
  also requires `file-write`; it is not counted as validation.
- Development branch `feat/i308-save-auto` is stacked locally on Fetch candidate
  `3202c6af0`; the shared HTTP fixture repair there resolves the pre-existing
  unused-read-amount Clippy failures on main. This checkpoint is not publication,
  full preflight, independent security approval, Windows evidence or T8 completion.


### Save URL recovery implementation checkpoint — 2026-10-27

- Local implementation commit `56c5a3ce9` adds optional `retain_backup` and
  `recovery_id` JSON arguments without changing public `SaveUrlInput` fields.
  Recoverable overwrites capture an existing regular file before download and
  reuse the existing bounded recovery store and atomic replacement. Restore
  requires Read+Write, performs no request, and consumes its receipt once.
  Limits remain 2 MiB per image, 64 records/16 MiB and 15 minutes per tool instance.
- Locked Tools `network,file-write` tests passed: 516 library tests plus enabled
  integration/doc tests; 15 Save URL tests include original restoration,
  wrong-instance/path/replay refusal, changed-content refusal, deterministic
  modification during download and over-limit download preserving original data.
  The proxy fixture is bounded by five seconds and joined by each download test.
  Clippy for the same library/tests passed with `-D warnings`; the isolated
  `--no-default-features --features file-write` check passed.
- Independent same-session read-only Agent static permission/security/API review
  approved the local delta, without independently running tests. The shared
  primitive checks content then renames atomically; it is not atomic CAS against
  external writers or retained inode identity. README states this limitation.
- This is local recovery implementation only. Save URL Auto admission, complete
  before/after model evidence and remaining G acceptance are not implemented by
  this commit. Full standard preflight, exact-head Windows/CI, publication and
  merge approval are still required. T8 and the handoff remain Partial.


### Save URL retained replacement checkpoint — 2026-10-27

- Local commit `a3ae723a1` implements a read/network-admitted capture followed by
  a fresh Read/Write-admitted replacement. Stage one retains complete bounded
  original/downloaded images without writing; stage two never downloads again.
  UTF-8 images supply existing private mutation evidence; binary images retain
  the generic approval path. Save URL model Auto eligibility is not enabled yet.
- Exact prepared proofs bind both stages independently. Relative destinations use
  normalized admitted path tokens, not ambient process cwd; stage two must match
  the captured target and existing recovery content checks. The initial local
  implementation incorrectly used process cwd; independent review caught this
  before commit. Added workspace-relative and pre-download link-retarget tests.
- Locked no-default `network,file-write` Tools tests pass: 483 library tests plus
  enabled integration/doc tests; 21 Save URL tests pass. Matching library/test
  Clippy passes with `-D warnings`. Logs: `/private/tmp/i308-save-staged-tools.log`
  and `/private/tmp/i308-save-staged-clippy.log` (local ephemeral evidence).
  The first focused attempt failed because a proof fixture used an invalid URL;
  the next failed because retargeting invalidates proof before capture, earlier
  than its assertion expected. Both corrected fixtures pass in the final run.
- Independent same-session read-only Agent static security/API review approved
  the corrected working executable delta; it did not run tests. Existing final
  content-check/rename race remains disclosed. This is local implementation,
  not full Auto acceptance, full preflight, Windows evidence or merged delivery.
  T8 and the complete handoff queue remain incomplete.


### Save URL staged Auto implementation — 2026-10-27

- Registered built-in recoverable saves now supply stage-owned invocation review
  configuration for the read/download stage. The shared ordinary classifier can
  approve that exact prepared plan once; a new plan separately reviews complete
  UTF-8 replacement images through existing retained file-mutation review.
  Download admission has transitive policy semantics (proxy/redirect effects),
  while the final Read/Write stage inherits no network authority.
- Added default-None descriptive configuration to the prepared invocation trait;
  it carries no authority. Command, mutation and invocation evidence are mutually
  exclusive. Registered instance ownership gates model review. External or
  replacement instances retain manual approval with matching transitive proofs.
  Dedicated prepared-network dispatch remains prior and cannot downgrade.
- `save_url_auto.rs` exercises 16 production Agent/subprocess proxy scenarios:
  allow, uncertain/error/stale at download and mutation, explicit Deny/human,
  external/replacement rejection, manual external/replacement approval, concurrent
  destination modification, nonrecoverable save fallback and binary fallback.
  Assertions cover zero connections before rejected capture, retained originals
  after rejected writes, exactly one redirected download, and zero stored grants.
- Local locked checks pass: Save URL 21/21, Agent library 510/510, Core library
  122/122, Agent library/tests Clippy with `-D warnings`. Logs are ephemeral local
  `/private/tmp/i308-save-auto-{agent,agent-full,core,clippy}.log`.
  Independent read-only security/API review approved the executable working delta,
  including manual external compatibility; it did not independently run tests.
- This is a local implementation stage, not merged delivery. Full standard
  preflight, native Windows/exact-head CI and final stable-commit review remain
  outstanding. The existing content-check/rename race remains disclosed. Recovery
  restore, default/nonrecoverable saves and binary mutations gain no Auto approval
  from this route. T8 full G/search/profile acceptance and all later original
  handoff tasks remain incomplete.

### Rebased full Save candidate review — 2026-10-27

Candidate `bafc87af73cbd7ce9ce34d53fea0987d9148d6a8` is based on Fetch
merge `d6f1e6a56c3d337168b587feb92add36223007c4`. Independent read-only
Agent security/API review approved the complete aggregate delta: bounded download,
instance-bound bounded recovery, exact admitted path capture, separate fresh
write plan, complete UTF-8 mutation evidence, positive builtin ownership and
unregistered manual compatibility. No new dependency/unsafe or required trait
method/public input fields. Reviewer did not run tests. It is a separate Agent,
not a separate natural person. Existing endpoint non-isolation, underlying chunk
allocation and content-check/rename race limits remain explicitly disclosed.

The prior unsandboxed run passed workspace tests (Tools997/997), then SDK
compilation and a concurrent xwin check failed ENOSPC; neither complete gate passed.
After authorized `cargo clean`, the active worktree moved outside target to
`/private/tmp/talos-i308-save-active`. Fresh full standard preflight runs with
debug symbols and incremental caching disabled to limit artifact size and passed,
including all workspace tests and the external Runtime SDK fixture (log:
`/private/tmp/i308-save-rebased-preflight.log`). Serial xwin passed (log:
`/private/tmp/i308-save-rebased-xwin.log`). Save publication/CI/merge and full T8 acceptance
remain outstanding. Historical pre-rebase SHAs above remain valid provenance,
not current-head validation claims.

### Remaining acceptance reconciliation after Save — working checklist

This is an execution checklist against the unchanged numbered acceptance, not a
reduced completion definition. Existing passing suites must be matched to actual
production composition before closing each row.

| Acceptance | Evidence to inspect / remaining concrete work |
|---|---|
| 1, operation/profile inventory | CLI `registry_builtin_tests`, Runtime `composition` equivalence, Desktop contributions/optional feature assemblies and MCP-only status. Record each conditional registration and concrete disposition; registration identity alone does not prove Auto dispatch. |
| 2–4, actual admission | Existing Agent write/edit/read/Todo/scheduler/ordinary HTTP/command fixtures; add missing real registered Git operation and protected/legacy WebSearch disposition evidence. Preserve no-model reads, atomic facets, refusal/cancel/stale/replay checks. |
| 5, higher-risk recovery | Reconcile `edit_auto`, mutation pipeline/recovery tests and Save recovery: special/config/security paths, destructive operation, partial failure, cancellation and stale target. Record unsupported native rename/batch inventory rather than inventing tools or rollback guarantees. |
| 6, host reachability | Inspect CLI/Runtime/Desktop/MCP actual resolver installation and visible decisions, beyond registry equality; match applicable feature/native Windows evidence to these paths. |
| 7, delivery gates | Save preflight/SDK/xwin, stable stage publication, exact-head native CI/review/CAS; then full H/G owner and user-doc reconciliation with implementation SHAs before Complete. |

Current production-entrypoint inspection (2026-10-28): standalone
`run_mcp_server` constructs `McpPermissionGate::new` with a default engine.
That gate uses Headless/Unavailable interaction and no resolver; it labels calls
as `McpRemote { server: "standalone-mcp" }` even for its hosted native registry.
This proves a fail-closed standalone disposition, not model Auto reachability.
Acceptance 6 remains open: reconcile intended hosted-native behavior and add
real gate/handler evidence before claiming MCP coverage. RuntimeBuilder Git
integration and Desktop ordinary-shell Auto tests do not cover this entrypoint.
Likewise CLI `format_auto_review_report` currently formats `prepared_command`
AllowOnce through its generic completion fallback; operation-specific visible
decision/rationale coverage must still be verified under acceptance 6.

### Real Pull Auto ownership follow-up — 2026-10-27

Production dispatch testing exposed a gap not covered by synthetic prepared-command
pipeline fixtures: a real external/replacement GitPullTool could reach model command
review because evidence forwarding omitted positive built-in ownership. Commit `115b302ad`
filters evidence at Agent registry dispatch; external instances retain exact
prepared manual admission, including successful explicit human approval. The
public pipeline API remains a trusted embedder surface, now documented accordingly.

The new separate `git_auto.rs` test exercises ten real local Pull scenarios with
actual HEAD/worktree effects, zero retained grants, assessor assertion-reached
checks and explicit discovery-only human fallback. Model allow executes; model
human/error/stale, explicit Deny/mandatory-human and unregistered/replacement
instances cannot silently execute. Explicit manual external/replacement approval
still succeeds. Focused tests and full Agent library 511/511 pass; independent
static security/API review approved the working delta without running tests.
Agent library/test Clippy passed. Full standard preflight failed three Runtime
Pull integration tests because caller `.tool(...)` registration no longer inherits
builtin Auto trust. The fixtures now select production `.shared_tools()` registration;
the focused Windows CI command explicitly enables `shared-composition` so this
feature-gated suite cannot silently disappear there. Focused rerun passed 3/3; both
governance validators passed. Fresh full preflight and Agent/Runtime Windows test
cross-check passed; independent static review approved exact `e14c76164`. Native
CI, publication and merge remain pending. This narrows a real registration trust gap; it does not add model
Auto support to Add/Commit/Push/Checkout or complete profile/search/G acceptance.


### Hosted-native MCP retained mutation stage — 2026-10-28

The local candidate forwards complete-image prepared edit/delete evidence only for
registry-verified builtin instances, with consistent Native preparation/admission
provenance. Ambiguous command/mutation/configuration evidence fails closed.
The additive trusted-embedder `authorize_prepared_file_mutation` API requires the
caller to establish builtin ownership; it does not infer trust from names or payloads.
Existing plan-binding regression now exercises this public API for allow, deny and
mismatched-plan cases; the untrusted internal path remains explicitly tested.

Real duplex rmcp tests exercise retained edit and delete with AllowOnce, model refusal,
same-name external replacement and a concurrent target update. Only AllowOnce changes
the expected target; concurrent contents are preserved. External assertion-completion
counters prevent contained assessor panics from masquerading as expected refusal.
The final focused test and Agent API regression pass. Full MCP all-features tests
passed before the assertion-counter-only test refinement; warnings-denied MCP Clippy
passed before the Agent regression/test refinement. Logs:
`/private/tmp/i308-mcp-all-after-mutation.log`,
`/private/tmp/i308-mcp-mutation-verified.log`,
`/private/tmp/i308-public-mutation-api-test.log`.

This is local implementation, not merged delivery or complete recovery evidence.
Save capture/configuration, prepared network and ordinary invocation Auto dispatch
through MCP remain outstanding, as do full acceptance, coherent final preflight,
independent exact-head security/API review, Windows CI and merge CAS. PR #797 still
publishes the older 1993432dc candidate with failing Windows Git tests; local host
success does not certify native Windows behavior. I308/T8 and the full goal remain Partial.


### Hosted-native MCP Save configuration stage — 2026-10-28

Local implementation adds a trusted-embedder prepared builtin configuration API; MCP
forwards configuration from the exact operation only after registry builtin ownership
verification. Ambiguous evidence still refuses. Save capture and complete-image write
remain distinct plan-bound admissions. The API does not authenticate arbitrary callers.

Real rmcp Save tests cover allow, capture refusal, write refusal, external replacement
and missing current-turn intent. Capture refusal performs no download; write refusal
follows one authorized download and preserves the original destination. Assessor
completion counters and parent-observed proxy connections supplement file assertions.
Locked MCP all-features tests and Agent/MCP all-features warnings-denied Clippy pass
(`/private/tmp/i308-mcp-save-full.log`, `/private/tmp/i308-save-config-clippy.log`).
Independent bounded shared-identity Agent static review approved the dirty code snapshot,
without running tests; immutable-head confirmation remains required.

This supersedes the preceding local Save configuration gap, not the remaining prepared
network/ordinary MCP routes. Recovery execution through MCP, recovery-storage inventory
on refusal, concurrent Save changes, native Windows, coherent standard preflight,
publication/exact-head CI/review/CAS/merge and full acceptance remain outstanding.
I308/T8 remains Partial; no completion evidence is claimed.

Save stale-target follow-up: the real MCP test now mutates the destination during
complete-image assessment before returning AllowOnce. Execution refuses and preserves
`concurrent update`, with two completed assessments and exactly one prior authorized
download. The six-scenario focused test passes
(`/private/tmp/i308-mcp-save-concurrent.log`). This supersedes the concurrent Save
regression gap above; recovery execution/storage and platform/remote gates remain open.

Stable Save/MCP candidate validation: standard `./scripts/release_preflight.sh`
with locked commands passed on the host, including both external Runtime SDK modes
(`/private/tmp/i308-save-config-preflight-host.log`). The initial sandbox run
(`/private/tmp/i308-save-config-preflight.log`) failed one Tools Push process-observation
test with EPERM (1009 passed / 1 failed); the unchanged host retry passed.
The Save concurrent regression and focused warnings-denied Clippy also pass. Both
governance validators pass with zero warnings. Fresh fetch retains target main
`4bf3488b2f4344b054754851b588686126c436f4`. This is local stage validation,
not new-head native Windows CI, merge authorization or whole-I308 completion.

MCP Save recovery-gate regression (local test-only follow-up): the real rmcp scenario
now extracts the receipt from a successful retained replacement. Recovery is denied by
the permission resolver by default with downloaded bytes unchanged; after an explicit
test-controlled `ApproveOnce` response for the `save_url` recovery request, original
bytes are restored, and reusing the one-use receipt fails without changing them. The
Auto assessor call count does not increase and the parent observes only the one download.
Locked focused test and warnings-denied test Clippy pass
(`/private/tmp/i308-mcp-save-restore.log`, `/private/tmp/i308-save-restore-clippy.log`).
This is a simulated human fallback, not UI evidence or restore-specific production policy;
cross-session receipt ownership/privacy/storage cleanup remain unverified. Independent
static test review approved immutable `55cc0e3cead7e6d9f21038d524a122d327747fb1`.
Exact-head CI observed during this work is for preceding `c32a49088`; it cannot validate
this commit. Updated candidate CI, full I308 gates and all later queue tasks remain open.

Save explicit-policy disposition follow-up: the real MCP fixture adds Headless, explicit
Network Deny and explicit Write Deny. All three refuse before model assessment or download
and preserve the destination. In particular, an explicit Write Deny on Save's declared
transitive effects rejects the whole operation at capture admission; it does not permit an
earlier network phase and rely on a later rollback. Nine locked MCP scenarios pass
(`/private/tmp/i308-mcp-save-policy.log`); focused warnings-denied test Clippy is recorded
separately after completion. This is host test evidence only, not native Windows CI or
full profile inventory.

MCP Git host-coverage follow-up: a real rmcp test now invokes production registered
`git_add`, `git_commit`, `git_checkout` and `git_push` contributions through the injected
permission pipeline. For each operation the Auto assessor checks the exact tool,
versioned command operation and current user intent, returns AllowOnce, and the test
verifies the actual Git index, commit subject, branch, or bare-receiver ref. The test
resolver permits configuration discovery but denies the operation-specific command
fallback, so the final effects require model admission. Focused test passes; all 49
`git_prepared` integration tests and warnings-denied test Clippy pass
(`/private/tmp/i308-mcp-git-four-ops.log`,
`/private/tmp/i308-mcp-git-prepared-all.log`,
`/private/tmp/i308-mcp-git-four-ops-clippy.log`). Independent bounded static review
approved the test. This supplies standalone MCP success-path evidence for four Git
mutations; it does not finish negative-policy/replacement matrices across every host,
native Windows, profile/feature reconciliation or the complete H/G acceptance.

Full host preflight for this follow-up is terminal exit 101
(`/private/tmp/i308-mcp-git-four-ops-preflight-host-final.log`): Tools 1009/1010
passed; `refused_connection_consumes_budget_before_the_next_source_can_resolve`
returned first-source Request rather than next-source RequestLimit. An all-feature
focused invocation executes that test and passes (1/1;
`/private/tmp/i308-search-refusal-focused-real.log`). An earlier invocation with
`--exact` and an incomplete name ran zero tests and provides no validation evidence.
The cause remains unproven; full preflight is not certified by the focused pass.
Earlier host TUI failure and sandbox Push EPERM remain separate observations.

Follow-up validation clarification: after the host preflight's parallel Tools run
reported two connection-refusal budget failures, the unchanged original connection
fixture module passed 4/4 in isolation, and a serial all-feature Tools run passed
1010/1010 (`/private/tmp/i308-tools-serial.log`, `--test-threads=1`, exit 0).
A temporary dynamic-port/held-socket experiment was reverted and is not evidence.
This establishes serial local stability only; parallel preflight reliability and the
underlying scheduling cause remain unknown. No production search code was changed.

Ordinary HTTP policy regression follow-up (2026-10-28): the production Agent
proxy/redirect fixture now checks `http_request` and `fetch_url` with a broad
deterministic Allow followed by an explicit domain-scoped Deny or Ask for an
undisclosed destination. All four cases terminate without assessor calls, accepted
or queued proxy connections, or installed grants. This verifies the existing
transitive-policy guard in the presence of a later broad Allow. The earlier default
Network Ask shadows that Allow, so this fixture alone does not prove the
deterministic-Allow ordering. No production code changed.
The focused HTTP matrix and all 513 locked all-feature Agent library tests pass;
all-target all-feature warnings-denied Agent Clippy passes. The unchanged parallel
Tools library suite also passes 1010/1010 on this attempt; earlier preflight failure
causes remain unknown, so this is not full preflight certification. MCP Git tests
pass 49/49, MCP Clippy and both governance validators pass. The preceding candidate
`3fffc458f212350ac0a70ba0d3671772fd377efa` was published to #797 and its own CI
run `40705827501` is live; none of that CI certifies this later test follow-up.
Ordinary MCP invocation routing, remaining profile/recovery/platform acceptance,
final review/merge gates and the downstream queue remain incomplete.

### Hosted-native MCP ordinary HTTP stage — 2026-10-28

The local handler now admits registered implementation-owned `http_request` and
`fetch_url` through the shared whole-invocation review, retaining exact input,
configuration and a one-use plan identity. The request-ID ledger reserves calls
before review; permission hooks complete before freshness-checked consumption.
Protected prepared-network tools and untrusted replacements do not enter this route.
Cancellation and the total invocation deadline drop the execution future and report
that effects may be incomplete; they do not promise remote rollback.

The real rmcp/subprocess proxy matrix passes for both tools: Auto success, actual
deterministic Allow, a later Allow shadowed by default Ask, incompatible scoped
Deny/Ask, uncertain/error/stale assessment, explicit human/Deny, Headless, absent
intent and caller replacement. Proxy assertions verify method, URL and POST body;
refusals produce no accepted or queued connection. Cancellation during assessment
then same-ID replay rejects without a second assessment or installed grant.
Command: `cargo test -p talos-mcp --all-features --locked --test native_http_auto`.
The full all-feature MCP suite and warnings-denied all-target MCP Clippy passed
before the final cancellation and deterministic-Allow test additions. The final
rerun failed linking other test targets with ENOSPC; subsequent Clippy and format
commands in that chain did not run. Authorized cleanup removed only this active
worktree target. The low-debug, incremental-disabled locked full MCP rebuild
passed (`/private/tmp/i308-mcp-http-rebuild.log`), as did all-target warnings-denied
MCP Clippy (`/private/tmp/i308-mcp-http-clippy.log`) and the strengthened final
scoped-policy matrix (`/private/tmp/i308-mcp-http-final-matrix.log`).
The subsequent transport-cancellation regression passes for both tools: the proxy
receives the complete request and withholds its response, the client cancels,
the proxy observes EOF, and same-ID replay is refused without another assessment
or grant (`/private/tmp/i308-mcp-http-transport-cancel.log`). This proves local
transport closure after cancellation, not remote rollback. Formatting passed separately.

Independent bounded static permission/security/API review found no blocker and
approved immutable commit `64de053969e36f7d96d65e2ec2913ed268374ca0` against
base `4bf3488b2f4344b054754851b588686126c436f4`; the review was static and did
not run builds. PR #801 targets `main` base `97cdc9bce2f372f57327ab4187ba6c84fbd263c4`.
Exact-head CI run `40795244372` has passed classification, Linux Desktop,
installer and owner reconciliation; workspace, Windows workspace and Windows
Desktop jobs remain pending. Remaining host/profile/recovery/platform
acceptance, full preflight and final merge
are not proven by this matrix. I308/T8 remains Active / Partial; the downstream
MODEL-014 → T4 → T5 → T1 → T6 queue remains incomplete.

## Retained mutation policy and Headless profile regression — 2026-10-29

The real MCP edit/delete Auto matrix now also covers an explicit Write Deny and
an injected Headless pipeline with an Auto resolver. For both retained edit and
delete, explicit Write Deny rejects before assessor invocation and preserves the
source file. In Headless mode the injected resolver can return AllowOnce; the
actual edit/delete occurs, while no persistent grant is installed. The test keeps
the default standalone MCP composition's separate fail-closed behavior covered
by `server_handshake`.

Locked focused MCP integration tests pass for retained edit/delete and Save
(`cargo test -p talos-mcp --all-features --locked --test native_mutation_auto
--test native_save_auto`, low-debug/nonincremental settings). This is local host
test evidence only; warnings-denied Clippy, governance validation, independent
review and fresh exact-head CI remain required. It does not close MCP ordinary
Shell/Exec Auto routing, WebSearch, recovery/profile/platform coverage, or I308.

## MCP Git operation-specific Execute Deny regression — 2026-10-29

The registered MCP Add/Commit/Checkout/Push Auto matrix now runs both success
and explicit operation-scoped Execute Deny for each operation. Deny returns a
permission-denied MCP error before any assessor call; HEAD, staged binary diff
and current branch remain unchanged, and Push creates no receiver main ref.
Both outcomes install zero persistent grants. These are independent operation
fixtures, not renamed Pull evidence. All 49 `git_prepared` integration tests pass
with `cargo test -p talos-mcp --all-features --locked --test git_prepared`
(low-debug/nonincremental build), and all-target/all-feature MCP Clippy passes
with `--locked -- -D warnings`. The preceding retained mutation checkpoint's
Clippy and both governance validators also passed before publication.

This extends local policy evidence only; fresh candidate review/CI, merge and
remaining I308 host/profile/recovery/platform acceptance are still outstanding.
The same four operation fixtures now inject an assessor transport/error result.
Each returns a permission-denied MCP error after exactly one assessor call,
without changing HEAD, the staged diff, current branch or Push receiver ref; no
persistent grant is installed. All 49 Git prepared integration tests passed
again after adding this failure branch.

The real hosted-native MCP retained edit/delete matrix now also injects a file
mutation assessor error for both operations. Review receives the complete image,
then fails; the operation refuses without modifying or deleting the original
file and leaves no persistent grant. Locked focused retained mutation and Save
tests pass (2/2 integration tests), as does all-target/all-feature MCP Clippy
with warnings denied and format/diff checks. This adds production-pipeline
assessor-error evidence only. It does not close special/config/security-path,
partial-failure, cancellation, or stale-identity recovery acceptance, nor the
broader I308 recovery and platform rows.

The hosted-native MCP retained mutation integration now also exercises the
recovery receipt through the same registered edit/delete tool instance. After
an Auto edit or delete, a separately resolved one-shot recovery approval restores
the original file; replaying the consumed receipt is rejected without another
Auto assessor call. The focused locked MCP mutation suite passes. This proves
one-use instance-local recovery in this composition, not persistence across
restart, cross-session ownership, special/config/security-path behavior,
partial-failure recovery, or cancellation/stale-identity coverage.

## Sensitive-path retained mutation Auto boundary — 2026-10-29

Retained file mutation review now applies the existing sensitive-target predicate
to the prepared target path before exposing complete images to the assessor.
Real registered MCP edit/delete fixtures with benign contents at `.env` and
`.aws/credentials` refuse via the denying human fallback with zero model calls,
unchanged originals and no persistent grants. This reuses the conservative
existing path policy; it is not an exhaustive credential classifier or a blanket
ban on all configuration changes.

Regression sensitivity was checked by temporarily removing the path guard:
`native_mutation_auto` failed with one assessor invocation instead of zero for
the `.env` scenario (`/private/tmp/i308-sensitive-path-red2.log`); after the
fixture was corrected to accept the scenario-specific intent, the failure was
specifically the unexpected model-call count. Restoring the guard passes the
locked all-feature MCP retained mutation and Save integration tests (2/2,
`/private/tmp/i308-sensitive-path-green2.log`), using nonincremental/low-debug
builds. This is local integration evidence; fresh protected-scope independent
review, complete candidate validation, exact-head CI and merge remain
outstanding, as do the broader I308 acceptance rows.

## MCP recovery stale-target integration — 2026-10-29

The real retained edit/delete Auto matrix now changes the destination after the
initial successful mutation and before a separately human-approved recovery.
Both the first recovery attempt and its repeat refuse to overwrite the new user
content; the assessor remains at one invocation and no persistent grant exists.
The locked all-feature `native_mutation_auto` suite passes
(`/private/tmp/i308-recovery-stale.log`); the latest full locked standard
`release_preflight.sh` also passes at the implementation candidate containing
this test (`/private/tmp/i308-sensitive-recovery-preflight-host.log`), including
workspace tests, doctests and the external Runtime SDK fixture. Sandbox execution
of that preflight had one process-observation EPERM in a Push test; the authorized
host run passed it. The governance validators pass with zero warnings. This proves
stale-content protection through this MCP composition, not atomic protection
against every concurrent rename race or restart-persistent recovery. I308 remains
Partial. This local candidate is not covered by PR #801's older head, independent
review, exact-head CI or merge.

## MCP retained mutation cancellation and replay — 2026-10-29

The real MCP retained edit/delete Auto fixture now sends JSON-RPC cancellation
after the assessor receives the complete mutation request. The server remains
responsive, the same request ID is reserved against replay, the original file is
unchanged, the assessor runs once and no grant is created. The locked focused
`native_mutation_auto` integration test and all-target/all-feature MCP Clippy
with warnings denied pass. This covers cancellation during assessment before
mutation; it does not establish rollback after filesystem effects begin. The
change is local and still needs review, exact-head CI and merge.

## MCP recovery refusal preserves prior effect — 2026-10-29

The real retained edit/delete fixture also denies the separate recovery request
after the initial Auto mutation. A denied edit restore leaves the edited bytes;
a denied delete restore leaves the target absent. The recovery assessor is not
called again and no grant is installed, so recovery requires its own permission
decision and denial does not execute the receipt. Focused locked MCP integration
test (`cargo test -p talos-mcp --all-features --locked --test
native_mutation_auto`) passes on this local follow-up candidate. This is explicit
Write Deny after the initial operation, not a human resolver denial; the MCP
transport correctly returns a permission error before tool execution.

## MCP ordinary command invocation admission — 2026-10-29

The MCP ordinary-invocation route now admits only registered native
`bash`/`powershell`/`exec` alongside ordinary HTTP/fetch. The shared pipeline
receives the tool-owned starting directory, exact normalized arguments, frozen
implementation configuration, resolved initial execution directories and their
resolution state. A one-use prepared admission is consumed after hooks and
freshness checks; execution is refused if configuration, directory list or
starting directory changes. The new MCP integration fixture executes real Git
config mutation through registered Exec and Unix Bash only after model
AllowOnce, and explicit Execute Deny rejects before assessor invocation or
mutation. Its locked test passes locally. Windows PowerShell runtime coverage,
assessor uncertainty/error/cancellation across this new MCP route, exact-head
CI and independent permission/security/API review remain outstanding. Default
standalone MCP remains Headless and fail-closed without an injected pipeline and
current-turn intent. Full I308 remains Partial.

The exact-head CI run `40820098820` completed with Linux preflight failure in
`cancelled_push_closes_transport_and_confirms_cleanup`: the fixture's Git commit
left background auto-maintenance rewriting the object layout between preparation
and Push discovery. The full local host preflight passed and the same test passed
there. The fixture commit now disables Git auto-detach so maintenance completes
before snapshot capture; this local corrective test change requires fresh full
validation, exact-head review/CI and merge evidence.

## Post-failure local validation and MCP ordinary command route — 2026-10-29

The fixture correction is committed as `bf057a41e` and the MCP registered
ordinary command route as `15be68e49`. Standard local validation completed with
exit 0 using the repository `./scripts/release_preflight.sh` entrypoint and
`CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0
RUST_TEST_THREADS=1`; this includes workspace check, warnings-denied Clippy,
workspace tests/doctests and both external Runtime SDK modes. Tools passed
1010/1010, including `cancelled_push_closes_transport_and_confirms_cleanup`
and the eight-stage local Push test. Governance and collaboration validators
reported zero warnings. An earlier sandbox attempt was incomplete: process
observation returned EPERM at the local Push test, and the next full run ran out
of disk while building the SDK coding-mode archive. The approved-host run above
is the successful evidence; the two failed attempts are not counted as passes.

The MCP command route admits only registered `bash`/`powershell`/`exec` when
they use ordinary invocation review. It binds exact arguments, the tool-owned
starting directory, configuration and the resolved initial execution-directory
set, then rechecks those values before consuming the one-use admission and
executing the retained instance. A real MCP integration fixture verifies
registered Exec and platform shell mutation after AllowOnce and explicit Execute
Deny before assessor invocation or mutation. This does not establish native
Windows PowerShell behavior, assessor uncertainty/error/cancellation cases on
this route, the full I308 matrix, or independent final permission/security/API
review. PR #801 still points at the prior head; fresh exact-head CI, authorized
review, latest-base rebase/CAS and merge remain required. I308 remains Partial.

## MCP ordinary invocation refusal and cancellation coverage — 2026-10-29

The real registered MCP Exec/Shell integration now exercises model uncertainty
and assessor error in addition to AllowOnce and explicit Execute Deny. All
non-Allow outcomes return an MCP permission error before Git configuration is
written and leave the session grant count at zero. A raw JSON-RPC cancellation
case waits until the ordinary-invocation assessor starts, sends
`notifications/cancelled`, verifies the server still answers `ping`, then
replays the same request ID and confirms it is rejected. The cancelled command
does not write Git configuration or retain a grant; the assessor is invoked
once. Locked `cargo test -p talos-mcp --test native_exec_auto` passes 2/2 and
`cargo fmt --all` passes. A full locked preflight has not been run on this
candidate. Native Windows PowerShell execution, changed-input/configuration
staleness on this MCP route, exact-head CI, independent permission/security/API
review and merge remain outstanding; this does not close the full operation /
profile matrix or I308 acceptance.
