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
| 2026-07-27 | Maintainer Decision | Maintainer explicitly authorized `discover_shared = true` as the Talos application default. `~/.agents/skills/` is the lowest-priority search root by default. Users can disable with `[skills] discover_shared = false`. This covers the shared-root default and runtime wiring only; it does NOT authorize default symlink following. Linked traversal policy remains safe-by-default (`follow_directory_links = false`). Implementation commit: `04999f1`. |
| 2026-07-27 | Config & runtime evidence | Commit `bfb8c22`: config deserialization matrix tests (missing/empty/explicit-true/explicit-false), runtime wiring tests (Config → discover_runtime_skills → SkillLoader), shared lowest-priority end-to-end test, README correction (moved shared discovery from "Not shipped yet" to "Currently shipped"). |
| 2026-07-27 | Traversal boundaries | Commit `b7e3704`: root symlink policy (symlink_metadata pre-walker, RootLinkDenied), pre-descent external rejection (WalkDir::filter_entry), canonical directory dedup at descent time, root-level SKILL.md fix, direct file symlink policy, DepthLimitReached warning (observation_depth = max_depth + 1), classify_walk_error/is_target_allowed extracted as free functions, RootLinkDenied warning kind added. |
| 2026-07-27 | Test authenticity | Commit `a0079dd`: real two-link chain, real cycle (follow=true, LinkLoop asserted), root link deny/allow, external pre-descent rejection, alias subtree single-descent proof, root-level SKILL.md, direct file link policy, depth boundary (exact/beyond/single-warning), unique-name budget, global budget across roots, misleading tests removed/rename, tiggers typo fixed. 78 skill tests pass. |
| 2026-07-27 | Oracle review + TOCTOU fix | Oracle review confirmed 8/10 questions correct. One MEDIUM-severity defense-in-depth gap: WalkDir `follow_root_links` defaults to `true`, creating a TOCTOU window where a swapped root symlink bypasses the explicit root-link check. Commit `e11481d`: added `.follow_root_links(false)` and pass `root_canonical` as walk root when root is an explicitly-allowed symlink. Two LOW-severity cosmetic issues (RootLinkDenied reused for file symlinks; inaccessible roots silently skipped) accepted as-is. 2508 workspace tests pass. |

### Maintainer-Authorized Shared Default

- **Decision date**: 2026-07-27.
- **Maintainer explicitly authorized**: `discover_shared = true` as the Talos application
  configuration default.
- **`~/.agents/skills/`** is added to the Talos application search root by default.
- **Users can disable** with `[skills] discover_shared = false`.
- **Shared root priority**: lowest. Workspace and Talos-native Skills retain precedence.
- **Scope exception**: This decision covers the shared-root default and runtime wiring only.
  It does NOT authorize default symlink following. Linked traversal policy remains
  safe-by-default.
- **04999f1 variance**: Commit `04999f1` implemented the shared default before this owner
  document was synchronized. This is a bounded governance variance: the maintainer gave
  explicit product direction, but the owner documentation lagged. This iteration
  (I163) is responsible for synchronizing the implementation and owner documents.
  This does not set a precedent for "implement-first-document-later" as a standard
  workflow.
- **Two-layer default**: Talos application defaults to shared discovery. Low-level
  `SkillLoader` constructors (`new`, `for_workspace`) do not implicitly opt embedders
  into HOME-based shared discovery unless the caller passes the application configuration.
- **discover_shared ≠ follow_directory_links**: `discover_shared` controls whether
  `~/.agents/skills/` is added as a search root. `follow_directory_links` controls
  whether filesystem links are followed. The two must not be conflated.
  `discover_shared = true` does not implicitly set `follow_directory_links = true`.
  `~/.agents/skills/` itself being a symlink must still pass the root-link policy.

### Governance Variance

1. Commit `1f5e451` was incorrectly tagged under TUI-035; it introduced unconditional
   `follow_links(true)` to SkillLoader. SKILL-003 owns all subsequent corrections.
2. Commit `ea3cc74` implemented the policy-controlled linked discovery before formal I163
   activation. The activation record is appended above for traceability.
3. Commit `04999f1` changed the shared discovery default based on explicit maintainer
   product direction. The owner documents (I163, SKILL-003, ADR-022, Program, Board)
   were not synchronized before the commit.
4. This iteration (I163) synchronizes the owner documents with the implementation. It
   does not modify Git history.
5. "Implement first, document later" is not a standard workflow. This bounded variance
   is recorded for transparency; future work should synchronize owner documents before
   or in the same commit as implementation changes.

### Security Residual: Level-2 Reference Paths

The `SkillManager::load_reference` function (`crates/talos-cli/src/skill_runtime.rs`)
validates that reference paths stay inside the active skill directory using
`Component::ParentDir` rejection and canonical-path containment. However:
- The reference path boundary check uses `canonical_candidate.starts_with(canonical_dir)`,
  which is a lexical prefix check, not a full trust-boundary proof.
- If the active skill directory itself contains symlinks that escape its canonical root,
  the containment check may not catch all escape vectors.
- The linked discovery policy (`SkillDiscoveryPolicy`) does not equate to reference
  containment — a skill loaded through a followed symlink may have references that
  resolve outside the logical skill directory.
- This iteration does NOT claim this risk is fully resolved. A follow-up security
  Story should audit `load_reference` for symlink-aware containment and document the
  trust boundary precisely.

### Windows Directory-Link Status

- **Unix symbolic links**: automated tests cover real symlinks (two-link chain, cycle,
  alias, broken link, external target, root link, file link). All `#[cfg(unix)]` tests
  pass on macOS/Linux.
- **Windows directory symlinks**: unverified. No Windows CI evidence exists. Windows
  directory symlinks require elevated privileges or developer mode; behavior may differ
  from Unix symlinks in WalkDir.
- **Windows junctions**: unverified. Junctions are a Windows-specific reparse point;
  WalkDir's `follow_links` behavior with junctions is not tested in this workspace.
- This iteration does NOT claim "cross-platform evidence complete" or "Windows junction
  supported and verified". The Windows status remains honestly documented as unverified.

### Public API Compatibility

- `SkillLoader` is a public struct with public fields (`skills`, `search_paths`,
  `discover_shared`, `workspace_root`, `discovery_policy`, `discovery_warnings`).
  Existing constructor-based usage (`new`, `for_workspace`, `for_workspace_with_options`,
  `for_workspace_with_discovery_policy`) remains compatible.
- External struct-literal construction (e.g., `SkillLoader { skills: vec![], ... }`) is
  source-incompatible if a new field is added, because the struct is not `#[non_exhaustive]`.
  The crate is pre-1.0 (v0.5.0) and does not claim stable 1.0 struct-literal compatibility.
- `SkillDiscoveryWarningKind` gained a new variant `RootLinkDenied`. This is an additive
  change; exhaustive matches on this enum will need a new arm. Pre-1.0 semver allows this.
- A future API-hardening Story may make `SkillLoader` fields private or mark the type
  `#[non_exhaustive]`. This iteration does not perform that refactor.
