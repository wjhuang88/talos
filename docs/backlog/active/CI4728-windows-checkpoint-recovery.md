# CI4728 Windows checkpoint history recovery

**Status**: Active (proposed upon claim merge)
**Selected Iteration**: None (regression maintenance)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Diagnose and correct Windows durable checkpoint history preparation and delivery failures in the registered actor test. |
| Claimed At | 2026-11-28 |
| Source Issue | None |
| Governance Claim PR | #1053 |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending independent review of this claim. |
| Implementation PR | Not started |
| Last Updated | 2026-11-28 |
| Handoff / Release Condition | Reproduce the specific failure, validate the correction on native Windows, and hand off residual failure modes with evidence. |

## Evidence and acceptance

The original PR #1034 job failed while consuming an old user search result. Local full-fixture testing passed ten times per side. Hosted Windows also reproduces on instrumented main. One restart reported linear index build failure and subsequent unavailability. A separate baseline restart obtained query results before delivery refused one result. The shared external error alone does not establish a common internal cause. [Original job](https://github.com/wjhuang88/talos/actions/runs/46118283216/job/140896945223), [instrumented PR failure](https://github.com/wjhuang88/talos/actions/runs/46123829691), [instrumented baseline failure](https://github.com/wjhuang88/talos/actions/runs/46125021272).

Retain the full fixture, existing deadlines, cancellation, source identity and permission checks. Capture a failing regression before implementation, verify that it passes after the fix, and repeat the exact actor test on hosted Windows. Record distinct failure modes independently. Update existing checkpoint user documentation if recovery behavior changes.
