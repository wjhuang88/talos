# Iteration I271: SDK Prompt Customization Boundary

> Document status: Complete / Closed

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | @wjhuang88 |
| Work Slice | SDK custom_prompt and hook prompt-contribution authority boundary |
| Claimed At | 2026-09-13 |
| Source Issue | #285 |
| Governance Claim PR | Direct commit f22fd5cb |
| Authorization Mode | Direct commit |
| Authorization Evidence | User-authorized continuation; SDK boundary only. |
| Implementation PR | Direct commits `5736d834`, `f8fac7b8` |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Excludes Memory, Todo/steering, Evolution, provider, UI, and release changes. |

## Scope

- Define and test how SDK custom and hook prompt contributions preserve runtime-owned authority.
- Keep explicit raw override compatibility as a separately documented escape hatch.

## Acceptance

- Ordinary customization cannot silently erase runtime safety/correctness invariants.
- Hook contributions are bounded and cannot replace the complete assembled prompt unintentionally.
- SDK documentation states the compatibility and override boundary.

## Completion Evidence

- Completion Commit: 5736d834, f8fac7b8

## Execution Checkpoint

- `dcffb640` documents the compatibility boundary for `custom_prompt`, `append_prompt`, and hooks.
- `5736d834` prevents hook modifications from erasing builder-owned Tools and Runtime Context sections; 33 prompt tests pass.
- `f8fac7b8` adds custom-prompt coverage for retaining runtime-owned sections; 34 prompt tests pass.
- I271 acceptance is complete for the bounded SDK/hook boundary; broader model-behavior coverage remains I270 scope.
- Production API enforcement and dedicated hook-boundary tests remain in scope.
