# WORKTREE-001: Session-owned worktree workspace admission

> Document status: Complete / Closed (CLI/TUI scope; sandbox follow-up remains Planned)
> Deliverable: a worktree verifiably created by the current Session becomes an additional workspace root for that Session without a second workspace-admission prompt.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Verified Session-owned worktree creation receipts, additional workspace capabilities, resume revalidation, tool/permission integration and adversarial tests |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | #889 |
| Authorization Mode | Independent review |
| Authorization Evidence | User requests automatic workspace admission for worktrees created by the Session; independent security design review identifies receipt and capability prerequisites |
| Implementation PR | #890 |
| Last Updated | 2026-11-09 |
| Handoff / Release Condition | Reviewed ADR and effective implementation claim; full preflight, cargo-xwin and native Windows identity checks; independent security review |

## Problem and evidence

The user's terminal shows successful creation of a sibling worktree followed by
`write` failing with `path escapes workspace root`, even with an always-approved
write rule. Current PermissionEngine, ManagedWorkspaceLease and file tools bind a
single initial root. No Session-owned worktree registration connects creation to
these boundaries. A broad tool Allow is not workspace admission.

Required Reads: ADR-064, ADR-089, docs/sop/AGENT-COLLABORATION.md,
docs/sop/LOCAL-DEV.md, docs/sop/TESTING.md; talos-permission lib.rs/session.rs;
talos-agent auto_resolver.rs/tool_execution.rs; talos-tools file_tools.rs,
file_tools/atomic_create.rs and bash_tool.rs; CLI/runtime Session composition and
resume. Affected user documentation: README workspace/permission behavior.

## Contract

- After authorized creation actually succeeds, admit only the canonical worktree
  root to the originating Session. The creating operation remains permission-gated.
- Registration is workspace membership, not blanket Allow: explicit Deny, sensitive
  paths, ordinary Ask, capability restrictions and sandbox boundaries remain active.
- A native execution receipt binds logical Session, invocation, source repository,
  target directory identity and verified bidirectional Git worktree metadata.
  Model output, transcript text, tool stdout and hook-rewritten results are not receipts.
- Shell-created worktrees are included in the requested behavior. Define and test
  deterministic supported invocation syntax; do not infer authority from substring
  matching or a before/after worktree-list scan. Ambiguous shell effects need a
  structured creation path or separate exact-root admission, not silent broad trust.
- Multiple roots must flow consistently through permission evaluation, automatic
  assessment leases, exact path authorizations, file-tool capabilities and sandbox
  construction. Adding or revoking a root advances workspace revision and invalidates
  prior assessments. Do not grant the shared parent directory.
- Persist creation evidence scoped to the logical Session. On `-c`, revalidate
  repository and directory identities before rebinding the new permission Session.
  Unrelated Sessions and pre-existing worktrees do not inherit membership.
- Revalidate before sensitive use. Removal, relocation, identity replacement,
  symlink/reparse redirection or invalid Git linkage revokes membership. A replacement
  directory at the same spelling must never inherit authorization.

## Acceptance and validation

Real CLI regression: authorized shell worktree creation followed immediately by
native read/write/edit/search succeeds within the registered root under the existing
policy, without changing workspace or granting a second directory approval.
Then resume the same logical Session and repeat after identity validation.

Negative tests: failed/partial creation; pre-existing target; concurrent external
creation; forged stdout or hook result; another repository or Session; explicit Deny;
credential/VCS-internal protection; symlink/reparse escape; directory deletion and
recreation; invalid metadata; stale workspace revision; ambiguous shell invocation.
Root capabilities and path identity require native Windows execution as well as
`cargo xwin ... --locked --target x86_64-pc-windows-msvc --tests` checks.
Run pinned `./scripts/release_preflight.sh`, exact-head CI and independent security
review before merge. No unsafe or new native dependency without an approved ADR.

## Design review and outstanding work

Independent Agent security design review found that changing only PermissionEngine
or scanning Git worktrees is insufficient. There are three enforcement layers and
shell success alone does not prove creation ownership. The reviewed recommendation
is trusted creation receipts plus per-root capabilities and resume revalidation.
This is design advice, not approval of implementation or a completed repair.

The effective claim includes a design checkpoint before permission expansion.
Settle bounded shell syntax, atomic creation/identity custody on Unix and Windows,
persistence schema and public API compatibility in ADR-099 and obtain independent
security review of that checkpoint before implementing automatic membership.
Approval/execution regression repair remains the first implementation checkpoint.
Prefer an existing audited host process boundary; document any host-git dependency,
unavailable-host behavior and replacement trigger. Do not implement broad directory
allowlisting as an interim fix. This owner is the residual destination.

## Scope inventory

I310 Active; I318/I302/I277 Review; I249 Planned/deferred; I164 Paused retained.
I162 Complete with historical Review outcome is not reopened. No Blocked owner
selected. Open #838/#849/#861 retain their existing scopes. BG-CONTINUE-001 owns
background completion continuation separately; its files and worktree are untouched.
No permission expansion or production code has been delivered by this intake record.

## Execution plan and business failure regression

User explicitly requested implementation and reported that external-workspace writes
have no human approval opportunity. Treat this as a workflow-blocking defect.
First reproduce the real Agent → auto resolver → human resolver → file tool chain.
An external path without an explicit Deny must reach manual approval when automatic
review cannot authorize it. Approval must issue exact path authorization and execute
successfully; denial must leave the target absent. The write tool currently rejects
absolute paths again when its optional atomic-create capability is installed, even
after successful exact-path authorization. Cover this defect with production-style
capability wiring, not only bare WriteTool tests. Do not equate this repair with the
separate automatic worktree admission requirement.

Implementation checkpoints: (1) approved external write and manual fallback;
(2) trusted worktree creation custody and multi-root permission/lease integration;
(3) durable logical-Session binding and resume/revocation; (4) adversarial/native
validation, exact-head security review and merged evidence. Owner remains Active
until all accepted behavior is implemented; partial milestones do not close it.

Decision: [ADR-099](../decisions/099-session-owned-worktree-admission.md).

## User-confirmed policy correction

The user explicitly clarified the rule: ordinary workspace files are allowed;
non-workspace files require human authorization. Session-created worktrees become
workspace roots. Do not keep the old default Ask for ordinary local file operations
or use automatic-model review as a barrier to the external-path human prompt.
Explicit Deny and sensitive-path restrictions remain authoritative. Limit the new
default to native ordinary file operations, not Execute/Network/Git administration
or plugin identity inferred from a tool name. Add a policy matrix regression for
local ordinary files, external files (approve/deny), registered worktrees and secrets.

The user's next clarification explicitly requires compatibility with automatic review.
External paths require authorization, not necessarily unconditional human approval:
retain existing deterministic automatic-review eligibility and exact one-shot grants.
When automatic review cannot approve, is ineligible or fails, fall back to human
approval. Do not broaden automatic eligibility merely because a path is external.
Both approval routes feed the same exact execution authorization. This clarification
supersedes any wording above that implies skipping automatic review for external paths.


## Implementation checkpoint — 2026-11-08

Approval execution repair, product ordinary-file defaults, retained root IO, durable
receipt validation and approval-generation revocation are implemented locally; no
implementation commit or delivery approval exists yet. Focused tests have verified
external approval through the real Agent auto-fallback chain and worktree creation,
resume, identity replacement and fresh external reapproval. Expanded read/edit/search
and UI tests are still converging. One full workspace test run passed, but its final
external SDK fixture failed because its independent lockfile needed the new capability
dependency; both fixture lockfiles are now synchronized and require revalidation.

Independent review found and corrected stale Once/Session grant reuse, missing
metadata identities, command semantic changes and optional-capability gaps. Atomic
creation custody now uses protected staging plus no-replace publication (ADR-099).
The Windows OS ABI extension has design-checkpoint approval only; native Windows
results and exact-head implementation approval are outstanding.

Remaining gates: stable-source full preflight, cargo-xwin MSVC tests, actual CLI/TUI
approval regression, native Windows contention/reparse tests, independent security
review and target-branch merge/source sync. Sandboxed execution does not register
host roots; SDK/print defaults and unsupported command forms are documented, not
represented as delivered automatic admission. Keep the owner Active; no Complete claim.


## Accepted scope adjustment — 2026-11-08

User explicitly selected: “先交付 CLI/TUI 修复，独立 OS 沙箱登记另立任务”.
WORKTREE-001 delivers the interactive CLI/TUI host execution path, inline Session
restoration, exact external approval execution, and their existing automatic-review
compatibility. Independent OS-sandbox root propagation and sandbox-owned creation
receipts are excluded from this delivery and owned by
[WORKTREE-SANDBOX-001](2026-11-08-sandbox-owned-worktrees.md). This overrides the
original combined sandbox-construction acceptance above without deleting its baseline.
SDK embedders opt in; ephemeral print mode has no persistent Session membership.
Other tools that have not adopted held worktree capabilities retain external approval;
original workspace trust is never inherited as blanket additional-root write authority.


## Stable candidate validation — implementation b8b7f760

Implementation Commit: b8b7f760304a29de9e17cfa5f16d99507c882e81

- Pinned Rust 1.99.0 `./scripts/release_preflight.sh`: passed, including both
  governance validators, workspace check/Clippy/tests and both external SDK modes.
- `cargo xwin check --locked -p talos-tools -p talos-agent -p talos-cli --all-features
  --tests --target x86_64-pc-windows-msvc`: passed. Existing test-only dead-code
  warning in checkpoint_history remains unrelated; no new cross-check warning.
- Real Agent authorized worktree creation → next native write: passed with exactly
  one creation approval. TUI external Once/Always/Deny execution: passed.
- Root read/write/edit/grep/glob, no-primary-capability write, resume, policy reload,
  protected receipts, invalid-root old Once/Session rejection and fresh external
  reapproval: passed. No-clobber/competing target/Unix symlink tests: passed.
- Independent Agent static security review APPROVE for implementation head above,
  base d29fbebe40f7077b5c0a254bc6748746918e0f99. Reviewer is separate from the
  implementation agent but shares the maintainer's GitHub identity; not a separate
  human approval. Native Windows identity/reparse results and exact candidate CI
  remain merge gates. No Complete status or completion evidence is claimed yet.

## First remote candidate findings — PR #890

Exact candidate cc3a63ac failed native CI; it is not merge evidence. Missing-target
workspace comparison mishandled macOS path aliases and Windows canonical prefixes.
The repair resolves existing ancestors with OS semantics and explicitly rejects
symlink/parent traversal escapes and dangling links. Native Windows also exposed
cap-std's deliberate directory rename lock during publication; ADR-099 records the
identity-only staging handle repair and post-publication capability transition.
macOS additionally reported two pre-existing long-history recovery test failures
(committed-source deadline and checkpoint outer timeout); these remain pending
validation rather than waived. All exact-head checks must pass on the next candidate.

Second native candidate 7ad6f4cf passed macOS full preflight (including the two
previous history failures) and Windows ordinary workspace policy tests. Windows
creation progressed past publication but Git rejected the canonical `\\?\` destination
spelling. The final follow-up gives Git a conventional argument only after unchanged
Win32 normalization and directory identity comparison, rejecting reserved/verbatim-only
names. Permission and receipt paths remain canonical. Native success is still required.

## Completion evidence

Completion Commit: 2fa9b18894f523e9c3de5e3f2d6166eea2229151

Implementation PR #890 merged after exact-head independent Agent security APPROVE
for bb75b0b26ff33acd80109fa09929b3c3f65c2ba9 against d29fbebe40f7077b5c0a254bc6748746918e0f99
and all checks passed in [CI run 42494047202](https://github.com/wjhuang88/talos/actions/runs/42494047202).
Local locked full preflight and MSVC cargo-xwin test cross-check passed. Native
Windows verified actual Agent creation-to-write, ordinary/external permission flows,
resume/revocation, stale grants, fresh approval, no-clobber and junction protection.
The final Windows run initially failed an unchanged history lease preparation test;
the same-head failed-job rerun passed the full workspace and CLI smoke. No test was
skipped or deadline relaxed. macOS full preflight and CLI/TUI Auto acceptance passed.
Review is by a separate Agent sharing the maintainer GitHub identity, not a separate
human approval. Merge-time claim, base, overlap and exact-head checks were unchanged.

Accepted CLI/TUI scope is complete. WORKTREE-SANDBOX-001 remains Planned/Unclaimed.
Nonblocking limitation: if Git path conversion or execution fails after directory
publication, an empty owned destination may remain; it is not registered and cannot
grant workspace authority.
