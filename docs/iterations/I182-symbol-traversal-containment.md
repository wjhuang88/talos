# Iteration I182: Symbol Traversal Containment

> Document status: Planned
> Published plan date: 2026-08-08
> Planned objective: prevent symbol-tool symlink recursion and unbounded directory-mode parser admission while preserving user-supplied root symlink resolution, normal-tree symbol results, and all unrelated parser/runtime behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: both directory symbol tools safely terminate on symlink cycles and finite depth/file/byte budgets, report bounded omissions, and retain byte-identical normal-tree JSON.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-08 |
| Work Slice | Implement only ARCH-034-R04-AG4 in `crates/talos-tools/src/symbol.rs`: non-following descendant entry classification, skip traversed directory/file symlinks while preserving user-supplied root symlink resolution, depth 64, 10,000 parser-admitted files, 2 MiB cap-plus-one file reads, 50 MiB parser-admitted aggregate bytes, one discriminated final JSON notice on bounded omission, and the focused compatibility/security tests; preserve public inputs, ordinary result objects/order, language/skip behavior, parser fallback, dependencies, permissions, and every AG-5 parser panic/deadline concern. |
| Claimed At | 2026-08-08 |
| Source Issue | None |
| Governance Claim PR | #176 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent review comment `5226241652` reviewed PR #176 head `eb0ab6f1af71ddebb6c1ccea26f979de9964f624` and returned NEEDS CHANGES. Independent security re-review must approve the finalized corrected head of governance PR #176; the approved SHA is recorded in the PR/CAS record rather than self-referenced here. |
| Implementation PR | Not started |
| Last Updated | 2026-08-08 |
| Handoff / Release Condition | Do not activate or create implementation work until the finalized governance-only claim is independently reviewed, passes exact-head CI/CAS, and is merged to `main`. |

## Closure Ledger

| Dimension | Record |
|---|---|
| Requested outcome | Begin the highest-severity accepted I181 implementation slice without combining other native/panic gaps. |
| Artifacts to create/update | ARCH-034-R04-AG4 owner, I182, R04 parent link, iteration/backlog indexes, Board, and governance manifest. |
| Existing assets to preserve | I181/PR #174 review evidence, I181/PR #175 closeout evidence, R04 `Partial` state, `symbol.rs` tool schemas/result objects/order/language detection/skip filters/fallbacks, and AG-1..AG-3/AG-5..AG-7 separation. |
| State/status owners | AG4 child and I182 first; R04 and derived indexes/views second. |
| Validation required | Source-boundary review, stale/overlap searches, `git diff --check`, both governance validators, architecture audit, scale assessment, exact-head Unix/Windows CI, independent security review, and merge-time CAS. |
| Evidence and uncertainty | Symlink following and unbounded directory reads are confirmed facts. Review `5226241652` accepted depth/file-count/aggregate budgets and required the 2 MiB parser cap, precise symlink/counter semantics, and discriminated notice contract now recorded in the AG4 owner. |
| Residual-work destination | ARCH-034-R04 retains AG-1..AG-3 and AG-5..AG-7, direct-file unbounded reads, and unbounded symbol-output serialization; AG-5 owns parser panic/deadline and wall-clock containment. |

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Unchanged under their published dependency/security/release gates. |
| ARCH-034-R04 | Partial | AG-4 selected as a non-overlapping child; parent remains Partial. |
| I181 | Review closed | Review evidence `aea26ad0`; no implementation authority is inherited. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable and non-overlapping. |
| Other Active/Review/Planned iterations | None | No competing iteration selection exists. |

No existing branch, open claim PR, implementation PR, or owner was found for AG-4/I182. The only
open PRs at selection were archival recovery PRs #120/#121.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R04-AG4 | ARCH-034-R04 | Ready | I181 review/closeout evidence and an independently reviewed effective claim | Bounded, cycle-safe directory symbol traversal with explicit omission reporting and normal-tree compatibility. |

### Scope

- Implement the exact scope, counter definitions, notice schema, and safety budget in
  ARCH-034-R04-AG4 after independent claim approval.
- Keep limits internal and share only the traversal accounting needed by the two existing walks.
- Skip traversed file/directory symlinks but preserve following of the user-supplied root symlink;
  use a 2 MiB + 1 bounded read before parser admission.
- Append at most one discriminated final JSON notice only when bounded work is omitted; preserve
  ordinary arrays and define the all-omitted result as a notice-only array.
- Add focused fixtures proving directory/file symlink, symlinked root, oversized, notice-only,
  depth/file/aggregate, notice absence, and byte-identical compatibility behavior.

### Non-Goals

- No AG-5 parser panic/deadline/wall-clock work or any other R04 child.
- No direct-file read containment for `list_symbols`, `find_references`, or `list_imports`, and no
  symbol-output byte cap; both remain explicit R04 residuals.
- No public API/schema, dependency, permission, ADR, sandbox, TUI, language mapping, sorting, or
  unrelated refactor.
- No production implementation on the governance claim branch.

### Acceptance

- Given a directory symlink cycle or a traversed file symlink, when either directory symbol tool
  runs, then traversal terminates without following the entry and reports the omission.
- Given a symlinked user-supplied root directory, when `list_symbols` runs, then the root still
  resolves and descendant traversal applies the new non-following rules.
- Given a file or traversal beyond a reviewed byte/file/depth budget, when the tool runs, then it
  does not admit the excess work and appends one deterministic final notice object with the exact
  `talos_notice`, `reasons`, `counts`, `admitted_files`, and `admitted_bytes` contract in AG4.
- Given every candidate is omitted, when the output is serialized, then the array contains only the
  distinguishable notice object and cannot be mistaken for a symbol result.
- Given an ordinary tree below all limits, when compared with the pre-change implementation, then
  serialized result content and order are byte-identical and the notice discriminator is absent.
- Given unsupported files or parser/language load failure, when the tool runs, then existing safe
  skip/error behavior remains unchanged.

### Planned Validation

- Focused `cargo test -p talos-tools --locked symbol` coverage including Unix symlink fixture.
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `python3 scripts/audit_architecture.py .`
- `scripts/assess_project_scale.sh .`
- Exact-head Unix/Windows CI, independent security re-review, and merge-time CAS.

### Documentation To Update

- Update ARCH-034-R04-AG4 and I182 execution/validation evidence.
- Synchronize R04, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`,
  and `.agent-governance/manifest.yaml`.
- No user guide change is planned because public inputs and ordinary successful output remain
  compatible; if the reviewed notice contract becomes user-facing documentation debt, record it
  before delivery closure.

### Risks And Rollback

- Risk: a containment marker could invalidate existing JSON consumers, or a walker rewrite could
  reorder normal results.
- Rollback: retain the existing native `read_dir` order and result-object serialization; revert the
  implementation if the byte-compatibility fixture changes.
- Risk: canonicalization-based cycle handling can cross intended roots or introduce host-specific
  identity behavior.
- Rollback: do not descend through directory symlinks; use non-following metadata rather than
  canonical traversal as the primary containment.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-08 | Planning | Selected AG-4 after I181/PR #174 and closeout PR #175 merged; inventoried I159-I162 as unchanged Blocked, found no Active/Review/Planned competitor, and found no overlapping branch/PR/owner. |
| 2026-08-08 | Claim submission | Draft governance-only PR #176 opened; this finalized exact-head record proposes `Claimed` ownership but has no effect until independently reviewed and merged to `main`. |
| 2026-08-08 | Review follow-up | Independent review comment `5226241652` returned NEEDS CHANGES on head `eb0ab6f1`: stale evidence SHA, parser/file-symlink/counter/notice ambiguity, root-symlink preservation, direct-file/output residuals, and four missing tests are corrected in the owner baseline; corrected exact-head re-review remains mandatory. |

## Verification Evidence

- Claim validation pending finalized PR number and independent security review.
- Runtime evidence is not applicable before claim activation and implementation.

## Completion Evidence

- Not applicable while I182 is Planned. Any terminal delivery state requires an already-existing
  implementation merge/evidence SHA; claim or status commits cannot self-certify it.

## Variance And Residuals

- AG-5 parser panic/deadline and wall-clock containment, direct-file unbounded reads, unbounded
  symbol-output serialization, and all other R04 accepted gaps remain separate.

## Retrospective

- Outcome: pending claim review.
- Documentation: owner-first synchronization proposed in the claim PR.
- Lessons: pending implementation evidence.
