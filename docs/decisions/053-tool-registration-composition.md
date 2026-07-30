# ADR-053: Explicit Tool Registration Contributions And Product Composition

- Status: Accepted
- Date: 2026-07-26
- Accepted: 2026-07-31
- Owners: Architecture / runtime maintainers
- Related: ADR-006, ADR-021, ADR-024, ADR-026, ADR-052
- Story: ARCH-034-R01
- Decision gate: Satisfied on 2026-07-31; I158 may activate under this contract

## Context

Talos has a generic `AgentTool` contract and `ToolRegistry`, but the concrete built-in tool inventory
is assembled repeatedly in product code. Print, TUI, and MCP construction paths maintain overlapping
registration lists while also injecting mode-specific state and permission wrappers.

This creates several risks:

- a new tool is added to one mode but omitted from another;
- the same tool receives different wrappers or construction inputs by accident;
- duplicate tool names can silently replace an existing registration;
- SDK composition can diverge from CLI composition;
- ownership of factories and product policy is unclear;
- later Cargo feature gates cannot reliably control one authoritative capability inventory.

ADR-052 requires CLI and SDK to retain separate public entrypoints while sharing one internal
composition. ARCH-034 finding F01 requires one explicit tool registration model. A decision is needed
before implementation because the change introduces a cross-crate public or semi-public contract.

## Decision

Talos will use **explicit tool contributions selected by an outer composition root**.

```text
talos-core
  AgentTool
  ToolRegistry
  ToolContribution descriptor/factory contract
  deterministic duplicate-name diagnostics

tool-owning crates
  authoritative contribution declarations/factories
  no dependency on CLI/TUI
  no global registration

product/runtime composition roots
  explicit profile selection
  runtime dependency injection
  permission wrapping
  capability/feature selection
  final ToolRegistry construction
```

### Approved additive Rust contract

The first implementation uses an **instance contribution**, not a context-erasing service locator or a
factory trait in `talos-core`. Tool-owning crates expose ordinary, explicitly typed factory functions;
after the outer composition root supplies the required workspace/session/runtime values, each created
`Arc<dyn AgentTool>` is carried with a stable source identity:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToolContributionSource(String);

impl ToolContributionSource {
    pub fn new(source: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

#[derive(Clone)]
pub struct ToolContribution {
    source: ToolContributionSource,
    tool: Arc<dyn AgentTool>,
}

impl ToolContribution {
    pub fn new(
        source: ToolContributionSource,
        tool: Arc<dyn AgentTool>,
    ) -> Self;

    pub fn source(&self) -> &ToolContributionSource;
    pub fn name(&self) -> &str;
    pub fn tool(&self) -> &Arc<dyn AgentTool>;

    pub fn map_tool(
        self,
        wrap: impl FnOnce(Arc<dyn AgentTool>) -> Arc<dyn AgentTool>,
    ) -> Self;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "duplicate tool registration '{tool_name}': existing source '{existing_source}', incoming source '{incoming_source}'"
)]
pub struct ToolRegistrationError {
    pub tool_name: String,
    pub existing_source: ToolContributionSource,
    pub incoming_source: ToolContributionSource,
}

impl ToolRegistry {
    pub fn register_contribution(
        &mut self,
        contribution: ToolContribution,
    ) -> Result<(), ToolRegistrationError>;
}
```

`register_contribution` is transactional for one contribution: on collision it returns the exact error
above and leaves the existing registry entry unchanged. Source strings are stable, display-safe
identifiers such as `talos-tools:file`, `talos-tools:git`, `talos-session:todo`,
`talos-cli:mcp-status`, or `plugin:<package>@<version>`; they contain no workspace path, arguments,
credentials, or dynamically projected tool input.

The existing `ToolRegistry::register` remains temporarily source-compatible and keeps its historical
replacement behavior during I158 migration. New product composition and plugin registration MUST use
`register_contribution`; after all equivalence tests pass, a later pre-1.0 cleanup may deprecate or
narrow the unchecked method with release-note guidance.

The concrete first owner modules are:

- `talos-core/src/tool.rs`: contribution/source/error contracts and checked registry insertion;
- `talos-tools/src/contributions.rs`: workspace, network, shell, Git, image, and code-intelligence
  factory groups with explicit typed arguments;
- `talos-session/src/tool_contributions.rs`: session-bound todo factory group;
- `talos-cli/src/registry.rs`: print/TUI/MCP profiles, runtime injection, permission adapters,
  capability selection, plugin loading, and final registry assembly.

Scheduler tools remain explicit caller-supplied contributions until their current construction owner
is consolidated separately. MCP `status` remains a CLI/MCP-product contribution. This keeps dependency
direction acyclic: core knows only `AgentTool`; tool/session crates depend on core; CLI depends on all
three; no implementation crate depends on CLI/TUI.

### 1. Contract ownership

`talos-core` owns only generic contracts and diagnostics:

- `ToolContribution` or an equivalent additive registration type;
- stable source identity used in collision errors;
- a deterministic registration error that reports:
  - duplicate tool name;
  - existing contribution source;
  - incoming contribution source;
- generic `ToolRegistry` behavior.

`talos-core` MUST NOT own:

- a concrete built-in tool list;
- constructors for tools implemented in other crates;
- CLI/TUI mode policy;
- permission defaults;
- model capability detection;
- product configuration.

### 2. Contribution ownership

A crate that implements a built-in tool owns its authoritative contribution declaration/factory.

Expected ownership:

- `talos-tools`: file, search, git, shell, network/web, image, and code-intelligence tools;
- `talos-session`: session-bound todo tools; scheduler tools remain explicit caller-supplied contributions until their current owner is consolidated;
- `talos-plugin`: explicit plugin contribution compatibility, not hidden discovery.

A contribution receives only the construction context it actually needs. It does not inspect global
state or discover product mode implicitly.

### 3. Composition ownership

The outer product/runtime composition root owns:

- choosing which contribution groups are active;
- print/TUI/MCP/runtime profile differences;
- workspace/session/config/provider dependency injection;
- permission wrappers;
- sandbox integration;
- model-capability gating;
- final registry construction.

Permission wrapping remains outside tool-owning crates so the refactor cannot silently change
Allow/Ask/Deny behavior.

### 3.1 Behavior-equivalence matrix

I158 must preserve the following current product inventory and wrapper policy before deleting any old
builder. “Checked contribution” describes registration/collision handling only; it does not imply
permission approval.

| Contribution group | Print / inline / RPC | TUI | MCP | Wrapping and gating that must remain equivalent |
|---|---|---|---|---|
| Shell and direct exec | Included | Included | Included | Print uses blocking permission adapter with unresolved `Ask` denied; TUI uses channel-based approval; MCP keeps its current unwrapped registry behavior. |
| Snapshot-aware file tools | Included | Included | MCP keeps current ordinary file constructors | Existing workspace roots, exact external-path authorization, and mode-specific permission adapters remain unchanged. |
| Search, stat, diff, document extraction | Included | Included | Included | Preserve each current wrapper choice and workspace injection; no new permission default. |
| Code-intelligence tools | Included | Included | Included | Print/MCP and TUI wrapper differences are characterized and preserved until an explicit later permission change. |
| Git read and mutation tools | Included | Included | Included | Current read-tool versus mutation-tool wrapper selection remains byte-for-byte testable by tool name and behavior. |
| Network/web tools | Included | Included | Included | Existing permission profiles and wrapper selection remain unchanged. |
| `read_image` | Included in registry | Included in registry | Excluded | One authoritative `talos-tools` contribution; outer profile retains Supported-model presentation/capability gate and existing result projection/continuation behavior. |
| Todo tools | Ephemeral session-bound set | Durable current-session set | Excluded | Factory ownership moves to `talos-session`; current session IDs, storage roots, and permission wrapping remain unchanged. |
| Scheduler/delay tools | Explicit caller injection | Explicit caller injection | Excluded | Remain caller-supplied contributions; no global discovery. |
| MCP `status` | Excluded | Excluded | Included | CLI-owned MCP product contribution. |
| Explicit WASM plugins | Explicit package selection | Explicit package selection | Existing MCP behavior unchanged | Package loading stays explicit; checked registration reports built-in/plugin and plugin/plugin collisions with both source identities. |
| Presentation policy | Applied after executable registry construction | Applied after executable registry construction | Existing MCP exposure retained | A registered but unpresented tool remains non-executable to the model. |

Equivalence evidence must compare sorted tool-name sets for print, TUI, and MCP and exercise at least
one read-only tool plus one permission-gated tool through real product paths. Capability omission for
`read_image`, plugin collisions, duplicate built-ins, and composing the same profile twice are required
negative cases.

### 4. No hidden registration

The following approaches are rejected:

- linker inventory or static constructor registration;
- a global singleton registry;
- process-global mutable registration;
- implicit plugin/tool discovery during crate initialization;
- a global event bus used as a registration mechanism;
- a new composition crate without a separate demonstrated need and ADR.

### 5. Collision behavior

Duplicate names MUST fail deterministically. Normal product composition MUST NOT silently replace an
existing tool.

The error must contain both source identities. Tests must cover:

- duplicate built-in contributions;
- built-in versus plugin collision;
- duplicate plugin registration;
- same profile composed twice.

### 6. Migration strategy

The change is additive first:

1. introduce the contribution contract and diagnostics;
2. express the current tool inventory through contributions;
3. keep existing mode builders temporarily;
4. prove registry-set and permission-wrapper equivalence;
5. switch one mode at a time;
6. remove duplicate lists only after all equivalence tests pass.

Pre-1.0 public API changes require release notes and migration guidance. Existing external users must
not be forced onto an unstable product-specific builder.

### 7. Runtime SDK boundary

`talos-runtime` may use the shared composition implementation, but it remains the supported SDK
facade. `talos-agent` does not become a second SDK entrypoint.

This ADR does not itself introduce `RuntimePreset::coding()` or `SandboxFallbackPolicy`; those belong
to ARCH-031-C after shared composition exists.

## Alternatives Considered

### Keep the three product registration lists

Rejected because omission and wrapper drift remain likely and feature gating lacks one authoritative
inventory.

### Put all concrete tools in `talos-core`

Rejected because it reverses dependency direction and turns the protocol crate into a product
assembly crate.

### Create a `talos-composition` crate immediately

Rejected. ADR-052 does not authorize a new composition crate. Start with a focused module in an
existing owner. A separate crate requires evidence of independent reuse or dependency pressure.

### Global inventory/auto-registration

Rejected because hidden initialization obscures product policy, complicates tests, and weakens
permission/capability review.

### ToolRegistryBuilder owned entirely by CLI

Rejected as the long-term contract because runtime composition would still duplicate tool ownership.
A focused CLI builder may exist as an adapter during migration.

## Consequences

Positive:

- one authoritative declaration per built-in tool;
- explicit and testable product profiles;
- deterministic collision diagnostics;
- easier Cargo feature gating;
- CLI and runtime can share composition without sharing public entrypoints;
- adding a tool no longer requires editing several lists.

Costs:

- additive core API and migration work;
- temporary coexistence of old and new builders;
- more explicit construction context;
- cross-crate equivalence tests.

## Security And Safety Constraints

- permission defaults and evaluation order do not change;
- model presentation filtering remains authoritative;
- a registered but unpresented tool remains non-executable to the model;
- plugin loading remains explicit;
- no new `unsafe`;
- no native dependency is introduced by this decision;
- security-sensitive wrapper differences must be tested per mode.

## Acceptance For ADR Approval

Before changing Status to Accepted, reviewers must confirm:

- the exact proposed Rust type shapes are additive or have a migration plan;
- no dependency cycle is introduced;
- source identity and duplicate error semantics are precise;
- print/TUI/MCP equivalence scenarios are documented;
- permission wrapping and capability gating remain explicit;
- the initial implementation owner/module is named;
- no new composition crate is required.

## Approval Record

Architecture/maintainer review on 2026-07-31 confirmed:

- the contract is additive and retains the old unchecked API only as a temporary compatibility seam;
- source identity and duplicate diagnostics are deterministic and redaction-safe;
- tool factories stay in their implementing crates while product policy stays in the outer root;
- print, TUI, MCP, plugin, session-tool, scheduler, and `read_image` equivalence cases are explicit;
- permission wrapping and model capability gating remain outer composition decisions;
- the initial implementation modules are named and introduce no dependency cycle;
- no global registration, service locator, event bus, or new composition crate is authorized.

ADR-053 is therefore Accepted and satisfies the activation gate for ARCH-034-R01/I158.

## Reversal Triggers

Revisit this ADR if:

- contribution context becomes so broad that it is effectively a service locator;
- explicit composition creates unacceptable compile-time or API weight;
- independent consumers prove a dedicated composition crate is needed;
- plugin/runtime-discovered tools require a distinct protocol.
