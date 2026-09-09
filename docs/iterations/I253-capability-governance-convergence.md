# Iteration I253: Capability Architecture Governance Convergence

> Document status: Active
> Claim and activation effective on main through PR #518 merge `53fc57b3a607ab33d8aad51faf4662f672d55efa`.
> Published plan date: 2026-09-09
> MVP deliverable: An auditable requirement-to-owner map for every governance acceptance item in Issue #466, with independently executable downstream Stories.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | CAP-001-G architecture audit, terminology migration contract, child decomposition and documentation synchronization only. |
| Claimed At | 2026-09-09 |
| Source Issue | #519 (parent Epic #466) |
| Governance Claim PR | #518 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized single-developer unattended completion of #499 then #466 and accepted ADR-072. Independent Agent-role review and exact-head CI were completed for #518; shared GitHub identity does not prove natural-person separation. Claim is effective after merge `53fc57b3`. |
| Implementation PR | #521 |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Handoff requires the documented governance audit and closeout; no runtime implementation authority. |

## Published Baseline

### Selected Story

CAP-001-G, child of CAP-001. Depends on accepted ADR-072 and completed CAP-001-P0/I246.
I252 is already Complete; its implementation is preserved, not repeated or broadened.

### Scope

1. Reconcile every Issue #466 governance acceptance row against current code and document evidence.
2. Inventory executable Plugin versus distribution Bundle references, compatibility-sensitive APIs,
   manifest/config fields, current Arborium consumers and runtime boundaries. Retain historical size
   measurements as historical, not as new measurements.
3. Preserve ADR-027 and historical ADR-029; add explicit supersession pointers and a staged migration
   contract without renaming any production API or stored field.
4. Create bounded child owners and corresponding Issues for remaining CAP, Bundle, Text, Language,
   distribution and Browser outcomes after collision checks. Give each dependencies, constraints,
   acceptance, validation, documentation and reciprocal parent links. Preserve completed A/I252.
5. Separate TOOL-008 feature trimming from runtime loading, DIST-001 policy from installation, and
   WEB-005 product/security semantics from the connector. Preserve completed owner evidence.
6. Synchronize architecture, requirement convergence, proposal/ADR indexes, backlog and Issue map.
   Board lists selected children only, never the entire Epic as implementation work.

### Non-Goals

No Rust/Cargo, runtime registry/resolver, public API or persisted-schema mutation, Plugin loading,
network installation, permission, sandbox, release, Browser or Desktop/Dashboard implementation.
No reopening historical completed Stories. No activation of CAP-001-B/C or domain children.
Implementation stages 3-7 in #466 must be preserved as owned downstream work, not declared shipped.

### Acceptance

- Every #466 governance acceptance bullet has a specific repository evidence pointer or remains
  explicitly unfulfilled; a green validator cannot substitute for semantic coverage.
- All six domain terms, object-specific lifecycles, shared Language Provider path, UI-independent
  text semantics, plugin source versus Bundle artifact layout, and domain-specific fallback are
  documented as current behavior or target contract without conflating the two.
- Migration records cover PluginManifest/Metadata, package root, cache identity and contributions;
  dual-read, controlled-write, unknown-field handling, rollback and later schema ADR gates are explicit.
- Each executable downstream Story has one Issue and one authoritative owner; dependencies are
  acyclic, and only the next dependency-ready child may be Ready (not Active or Claimed).
- No runtime loading, parser-size reduction, installation or Browser behavior is claimed on the
  strength of descriptors, policy documents or Cargo feature trimming.
- Existing completed owners and historical plans remain intact; owner-first derived synchronization
  and Issue updates agree with actual evidence.

### Validation And Documentation

Governance-only infrastructure exception: no new binary behavior. Run both repository governance
validators with explicit base, YAML parsing, local links/changed-file inventory and diff checks.
Independently review the full #466 acceptance matrix, not only validator output. No Rust build is
required for documentation-only changes. Required documentation: CAP-001 and child owners,
docs/reference/ARCHITECTURE.md, docs/roadmap/REQUIREMENT-CONVERGENCE.md, relevant proposals/ADRs,
backlog/iteration indexes, Board, manifest and Issue/doc matrix.

### Risks And Rollback

Main risk: narrowing #466 to already-delivered descriptors or claiming downstream implementation
through planning. Preserve original Issue requirements and distinguish governance completion from
product delivery. Revert only this slice's documentation if needed; never erase historical evidence.

## Inventory And Startup Contract

Baseline: main `c878eabc`; 2026-09-09 API open-PR inventory is empty.

| Item | State | Disposition |
|---|---|---|
| I249 | Planned / Unclaimed | Retain dependency pilot without activation; full upgrade I250 is Complete. |
| I164 | Paused / superseded | Retain, do not resume. |
| I252 | Closed (completion evidence preserved) | Preserve descriptor implementation and retrospective review evidence. |
| I251 / #499 | Closed (completion evidence preserved) | Preserve closure; #499 is CLOSED. |
| Other iterations | Terminal or legacy historical | No Active/Review/Blocked current delivery header; I162 Complete/Review-outcome is terminal. |
| CAP-001 / #466 | Refinement / Unclaimed | Parent remains unselected; select only this governance child. |

Work mode: Standard, single-developer unattended. Ordered work: audit, migration/decomposition,
owner/index synchronization, independent governance acceptance audit, evidence-based closeout.
Allowed actions: scoped docs, local validation, stable PR/Issue coordination under existing
maintainer authorization. No destructive operations, purchases, release or runtime changes.
Use one claim stage then one locally converged governance implementation candidate; preserve
exact-head review/CI and CAS. Pause on a decision that would change accepted ADR-072.

### Child Issue Map (2026-09-09)

| Owner | Issue | State | Dependency disposition |
|---|---:|---|---|
| CAP-001-B | #512 | Ready / Unclaimed | Only dependency-ready child; not activated |
| CAP-001-C | #513 | Refinement / Unclaimed | Waits for CAP-001-B |
| TEXT-001 | #511 | Refinement / Unclaimed | Waits for CAP-001-C |
| LANG-001 | #510 | Refinement / Unclaimed | Waits for TEXT-001 and CAP-001-C |
| LANG-002 | #516 | Refinement / Unclaimed | Waits for LANG-001 |
| LANG-003 | #517 | Refinement / Unclaimed | Waits for LANG-002, BUNDLE-001 and DIST-001-A |
| BUNDLE-001 | #514 | Refinement / Unclaimed | Waits for CAP-001-C |
| DIST-001-A | #509 | Refinement / Unclaimed | Waits for BUNDLE-001 and CAP-001-C |
| DIST-001-B | #515 | Refinement / Unclaimed | Waits for DIST-001-A, CAP-001-B/C and BUNDLE-001 |
| BROWSER-001 | #508 | Refinement / Unclaimed | Waits for CAP-001-B/C and WEB-005 |

These Issues are intake tracking only. Creating them does not activate an owner or grant
implementation authority; each requires a separately selected runnable iteration and effective
Collaboration Claim.

## Required Reads

- Issue #466 full body/comments and Issue #467; ADR-072; CAP-001 and I252.
- PLUGIN-001, TOOL-008/012/014, DIST-001, WEB-005 and DESKTOP-001 owners.
- ADR-027/028/029/030, plugin-encapsulation-format, optional-runtime-asset-distribution,
  web-005-browser-session-continuity-design, architecture and requirement-convergence documents.
- Issue #308 for Session/Preset product semantics; CAP-001-P0 compatibility evidence.

## Execution And Completion Evidence

Completion Commit: Pending. Status changes cannot self-certify.
Next: complete the post-claim governance audit locally from merge `53fc57b3`;
push only after the full acceptance matrix and migration evidence converge.

### 2026-09-09 Atomic Claim Candidate Checkpoint

PR #518 now proposes Active / Claimed for I253 and CAP-001-G only. This checkpoint
supersedes the earlier claim-preparation current state, not the Published Baseline.
Both ownership and activation remain ineffective until merge. No implementation branch
may start before that merge or a later main commit. CAP-001 and all ten downstream
children retain their existing unclaimed states; ADR acceptance grants no runtime authority.

Fresh fetch confirms main remains `c878eabcee9ca3064b15291ce8fa073874915791`.
Inventory: I249 remains Planned/Unclaimed and deferred; I164 remains Paused/superseded;
all other current iteration headers are terminal, with I162's Complete/Review-outcome
treated as terminal. I253 is the sole proposed activation. PR #518 is the only open PR;
there is one worktree and no stash. Earlier CI `34322110835` proved the planning head
`111199b9`, not this changed claim candidate; fresh exact-head CI/review are required.

Next gate: validate this finalized candidate locally, push it once, obtain independent
Agent-role governance review, then perform merge-time CAS. After merge, audit the full
#466 acceptance matrix and migration contracts locally; do not claim the intake map
alone completes #466. No Rust build is required for this documentation-only candidate.

The earlier dated Child Issue Map is superseded for LANG-002 dependency detail: the
Rust WASM vertical slice requires LANG-001, CAP-001-C and DIST-001-A (verified manual
installation), not just LANG-001. BUNDLE-001 owns the actual compatible schema/manifest
migration after its decision gate, not only the migration plan. These corrections
preserve #466's installation/loading and WASM requirements without activating children.

### 2026-09-09 Claim Review Disposition

The dated Child Issue Map remains historical. Current dependencies are owned by the
child documents: TEXT-001 depends on completed CAP-001-A, LANG-001 on TEXT-001 and
CAP-001-A, and BUNDLE-001 on ADR-072 plus manifest compatibility evidence and a dedicated
migration decision. CAP-001-C is not a prerequisite for those contracts. TEXT-001
remains Refinement pending this governance audit and its consumer ownership inventory;
only CAP-001-B has completed readiness preparation. This is scheduling, not a fabricated
technical dependency. LANG-002 still needs CAP-001-C and DIST-001-A for real WASM loading.

The following acceptance remains unfulfilled and belongs to post-claim I253 execution,
not this activation candidate: ADR-029 supersession note; linked supersession of the
old Plugin encapsulation proposal and ADR/proposal indexes; plugins/ runtime-source
versus bundles/ artifact layout; concrete PluginManifest skills/tools/hooks versus
missing mcp/provides evidence; full #466 acceptance matrix and migration audit; and
distinct executable Issue mapping for CAP-001-G with a reciprocal Epic child link.
Issue #466 remains the parent requirement source, not authorization to implement all
children. These rows must be completed before governance closeout; intake alone is
not completion evidence.

### 2026-09-09 Effective Claim And Audit Checkpoint

PR #518 merged to `main` as `53fc57b3a607ab33d8aad51faf4662f672d55efa` after exact-head CI
`34357514961`, independent Agent-role APPROVE bound to head `24d0510e` / base `c878eabc`,
and merge-time CAS. I253/CAP-001-G is therefore the effective sole governance claimant. This
checkpoint records activation only; it does not close #466 or authorize any child implementation.

#### Current code-truth and layout audit

| Acceptance surface | Evidence at `53fc57b3` | Result |
|---|---|---|
| Capability/Provider/Plugin/Bundle/Carrier/Asset vocabulary | ADR-072; descriptors in `crates/talos-core/src/capability.rs` | Contract and descriptor exist; registry/resolution and installation do not |
| Plugin runtime source | `crates/talos-plugin/` (`manifest.rs`, `registry.rs`, optional `wasm.rs`) | Hook/manifest/WASM adapter substrate exists; no repository-root `plugins/` runtime source tree |
| Bundle artifacts | No repository-root `bundles/` directory or checked-in bundle artifact | Distribution layout remains a downstream BUNDLE/DIST outcome |
| PluginManifest fields | `plugin { name, version, carrier, artifact, description?, talos_protocol? }`; top-level `skills`, `tools`, `hooks`; no `mcp` or `provides` field | Current parser facts recorded; schema expansion is not implemented |
| Provenance and registration | Existing `talos-plugin` manifest/hooks and `ToolProvenance` consumers | Existing Tool provenance is preserved; general Capability registry and carrier contributions remain CAP-001-B/C work |
| Startup and fallback | ADR-072 requires network-independent startup and domain-specific safe fallback | Accepted contract only; no resolver or installer behavior is claimed |
| Shared text/language boundary | `crates/talos-text/Cargo.toml` and I246 completion `9a3c1d86` | UI-neutral crate exists and is consumed by TUI/tools; `code-intelligence` still statically selects its declared Arborium language set, so on-demand provider loading remains unfulfilled |
| Arborium consumer ownership | `crates/talos-tui/Cargo.toml`, `crates/talos-tools/Cargo.toml`, `crates/talos-text/Cargo.toml` | Parser ownership is centralized in `talos-text`; no claim of dynamic loading or binary-size reduction |

#### #466 acceptance disposition

| Requirement group | Disposition |
|---|---|
| Terminology ADR and ADR-027 preservation | Decision evidence: accepted ADR-072; ADR-027 remains authoritative |
| ADR-029/proposal supersession pointers | Added in this audit; historical ADR/proposal text is retained |
| Owner and Issue decomposition | Child Issues #508-#517 exist; CAP-001-G is tracked by executable Issue #519 with reciprocal Epic link; no child is activated |
| Capability/Provider descriptors | Existing I252 implementation evidence; no registry/resolver claim |
| Text/Language/Browser/Distribution implementations | Unfulfilled; each remains a separately governed child owner |
| Plugin source versus Bundle artifact layout | Current absence recorded above; implementation remains BUNDLE/DIST scope |
| Manifest migration matrix | ADR-072 records the boundary; I253 still owes concrete field/cache/contribution compatibility matrix before closure. Schema mutation and dual-read/controlled-write implementation remain BUNDLE-001 scope |
| Full product delivery in #466 stages 3-7 | Unfulfilled and intentionally not claimed by I253 |

The audit is evidence for governance closure preparation, not product completion. Remaining rows
must be resolved or explicitly owned before I253/CAP-001-G can become Complete/Closed.
This initial grouped disposition is not the required full bullet-by-bullet #466 acceptance matrix.

#### Full #466 acceptance matrix (2026-09-09)

The matrix below mirrors the Issue body in order. `Satisfied` means the governance artifact or
existing evidence is present; it does not mean a downstream runtime feature shipped.

| ID | Issue acceptance item | Disposition and repository evidence |
|---|---|---|
| T1 | Accepted ADR defines Capability, Provider, Plugin, Bundle, Carrier and Asset | Satisfied by accepted ADR-072, with definitions and lifecycle ownership in its Decision section. |
| T2 | ADR supersedes only the “Plugin is a package format” portion of ADR-029 | Satisfied by the dated supersession note in ADR-029 and the explicit preservation clause in ADR-072. |
| T3 | ADR-027 WASM runtime, sandbox and dylib conclusions remain intact | Satisfied by ADR-072 compatibility text and ADR-027 remaining Accepted; no code change is claimed. |
| T4 | Active documentation no longer uses Plugin and Bundle interchangeably | Satisfied for the audited active architecture/proposal/index surfaces; historical wording is retained with supersession notes. A future repository-wide terminology lint remains a residual. |
| T5 | Manifest compatibility and migration handling are decided | Satisfied at governance level by ADR-072 and the concrete matrix below; schema implementation remains BUNDLE-001. |
| O1 | CAP-001 Epic and stable child Story IDs exist | Satisfied by CAP-001 owner and its child map, Issues #508-#519 including executable CAP-001-G/#519. |
| O2 | At least the first child is independently implementable and Ready | Satisfied by CAP-001-B / #512 being Ready / Unclaimed; no activation is implied. |
| O3 | TOOL-008 trimming is separate from runtime Provider loading | Satisfied by the CAP-001 map and I246/talos-text evidence; parser trimming is not described as resolver delivery. |
| O4 | DIST-001 policy is separate from installation implementation | Satisfied by DIST-001-A / #509 and DIST-001-B / #515 child boundaries. |
| O5 | WEB-005 product/security ownership is separate from Browser connector implementation | Satisfied by WEB-005 retaining semantics and BROWSER-001 / #508 owning connector implementation. |
| O6 | Parent and child owner documents link in both directions | Partially satisfied: local parent links and child Required Reads are synchronized; remote parent/child Issue backlinks remain a required reconciliation action. |
| A1 | `talos-text` is UI-independent and does not aggregate all parsers by default | Partially satisfied: crate is UI-independent and optional, but enabling `code-intelligence` currently selects its declared static language features; dynamic provider loading is explicitly downstream. |
| A2 | TUI and symbol tools consume one Language Provider path | Partially satisfied: both depend on `talos-text`/the shared seam (I246 `9a3c1d86`); a provider registry and cross-consumer vertical slice remain LANG-001/LANG-002. |
| A3 | Browser discovery/loading is separate from TOOL-014 model disclosure | Satisfied as an ownership contract in ADR-072, TOOL-014 and BROWSER-001; no Browser connector is claimed. |
| A4 | Bundle installation is separate from Plugin activation | Satisfied by ADR-072 lifecycle and explicit “installation never implies activation”; implementation remains DIST/BUNDLE scope. |
| A5 | Provider registration is separate from Tool/schema exposure | Satisfied by ADR-072 and preserved TOOL-012/014 presentation policy; no automatic prompt exposure is claimed. |
| A6 | Missing optional capabilities have domain-specific safe fallback | Satisfied as an accepted contract in ADR-072; runtime fallback behavior remains downstream acceptance work. |
| A7 | Startup remains network-independent | Satisfied as an accepted ADR-072 invariant; no resolver or installer implementation is claimed. |
| R1 | Root `plugins/` semantics are runtime Plugin/Provider source | Satisfied by ADR-072 and architecture documentation; the directory is not yet present and no source is claimed. |
| R2 | Bundle source/artifact organization is separate | Satisfied by ADR-072 layout contract; no checked-in `bundles/` artifact is claimed. |
| R3 | Each executable Story has one Issue and one authoritative owner | Partially satisfied: #508-#517 map to child owners and CAP-001-G maps to #519, but legacy remote child bodies must be reconciled before their source/dependency facts can be treated as authoritative. |
| R4 | Backlog, Requirement Convergence and indexes are synchronized | Partially satisfied: local owner/index synchronization is in this candidate; remote reconciliation is a separate required action. |
| R5 | Board lists selected child work, not the whole Epic | Satisfied: Board mirrors I253 and lists child readiness without activating CAP-001. |
| R6 | Existing completed Stories retain original scope and evidence | Satisfied: I246/I252 and PLUGIN-001 evidence are referenced without reopening or broadening their status. |

#### Concrete manifest and package migration matrix

This is a governance contract for BUNDLE-001; it does not alter the current Rust schema. The
legacy parser facts are from `crates/talos-plugin/src/manifest.rs` and `registry.rs`.

| Existing surface | Target surface | Dual-read / controlled-write rule | Unknown and invalid data | Rollback boundary |
|---|---|---|---|---|
| `PluginManifest` | `BundleManifest` | Read legacy `PluginManifest` while schema version is supported; emit `BundleManifest` only after a separately accepted schema ADR and migration flag. | The current serde parser ignores unknown fields and does not retain them for round-trip; a target migration must either preserve unknown fields explicitly or reject unsupported target data before controlled write. Malformed or incompatible version values fail closed. | Revert the metadata adapter; do not mutate runtime state. |
| `PluginMetadata` | `BundleMetadata` | Map declared legacy fields in memory; keep legacy deserialization during the dual-read window. | Missing identity/version or invalid carrier is rejected. Current unknown metadata is not retained, so lossless preservation is a BUNDLE-001 implementation requirement, not a current fact. | Disable target serialization and continue legacy read-only interpretation. |
| `[plugin]` package root | `[bundle]` package root | Accept one legacy root or one target root, never merge both; controlled writes choose `[bundle]` only when schema version is explicit. | Duplicate roots and incompatible versions are rejected; current parser does not preserve unrelated top-level tables on serialization, so target preservation must be explicitly implemented or unsupported target tables rejected. | Restore the legacy root without deleting unknown tables. |
| `name` + `version` package identity | Bundle identity and version | Derive target identity from the same normalized name/version; do not silently rename or reuse an installed identity. | Normalization collisions and invalid semver fail closed. | Keep the old cache entry and invalidate only the attempted target entry. |
| Plugin installation/cache key | Bundle installation/cache key | During dual-read, look up legacy key first, then target key; write target key only after verified migration. | Cache metadata with mismatched identity, digest or schema is unusable, not silently repaired. | Remove only the newly written target metadata; legacy cache remains readable. |
| `artifact` executable path | Nested Plugin artifact declaration | Read legacy artifact relative to package root; target writes a nested plugin declaration after carrier/schema gate. | Absolute, escaping or unsupported-carrier paths are rejected; unknown declarations are not executed. | Keep the prior artifact mapping and do not activate a partially migrated plugin. |
| Top-level `skills` | Typed Bundle skill contributions | Preserve list values on read; controlled write emits typed contributions only after schema ADR. | The target schema must explicitly preserve unknown contribution fields or reject them before a controlled write; invalid paths/duplicates fail closed. | Restore legacy list without deleting unrelated contributions. |
| Top-level `tools` | Typed Bundle/Plugin capability contributions | Read existing tools as legacy contributions; target registration is opt-in and separate from disclosure. | Duplicate IDs, unsupported capability IDs or unsafe permissions are rejected. | Disable only the target contribution registration. |
| Top-level `hooks` | Typed Bundle hook contributions | Read legacy hooks; target writes explicit hook ownership and carrier after migration gate. | Invalid event/carrier/path is rejected; no silent hook execution. | Keep hooks inactive and restore legacy metadata. |
| Missing current `mcp` / `provides` fields | Future typed MCP/provides declarations | Do not infer fields from proposal examples; dual-read begins only after a future schema ADR. | The target schema must explicitly preserve unknown fields or reject them before a controlled write; neither case permits execution or registration before that gate. | Ignore target-only declarations and retain the original manifest bytes. |
| Schema/version marker | Versioned migration contract | Require an explicit schema version before controlled write; migration is atomic per manifest and records source version. | Unsupported versions fail closed; no best-effort downgrade. | Restore source bytes and source version marker. |

Required implementation follow-ups are therefore BUNDLE-001 (schema/manifest adapter and tests),
DIST-001-A/B (installation and consent), and CAP-001-B/C (registry and contribution boundaries).
The matrix is complete as a planning contract while those behaviors remain unimplemented.

#### Remote linkage and Issue reconciliation residual

Local owner links, acceptance references and the dependency DAG are authoritative for planning.
Remote child Issue bodies #508-#517 still contain pre-creation placeholder wording and several
obsolete dependency claims. Before I253 closeout, they must be reconciled from the authoritative
owners and parent #466 must carry the complete child Issue map, including #519. This is a remote
governance synchronization task, not child activation or implementation authority.

#### CAP-001-G Issue mapping checkpoint

#466 is the parent governance Epic and cannot also serve as this Story's executable Issue. The
distinct CAP-001-G Issue is #519, with the reciprocal parent link recorded in the Epic child map
and the Issue body. R3 is satisfied for Issue mapping; I253/CAP-001-G stays `Active / Claimed`
until the remaining audit evidence and synchronization are complete, and no child is activated by
this audit.

Claim evidence: independent [APPROVE](https://github.com/wjhuang88/talos/pull/518#issuecomment-5602756359),
[merge-time CAS](https://github.com/wjhuang88/talos/pull/518#issuecomment-5602797551),
and [Issue synchronization](https://github.com/wjhuang88/talos/issues/466#issuecomment-5602843043).
The execution branch `docs/i253-capability-governance-audit` starts at the claim merge.
No runtime files have changed. Continue locally; do not push this incomplete audit as a candidate.

### 2026-09-09 Executable Issue Mapping Checkpoint

Executable Issue [#519](https://github.com/wjhuang88/talos/issues/519) now tracks CAP-001-G;
#466 remains its parent Epic. This supersedes the preceding pending-Issue statements without
changing the published selection baseline or claim scope. The parent owner child map and
current source fields identify #519; the remaining semantic acceptance audit is still required
before I253 can be treated as fully complete.
I253 remains Active/Claimed; no implementation child is activated.

Remote reciprocal reference: parent #466 comment
[5603561909](https://github.com/wjhuang88/talos/issues/466#issuecomment-5603561909)
links #519; #519 names this iteration, the Story owner and parent #466. Current Board,
backlog, iteration index, requirement map and manifest now identify executable Issue #519.
Local checks run after Issue mapping: project governance and exact-base collaboration
validators both exited 0 with 0 warnings; diff check passed. No Rust/Cargo checks were
run for this documentation-only audit. These are local checks, not exact-head CI evidence.
Resume on `docs/i253-capability-governance-audit` at claim merge `53fc57b3`: finish the
T4/O6/R4 semantic audit and distinguish accepted target contracts from current behavior,
preserve Published Baseline and dated claim checkpoints, then validate and submit one
stable candidate for independent review. The audit is not yet Complete or submitted.

### 2026-09-09 Intake And Audit Follow-Up

Issue #520 is registered separately under INTEGRATION-001, Intake/Unclaimed, with no selected
iteration or implementation authority. Its two contract questions do not expand I253 or
BROWSER-001. The local backlog and Issue matrix include #519/#520; global remote reconciliation
is still required before submission of this governance candidate.

The earlier Child Issue Map is historical, as already stated in Claim Review Disposition.
For current scheduling use the child owners: TEXT-001 requires CAP-001-A/ADR-072 and consumer
ownership preparation; LANG-001 requires TEXT-001/CAP-001-A; BUNDLE-001 requires ADR-072,
manifest compatibility evidence and its migration decision. LANG-002 requires LANG-001,
CAP-001-C and DIST-001-A. These superseding facts do not alter the published baseline.

Local review added child Required Reads and identified stale remote child source/dependency
fields and missing full parent Issue backlinks. O6/R3/R4 remain partial pending reconciliation;
the earlier R3-satisfied sentence refers only to creation of #519, not full remote convergence.
Do not push this incomplete audit as a stable candidate or mark I253 Complete.
