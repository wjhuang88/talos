# Iteration I219: PERM-006-B First-Class Scoped Grants

> Document status: Planned / Unclaimed
> Published plan date: 2026-08-22
> Planned objective: replace compatibility runtime permission rules with one first-class scoped
> grant compiler, explicit in-memory Session store and revision-safe approval/admission contract
> shared by the official CLI, TUI and embedded Runtime surfaces.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: real `talos permissions preflight`, CLI/TUI permission wrappers and the embedded
> Runtime use the same proposal/preview/grant contract; repeated same-scope Session approval is
> reused only in that Session, Once is single-use, a different scope still asks, and every current
> policy Deny remains terminal.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Codex / GPT-5.6 mainline governance session 2026-08-22 |
| Work Slice | Implement only PERM-006-B / I219: first-class grant/compiler/store types in `talos-permission`; explicit in-memory Session ownership; proposal/approval revision and restriction CAS; consuming Once and pre-admission fencing; exact-only path scope; full provenance and multi-facet matching; all-policy Deny dominance; existing Bash classifier descriptor reuse; shared CLI/TUI/Runtime compilation, preview and official wrapper integration; session transition clearing; and the ADR-066 v0.9 public API/schema migration with automated and real preflight evidence. Preserve the existing agent-owned pipeline boundary for PERM-006-C. No persistent/task/cross-process grants, typed-effect migration, model-assisted auto or `/auto`, sandbox/fallback policy change, TOOL-024/background jobs, release, version, tag, publication, Desktop or Dashboard work. |
| Claimed At | Not applicable |
| Source Issue | #54 (dependency for #55 and #59; related #188) |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending exact-head CI, independent Agent-role permission/security/API review and merge-time CAS. Shared GitHub identity proves Agent-role separation only, not natural-person identity separation. |
| Implementation PR | Not started |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Claim and activation remain ineffective until this governance record reaches `main`. After merge, implementation must start from that merge or later and requires a separate exact-head permission/security/code review before merge. |

## Published Baseline

Planning target: `main@17e0b6488d530ea0e85991fd4d49ad5eb3cb2c07`.

### Current Nonterminal Inventory And Disposition

| Iteration(s) | State | I219 disposition |
|---|---|---|
| I197, I198, I201, I210 | Review | Preserve their owners and deferred/corrective validation; no authority transfer. |
| I206, I207, I208 | Planned / Unclaimed | Preserve the ordered steering sequence; do not activate or supersede. |
| I213 | Planned / Claimed | Independent Dashboard lane remains unactivated and non-overlapping. |
| I164 | Paused / superseded | Do not restore. |
| I219 | Planned / Unclaimed | Propose the sole mainline Active iteration for PERM-006-B only. |

PRs #120/#121 remain archival Drafts. No other open PR owns PERM-006-B. ADR-066 is Accepted through
PR #358 merge `17e0b648`; PERM-006-A/I189 is Complete/Closed at Completion Commit `6b577d6a`.

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-006-B | PERM-006 / Issues #52 and #54 | Ready / Unclaimed | PERM-006-A Complete; ADR-066 Accepted; preserves PERM-004/PERM-005 and SEC-001 | One session-scoped grant contract is exercised through the real preflight, CLI/TUI and embedded Runtime paths with Deny, scope, provenance, lifecycle and API migration evidence. |

### Scope

- Add distinct non-serialized grant/proposal/store authority types in `talos-permission`; keep
  `PermissionEngine::rules()` policy-only and make every effective policy Deny dominate grants.
- Compile the complete authoritative `PermissionRequest` into one atomic proposal whose bounded
  preview is the sole installation description; bind full provenance, facets, Session and all
  policy/mode/workspace/registration/restriction/store revisions required by ADR-066.
- Support exactly consuming `Once` and in-memory `Session`; make clear/new/resume/fork/runtime
  rebuild prevent later reuse and make clear-before-admission prevent an unstarted invocation.
- Compile workspace and external paths exact-only. Reuse existing audited Bash classifier
  descriptors without parsing raw shell input again. Preserve current typed Domain/Remote limits.
- Replace in-tree compatibility runtime-rule insertion at CLI/TUI approval, `/attach`, permission
  preflight and embedded Runtime approval surfaces while retaining existing wrappers for C.
- Apply the documented public source/schema migration: separate grant decision provenance and
  replace `ToolAuthorizationScope::Persisted` with truthful `Policy`, `Once` and `Session` values.
- Update CLI/SDK/migration documentation and crate public rustdoc for observable/public changes.

### Non-Goals

- No PERM-006-C agent-owned evaluate-to-execute pipeline convergence, wrapper removal, unified hook
  ownership, cancellation migration or Bash sandbox pipeline change beyond mechanical API updates.
- No PERM-006-D typed effect/resource redesign or PERM-006-E final conformance closeout.
- No persistent, task, scheduler, inherited, cross-process or permission-config grant storage; no
  trusted-workspace broadening.
- No model-assisted decision behavior, `/auto`, PERM-007 implementation or Issue #188 behavior.
- No sandbox fallback behavior, TOOL-024/background process, Windows Job Object, release, workspace
  version, tag, crates.io publication, Desktop or Dashboard implementation.

### Acceptance

- Given an effective Configured or Explicit policy Deny and any matching grant, when the complete
  request is evaluated, then Deny wins, including Allow-before-Deny conflict fixtures.
- Given human or configured SDK-host approval, when `Once` is selected, then one official adapter
  invocation may start and the authority is neither cloneable nor stored; a second use fails closed.
- Given a Session approval, when a later request has the same compiled scope and complete
  provenance, then it is reused only in that Session; sibling paths, other provenance/providers,
  uncovered facets and a new/resumed/forked Session still Ask or Deny.
- Given a policy/mode/workspace/registration/restriction/store change while approval is pending, or
  clear before admission, when commit/admission runs, then no stale grant authority starts a tool.
- Given a hybrid request, when any facet fails compilation, approval or matching, then installation
  is atomic with zero partial authority and execution remains unresolved/fail-closed.
- Given `talos permissions preflight` for write, safe-template Bash and unsafe Bash operations, when
  rendered, then preview comes from the proposal, write is exact-only, the safe template uses the
  existing classifier descriptor, unsafe Bash remains exact, and no tool executes or grant installs.
- Given sentinel secrets and distinct native/MCP/plugin identities, when Debug/report/preview/log/
  schema and matching tests run, then secrets/private fingerprints never appear and identities do
  not collide.
- Existing permission-rule and decision serialization remains compatible; documented exhaustive
  Rust/report schema breaks are covered as v0.9+ migration without changing the workspace version.

### Planned Validation

- `cargo fmt --all --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test -p talos-core --locked`
- `cargo test -p talos-permission --locked`
- `cargo test -p talos-tools --locked`
- `cargo test -p talos-agent --locked`
- `cargo test -p talos-cli --locked`
- `cargo test -p talos-runtime --locked`
- `cargo test -p talos-mcp --locked`
- `cargo test -p talos-plugin --locked`
- `cargo test --workspace --locked`
- Real `talos permissions preflight` fixture for exact write, reusable safe Bash descriptor and
  exact unsafe Bash, proving no execution or installation.
- Cross-surface structural equivalence, Session lifecycle, concurrent proposal/CAS, clear-before-
  admission, provenance collision, multi-facet atomicity and sentinel-secret regression suites.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- Exact-head independent Agent-role permission/security/API/code review and CI, including Windows
  workspace validation for the stable implementation candidate.

### Documentation To Update

- `README.md` permission preflight and Session-only `AlwaysApprove` behavior.
- `docs/reference/RUNTIME-SDK-CONTRACT.md`.
- `docs/reference/I219-PERM006B-SCOPED-GRANT-MIGRATION.md`.
- `crates/talos-permission` crate-level/public API rustdoc.
- PERM-006/PERM-006-B owners, Issue #54, and owner-first derived status views.

### Risks And Rollback

- Risk: duplicated normalization or shell parsing broadens authority or makes preview differ from
  installation. Mitigation: consume authoritative typed facets and shared path/classifier outputs.
- Risk: waiting for approval while holding session state deadlocks, or a stale response crosses a
  newer Deny/restriction. Mitigation: snapshot under lock, wait unlocked, CAS and fence on commit.
- Risk: current TUI approval state survives logical Session transitions. Mitigation: clear/rebind
  only at the successful transition publication fence and test failed transitions retain old state.
- Risk: private fingerprints or provider identity leak through observer surfaces. Mitigation:
  non-serialized private fields, custom safe projections and sentinel-secret tests.
- Rollback: disable Session insertion and clear the affected in-memory store; configured policy and
  durable data remain unchanged. Revert the source migration before any v0.9+ publication.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-22 | Selection proposal | I219 is the only proposed Active iteration. This open governance branch has no claim or activation effect; no implementation branch or Rust/Cargo edit is authorized before reviewed exact-head merge to `main`. |

## Verification Evidence

- Claim candidate: pending governance PR number, exact-head CI, independent permission/security/API
  review and merge-time CAS.
- Implementation and runtime evidence: pending effective claim and later stable implementation PR.

## Completion Evidence

- Completion Commit: Pending
- A later status-only closeout must cite pre-existing implementation commit(s); this claim record
  cannot certify implementation completion.

## Variance And Residuals

- PERM-006-C remains blocked and owns full agent pipeline convergence/cancellation after B.
- PERM-006-D/E, PERM-007/#188 and TOOL-024/#59 retain separate claims and dependency order.

## Retrospective

- Pending implementation and verification.
