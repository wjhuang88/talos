# GIT-FEATURE-001: Restore standalone Git feature dependency on ignore

**Status**: Intake / Unclaimed
**Type**: Build regression / dependency feature closure
**Parent Epic**: GIT-001
**Source Issue**: #1041
**Selected Iteration**: None

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #1041 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-11-27 |
| Handoff / Release Condition | Establish effective claim, repair existing feature edge, verify isolated builds and native CI, integrate to main before resuming #1034. |

## Source evidence and ownership

Latest inspected main `0561e9f2d48f1699477248fcdd3008a7f676c47d`: `crates/talos-tools/src/git_write/status_state.rs:91` uses `ignore::gitignore::GitignoreBuilder`. The existing optional ignore 0.4.33 dependency is enabled via search but absent from git feature. Whole-workspace feature unification masks the missing edge. This is a regression in [GIT-001](GIT-001-embedded-git-tools.md), separate from I324, GIT-002 planning and PROVIDER-003-B.

[CI4678 attempt 1](https://github.com/wjhuang88/talos/actions/runs/45942616688/attempts/1) Windows job 140322136411 failed isolated Git compile with E0433; [attempt 2](https://github.com/wjhuang88/talos/actions/runs/45942616688/attempts/2) macOS job 140338043687 passed release preflight but failed STORE-002 E explicit local-storage feature check on the same E0433. Both on #1034 head f57ea8c9021bbcfe6990b7d346fede8bb1c1b952. B's 24 named provider tests passed on Windows and macOS; these are historical exact-head receipts, not overall CI success. Agent timing failures remain distinct; macOS attempt 2 passed 768/768, Windows timing failure not yet resolved.

## Proposed scope and acceptance

- Add the existing `dep:ignore` feature edge to `git` in `crates/talos-tools/Cargo.toml`. No new dependency version/package or lockfile change expected. Preserve Git status traversal, authorization, defaults, and native CI; no code/implementation authorized by intake.
- Run `cargo check -p talos-tools --no-default-features --features git --locked`, actual Windows Structured Git executable and path identity guard tests, and `cargo check -p talos-runtime --features local-storage --locked` plus actual STORE-002 E explicit-feature commands.
- Run applicable pinned locked preflight, native CI, both governance validators and independent exact-head review. Record all failures/skips and perform merge CAS. Do not weaken deadlines or disable gates.
- Integrate the independently reviewed correction to main before synchronizing #1034; run its fresh exact-head CI. Keep #1024 and #132 open until B review and completion.

## Required reads

[AGENT-COLLABORATION](../../sop/AGENT-COLLABORATION.md), [GIT-WORKFLOW](../../sop/GIT-WORKFLOW.md), [GIT-001](GIT-001-embedded-git-tools.md), [Git tools handoff](../../tasks/2026-11-27-git-tools-handoff.md), CI4678 logs and [#1041](https://github.com/wjhuang88/talos/issues/1041). Intake only; no effective claim or completion.
