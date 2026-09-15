# ADR-076: Language Provider Default Distribution Policy

**Status:** Accepted (LANG-003 / Issue #517; maintainer acceptance 2026-09-15)

## Context

LANG-002 proves the Rust provider contract and verified Bundle installation. Remaining language
migration must not silently enlarge the default binary or change existing `cargo build/run` scope.

## Decision (proposed)

1. The default Talos build keeps its current language behavior and does not download or activate
   providers at startup.
2. Additional language providers are distributed as verified Bundles and loaded only after an
   explicit user choice. Built-in providers require a separate measured change-control record.
3. Every migrated language must document provider identity, compatibility version, fallback to plain
   text, and its consumer coverage (highlighting and symbol/query where applicable).
4. Release evidence reports static parser footprint separately from provider/Bundle assets; size
   alone cannot justify removing an existing fallback.
5. Missing, corrupt, incompatible, timed-out or unavailable providers degrade to the documented
   plain-text/explicit-unavailable result without network-dependent startup.

## Consequences

Existing defaults remain stable. Distribution and migration can proceed incrementally, but each
language needs its own compatibility fixture and owner evidence. This ADR authorizes no code,
Cargo feature inversion, parser removal, or automatic download until accepted and claimed.

## Acceptance evidence required

- per-language provider and fallback matrix;
- default/no-default/release build comparison;
- offline verified-Bundle fixture;
- consumer conformance tests;
- explicit review of any default or persisted-format change.

## Supersession

This proposal refines LANG-003 under ADR-072 and does not supersede ADR-027 or ADR-073.
