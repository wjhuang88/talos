# Iteration I278: Repository-Root Plugin Source Delivery

> Document status: Review
> Published plan date: 2026-09-18
> Planned objective: Deliver the omitted repository-root plugins/ source and independent build/package path from #466.
> Baseline rule: preserve this objective and acceptance; append execution facts.
> MVP deliverable: build an optional Plugin/Provider from root plugins/ source, package/install it as a verified Bundle and execute its real input-dependent capability.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | CAP-001-D real root Rust/Python Plugin sources, independent build/package, compatible bounded language transport and real-consumer validation under ADR-079; no Desktop or default-distribution change |
| Claimed At | 2026-09-18 |
| Source Issue | #466 |
| Governance Claim PR | #575 |
| Authorization Mode | Independent review |
| Authorization Evidence | #575 merged as 2d4e064f; exact head dbddf44a / base d6b566be; CI 35359610144 passed scoped checks; independent security/API/governance APPROVE 5731895048 and CAS 5731908465 |
| Implementation PR | Not started |
| Last Updated | 2026-09-19 |
| Handoff / Release Condition | Effective claim #575 / 2d4e064f; ADR-079 and all baseline acceptance mandatory before closure; I277/#29 human/device rows remain Deferred, non-blocking and unverified |

## Published Baseline

Selected Story: [CAP-001-D](../backlog/active/CAP-001-D-root-plugin-source-layout.md).
Dependencies: existing CAP-001-B/C, language contracts, Bundle/manual installation and
ADR-027/072/073. Verify current target-branch dependency evidence and effective claim before code.

Scope: inventory concrete providers versus fixtures; implement root source organization and
independent build/package entrypoints; exercise installed artifacts; update affected consumers,
tests and layout documentation. Host infrastructure remains in `crates/talos-plugin/`.

Acceptance: every CAP-001-D acceptance item is mandatory. Empty directories, source relocation
without executable packaging, and canned fixture results cannot establish completion. Prove
input-dependent capability behavior, safe failure, dependency isolation and reference consistency.

Non-goals: Desktop, new Browser capabilities, marketplace, implicit activation/acquisition,
default distribution changes, release or publication. Preserve published prior iteration evidence.

Validation: focused build/package/install/consumer and failure tests; locked preflight for code;
governance validators and whitespace checks; stable-candidate CI and appropriate independent review.
Documentation: Plugin build/package/usage guide, host README, architecture layout, ADR-072,
Story/iteration and owner-first derived indexes.

Risk: existing artifacts may prove only a test seam, not distributable implementation. Inventory
this before moving files and implement the missing bounded delivery rather than relabel fixtures.
Rollback: revert the source/build migration without changing installation identity or host policy.

## Scheduling And Inventory — 2026-09-18

The maintainer explicitly prioritizes this omitted requirement for the next iteration cycle.
The local iteration-header inventory found I277 Review and I249 Planned, with no Active or
Blocked current header. I164 remains Paused under its superseded layout target. Completed-owner
historical Review checkpoints are not new active claims.

| Owner | Disposition |
|---|---|
| I277 | Continue current Desktop corrections and machine/review/merge gates first. Retain deferred human validation in #29; do not fabricate Complete. |
| I249 | Keep published dependency pilot Planned/Unclaimed; not selected ahead of I278. |
| I164 | Keep Paused; no reactivation or layout work authorized. |
| I278 | Planned/Unclaimed, first next implementation cycle; establish target-branch claim at handoff. |

No plugin implementation begins in the I277 branch. Keep this planning change separate from the
Desktop implementation commit/candidate. Reuse #466 as source; no per-subtask Issue is required.

Evidence correction: I253's R1 `Satisfied` label explicitly excluded source delivery. Its cited
ADR-072/architecture layout contract is not explicit in the current files. This plan owns both
the missing contract clarification and implementation; the historical label is not acceptance.

## 2026-09-18 Source And ABI Investigation

Requested outcome: complete CAP-001-D source/build/package/install/real-consumer delivery.
Preserve the Published Baseline, existing legacy fixtures, I277 deferral evidence and unrelated
owners. Synchronize I278/CAP-001-D first, then parent/manifest/Board/indexes. Required evidence
remains focused real-artifact tests, locked preflight, independent security/API review and CI.
Any unresolved delivery gap remains owned here rather than creating per-subtask Issues.

At `main@d6b566be`, no open PR overlaps this slice. I277 implementation #571/#573 is merged;
remaining human/device acceptance is explicitly Deferred by the maintainer and non-blocking.
I249 stays Planned/Unclaimed and I164 Paused. I278 remains Planned/Unclaimed pending its atomic
claim and decision review; read-only investigation grants no implementation authority.

| Existing artifact | Verified behavior | Disposition |
|---|---|---|
| `crates/talos-plugin/tests/fixtures/language-provider/provider.wat` | Constant zero response, plain-text fallback | Keep as a negative/empty-response compatibility fixture; not production Rust capability. |
| `crates/talos-plugin/tests/fixtures/language-provider-python/provider.wat` | Same constant response | Keep as an installation/lifecycle fixture; not production Python capability. |
| Inline WAT response fixtures in `crates/talos-plugin/src/wasm.rs` | Canned protocol responses | Preserve decoder tests; add separate real-artifact consumer evidence. |
| Built-in `talos-text` language implementation | Actual statically linked source processing | Preserve current defaults; no broad parser migration. |
| `crates/talos-plugin` | Host load/install/lifecycle/transport infrastructure | Keep in `crates/`; concrete optional sources belong under root `plugins/`. |

The legacy host writes at guest address zero and lacks store memory/table limits in the language
execution path; response copying also precedes its size check. These are verified integration gaps,
not evidence of a deployed escape. Real Rust guest delivery requires an explicit buffer contract
and bounded execution before claiming safety. [ADR-079](../decisions/079-rust-language-plugin-guest-buffer-boundary.md)
proposes an additive allocator handshake and only guest symbol-export unsafe attributes; no raw
pointer memory operations. Independent security/API review is required before acceptance.

Actual consumer evidence must use the shared TUI highlight and symbol-tool paths. The current print
registration path discards the language context, so it is not existing language-consumer evidence.
The implementation must account for every affected path without presenting fixtures as real work.

### Decision Review And Claim Preparation

Independent Agent-role security/API review of ADR-079 initially requested two changes: bound
admission/version probing as well as execution, and reject new guests on old hosts before any
request memory write. The corrected proposal adds ABI v2 with mandatory allocation, retains
new-host/v1 support, and explicitly covers public probe callers and artifact start/version rules.
Reviewer `/root/i278_abi_decision_review` issued APPROVE against base `d6b566be25205a5abe1202dabeb0cee09ea81cce`
and ADR content blob `1a6182b425741a0f6da0825b69c4ab8595241e72`. Shared workspace/account;
Agent-role separation only. This is decision evidence, not implementation safety acceptance.
The draft governance candidate will bind the actual claim PR before becoming reviewable.

### Atomic Claim Proposal — PR #575

PR #575 proposes Active / Claimed and ADR-079 acceptance together. Neither is effective until
the finalized candidate reaches `main` after exact-head CI, independent security/API/governance
approval and merge-time CAS. Implementation must start from that merge or a later main commit.
The maintainer's standing single-maintainer/unattended instruction delegates independent review
to a separate Agent role; it does not waive protected review or claim a separate human identity.

I277's #571/#573 machine/review/merge gates have passed; its remaining manual/device checks are
Deferred and stay with I277/#29. I278 does not modify Desktop code, its runtime binding or its
acceptance baseline. I249 remains Planned/Unclaimed; I164 remains Paused. Existing I277 local/
remote branches are predecessor evidence, not competing Plugin claims. The PR list was empty
before opening #575. Source #466 remains the closed architecture parent; CAP-001-D owns this
recovered implementation requirement without reopening or broadening historical completion.

Expected implementation inventory: root `plugins/` sources/build/package guide and lockfile;
bounded language transport and tests in `crates/talos-plugin`; additive protocol support in
`crates/talos-text`; real consumer integration tests in TUI/tools/runtime/CLI only as needed;
scoped build/CI wiring; ADR-072/architecture/layout and owner evidence. Runtime default dependency
expansion, native loading, permission policy, Desktop and new Browser capability are excluded.
Any newly discovered incompatible public API or unsafe-memory need requires decision review,
not an implementation-time exception. Local convergence precedes the stable implementation PR.

### Activation Checkpoint — 2026-09-18

PR #575 merged at `2d4e064f94a7c54d65444dde195c3069a8509ddd` after exact head
`dbddf44abf3908639ff1cc971c2d1c8c1b870d68` / base
`d6b566be25205a5abe1202dabeb0cee09ea81cce`, CI `35359610144` (four successful checks,
Windows Rust/Linux Desktop skipped under verified docs-only routing), independent Agent-role
APPROVE [5731895048](https://github.com/wjhuang88/talos/pull/575#issuecomment-5731895048), and
merge-time CAS [5731908465](https://github.com/wjhuang88/talos/pull/575#issuecomment-5731908465).
Both governance validators passed with zero warnings. ADR-079 and the claim are now effective;
earlier proposal wording records the pre-merge phase, not current authority.
The sole implementation branch `impl/i278-root-plugin-sources` starts exactly at that merge.
No implementation PR is open. Converge locally before pushing a stable candidate.

### Local Implementation Checkpoint — 2026-09-18

First host transport changes are local only: ABI-v2 allocation with v1 compatibility, bounded
admission/start/version/allocator/run, memory/table ceilings, copy-before-limit ordering corrected,
guest Unicode span validation, dependency compilation panic containment and per-store deadline
custody. Independent early audit found a lost-final-epoch-tick race; the timer now repeats its tick
after expiration until completion, with an explicit regression that absorbs its first tick.
Generic Tool execution shares this guard because its engine may be shared with language calls;
retain its fuel-to-Timeout mapping and verify the newly enforced memory/table ceilings.

Validation so far: locked Plugin check passed; 71 Plugin unit tests passed with both wasm-only and
wasm/code-intelligence features; strict library Clippy passed. Strict all-target Clippy exposed
30 pre-existing test `unwrap()` diagnostics in `install.rs` and `resolution.rs`; these are not
waived and remain to fix within local convergence before final checks. Later code edits require
fresh focused validation; these results do not certify the final candidate.

Rust 1.97.0 `wasm32-unknown-unknown` standard library is installed. Current package discovery found
`syn` 3.0.6 and `rustpython-parser` 0.4.0; no new dependency or guest source has yet been added.
Real Rust/Python sources, independently locked builds, packaging, installed-artifact consumer
tests, path/reference docs, final preflight/security/API review and implementation/closure remain
unfinished. Resume in this sole implementation branch; do not create another claim or subtask PR.
Disk remains constrained after safe removal of incremental caches and six obsolete test/build
executables; preserve standalone CLI/Desktop acceptance binaries and avoid duplicate target trees.

### Parser Reuse Correction — 2026-09-19

The first uncommitted guest experiment chose syn/RustPython without proving that the existing
Tree-sitter stack could not build for WASM. The maintainer challenged this substitution. Arborium
2.18.2 explicitly supports WASM; both Rust/Python guests now compile with the original grammars,
and symbol visitors are compiled directly from the existing talos-text source-only modules.
No parser replacement is required. The standalone lockfile no longer includes RustPython;
syn remains only as a transitive procedural-macro build dependency. The guest uses the host's
fuel/epoch deadline rather than unavailable wasm32-unknown-unknown wall-clock APIs.

Locked WASM release builds passed; four reused native symbol-query tests passed. Actual host ABI
execution rejects both guests at the existing 1,000,000 fuel budget. An explicit diagnostic
1,000,000,000 budget permits Rust highlight and Unicode/comment-sensitive symbol operations;
Python then exposes overlapping Arborium captures rejected by the stricter guest span check.
These are unresolved delivery gaps, not a reason to replace Tree-sitter or claim completion.
Measure initialization/request costs and reconcile captures with the actual shared renderer
semantics before installed-consumer acceptance. Global production budgets remain unchanged.
The local verify_language_guests example reproduces this evidence; it is not yet package/install
or full consumer acceptance. No implementation candidate has been pushed.

### Real Artifact / Installation Checkpoint — 2026-09-19

Independent local resource-policy review confirmed finite language fuel adjustment fits ADR-079,
but found that first-capture deduplication lost Arborium's pattern priority. The guest now calls
Arborium's own `spans_to_flat_tokens`, preserving pattern metadata through normalization and
mapping its canonical theme tags to wire captures. Native tests assert function-name styling,
nested strings, Unicode boundaries and comment-excluded Python symbols. Six guest tests pass.

Cold Wasmtime measurements on this macOS host: Rust tiny highlight ~897M fuel / 56ms;
13KiB/1,000 declarations ~1.054B / 73ms; Python 22KiB/1,000 declarations ~260M / 25ms.
All four symbol operations passed those fixtures. Near-256KiB comment-heavy inputs exercise
bounded response fallback and fuel exhaustion in reference/import traversal; these cases are
not claimed as successful full analysis. Language default fuel is now 2B, with the same 500ms
deadline and explicit smaller-limit behavior; generic Tool fuel is unchanged. Host regression
coverage includes an infinite run with u64::MAX fuel terminated by the real wall deadline.
The wasm/code-intelligence Plugin library suite passed 73 tests after this change.

The independent Rust packager uses current sha2 0.11.0 / wasmparser 0.259.0, validates no imports,
no start section and constant ABI-v2 version, and writes actual SHA-256 plus matching versioned
Bundle and legacy activation descriptors. It refuses an existing output directory. Real Rust
and Python packages were installed through `install_bundle`, explicitly loaded/initialized/
activated, then exercised through SharedLanguageProvider with two different inputs each.
Pre-activation access was rejected, caller-supplied symbol file labels were preserved, and stop
revoked retained contexts. Verification-owned temporary installation directories were removed.

Commands: `verify_language_guests` (explicit 2B diagnostic budget),
`verify_installed_languages` (production default budget), and the locked Plugin library suite.
These prove real transport and installed shared-context behavior, not yet TUI-renderer or
AgentTool invocation acceptance. Remaining: real consumer integration, complete fault/package
tests, separate-per-language rebuild/repackage under the finalized lockfile, CI routing/default
dependency isolation, architecture/I253/layout docs, strict all-target Clippy cleanup, preflight,
final independent security/API review, stable candidate and owner-first closeout. No PR pushed.

### Consumer Acceptance Checkpoint — 2026-09-19

`bash scripts/validate_language_plugins.sh` passed its complete pipeline: separate locked
Rust/Python WASM builds, six guest tests, strict standalone-workspace Clippy, verified packaging,
real transport matrix, installed lifecycle, TUI segment/color reconstruction and all four actual
symbol AgentTools. Consumers check two distinct source states, comments/Unicode, canonical
keyword color, caller path attribution and post-stop rejection. The optional acceptance features
require real Bundle input and fail if it is absent; ordinary workspace tests do not silently
pretend those artifacts exist. CI runs this pipeline on the full-code route. New classifier
tests retain reduced validation for the two build-guide README files while source/lock/query
changes remain full-code; all 11 classifier tests passed.

Independent early review then requested narrower guest target cfg and stronger static package
checks. These now reject WASI exports, component encoding, memory64/shared memory, wrong ABI
signatures, imports, starts and nonconstant/incompatible version. The package input read itself
is bounded and incomplete writes produce an explicit recoverable error. The negative package
matrix and strict standalone Clippy passed after those edits. The installed diagnostic now also
checks missing/corrupt replacement preserves the existing installation; rerun this changed path.

Standard locked preflight completed check/Clippy and progressed through workspace tests, but
`talos-skill::tests::test_dedup_project_shadows_shared` hit sandbox OS PermissionDenied. The same
standard command is rerunning with explicit sandbox escalation; do not waive or call it green.
Log: `/private/tmp/talos-i278-preflight.log` (temporary observation, not durable evidence).

Disk exhaustion during TUI test compilation was resolved by preserving `target/debug/talos` and
`target/debug/talos-desktop-mock`, cleaning 12.5GiB of rebuildable root cache, restoring both
binaries and rebuilding with debug info/incremental disabled. TUI acceptance then passed; later
verification must use consistent local profile environment to avoid duplicate caches.

Architecture and ADR-072 now state the root source boundary; I253 has an appended R1 evidence
correction. Remaining work includes finalized-artifact rerun after packaging edits, permission/
failure evidence review, strict host all-target checks, successful preflight and final independent
exact-head review/candidate/merge/closeout. Print/inline still discard loaded language contexts:
do not claim those modes use the guest based on TUI/AgentTool acceptance. Review this affected
consumer disposition before final handoff. No remote implementation candidate is open.

### Stable Local Candidate — 2026-09-19

Standard locked `release_preflight.sh` completed successfully after rerunning outside the
filesystem sandbox; no test was waived. The finalized standalone build/package/installed-consumer
script passed, including missing/corrupt replacement preservation. Host all-target strict Clippy
passed. The subsequent CLI acceptance addition passed its focused test and strict Clippy: both
real installed language guests execute under production Agent Allow, while explicit Deny returns
an error and invokes the tool zero times. The counting delegate retains the tool's permission
profile, read-only property and family; it does not implement permission decisions. Production
error presentation normalizes the deny reason to `Permission denied: approval denied`.

Public usage documentation explicitly preserves the existing print/inline language-context
limitation. TUI/shared-context and four symbol tools have real installed-artifact evidence; no
all-mode or semantic name-resolution support is claimed. Local implementation is now Review /
Claimed pending stable-head independent permission/security/API review, remote CI and merge.

Changed-file inventory (all within the effective I278 slice):

- `plugins/`: independent locked guest/package workspace, real Rust/Python grammars, shared
  existing symbol queries, negative package matrix, source/build/usage guide and artifact ignores.
- `crates/talos-plugin/`: bounded ABI-v2/v1 host and regression tests, diagnostic examples,
  host guide, example dependencies and test-only unwrap cleanup needed by strict Clippy.
- `crates/talos-text/src/wasm_provider.rs`: compatible bounded language wire representation.
- `crates/talos-tools/`, `crates/talos-tui/`, `crates/talos-cli/`: opt-in real consumer tests and
  their manifests only (TUI test appended to highlight module); no production permission change.
- Root `Cargo.lock`, `.github/workflows/ci.yml`, language acceptance script and CI classifier/tests:
  optional test dependency links and source-aware acceptance routing.
- I278/CAP-001-D, CAP-001 parent, I253 appended correction, ADR-072/079 and decision index,
  Architecture, manifest, Board, Backlog and iteration index: owner-first delivery/layout evidence.

No Desktop source, I277 owner, default guest dependency, release/version/publication or new
permission policy is included. Generated WASM/Bundle/target outputs stay ignored. Remaining work:
stage/secret review, commit and exact-head independent review, stable candidate CI/CAS/merge,
then completion evidence and source-Issue handoff. No Completion Commit exists yet.
