# ARCH-034-R01: Tool Registration Composition Consolidation

| Field | Value |
| --- | --- |
| Story ID | ARCH-034-R01 |
| Type | Architecture / Technical Story |
| Parent Epic | ARCH-034 |
| Status | Refinement — ADR decision required before Ready |
| Priority | P1 |
| Source | ARCH-034-F01 audit finding; maintainer requirement, 2026-07-22 |
| Estimated effort | M (3–5 developer days, excluding review/acceptance) |
| Depends on | ARCH-034-A accepted audit; ADR-006; tool permission boundary |
| Blocks | Future tool additions, including follow-on registration of I154 `read_image` |

## Problem

`talos-core` already owns the generic `AgentTool` contract and empty `ToolRegistry`; it does not
hard-code concrete tool instances. The actual registration composition is fragmented in
`talos-cli/src/registry.rs`: print, TUI, and MCP builders each manually construct and register a
largely overlapping tool list, with mode-specific permission wrappers and runtime inputs mixed into
the same lists.

Adding one built-in tool therefore requires repeated edits across mode builders and makes omission,
inconsistent wrapping, inconsistent workspace/session construction, and silent name replacement
plausible. The audit records this as ARCH-034-F01.

## Goal / Value

Create one explicit, testable registration composition model:

- `talos-core` owns only generic registration contracts, descriptor/diagnostic types, and the
  registry—not concrete tool inventory or runtime-specific dependencies.
- Tool-owning crates actively contribute declared factories/descriptors through that contract.
- Product composition roots (`talos-cli`, and later `talos-runtime` where applicable) explicitly
  choose contributions by mode, inject runtime dependencies, and apply permission wrappers.
- A new tool has one authoritative built-in declaration and a testable mode/capability policy
  rather than three hand-maintained registration lists.

## Scope

- Define an ADR-backed, additive core registration/contribution contract and collision policy.
- Extract built-in tool declarations/factories from the three CLI registry builders into owning
  crates or focused crate-owned contribution modules.
- Make print, TUI, and MCP registries compose their selected tools through the same declarative
  inventory while retaining their deliberately different runtime context and permission wrappers.
- Preserve explicit plugin registration/loading and its collision protection.
- Provide registration diagnostics that identify the duplicate tool name and both contribution
  sources; normal product composition must reject accidental duplicate registrations rather than
  silently replace an existing tool.
- Add equivalence tests for the existing mode-specific registry sets and wrapper/permission
  behavior; document the extension path for a future built-in tool.

## Explicit Exclusions

- No global tool singleton, linker/inventory auto-discovery, static constructor, global event bus,
  or hidden side-effect registration.
- No `talos-core` dependency on `talos-tools`, `talos-agent`, `talos-cli`, TUI, permissions, or
  provider implementations.
- No behavior change to tool permission defaults, workspace trust, plugin policy, model tool
  presentation, MCP protocol, or runtime tool execution.
- No generic “tool configuration DSL,” dynamic loading redesign, plugin protocol expansion, or
  broad tool-list product changes.
- No implementation of I154 `read_image` as part of this story; I154 remains independently scoped.

## Design Direction To Validate In ADR

The preferred direction is explicit contribution, not runtime auto-registration:

```text
talos-core:  ToolRegistry + ToolContribution contract + collision diagnostics
      ↑
tool-owning crates: declared factories/descriptors, no CLI/TUI dependencies
      ↑
talos-cli/runtime: mode profile + runtime context + permission wrapper + final registry
```

Each contribution receives only the construction context it actually needs. The outer composition
root decides whether a contribution is applicable to print/TUI/MCP mode and wraps it with the
existing mode-appropriate permission adapter. Capability-dependent tools remain a composition
decision because model capability is runtime/session state, not a property a tool crate can safely
infer on its own.

The ADR must compare this with a narrowly scoped `ToolRegistryBuilder`, establish the public API
and pre-1.0 semver migration, decide whether descriptors and factories are one type or separate,
and reject any approach that relies on hidden global initialization.

## Affected Crates / Seams

- `talos-core`: registration contract, collision/error diagnostic, registry API only.
- `talos-tools`: file/git/network/symbol tool contributions and shared factory ownership.
- `talos-agent`: runtime-owned tools such as todo/scheduler contributions where applicable.
- `talos-cli`: TUI/print/MCP composition profiles, dependency injection, and permission wrappers.
- `talos-plugin`: explicit plugin contribution compatibility and duplicate-name behavior.
- `talos-runtime`: audit whether its embedded construction needs the same composition seam; do not
  add a dependency without an evidence-backed need.

## Required Reads

- `docs/reference/ARCHITECTURE.md` — Tool Presentation and crate boundaries
- `docs/reference/ARCHITECTURE-AUDIT-2026-07.md` — F01 and R01
- `docs/reference/ARCHITECTURE-AUDIT-2026-07-findings.json` — ARCH-034-F01
- `docs/backlog/active/ARCH-034-architecture-sustainability-audit.md`
- `docs/backlog/active/ARCH-034-B-finding-remediation-program.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/021-tool-call-protocol-architecture.md`
- `docs/decisions/026-multi-resource-tool-permissions.md`
- `crates/talos-core/src/tool.rs`
- `crates/talos-cli/src/registry.rs`
- `crates/talos-tools/src/lib.rs`

## Readiness Gate

This story becomes Ready only after a dedicated ADR records:

1. the additive core API and downstream migration path;
2. ownership of built-in factories/descriptors versus mode composition;
3. exact duplicate-name/source diagnostic behavior;
4. why plugin registration, permission wrapping, and model capability gating remain explicit;
5. the absence of cycles and global/implicit registration; and
6. a behavior-equivalence matrix for print, TUI, MCP, plugins, and runtime-owned tools.

## Acceptance

### Structural / technical

- [ ] `talos-core` remains free of concrete tool inventory and dependencies on implementation or
      product crates.
- [ ] Every built-in tool has one authoritative contribution declaration/factory, owned by its
      implementing crate or a clearly justified focused owner.
- [ ] Print, TUI, and MCP registries are assembled from that common inventory/profile mechanism;
      no three duplicated per-tool registration lists remain.
- [ ] Mode-specific runtime values and permission wrappers are supplied only at the outer
      composition root, preserving current Allow/Ask/Deny behavior.
- [ ] Duplicate names fail deterministically with both source identities; plugins retain explicit
      load/collision behavior.
- [ ] `ToolPresentationPolicy` continues to filter the final executable registry; a registered
      but unpresented tool remains non-executable to a model as it is today.
- [ ] I154 `read_image` can be added by one contribution declaration plus an explicit
      capability-gated composition rule, without copying a registration list.

### Regression / validation

- [ ] Snapshot or set-equivalence tests show that existing print, TUI, and MCP tool names remain
      unchanged unless an explicitly documented mode exclusion already exists.
- [ ] Tests cover wrapper selection, workspace/session injection, duplicate built-in detection,
      duplicate plugin detection, and capability-gated omission.
- [ ] Existing tool permission, plugin, MCP, and presentation tests remain green.
- [ ] `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace
      --locked -- -D warnings`, `cargo test --workspace --locked`, governance validation, and
      `git diff --check` pass.

## Documentation / State Owners

- Update `docs/reference/ARCHITECTURE.md` with the final composition model.
- Update the new ADR, this story, ARCH-034-F01 disposition, parent ARCH-034, product backlog, and
  Board when status changes.
- Update developer-facing tool-extension guidance to replace the obsolete “register in every
  builder” instruction in `TOOL-003` and related docs.
- User-facing README/site changes are not expected unless tool availability changes; record a
  residual rather than inventing user-facing documentation.

## Risks / Rollback

- Risk: a common factory hides mode-specific security differences. Mitigation: wrappers and mode
  selection stay outer-layer and have equivalence tests.
- Risk: public trait/API churn. Mitigation: ADR first, additive migration where possible, and a
  pre-1.0 minor release note if consumers must update.
- Rollback: retain the old builders until the new profile produces an equivalent registry in tests;
  revert the uncommitted migration if a mode/security discrepancy appears.

## Residual Destination

- A need for third-party/runtime-discovered tool loading belongs in a separate plugin/runtime
  proposal and ADR; it is not an implicit extension of this story.
