# VALIDATION-003: Model Validation And Read-Only Agile Tools

| Field | Value |
|---|---|
| Story ID | VALIDATION-003 |
| Status | In Progress (#845 merged; I315 implementation and validation ongoing) |
| Priority | P1 |
| Source Issue | #844 |
| Owner | I315 |
| Depends On | I313/T5 complete; VALIDATION-001 shared service; GOV-003 supported summary |

This bounded child owns handoff T1, not GOV-003 automation or T6 memory limits.
[Iteration and claim](../../iterations/I315-model-validation-agile-tools.md) owns
activation. No implementation before finalized target-branch claim merge.

## Deliverable

The model discovers and calls the configured workspace's read-only agile summary
and fixed validation profiles through shared business services. Actual host
execution requires normal permission admission and bounded owned execution.
T1 follows T5 disposition; T6 remains the next independent queue item.

## Applicability acceptance

- For agile_summary, a governance manifest independently enables discovery. Otherwise any two of
  docs/BOARD.md, docs/iterations/README.md and docs/backlog/PRODUCT-BACKLOG.md do.
  Test all pairs, each single marker, no markers, and manifest alone.
- Do not apply agile_summary's governance-only eligibility to
  validate_project. VALIDATION-001 explicitly supports language-neutral project
  detection and language-manifest-driven adapters. Validation discovery should use
  the existing detector registry and expose supported plans in a recognized project
  even without governance documents. A Rust-only Cargo.toml fixture must discover
  validation but not agile_summary; a governance-only fixture must allow internal
  governance validation without requiring Cargo. Recognizing Node/Python/Go/Java
  does not authorize inventing host profiles that the shared service lacks: return
  explicit unsupported/prerequisite evidence. Empty unrelated directories do not
  enable either tool merely because the conversation contains the word project.
- Small and newly Planned governed projects qualify without an Active iteration.
- Explicit progress/backlog/governance requests select the read-only summary.
  Explicit validation or a required check of an accepted task selects validation.
  Ordinary conversation and isolated code explanations do not justify invocation.
- Tool descriptions and user documentation include positive and negative examples.
  Description assertions are not proof of deterministic model intent routing.
- Missing sources/prerequisites produce truthful unavailable evidence. Do not
  create placeholder governance files to enable discovery.
- Discovery never grants execution authority. Plan has zero host spawns; run
  passes Execute admission. Denial is tested through the real permission pipeline
  with a marker proving zero execution, not only a permission-profile unit test.
- Bind to the host-selected workspace; reject arbitrary commands, workspace
  overrides, unknown fields/types/modes/profiles before side effects.
- No task/owner/Board/Issue writes, iteration activation, publication or release
  operation is exposed. Built-in validators may create normal build artifacts;
  document these run-mode side effects rather than calling execution read-only.

## Shared service and host coverage

Reuse conversation governance_summary and validation plan/evidence semantics;
inspect all existing CLI/TUI callers before changing the shared execution API.
Verify CLI print/inline/RPC/TUI and Runtime/Desktop contribution paths explicitly;
record unsupported hosts instead of implying universal registration from a CLI test.
Existing supported profiles are governance, i076 and workspace; schema must not
advertise arbitrary agile subcommands or unsupported validation operations.

Current-source finding: the preserved candidate defines the tools in CLI registry
only. Runtime composition currently has no governance/validation contribution and
does not depend on conversation. Do not close host coverage from the CLI inventory
test. Choose an additive shared contribution factory and execution adapter after
checking dependency direction; adding a required public SharedToolContributions
field would affect downstream struct literals and needs compatibility review.
OwnedCommandExecutor provides operation(), and execute() is on the returned
OwnedCommandOperation. Retain operation-local cleanup receipts so cancellation of
one validation does not wait for unrelated tool work.

## Required execution correction

Current conversation/validation.rs waits for exit before draining pipes, then
allocates full output before truncating. It lacks caller cancellation and kills
only the direct child. Errors after spawn are incorrectly reported NotStarted.
These are implementation gaps, not accepted limits.

Prefer a shared host execution adapter with cancellation and owned process cleanup;
reuse talos_sandbox::OwnedCommandExecutor where supported, without confusing
lifecycle ownership with permission admission. Keep planning/summary independent
of process authority. Inspect dependency direction and public API compatibility
before selecting adapter placement. Do not introduce a second subprocess runner.

Runtime regressions must cover normal success, nonzero exit, invalid input,
spawn failure vs started failure, simultaneous stdout/stderr beyond pipe capacity,
exact output limit and overflow, timeout, cancellation during output, descendant
holding pipes open, cleanup confirmation, and panic/error containment. Cancellation
must stop subsequent profile checks. Bound retained output during collection.
Use deterministic harmless fixtures and prove elapsed bounds and child cleanup.
Native Windows must reuse its supported lifecycle adapter and have native
validation; actual platform/launch failures return explicit evidence. Blanket
Unavailable does not satisfy Windows acceptance; Unix evidence is not portable proof.


## Acceptance supplements from independent design review

- Use the same bounded adapter for new model tools and existing CLI host validation;
  do not leave the old wait-before-drain runner as a second supported path.
- Use actual existing Windows Job-owned execution, not blanket Unavailable. Unix
  retains ADR-082's accepted ordinary-owned-group guarantee; deliberate escape is
  not universal tree containment. No new unsafe authority is implied.
- Drop execution before bounded operation-local cleanup wait; register aggregate
  cleanup ownership for Runtime shutdown. Cancellation stops subsequent checks.
- Overflow is failure, not truncated success. Distinguish spawn refusal/failure
  from started timeout/cancel/overflow/cleanup failure without parsing error strings.
- Record CLI public sync API compatibility and host-bound workspace/allowed build
  side effects. Any breaking API follows ADR/migration rules.
- No task/Issue/Board/owner mutation, arbitrary commands, release/tag, paid probing
  or T6 memory limit changes. Tool descriptions guide selection but are not a
  deterministic natural-language intent router.

## Verification and completion

Run meaningful real permission/runtime regressions above, pinned locked checks,
Clippy, standard preflight with SDK modes, native platform CI, independent protected
review and merge CAS. Update README en/zh and tool/SDK docs; owner-first evidence
before derived views. No Completion Commit exists yet. The local sandbox stage is
`9bf942651`; remaining implementation, verification boundaries and merge gates are
tracked in I315.

## Required Reads

- ../../tasks/2026-10-10-session-todo-handoff.md (T1)
- VALIDATION-001-internal-validation-service.md
- GOV-003-builtin-project-governance.md
- ../../decisions/082-sandbox-command-cancellation-ownership.md
- ../../sop/AGENT-COLLABORATION.md
