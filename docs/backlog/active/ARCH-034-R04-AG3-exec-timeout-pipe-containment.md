# ARCH-034-R04-AG3: Exec Timeout Pipe Containment

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-3 / direct exec operation-deadline gap |
| Status | Ready — bounded liveness correction defined; claim and iteration required |
| Priority | P1 |
| Selected Iteration | None |
| Preserved behavior | Exec schemas, permission facets, command/pipe ordering, timeout values, markers, retained-output limits, and normal completion output |

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
| Handoff / Release Condition | Establish an effective child claim from current `main` after a fresh non-terminal inventory and overlap CAS. |

## Confirmed Baseline

`run_step_inner` kills and waits for the direct child when its timer expires, but
then awaits independent stdout/stderr reader tasks without a second bound. A
descendant that inherited either pipe can keep it open indefinitely. Bash already
returns at its absolute deadline and is compatibility evidence, not shared code
authority.

## Scope And Acceptance

- Make the existing timeout an operation deadline, including reader/stdin task
  joins after the direct child is killed.
- Preserve bounded output already observed before timeout and the existing
  `marker: timeout`/format contract.
- Add a Unix grandchild-held-pipe fixture that returns within deadline plus a
  deterministic margin; keep numeric timeout coverage cross-platform.
- Cover single step, sequential/parallel steps and pipe-chain ownership without
  leaking Tokio tasks.
- Preserve byte-for-byte normal completion output and error classification.

## Exclusions And Residuals

No process-group/Job Object implementation, descendant-kill guarantee, rlimit
parity, background jobs, schema change or permission change. TOOL-024 owns
supervised process trees; this child only prevents inherited pipes from defeating
the existing foreground deadline.

## Minimum Validation

Focused exec tests, `cargo test --locked -p talos-tools exec_tool`, locked release
preflight, Unix/Windows exact-head CI and independent security review.
