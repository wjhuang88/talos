# Iteration I271: SDK Prompt Customization Boundary

> Document status: Partial / Open

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | @wjhuang88 |
| Work Slice | SDK custom_prompt and hook prompt-contribution authority boundary |
| Claimed At | 2026-09-13 |
| Source Issue | #285 |
| Governance Claim PR | Direct commit f22fd5cb |
| Authorization Mode | Direct commit |
| Authorization Evidence | User-authorized continuation; SDK boundary only. |
| Implementation PR | Not started |
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

- Completion Commit: pending (runtime enforcement and hook API tests remain)

## Execution Checkpoint

- `dcffb640` documents the compatibility boundary for `custom_prompt`, `append_prompt`, and hooks.
- `5736d834` prevents hook modifications from erasing builder-owned Tools and Runtime Context sections; 33 prompt tests pass.
- Production API enforcement and dedicated hook-boundary tests remain in scope.
