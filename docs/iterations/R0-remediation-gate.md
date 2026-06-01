# R0: Remediation Gate

**Purpose**: Close known architecture, security, and session-correctness findings before I009 exposes
Talos through plugin/MCP/RPC extension surfaces.

## Status: PLANNED

R0 is a remediation round, not a product feature iteration. It exists because post-I008 diagnosis
found shipped-code gaps that should not be carried into the extensibility work.

## Selected Stories

- [ ] #ARCH-S1: Link sandbox `unsafe` blocks to ADR-007
- [ ] #ARCH-S2: Deprecate zero-security `Agent::new()`
- [ ] #ARCH-S3: Wire `ProcessHardening` into child execution
- [ ] #ARCH-S4: Unify duplicated live `ApprovalChoice` definitions
- [ ] #ARCH-S5: Keep the SQLite session index current on normal turns
- [ ] #ARCH-S7: Fix CLI search highlight output leaking literal `BOLD`
- [ ] Triage #ARCH-S6: repair interactive fork identity now, or defer to #I010-S7 if it touches run-path migration

## Execution Plan

1. Safety documentation and API guardrails: #ARCH-S1, #ARCH-S2.
2. Runtime security correction: #ARCH-S3.
3. Approval type cleanup with no event-loop migration: #ARCH-S4.
4. I006 session correctness: #ARCH-S5 and #ARCH-S7.
5. Fork triage: decide whether #ARCH-S6 is a contained session fix or part of the #I010-S7 AppServerSession migration.

## Acceptance Criteria

- [ ] No known security false-complete remains untracked.
- [ ] Bash subprocess hardening applies to the child process at runtime.
- [ ] Production paths no longer use the zero-security `Agent::new()` constructor.
- [ ] `talos -r` / `talos --search` include newly written normal sessions.
- [ ] Search highlight output never prints literal `BOLD`.
- [ ] #ARCH-S6 has an explicit execution target: R0 if self-contained, I010-S7 if it requires run-path migration.
- [ ] `cargo check --workspace` exits 0.
- [ ] `cargo test --workspace` exits 0.

## Verification Notes

Append command outputs and runtime evidence here as each story closes. Do not mark R0 complete based
only on unit tests when a finding is about runtime wiring.
