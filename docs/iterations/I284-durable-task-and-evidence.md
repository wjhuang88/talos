# Iteration I284: Desktop Durable Tasks And Evidence

> Document status: Active — Claimed
> Plan date: 2026-09-22
> Target window: 2026-10-06 to 2026-10-12
> Planned objective: A Desktop user can reopen a saved conversation and inspect real work status, changes and revision-bound evaluation evidence.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A Desktop user can reopen a saved conversation and inspect real work status, changes and revision-bound evaluation evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | DESKTOP-001-D6 / I284: Desktop Durable Tasks And Evidence |
| Claimed At | 2026-09-23 |
| Source Issue | #29 |
| Governance Claim PR | #606 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I283/D5 implementation and governance closeout merged as `81a5d27c`; I284 is the next serial child |
| Implementation PR | Not started; ineffective until activation #606 reaches `main` |
| Last Updated | 2026-09-23 |
| Handoff / Release Condition | Resolve readiness and establish effective serial child claim before implementation; no release |

## Published Baseline

### Selected Story And Dependencies

[DESKTOP-001-D6](../backlog/active/DESKTOP-001-D6-durable-task-and-evidence.md), parent DESKTOP-001;
Planned / Unclaimed, not yet Ready or activated.
Prerequisites: I283 implementation merged with technical gates; ADR-042/061; durable-session and WORK-001 projection compatibility map.
Follow [the four-week task](../tasks/2026-09-22-desktop-four-week-delivery.md) for
serial scheduling, inventory, resource limits, authorization and deferred acceptance. Dates do not
activate implementation. Resolve readiness, then use one atomic claim/activation transition.

### Scope

- Use existing durable Session identity/storage for recent task navigation and transcript resume; no Desktop business database or automatic tool replay.
- Show authoritative Work/Goal projection when available. Missing projection remains unavailable, never replaced by fixture work.
- Present read-only actual file/artifact change evidence with source/task identity; do not infer a complete task diff from arbitrary workspace dirt.
- Integrate existing completion-claim/evaluator/Mission gate interfaces where supported, with explicit evaluation action rather than autonomous resubmission. No executor self-certification.
- Show evaluation missing/fail/inconclusive/stale/current states and Delivery eligibility from the shared gate. Persist only through supported contracts; never resurrect an old PASS after restart.

### Non-Goals

New durable Mission/Evaluation schema, automatic evaluation on every turn, multi-window/reconnect, full artifact editor/merge UI, #308 presets or complete autonomous Mission planning.

### Acceptance

- Given two sessions, reopening one restores only its own durable transcript and workspace association; switching tasks cannot mix approvals or tool results.
- Given an interrupted session, restart/resume does not repeat writes or invent missing output; storage errors become a safe visible failure.
- Given execution artifacts or changes, show their actual provenance, or explicitly mark unavailable attribution; inspection itself performs no mutation.
- Given missing/stale evaluation or a changed Goal revision, the UI cannot display current PASS or eligible Delivery; final model text does not certify success.
- A live Desktop-to-shared evaluation fixture covers pass/fail/staleness; absent durable evaluation storage displays unavailable after restart rather than claiming persistence.

### Planned Validation

Restart/resume/session-isolation fixtures, storage failure and no-replay tests, shared work/evaluation projection and revision-staleness matrix, read-only evidence boundary review; native H4 and integrated H6.

Use the pinned toolchain and --locked. Before a stable code candidate, run focused checks,
./scripts/release_preflight.sh, both governance validators, diff/secret review and applicable
independent Agent-role review. Exact-head remote CI validates the stable stage. Planning-only
changes do not require Rust builds. Required human rows remain Review until accepted.

### Documentation To Update

- Desktop crate README with delivered launch/behavior instructions (create in I282).
- README.md and README.zh-CN.md; site English/Chinese capability instructions where affected.
- Child owner, this iteration, four-week task, parent summary, indexes and #29 acceptance evidence.

### Risks And Rollback

Shared P4 is not proof of durable evaluation storage. Resolve facade access before implementation; any required new public boundary/schema needs decision review. An unavailable-state fallback is honest but does not close unmet baseline acceptance.
Preserve working mock/preceding stage. Disable incomplete live features rather than weakening
safety. Roll back only the failing slice; no destructive cleanup of user sessions or workspaces.

## Actual Activation And Execution

2026-09-23: activation PR #606 merged to `main` as `4e150b3e` after I283 technical and governance
closeout. The first local implementation slice binds configured Desktop RuntimeHost instances to
workspace-scoped durable Sessions through the shared `talos-session` facade. Successful turns are
persisted and readable after shutdown; no automatic tool replay or Desktop-owned database was added.
Local implementation commits `08df087e`, `beed4624`, `ad4a8665`, `1f466182`, and `9f549cd6`
now provide stable workspace+goal identities, durable host persistence, and a read-only tool
provenance projection. The old unparameterized configured-host entry was removed after the
focused check exposed it as unused.

## Verification Evidence

Implementation checks: `cargo test -p talos-session --locked` (189 unit/integration tests plus
6+1+15+1+2+2+2 integration tests) passed; the full Desktop mock suite passed 80/80 with
`--locked`, and `cargo check -p talos-desktop --locked` passed. The durable host integration test
verifies a successful turn is persisted and reopened through the workspace-scoped binding; task
discovery, source attribution and explicit unavailable evaluation display are covered. A full
artifact diff viewer and persisted evaluation staleness remain unfinished in this iteration.

## Completion Evidence

Completion Commit: pending.
Only already-existing implementation/evidence commits may close this iteration.

## Variance And Residuals

The current UI now has workspace-scoped recent-session discovery and explicit resume, and it
exposes tool request provenance and requested paths, but does not claim that they prove a file
changed. It still has no full artifact diff viewer or persisted evaluation state. The shared
Work/Evaluation contracts are storage-neutral; no durable Mission/Evaluation projection is exposed
to this Desktop host. H4/H6 remain human acceptance rows in #29. Carry eligible human rows without
transferring protected security gates.

## Local Review Correction Checkpoint — 2026-09-23

PR #607's prior exact head `8e8ebf6dde23af4286badd1b66d1dfcf41d62c24` received two blocking
findings. Local corrections replace lossy workspace/goal sanitization with domain-separated
SHA-256 identities, preserve legacy task bindings while redacting their old embedded IDs from the
Recent Tasks UI, and make Resume open an existing identity only. The existing-only mode survives
host restart attempts; a missing binding reports an error and does not create a replacement.
Interrupted pending-write restart coverage now proves transcript restoration without tool replay.
Tool request paths are presented as request metadata; actual artifact-change attribution remains
explicitly unavailable. Recent-task SQLite reads now run on GPUI's background executor.

Local verification on the unpushed candidate passed `cargo check -p talos-desktop --locked`,
`cargo clippy -p talos-desktop --features desktop-ui --all-targets --locked -- -D warnings`,
and `cargo test -p talos-desktop --features desktop-ui --locked` (83/83). Formatting, diff checks,
and both governance validators passed; each validator reported 0 warnings. The PR's old exact head
`8e8ebf6` failed the same Desktop Clippy gate on a nested condition in the previous Resume path;
the local candidate replaces that path, and the exact CI command now passes locally. Full
The full `./scripts/release_preflight.sh` passed on this local candidate, including workspace
check, Clippy, tests, doc-tests and both Runtime SDK fixtures. Fresh exact-head CI and independent
review are still pending. Do not reuse CI/review for old head `8e8ebf6` as evidence for the
corrected candidate.

I284 remains Active / Claimed, not Complete. The live UI has no shared durable Work/Evaluation
source and no actual changed-artifact evidence contract; its unavailable state is truthful but
does not satisfy the full evidence objective. The actual artifact diff/change viewer and supported
shared evaluation projection remain unresolved within the published acceptance. H4/H6 still
require natural-person acceptance through #29.
