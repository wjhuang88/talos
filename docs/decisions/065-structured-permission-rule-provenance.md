# ADR-065: Encapsulated Permission Rules And Diagnostic Provenance

- Status: Accepted (effective when this exact reviewed status reaches `main`)
- Date: 2026-08-22
- Owners: PERM-006-A / I189
- Related: Issues #52 and #53; ADR-024; ADR-026; ADR-047; ADR-064

## Context

I189 must add one structured permission request/context/report evaluator while preserving every
existing `Allow`, `Ask`, `Deny(String)` result and compatibility-visible denial message. The report
must truthfully distinguish configured rules, runtime/session grants, built-in defaults, workspace
trust, workspace boundaries and nature fallback without exposing raw tool input, concrete resource
values, provenance names or free-form denial text.

The current representation cannot satisfy that contract:

- `PermissionEngine.rules`, `workspace_root` and `trusted_workspace` are public and downstream code
  can construct the engine with a struct literal;
- `PermissionRule` is the same public configuration DTO whether it came from JSON configuration,
  `add_rule`, the built-in defaults or `add_runtime_allow_rule`;
- `add_runtime_allow_rule` inserts an ordinary rule into the same `Vec<PermissionRule>` and retains
  no source identity;
- configured rules are prepended to that same vector, and public mutation can reorder or replace
  any entry.

Consequently, an evaluator cannot reconstruct truthful provenance from the current value. A hidden
sentinel in an otherwise ignored public field, structural guesses about rule position, or a global
side map would be spoofable or drift-prone. Adding a private provenance field to either public
struct would itself break downstream struct literals. The repository has already published v0.8.0,
so this pre-1.0 source break must be disclosed for a future v0.9.0-or-later publication; I189 does
not change the workspace version or publish a release.

## Decision

### 1. Encapsulate `PermissionEngine`

I189 may make the fields of `PermissionEngine` private and replace direct construction/mutation
with documented methods:

- `PermissionEngine::new()` retains the current five built-in default rules;
- `PermissionEngine::empty()` creates an engine without built-in rules for tests and embedders;
- `PermissionEngine::from_rules(Vec<PermissionRule>)` creates an engine whose supplied entries have
  explicit-rule provenance;
- `rules()` returns a read-only rule slice in evaluation order;
- existing workspace setters and `workspace_root()` remain; an additive trusted-workspace getter is
  provided;
- `add_rule`, `add_runtime_allow_rule` and `load_from_config` remain the only mutation paths needed
  by the current repository.

Existing serialized `PermissionRule` configuration, constructors and decision serialization remain
unchanged. `PermissionRule` itself remains a public DTO with its current fields; only direct
`PermissionEngine` field construction and mutation are removed.

### 2. Store source identity beside each rule

The engine stores ordered rules plus private, index-aligned metadata. Every insertion path assigns
one closed source class:

- `Default` for the five rules installed by `new()`;
- `Configured` for rules loaded from serialized configuration;
- `RuntimeGrant` only for rules inserted through `add_runtime_allow_rule`;
- `Explicit` for rules supplied through `from_rules` or `add_rule`.

Each entry receives an opaque, engine-local diagnostic identifier. Identifiers are unique and stable
for that engine entry across evaluation and unrelated later insertions. They are not authorization
tokens, are not persisted, do not encode resource values and cannot be supplied by configuration.
Removing and reloading rules may create new identifiers; consumers must not treat them as durable
foreign keys.

### 3. Keep authority and diagnostics separate

Rule source metadata never changes matching, precedence, grant lifetime or outcome. Existing
matching remains authoritative:

1. first matching rule within a facet;
2. explicit `Deny` before trusted-workspace handling;
3. trusted-workspace repo-contained `Write` behavior;
4. external concrete-path approval boundary, including exact scoped `Allow` reuse;
5. matched non-Deny rule;
6. existing nature fallback;
7. multi-facet aggregate `Deny > Ask > Allow`, preserving the first input-order Deny message for
   compatibility.

The structured report contains only closed outcome/reason/source enums, facet ordinal, nature,
resource kind/presence, opaque rule identifier, mode/interaction labels and a coarse tool-source
class. The compatibility decision retains the original Deny string privately and is excluded from
report serialization and safe `Debug`. Raw input, concrete resource and description strings,
workspace paths, MCP server names and plugin identity strings are never observer fields.

### 4. Mode is context, not policy in I189

I189 records runtime mode and interaction capability, but it does not resolve approval. It does not
convert `Ask` to `Allow` or a mode-specific `Deny`. CLI, Runtime and MCP composition roots retain
their current Ask handling and exact visible messages. PERM-006-C owns later single-pipeline
approval resolution; ADR-064 governs any later model-assisted resolver.

### 5. Hook compatibility remains additive and deferred

I189 does not add a field to `HookEvent::AfterPermissionCheck` and does not add a new
`HookEventKind`. Both would change a published public enum/constructor surface. Existing hooks keep
receiving the projected tool call and compatibility decision. The authoritative report is exposed
through the additive `talos-permission` API; PERM-006-C must select a separately reviewed additive
hook transport or a versioned hook migration when it converges the execution pipeline.

## Public API Migration

For downstream Rust users moving from v0.8.x to a future v0.9.0-or-later release:

| v0.8.x form | Replacement |
|---|---|
| `PermissionEngine { rules: Vec::new(), workspace_root: None, trusted_workspace: false }` | `PermissionEngine::empty()` |
| direct `engine.rules.push(rule)` | `engine.add_rule(rule)` or `engine.add_runtime_allow_rule(rule)` according to the real source |
| read `engine.rules` | `engine.rules()` |
| assign `engine.workspace_root` | `engine.set_workspace_root(root)` |
| assign `engine.trusted_workspace` | `engine.set_trusted_workspace(value)` |

No JSON/TOML permission configuration migration is required. No durable data is rewritten.

## Consequences

- The report can identify runtime grants without hidden markers or caller-controlled provenance.
- Direct Rust struct construction of `PermissionEngine` becomes a documented pre-1.0 source break.
- Downstream mutation becomes auditable and cannot silently desynchronize rule/source metadata.
- I189 must update all in-tree struct literals and add a public API migration note before completion.
- A future crates.io publication containing this change must use v0.9.0 or later and mention the
  migration; release/publish work remains outside I189.

## Acceptance Evidence

- Decision-content commit: `dae98460c29c72cb61da391ddf998630e67d6f15`.
- The status commit cannot self-certify the decision. Acceptance is effective only after the final
  exact PR head passes both governance validators, CI, independent Agent-role security/API review,
  merge-time CAS and target-branch merge.
- This acceptance authorizes only the I189 migration and structured evaluator already bounded by
  its effective claim. It does not authorize PERM-006-B/C/D/E, PERM-007 behavior, TOOL-024,
  version changes, release or crates.io publication.

## Rejected Alternatives

### Encode source in an ignored `PermissionRule` field

Rejected because fields are public, serialized and caller-controlled. A sentinel could be spoofed,
leak through diagnostics or collide with legitimate legacy values.

### Infer source from rule shape or vector position

Rejected because configured, runtime and explicit rules can have identical shapes, and public code
can reorder the vector. A plausible label is not truthful provenance.

### Store provenance in a global side map

Rejected because engine moves/clones/lifetimes would make identity fragile, global state would
complicate concurrency and tests, and stale entries could misattribute authority.

### Defer all provenance to PERM-006-C

Rejected because provenance is an explicit I189 acceptance condition and the later grant/pipeline
children require it as their typed foundation.

## Validation

- Compile-time migration of every in-tree `PermissionEngine` struct literal.
- Existing serialized permission configuration and `PermissionDecision` round-trip tests unchanged.
- Table tests for Default, Configured, RuntimeGrant, Explicit, WorkspaceTrust,
  WorkspaceBoundary and nature fallback sources.
- Compatibility equality for `evaluate`, `evaluate_with_nature` and `evaluate_profile`.
- All permutations of representative multi-facet profiles preserve aggregate severity; separate
  tests preserve first-Deny message compatibility.
- JSON and `Debug` negative tests with sentinel secrets in input, resource, description, Deny text,
  MCP server and plugin identity.
- Locked permission, agent, CLI, runtime, MCP, plugin and workspace validation.
- Exact-head independent security/API review before merge.

## Reversal Triggers

Revisit or supersede this ADR if:

- a non-breaking representation can prove truthful source identity without caller-controlled
  markers;
- a future durable audit requirement needs identifiers to survive restart;
- PERM-006-C requires a report transport that cannot remain additive at the hook boundary; or
- the migration removes a currently supported construction path without an equivalent method.
