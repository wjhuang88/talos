# Iteration I180: Architecture Documentation Truth

> Document status: Review
> Published plan date: 2026-08-07
> Planned objective: reconcile current architecture, crate/composition, extension, and historical-status documentation with source evidence without changing runtime behavior or decision semantics.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: current-state architecture prose accurately describes the workspace and tool/extension composition boundaries, historical snapshots are visibly historical, and security-gated ADR/R0 semantics remain owned by R04.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Reconcile `docs/reference/ARCHITECTURE.md` current-state workspace, crate, CLI, tool-contribution, extension, and composition descriptions against root `Cargo.toml` and current source; explicitly distinguish historical iteration-era snapshots from current facts; update directly affected architecture indexes/registers with non-semantic factual status only; preserve every runtime/API/dependency/decision/security behavior and route any ADR-007/R0 semantic or process-hardening conclusion to ARCH-034-R04. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #170 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, scale assessment, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #171 |
| Last Updated | 2026-08-08 |
| Handoff / Release Condition | Release if a claimed current fact lacks source evidence or requires decision/security interpretation; any ADR-007/R0 semantic or process-hardening change remains blocked on independent R04 security review. |

The finalized `Claimed` record became effective on `main` at merge commit
`1ca03fdf8d262eba4d1de2374e43f2c1a94882dd`; the implementation branch starts from that commit.

## Closure Ledger

| Dimension | Record |
|---|---|
| Requested outcome | Close the remaining non-security architecture documentation finding without changing logic or decision semantics. |
| Artifacts to create/update | I180, ARCH-034-R11, current architecture reference, directly affected indexes/registers, Product Backlog, Board, and governance manifest. |
| Existing assets to preserve | August audit evidence, published iteration baselines, accepted ADR text/semantics, R01 exception verdicts, recovery PRs #120/#121, and all runtime/public behavior. |
| State/status owners | ARCH-034-R11 and I180 first; iteration index, Product Backlog, Board, manifest, and parent/register facts second. |
| Validation required | Source-to-doc trace review, DOC-CHECK, both governance validators, scale assessment, local-link/search checks, `git diff --check`, and exact-head Unix/Windows CI. |
| Evidence and uncertainty | Root Cargo/source/current owner states are facts; prose classifications are inferences until traced. ADR-007/R0 security meaning remains explicitly out of scope and unknown pending R04 review. |
| Residual-work destination | ARCH-034-R04 owns security/unsafe/process-hardening semantic reconciliation; no such conclusion may be closed by I180. |

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Unchanged; their feature/composition/security/publication dependency chain remains blocked. |
| I164 | Paused | Historical superseded target; no activation. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable; no implementation authority. |
| ARCH-034-R04 | Refinement | Security/unsafe/process-hardening semantics remain excluded pending independent security review. |
| I179 / ARCH-034-R10 | Closed | Implementation merge `dafc9be0`; closeout merge `76b81a8e`; no overlap with documentation truth. |
| Other Planned backlog items | Unselected | Retain their existing owners; none authorizes or overlaps this architecture-reference reconciliation. |

No other Active, Review, or Planned iteration exists. R11 is selected after I179 closure because it
is the final Ready non-security child in the August architecture register.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R11 | ARCH-034 | Ready | I171 architecture register, I158/R01 composition verdicts, I179 closure, root Cargo/source truth, and R04 exclusion | One documentation-only reconciliation of current architecture and historical labels. |

### Scope

- Trace current workspace/crate responsibilities and dependency/composition statements to root `Cargo.toml` and current source.
- Reconcile CLI runtime ownership, tool contributions/profiles, plugins/MCP extension paths, and recorded R01 exceptions.
- Mark iteration-era or superseded architecture passages as historical without rewriting their original evidence.
- Update current indexes/registers only where needed to point readers to the authoritative August/current-state record.
- Record non-semantic ADR-007/R0 status facts only when source evidence is direct and security meaning is unchanged.

### Non-Goals

- No production, test, manifest dependency, public API, runtime behavior, permission, sandbox, or process-hardening code change.
- No ADR acceptance/reversal, security conclusion, unsafe justification change, or R04 completion claim.
- No historical evidence rewrite, architecture redesign, new crate, dependency, proposal, or speculative target state.
- No changes to recovery PRs #120/#121 or I159-I162 activation gates.

### Acceptance

- Current-state architecture tables and prose match root workspace membership and source-owned composition boundaries.
- Tool contributions, CLI profile selection, scheduler/status exceptions, plugin/MCP paths, and `talos-core` ownership agree with current code and R01 evidence.
- Historical/iteration-era text is explicitly distinguished from current truth; no accepted decision is backdated or reinterpreted.
- ADR-007/R0 security semantics remain unchanged and visibly routed to R04 where reconciliation requires review.
- DOC-CHECK, governance/claim validators, scale assessment, link/search/diff checks, and exact-head CI pass.

### Planned Validation

- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `scripts/assess_project_scale.sh .`
- `./scripts/release_preflight.sh`
- `git diff --check`
- Source-to-doc trace matrix for Cargo membership, CLI/runtime ownership, tool contributions, profiles, plugins/MCP, and R01 exceptions.
- Local Markdown link and stale-current-claim searches.
- Exact-head Unix/Windows CI, governance reconciliation, installer fixture, and rebuilt CLI smoke.

### Documentation To Update

- Update `docs/reference/ARCHITECTURE.md` and only directly affected architecture current-state/index records.
- Synchronize ARCH-034-R11, I180, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the governance manifest.
- No product README/user guide change is expected because this work changes architecture documentation only.

### Risks And Rollback

- Risk: current prose could accidentally reinterpret accepted decisions, erase historical context, or claim a target architecture not present in source.
- Rollback: revert any unsupported prose; route security/decision ambiguity to R04 or a new ADR instead of resolving it in documentation.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Planning | I180 selected after inventorying non-terminal work, confirming I179/R10 closure, and finding no overlapping effective claim or implementation PR. |
| 2026-08-07 | Claim submission | Draft governance claim PR #170 opened; the exact finalized `Claimed` record is submitted for claim-only CI, scale validation, and merge-time CAS. No documentation implementation authority exists until #170 merges to `main`. |
| 2026-08-07 | Claim activation | PR #170 squash-merged to `main` as `1ca03fdf8d262eba4d1de2374e43f2c1a94882dd` after exact-head CAS, CI `31193474992`, both governance validators, scale assessment, and `git diff --check` passed. Implementation branch `docs/i180-architecture-documentation-truth` starts from that commit. |
| 2026-08-07 | Implementation | Current-state architecture documentation reconciliation is implemented on this branch; production/runtime behavior and ADR-007/R0 semantics remain unchanged. |
| 2026-08-08 | Review submission | Source implementation commit `fd8ac75d` pushed and draft implementation PR #171 opened. Owner/derived status moves to Review; exact-head CI and merge evidence remain pending. |

## Verification Evidence

### Source-to-Document Trace

| Current fact | Source evidence | Architecture surface |
|---|---|---|
| Workspace membership and publication boundary | Root `Cargo.toml`; crate manifests; ADR-052 and publication matrix | Cargo workspace and distribution sections |
| CLI composition ownership | `crates/talos-cli/src/main.rs`, `mode_runners.rs`, `mode_runtime.rs`, `tui_runtime_builder.rs`, `registry.rs`, `runtime_adapter.rs`, `session_transition.rs` | CLI Runtime Boundary and dependency graph |
| Built-in contribution ownership | `crates/talos-tools/src/contributions.rs`; `crates/talos-session/src/tool_contributions.rs` | Tool Contribution And Profile Composition |
| Scheduler and MCP `status` exceptions | `crates/talos-agent/src/scheduler.rs`; `crates/talos-cli/src/registry.rs`; I158/R01 closeout | Composition exception table |
| Plugin carrier and extension path | `crates/talos-plugin/src/manifest.rs`, `wasm.rs`; ADR-027 | Plugin System |
| MCP session lifecycle | `crates/talos-cli/src/mcp_runtime.rs`; `crates/talos-mcp/src/client/` | MCP Session Boundary |
| Storage/file boundaries | `crates/talos-session/Cargo.toml`, `crates/talos-evolution/Cargo.toml`, `crates/talos-memory/Cargo.toml`, `crates/talos-exploration/Cargo.toml`, and `crates/talos-cli/src/storage.rs` | Storage Architecture |
| Historical versus current labels | August architecture audit and finding register | Current-state note and historical storage phases |

The claim-only exact-head CI evidence remains recorded above; implementation PR #171 exact-head CI
is pending the review gate.

### Local Implementation Validation (2026-08-08)

- `./scripts/release_preflight.sh`: PASS for site/installer validation, governance, locked workspace
  check, Clippy, tests, and doctests.
- `scripts/validate_project_governance.sh .`: PASS, 0 warnings.
- `bash scripts/validate_collaboration_claims.sh .`: PASS, 0 warnings.
- `scripts/assess_project_scale.sh .`: PASS; high-risk, release-managed, on-demand remains the
  recommended profile/mode.
- `scripts/audit_architecture.py .`: PASS; 21 crates and 0 dependency cycles.
- Stale-current-claim search, documentation-only scope check, and `git diff --check`: PASS.

## Completion Evidence

- Completion Commit: not assigned; retain Review until PR #171 merges and an existing merge SHA can
  be cited by a separate closeout change.

## Variance And Residuals

- R04 remains Refinement pending independent security review and owns all ADR-007/R0 semantic reconciliation.
- ARCH-034 remains open after R11 until R04 receives a truthful independent disposition.

## Retrospective

- Outcome: pending.
- Documentation: pending implementation result.
- Lessons: none recorded.
