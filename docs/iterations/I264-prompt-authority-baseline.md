# Iteration I264: Prompt Authority Architecture Baseline

| Field | Value |
|---|---|
| Status | Planned / Unclaimed |
| Source | PROMPT-001 / Issue #285 |
| Deliverable | Source-backed inventory and accepted authority/precedence decision boundary for prompt inputs; no semantic prompt rewrite. |
| Depends On | ADR-033, current prompt/context/evolution/memory/plugin owners |
| Excludes | No Rust prompt behavior changes, provider changes, SDK breaking change, or model-harness implementation. |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Source Issue | #285 |
| Claimed At | Not applicable |
| Governance Claim PR | #547 (merged; effective) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim-only architecture baseline; no prompt/runtime implementation authorized before merge. |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Baseline inventory complete; follow-up ADR and child slices required. |
| Implementation PR | Not started |

## Acceptance

- Inventory every current prompt contributor, scope, authority, provenance and cache behavior.
- Record conflicts and choose an ADR/migration boundary without changing runtime behavior.
- Decompose independently runnable authority, decomposition and behavior-harness children.
- Preserve current prompt assembly and SDK compatibility until later implementation claims.

Completion Commit: pending

## Source-backed contributor inventory (2026-09-13)

| Contributor | Source | Scope / authority | Provenance and cache behavior |
|---|---|---|---|
| Identity | `prompt/builder.rs`, `prompts/identity.txt` | Stable runtime identity; replaced only by explicit `custom_prompt` | Cacheable stable prefix; template variables are rendered during build |
| Tool definitions | `prompt/builder.rs` tool registry | Stable capability descriptions grouped by family | Cacheable; families and tools sorted deterministically |
| Skill index / activated skill | `prompt/builder.rs` | Capability discovery plus explicitly activated skill content | Cacheable; activation changes stable prompt content |
| AGENTS context | `context.rs` and loaded `AGENTS.md` files | Workspace and global instructions | Dynamic; filesystem provenance and ancestor order are retained |
| Memory | `with_memory_section`, `talos-memory` | Advisory recalled context | Dynamic and optional; ADR-033 keeps automatic associative injection disabled |
| Session todos | `with_todo_section` | Turn/session continuity guidance | Dynamic suffix |
| User preferences | `with_user_preferences` | User-provided preferences | Dynamic suffix |
| Runtime context | `prompt/builder.rs` | Current datetime | Dynamic and recomputed per build |
| Append/custom prompt | `with_append_prompt`, `with_custom_prompt` | Explicit caller input; custom replaces identity, append adds instructions | Dynamic; caller provenance is not yet represented as a typed authority |
| Evolution hooks | `talos-evolution` hook/adapter modules | Learned patterns and hook contributions | Hook-mediated; authority and cache effects require a dedicated child decision |

### Baseline findings and boundaries

The current builder has an explicit cacheable prefix (identity, tools, skills) and a dynamic
suffix (context, memory, todos, preferences, runtime and append). `custom_prompt` replaces only
the identity section, while append content is added after runtime context. This is an observed
assembly fact, not a new precedence decision. I264 records it for later ADR and child-slice work;
no runtime behavior is changed here.

Open decisions for subsequent slices are typed provenance/authority for each contributor,
nearest-scope AGENTS precedence, instruction-aware truncation, and scoped hook patches. Existing
memory safety boundaries and SDK compatibility remain unchanged until separately claimed.
