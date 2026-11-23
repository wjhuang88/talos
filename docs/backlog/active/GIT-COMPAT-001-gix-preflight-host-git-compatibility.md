# GIT-COMPAT-001: Targeted Older-Git Preflight Query Compatibility

**Status:** In Progress / Claimed — #920 merged to main `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80`; implementation and ADR-089 amendment not yet accepted.

**Type:** Git Tools / Security-Preserving Compatibility Repair

**Source Issue:** [#919](https://github.com/wjhuang88/talos/issues/919)

**Source design:** [gix-native system-Git preflight proposal](../../proposals/gix-preflight-system-git-compatibility.md)

**Related authority:** ADR-089, I300, completed I309 (gix 0.90.0), GIT-SNAPSHOT-001 (separate Review owner), Windows native report of 2026-11-10.

## Outcome

**Maintainer correction (2026-11-19): retain existing authorized Git CLI preflight queries that are supported by older Git; replace only specifically proven unsupported actions, initially `git var GIT_ATTR_SYSTEM` and `git var GIT_ATTR_GLOBAL`.** Use bounded gix/Talos-native replacements over admitted evidence, not a whole-preflight rewrite. Maintain exact prepared Read/Command/Remote facets, immutable sources, executable binding, stale-plan rejection and the selected system Git as final writer. Git 2.36.1 Windows Checkout/Commit acceptance is still failed.

## Bounded scope (2026-11-19 effective maintainer correction)

1. **Command-level compatibility inventory**: keep current ADR-089-authorized Git CLI inspection commands when actually supported; record exact Git 2.36.1/native output before declaring another CLI query unsupported. The 66 attribute-path failures identify `git var GIT_ATTR_SYSTEM/GLOBAL` as the first replacement pair, while five config-root failures require more direct diagnosis.
2. **Narrow replacement only**: implement an audited gix/Talos source-evidence substitute for these unavailable attribute-path queries, with exact selected-writer semantics. A path name is not authority to stat/open/read; preserve missing/empty sources, ancestor and symlink identity, explicit exact Read, budget and revalidation. Unknown compiled attribute defaults must fail closed **for the affected operation**, not block all other approved CLI discovery.
3. **Retain permitted CLI**: preserve known-compatible bounded `git config`, `git --exec-path`, `git check-attr`, ref/branch and other inspected operations under their existing argv, permissions and receipts. Revisit only when a specific old-version incompatibility is independently demonstrated. No unauthorized new CLI probing, broad version/Scoop heuristics or hidden gix process/file I/O.
4. **Separate environment and semantic issues**: characterize Windows/Scoop `GIT_INSTALL_ROOT` without blanket `GIT_*` admission; separately triage URL rewrite, tracking, receiver and fixture failures. Existing hooks/filters, transport, maintenance and permission effects remain.
5. **Native conformance**: prove the replaced query outcomes against Git 2.36.1 and newer; detect unauthorized new helper children while allowing retained approved CLI; verify full 17 MiB Checkout/Commit and mutation-before-write refusal.
6. Preserve exact-head CI, independent security/API/ADR review on **changed** authority, native Windows acceptance and accurate user documentation before Complete.

## Exclusions and dependency gates

- Preserve I300's completed acceptance and ADR-089 security invariants. **ADR-089 currently names fixed Git CLI discovery commands**, so replacing those commands needs an explicit reviewed amendment/maintainer architecture decision *before* production implementation; no unreviewed permission change is authorized.
- Does not own GIT-SNAPSHOT-001's 16MiB/digest/auxiliary budget fixes, TUI-APPROVAL-001, PERM-007, process-hardening, sandbox, search, provider, Server or other Git operation expansion.
- No replacing system Git with gix; no new dependency/version bump/unsafe; no automatic fallback from missing admission.
- Implementation may start only after effective governance claim and activation on main, recorded no-overlap CAS and any necessary ADR decision.

## Acceptance and verification

- Instrumented test sentinel proves **no unapproved Git/helper children in the new query replacement**. Previously approved Git CLI inspections may continue; the bound system Git still performs final writes. Unauthorized/missing attribute source evidence fails closed.
- Real Git 2.36.1.windows.1: native Git tools tests and large checkout/commit pass in original Scoop environment, with any remaining failures independently classified; newer Git regression matrix passes.
- Changes after plan creation (config, missing source created, executable or symlink path, relevant environment/source) refuse before mutation. Malicious/unknown config, executable hooks/filters and budget failures remain fail closed.
- `cargo fmt --all -- --check`, locked tool tests, workspace preflight, both governance validators, Windows native CI, exact-head checks and independent Git authorization/security/API review pass. No `Complete` without existing implementation SHA.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | GIT-COMPAT-001 / #919 I324 only: preserve supported authorized CLI preflight; replace verified unsupported Git 2.36 query forms (initially git var GIT_ATTR_SYSTEM/GLOBAL) with audited, narrow substitutes; preserve exact Read/executable/facet/revalidation and system-Git final writes; no snapshot/TUI work |
| Claimed At | 2026-11-11 |
| Source Issue | #919 |
| Governance Claim PR | #920 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim PR #920 merged as `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80` after exact-head CI #4123 (`43248451893`) passed remote owner reconciliation and both governance validators (0 warnings). Single-maintainer documentation-only claim; **no** source implementation, accepted ADR amendment or independent Git security/API review is implied. |
| Implementation PR | Draft #928 (T0 source/API audit only; no production implementation yet) |
| Last Updated | 2026-11-20 |
| Handoff / Release Condition | Independent security/API review and native Windows acceptance before delivery; exact-head CI and governance CAS for claim/implementation. |

## Delivery evidence

2026-11-11: Issue #919 records the maintainer's request to implement the existing proposal. This #920 governance baseline is documentation only. No production code, platform test pass, activated claim or completed compatibility behavior is asserted.

## Historical architecture clarification — 2026-11-12 (SUPERSEDED)

Historical 2026-11-12 direction, **superseded on 2026-11-19**: the maintainer had clarified that **gix must replace Git CLI for preflight**, not that Talos should add hardcoded version/installation compatibility branches. This supersedes the original proposal's suggestion to use host probes. The final system Git writer and Talos admission/stale-evidence checks remain unchanged. No CLI preflight fallback is approved. Any `gix::config_path` or related convenience API that internally invokes Git or captures global ambient state is disallowed in this plan unless reworked into a pure, per-invocation, externally admitted interface. Open decision: how to prove compiled installation default attribute paths match the selected system Git without a host subprocess; if unprovable, stop safely instead of guessing. See Issue #919.

## T0 implementation follow-up — 2026-11-12

The governance claim is effective on `main@067dd3d4`. The [source/API audit](../../reference/GIT-COMPAT-001-I324-GIX-PREFLIGHT-AUDIT.md) on implementation branch inventories all preflight CLI families and the existing `target_config::Sources` staged exact-Read implementation. The gix 0.90.0 `Repository::attribute_global_paths` API exists but **may read ambient config**; `gix::config_path` may spawn Git. Neither API may be blindly called before admission. Installation-compiled defaults remain an unproven binding and must fail closed until an independently reviewed source of truth exists. ADR-089 amendment remains Proposed, not Accepted; production behavior change is gated. No implementation SHA/native pass/review evidence yet.

## Native validation follow-up — 2026-11-17

Claim #920 is effective; Draft implementation #928 and Proposed ADR review #931
remain unmerged. [Native Windows evidence](../../reference/WINDOWS-VALIDATION-2026-11-17-I324.md)
tests #928 at `71c923b95e4aff29ac7a7d71703d26f8d42ff530` with Rust 1.99.0 MSVC
and Git 2.36.1.windows.1: candidate primitives 12/12 pass; full Git suite in the
original Scoop environment is 272 passed / 113 failed. Removing only
`GIT_INSTALL_ROOT` in the test process yields 305 passed / 80 failed. The controlled
17 MiB stale-source rejection passes; actual large Checkout/Commit still fails
attribute discovery. Unix-only sentinel/gitdir tests supply no Windows coverage.
Formatting and full preflight/governance gates did not pass (details and raw
evidence linked above). No production replacement, compatibility acceptance,
independent security approval or completion is claimed. Preserve the published
scope and existing claim; remaining acceptance is open under #919.

## Current maintainer correction — 2026-11-19

The latest [Issue #919](https://github.com/wjhuang88/talos/issues/919) is authoritative: this is a **query-level older-Git compatibility repair**, not a full gix-native, zero-Git-CLI preflight conversion. The revised [ADR-089 proposal PR #931](https://github.com/wjhuang88/talos/pull/931) remains Proposed / NOT ACCEPTED. Existing test-only pure-gix work in Draft #928 is retained as reusable evidence; it does not authorize eliminating approved CLI, changing production policy or declaring native Git 2.36 acceptance.


## 2026-11-20 target-tree compatibility finding

A second independently documented old-Git-incompatible **query option**, `git check-attr --source=<tree-ish>`, was introduced in Git 2.40. The existing I324 prepared Checkout target analysis uses `--source=<target.commit>` in addition to a separate, older-compatible `--cached` query. They cannot be interchanged when committed target, staged index and worktree differ. See [T0h audit](../../reference/GIT-COMPAT-001-I324-GIX-PREFLIGHT-AUDIT.md) and [Git 2.40 release notes](https://github.com/git/git/blob/master/Documentation/RelNotes/2.40.0.adoc). **Current scope remains command/option-level compatibility, not all-CLI removal**: retain normal/cached check-attr where supported; investigate an admitted, byte-fed gix target-tree substitute. Test-only candidate is on Draft #928, with no accepted runtime authorization or native Git 2.36.1 Checkout/Commit success. Additional `git var` config/shell names still require exact native classification.

## 2026-11-21 native runbook feedback

[Native report](../../reference/WINDOWS-VALIDATION-2026-11-21-I324.md) for #928
`2bbd3c3441ea14e3ac853ced8101de6424463cf8`: official pinned MinGit and selected
Scoop Git 2.36.1 both classify all five var queries unsupported (129), normal/cached
attributes working, source option unsupported (129). Initial scripts failed only
at fixture cleanup; bounded read-only-file cleanup repair reran both modes to
exit 0. Format passes; 15/15 gix candidates pass including Windows gitdir identity;
17 MiB actual Checkout/Commit still fails preparation in the original environment
on the Git-variable guard (101). Raw logs remain local TEMP. Scope stays targeted;
no product acceptance, production authority change, ADR acceptance or Complete.

## Maintainer decision — 2026-11-23: implementation direction approved in principle

The maintainer explicitly decided in the I324 handoff conversation:

> 原则同意 ADR-089 I324 局部兼容替换技术方案，仅授权在现有安全边界内继续开发与验证，不代表最终 ADR 接受、生产启用或 PR 合并。

**Authorization classification:** technical direction and continued **development/verification** only. This does **not** change the effective Accepted ADR-089 contract or authorize any production policy/permission expansion. The requested design is the 2026-11-19 narrowly scoped correction, **not** a whole-preflight CLI removal. Retain supported fixed-argv CLI inspections under their existing authority; investigate/review only the six forms demonstrated unsupported in native Git 2.36.1: `git var GIT_ATTR_SYSTEM`, `GIT_ATTR_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_GLOBAL`, `GIT_SHELL_PATH`, and target-tree `git check-attr --source`. Configuration-root, attribute-root, shell and tree attribute replacements require independent source/effect/identity review, not one blanket approval.

**No new authority:** selected system Git remains the final bound writer. Preserve exact Path/ancestor/link metadata and object Read admission, negative-source and compiled-default provenance, explicit Command/Remote facets, hooks/filters/receiver effects, one-use approval and stale-plan rejection. Unknown defaults fail closed per operation; no unapproved gix I/O/process, guessed Git installation root, blanket `GIT_*` exception or new CLI command may be inferred. The inherited Scoop `GIT_INSTALL_ROOT` investigation is separate.

**Status and gates:** ADR-089 amendment [Draft #931](https://github.com/wjhuang88/talos/pull/931) stays **PROPOSED / NOT ACCEPTED**; implementation [Draft #928](https://github.com/wjhuang88/talos/pull/928) remains unmerged and #919 remains Open. As of this checkpoint, [exact-head CI #4542](https://github.com/wjhuang88/talos/actions/runs/45116883372) and [ADR CI #4543](https://github.com/wjhuang88/talos/actions/runs/45116887147) are green on their then-current heads, but neither CI nor approval-in-principle substitutes for independent Git authorization/security/API review, trustworthy operation-specific source provenance, native Windows Git 2.36.1 original-environment >16 MiB Prepared Checkout/Commit acceptance, subsequent exact-head CI/governance checks, or the maintainer's separate final acceptance. Do not activate or merge implementation until those gates are satisfied.

Decision receipt: [#919](https://github.com/wjhuang88/talos/issues/919#issuecomment-7093132593), [#928](https://github.com/wjhuang88/talos/pull/928#issuecomment-7093133239), [#931](https://github.com/wjhuang88/talos/pull/931#issuecomment-7093131969).
