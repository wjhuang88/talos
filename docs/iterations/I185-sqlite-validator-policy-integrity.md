# Iteration I185: SQLite Validator Policy Integrity

> Document status: Planned
> Published plan date: 2026-08-10
> Planned objective: eliminate the duplicate ADR-008/validator SQLite-consumer policy source and preserve actionable Cargo diagnostics while keeping every runtime, dependency and accepted-consumer behavior unchanged.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: maintainers can run the standard validator and prove that one structured policy controls the exact five-consumer ADR boundary, invalid policy/metadata fails closed, and non-UTF-8 Cargo stderr remains actionable without changing Cargo invocation or Talos runtime behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 (proposed) |
| Executing Agent | Codex / GPT-5.6 architecture session 2026-08-10 (proposed) |
| Work Slice | Implement only ARCH-034-R04-AG12: introduce one versioned structured SQLite-consumer policy named normatively by ADR-008 and loaded by `scripts/validate_sqlite_consumers.py`; preserve the exact accepted set and all locked graph/bundled/version/isolation semantics; explicitly retain `cargo metadata --locked` with fail-closed host/toolchain/cache failures and no `--frozen`/offline fallback; decode Cargo stdout strictly as UTF-8 while escaping undecodable stderr bytes; add controlled cross-platform fixtures and synchronize evidence. No Rust, Cargo, runtime, SQLite consumer, schema, migration, timeout, retry, network policy, AG-11 or other R04 child change. |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review (proposed) |
| Authorization Evidence | Claim review and exact-head CI pending. The proposal has no ownership effect until the finalized record is merged into `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | Finalize the actual claim PR number, obtain exact-head independent approval and CI, repeat merge-time CAS, then start implementation from that claim merge or later `main`. |

## Closure Ledger

| Dimension | Record |
|---|---|
| Requested outcome | Resolve the behavior-preserving AG-12 architecture/governance residual. |
| Artifacts to create/update | AG-12 and I185 owners, R04 link, iteration/backlog indexes, Board and manifest; after claim merge, ADR-008/reference policy, SQLite validator and controlled fixtures. |
| Existing assets to preserve | I183 completion evidence, the five accepted consumers, workspace-boundary graph semantics, bundled/version/isolation assertions, current `--locked` resolution behavior, AG-11 and all production code. |
| State/status owners | AG-12/I185 first, then R04 and derived views. |
| Validation required | Validator positive/negative fixtures on Unix/Windows, both governance validators, architecture audit, scale assessment, locked release preflight, exact-head CI, independent review and merge-time CAS. |
| Evidence and uncertainty | Duplicate policy literals and diagnostic masking are confirmed repository facts. The selected solution changes governance enforcement/diagnostics only; runtime equivalence is proven by a zero Rust/Cargo diff and full locked validation. |
| Residual-work destination | AG-11 and every other R04 child remain separately owned and unmodified. |

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Unchanged under published TUI-037, sequential dependency, security and release gates. |
| I184 | Review / closeout PR #189 pending | Non-overlapping TUI policy closeout; no TUI state or implementation is included in I185. |
| ARCH-034-R04 | Partial | Select only AG-12; parent remains Partial. |
| AG-1/2/3/5/6/8/9/10/11 | Ready or Refinement, unclaimed | Preserve each independent behavior/security/evidence boundary; no authority is inherited. |
| Recovery PRs #120/#121 | Open archival evidence | Keep immutable, unmerged and non-overlapping. |

No open branch, claim PR, implementation PR or effective claimant was found for AG-12/I185 at
selection. PR #189 is limited to I184 closeout and does not overlap this validator-only scope.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R04-AG12 | ARCH-034-R04 | Refinement promoted to Planned by this exact contract | I183/AG-7 completion and an independently reviewed effective I185 claim | One authoritative SQLite consumer policy plus fail-closed, diagnostically useful validation with no runtime behavior change. |

### Scope

- Replace the validator-local accepted-consumer constant with a versioned structured policy file
  that ADR-008 identifies as the normative exact allowlist.
- Validate policy schema/version and exact set equality against the existing locked resolved graph.
- Preserve `cargo metadata --locked` and explicitly document missing Cargo, resolution, cache and
  network failures as fail-closed validation errors.
- Decode metadata stdout strictly as UTF-8 JSON; render invalid stderr bytes using deterministic
  escapes so the originating Cargo failure remains inspectable.
- Add deterministic self-test fixtures for policy and decoding failures and keep the standard
  success summary truthful.

### Non-Goals

- No Rust source, Cargo manifest/lock, dependency, feature, consumer, quarantine or runtime change.
- No SQLite schema, migration, query, timeout, retry, panic or corruption-containment change;
  AG-11 remains separate.
- No `--frozen`, offline fallback, network-policy change, dependency download wrapper or removal of
  the pinned Cargo requirement.
- No other R04 child, architecture fitness gate or general validator refactor.

### Acceptance

- Given the accepted structured policy and current locked metadata, when validation runs, then the
  same five consumers, three layered packages, one `rusqlite`/`libsqlite3-sys` version, bundled
  features and zero `talos-models` dependents pass with the existing summary facts.
- Given a policy with an unknown version, malformed field, duplicate/missing consumer or metadata
  consumer drift, when validation runs, then it fails closed with a specific recovery diagnostic.
- Given invalid UTF-8 Cargo stdout, when metadata loading runs, then no metadata is trusted and the
  validator fails closed.
- Given Cargo exits unsuccessfully with non-UTF-8 stderr bytes, when metadata loading runs, then the
  validator reports the Cargo failure and deterministic escaped bytes rather than only a decoder
  exception.
- Given a clean implementation diff, when reviewed, then it contains no Rust/Cargo/runtime files and
  does not alter ADR-008's accepted five-consumer semantics.

### Planned Validation

- `python3 scripts/validate_sqlite_consumers.py . --self-test`
- Controlled invalid-policy, invalid-stdout and non-UTF-8-stderr fixture execution on Unix and Windows.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `python3 scripts/audit_architecture.py .`
- `scripts/assess_project_scale.sh .`
- `./scripts/release_preflight.sh`
- Exact-head Unix/Windows CI, independent review and merge-time CAS.

### Documentation To Update

- ADR-008 and its structured SQLite policy/reference entry.
- AG-12, I185, R04, `docs/iterations/README.md`, Product Backlog, Board and manifest evidence.
- No user guide change: this operator/governance correction does not alter Talos commands or runtime.

### Risks And Rollback

- Risk: moving the allowlist could make an unreadable/malformed policy silently bypass checks.
- Rollback: validate schema/version/types before graph evaluation and fail closed before success output.
- Risk: permissive stderr decoding could also make invalid metadata stdout appear valid.
- Rollback: keep stdout strict and apply byte escaping only to error-channel presentation.
- Risk: changing Cargo flags could alter resolution/network semantics.
- Rollback: preserve the exact `cargo metadata --locked --format-version 1` invocation.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-10 | Selection | Inventoried I159-I162 as Blocked, I184 closeout as non-overlapping Review work, R04 as Partial, all sibling children as separately unclaimed, archival PRs #120/#121 as immutable, and no AG-12 overlap. Draft claim submission is pending and has no target-branch effect. |

## Verification Evidence

- Pending claim validation and implementation evidence.

## Completion Evidence

- No completion evidence while Planned. A later closeout must cite an already-existing implementation commit.

## Variance And Residuals

- AG-11 and every other R04 child remain independently unclaimed and outside I185.

## Retrospective

- Pending execution.
