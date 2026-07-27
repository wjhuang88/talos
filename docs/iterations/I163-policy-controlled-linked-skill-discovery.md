# Iteration I163: Policy-Controlled Linked Skill Discovery

> Document status: Active
> Published plan date: 2026-07-27
> Planned objective: Correct SKILL-003 linked skill discovery — safe-by-default policy, pre-descent external target rejection, canonical dedup, global budget, deterministic ordering, observable warnings.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: SkillDiscoveryPolicy with safe defaults; no unconditional follow_links(true); canonical directory+file dedup; global entry budget; deterministic first-wins; structured warnings for all variants.
> Activation rule: this iteration is not implementation authority until the Activation Record is appended below.

## Published Baseline

### Selected Stories

| Story | Parent | Status | Activation Gate | Deliverable |
|---|---|---|---|---|
| SKILL-003 | None | Refinement → I163 activation | I156 Complete (2026-07-27); no conflicting Active iteration | Policy-controlled linked discovery with safe defaults |

### Dependencies

- I156 Complete (confirmed 2026-07-27, `6909675`)
- ADR-022 (agent config compatibility boundary)
- No conflicting Active iteration

### Scope

Fix the 15 blocking defects from commit `1f5e451`/`ea3cc74`:
1. Unconditional follow_links(true) → policy-controlled
2. External target checked after traversal → pre-descent rejection
3. Root-level SKILL.md skipped by canonical dedup → fix
4. Alias subtree not deduplicated → canonical directory entry tracking
5. Non-deterministic ordering → sort_by_file_name
6. PermissionDenied unreachable → fix classification
7. DepthLimitReached unreachable → active boundary check
8. Entry budget per-root not global → global break
9. Budget test confounded by name dedup → use unique names
10. No real two-link chain test → create real symlinks
11. Old misleading cycle test → fix or remove
12. No Windows evidence → honest documentation
13. Public struct source break → document accurately
14. SKILL-003 has no iteration → I163 provides authority
15. ea3cc74 mixed TUI-036/TUI-037 → variance recorded

### Forbidden Changes

- No new crate or dependency
- No `unsafe`
- No renderer rewrite
- No transcript/export/session changes
- No publish/tag/release

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-27 | Activation | I156 Complete; no Active iteration conflicts; SKILL-003 Refinement → I163. Baseline `be77b18`. Primary executor: `glm-5.2`. |
