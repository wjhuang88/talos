# CAP-001-D: Repository-Root Plugin Source Delivery

**Status**: Review / Claimed
**Type**: Technical delivery Story
**Parent Epic**: CAP-001 / #466
**Selected Iteration**: I278

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
| Handoff / Release Condition | Effective claim #575 / 2d4e064f; ADR-079 and full baseline acceptance apply; I277 deferred human/device checks do not block this slice |

## Goal And Scope

Restore the unimplemented Repository Layout Direction in #466. Maintainers must be able to
find and independently build Talos-supplied optional runtime Plugin/Provider implementations
under repository-root `plugins/`, then package and exercise the resulting artifact through the
existing verified Bundle installation and explicit activation path.

- Inventory existing shipped implementations, examples and test-only fixtures separately.
- Keep host infrastructure in `crates/talos-plugin/`; place concrete optional implementations
  under `plugins/<domain>/<provider>/`, beginning with existing Rust/Python language work.
- Provide source, manifests, reproducible build commands and Bundle packaging instructions.
  Do not promote a canned/no-op test fixture into a production Provider by relocating it.
- Update imports, fixture consumers, build checks and contributor documentation. Preserve
  intentional negative-test fixtures and remove only superseded duplicate implementation source.
- Restore explicit layout guidance in ADR-072/architecture documentation, and correct the
  I253 R1 evidence interpretation through an appended checkpoint, preserving historical records.

## Dependencies And Constraints

Use CAP-001-B/C, LANG-001/002/003, BUNDLE-001 and DIST-001-A contracts and implementation
evidence. ADR-027/072/073 remain authoritative. Source layout is distinct from installed
`.talos/plugins/` locations and Bundle artifact directories. Installation never grants activation
or permission. Optional implementations must not enter the default host binary through static
dependency expansion. Check current stable versions before introducing a dependency.

## Acceptance

- A fresh checkout contains actual independently buildable Plugin/Provider source in root
  `plugins/`, not only an empty directory, README, descriptor, or copied test payload.
- Build and package from that source; verify/install the resulting Bundle and explicitly load
  the Plugin using existing paths. Exercise input-dependent behavior through real consumers;
  record which providers are delivered and preserve existing compatibility behavior.
- Test missing/corrupt/incompatible artifacts and existing permission/sandbox boundaries.
- Prove default dependency isolation and update all affected source-path references.
- Record an explicit disposition for each inventoried implementation/fixture and every #466
  layout requirement. Future Browser/examples in the original illustrative tree are not grounds
  to claim those independent features shipped.
- Update user/contributor build, package and usage instructions and reconcile parent/child status.

## Exclusions

No new Browser connector, marketplace, automatic activation/download, default-distribution
policy change, broad parser migration, Desktop implementation, release or publication.

## Validation And Documentation

Run source/build/package checks, focused Plugin/Bundle/language and actual-consumer tests,
dependency isolation checks and required locked preflight for code changes. Obtain independent
security/API review where protected boundaries change. Documentation-only planning uses the two
governance validators and diff checks, not Rust compilation.

Documentation targets: root plugin build/packaging guide, `crates/talos-plugin/README.md`,
architecture layout section, ADR-072 clarification, I278, this owner and derived indexes.
Any newly discovered implementation gap remains explicit in I278; do not silently narrow acceptance.

### 2026-09-18 Implementation Readiness

Read-only I278 inventory confirms Rust/Python WASM fixtures are no-op modules, not movable real
implementations. I278 owns actual source delivery and the compatible guest-buffer integration
needed to execute it. [ADR-079](../../decisions/079-rust-language-plugin-guest-buffer-boundary.md)
is Proposed for that boundary; independent security/API decision review precedes acceptance and
the effective claim. No implementation has begun. I277's remaining human/device checks are Deferred
by the maintainer and do not block this slice.

PR #575 proposes this claim and activation atomically with ADR-079 acceptance after independent
Agent-role security/API review. These changes become effective only on target-branch merge;
no code has started. I278 records the full non-terminal inventory, predecessor evidence,
expected changed-file boundaries and implementation/validation sequence.

Activation checkpoint 2026-09-18: #575 merged as `2d4e064f94a7c54d65444dde195c3069a8509ddd`.
ADR-079 and this In Progress / Claimed state are effective. I278 records exact head/base, CI,
independent review and CAS. Implementation branch begins at that merge; no completion is claimed.

## Required Reads

Review checkpoint 2026-09-19: local source/build/package/install, real TUI/symbol consumers,
production Agent Allow/Deny, failure tests, strict Clippy and standard locked preflight passed.
I278 records the complete changed-file inventory and preserved print/inline limitation. Stable-head
independent security/API review, CI, merge and completion evidence remain pending.

- [Parent CAP-001](CAP-001-progressive-capability-provider-architecture.md) and Issue #466.
- [I278](../../iterations/I278-root-plugin-source-delivery.md) and I253 acceptance row R1.
- [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md).
- [LANG-002](LANG-002-rust-language-provider-vertical-slice.md),
  [LANG-003](LANG-003-language-provider-migration-default-distribution.md),
  [CAP-001-C](CAP-001-C-plugin-capability-carriers.md),
  [DIST-001-A](DIST-001-A-verified-manual-bundle-installation.md).
