# ARCH-034-R04-AG12: SQLite Validator Policy Integrity

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | PR #184 review residuals 3-5 / validator policy linkage and diagnostics |
| Status | Complete |
| Priority | P2 |
| Selected Iteration | I185 (Complete) |
| Preserved behavior | ADR-008 accepted consumers, locked graph semantics, bundled SQLite behavior and standard governance results |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 architecture session 2026-08-11 |
| Work Slice | Implement only I185/ARCH-034-R04-AG12: replace the validator-local accepted-consumer constant with one versioned structured policy file that ADR-008 names as its exact machine-readable allowlist; preserve the five accepted consumers, workspace-boundary graph rule, bundled-feature/version/isolation checks and `cargo metadata --locked` invocation; document the current Cargo/toolchain/cache dependency as fail-closed without adding `--frozen` or offline fallback; preserve invalid-stdout rejection while rendering non-UTF-8 Cargo stderr with escaped offending bytes; add focused policy-drift and decoding fixtures; synchronize governance evidence only. No Rust, Cargo manifest/lock, SQLite consumer, runtime, schema, migration, timeout, retry, network-policy or AG-11 change. |
| Claimed At | 2026-08-11 |
| Source Issue | None |
| Governance Claim PR | #190 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim merge `5fe56fa8c0320dbb6a70443f19b16b388339ab5e`; PR #191 final head `45f70802bf3b593c6228d5a103dfcee351620920` passed CI `31556720252`, independent approval `5261491057` and merge-time CAS, then merged as `af9783229bfc8ee592813440ecfcdb6efc90a3c2`. |
| Implementation PR | #191 |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | None - AG-12 is complete; AG-11 and all other R04 children remain independent. |

Status: Complete

Completion Commit: `af9783229bfc8ee592813440ecfcdb6efc90a3c2`

## Closure Ledger

| Dimension | Record |
|---|---|
| Requested outcome | Close the validator-integrity architecture residual without changing Talos runtime behavior. |
| Artifacts to create/update | AG-12, I185, R04 child index, iteration/backlog indexes, Board and governance manifest; implementation later changes only ADR/reference policy, validator and fixtures. |
| Existing assets to preserve | I183/AG-7 completion evidence, ADR-008's five accepted consumers and boundary semantics, validator clean-run output, Cargo `--locked` behavior, AG-11 ownership and all Rust/Cargo/runtime state. |
| State/status owners | AG-12 and I185 first; R04 and derived indexes/views second. |
| Validation required | Controlled policy/metadata/diagnostic fixtures, both governance validators, architecture audit, scale assessment, locked release preflight, exact-head Unix/Windows CI, independent review and merge-time CAS. |
| Evidence and uncertainty | The duplicate allowlist, unstated host/cache contract and masked localized stderr are confirmed from ADR-008, the validator and reviews `5235077449`/`5235367999`. Implementation commit `74199395` adds the structured policy, fail-closed policy loading and escaped diagnostics without runtime files. No runtime defect is inferred from these governance-only findings. |
| Residual-work destination | AG-11 retains runtime SQLite containment evidence; all other R04 children retain their existing behavior/security boundaries. |

## Confirmed Baseline

- The validator's exact `ACCEPTED_CONSUMERS` set is checked against locked dependency reality, but
  ADR-008 clause 2 is not parsed or mechanically compared with that set. A future accepted consumer
  change could update Cargo plus the constant while leaving the ADR prose stale.
- Standard project governance now invokes Python and `cargo metadata --locked`; the repository has
  not explicitly selected whether this host dependency should remain online-capable, use
  `--frozen`/offline behavior, or degrade when the toolchain/cache is unavailable.
- Explicit UTF-8 decoding is correct for Cargo JSON. A non-UTF-8 localized Cargo stderr is caught
  safely, but the decode failure can mask Cargo's actionable diagnostic.

## Scope And Acceptance

- Add one versioned structured policy file as the exact accepted-consumer source; ADR-008 names it
  normatively and the validator loads it instead of carrying a second literal allowlist.
- Keep `cargo metadata --locked` as the selected host/toolchain/cache contract. Missing Cargo,
  unavailable dependency resolution and cold-cache/network failure remain explicit fail-closed
  errors; I185 adds no `--frozen`, offline fallback or silent degradation.
- Preserve actionable Cargo stderr on decode anomalies while continuing to decode Cargo JSON as
  strict UTF-8 and fail closed on invalid metadata; undecodable stderr bytes are escaped rather
  than discarded or allowed to replace the primary Cargo failure.
- Add focused fixtures for missing/malformed/mismatched policy data, metadata drift, invalid stdout
  and non-UTF-8 stderr diagnostic behavior.

## Exclusions And Residuals

No consumer, dependency, manifest, runtime SQLite, schema, migration, timeout, retry or containment
change. AG-11 remains the separate runtime containment evidence owner.

## Minimum Validation

Focused validator fixtures on Unix and Windows, both governance validators, locked release
preflight, exact-head CI and independent review.

## Completion Evidence

- PR #191 final head `45f70802bf3b593c6228d5a103dfcee351620920` passed full CI
  `31556720252`, independent review `5261491057`, local release preflight and merge-time CAS.
- Merge `af9783229bfc8ee592813440ecfcdb6efc90a3c2` contains the structured policy, strict policy
  parser, fail-closed metadata handling, escaped Cargo diagnostics and 17 validator fixtures, with no
  Rust, Cargo manifest/lock, consumer or runtime change.
