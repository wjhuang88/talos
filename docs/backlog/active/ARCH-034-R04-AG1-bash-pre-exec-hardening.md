# ARCH-034-R04-AG1: Bash Pre-Exec Hardening

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-1 / Unix Bash native hardening boundary |
| Status | Refinement — fail-closed post-fork contract and ADR-007 amendment required |
| Priority | P1 |
| Selected Iteration | None |
| Preserved behavior | Bash schema, permission decision, shell identity, output text/order, timeout, environment filtering, and configured resource-limit values |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Independent security review required |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Accept a corrected ADR-007 contract and an independently reviewed claim before touching production `pre_exec` or `unsafe` code. |

## Confirmed Baseline

Unix `BashTool` prepares dangerous names before `pre_exec`, then calls
`libc::unsetenv` and three `libc::setrlimit` operations inside the closure. All
native return values are ignored and the closure always returns `Ok(())`, so a
failed hardening call can still execute the requested shell. ADR-007 incorrectly
describes this shipped site as planned and claims `unsetenv` is in the normative
POSIX async-signal-safe set.

## Scope And Acceptance

- Decide which environment changes must occur through safe pre-spawn
  `Command::env_remove` configuration and which operations may remain post-fork.
- Keep the post-fork closure allocation-, lock-, formatting- and panic-free.
- Fail the spawn deterministically when any required native limit cannot be
  applied; never execute a less-hardened child silently.
- Correct ADR-007's shipped-site inventory and async-signal-safety claims before
  implementation merge.
- Add controlled Unix fixtures for native failure, environment removal and all
  three child limits without mutating the Talos parent process.
- Preserve Windows behavior and all Bash permission, output and timeout contracts.

## Exclusions And Residuals

No new native dependency, broad process supervisor, process-tree cleanup,
permission change, exec-tool rlimit parity or parent-process hardening. AG-2 owns
the parent API; AG-3 and TOOL-024 own their distinct liveness/supervision scopes.

## Minimum Validation

Focused Unix tests, `cargo test --locked -p talos-tools bash_tool`, locked release
preflight, Unix/Windows CI, both governance validators, ADR index validation and
independent security review of the exact implementation head.
