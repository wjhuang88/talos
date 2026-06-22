# I041: Interactive Session Lifecycle & Operation-Scoped Permissions

> Document status: Active
> Published plan date: 2026-06-22
> Planned close date: 2026-07-20 (≈ 4 weeks)
> Planned objective: Talos gains interactive `/new` and `/resume` and `/fork` commands
>   through the SESSION-001-A runtime transition service, and the permission engine
>   switches to operation-scoped (ToolNature + resource) matching with a no-repeat-approval
>   UX for already-authorized resources.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `/new`, `/resume`, `/fork` slash commands; nature-based permission rules
>   that one Allow covers all Write/Network/Execute tools for the same resource; existing
>   tool-name-based rules continue to load and apply unchanged.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SESSION-001-B | SESSION-001 | Proposed | SESSION-001-A ✅ (I040), CMD-001 ✅ | `/new` and `/resume` slash commands consume SessionTransition; workspace-scoped resume candidates in deterministic order |
| SESSION-001-C | SESSION-001 | Proposed | SESSION-001-A ✅ (I040), CMD-001 ✅ | `/fork` slash command clones durable history into a child identity, source session remains byte-for-byte unchanged |
| PERM-002 | (root) | Refinement | PERM-001 ✅, ToolNature enum ✅ | Permission rules match on ToolNature + resource (path / domain) instead of tool name; "always approve" creates a scoped rule |

### Execution Order

```
Week 1 ── PERM-002 foundation
         • Resource extraction from tool input by nature
         • Nature + resource matcher; backward-compat shim for tool_name-only rules
         • Config format with `nature`, `resource`, `resource_kind`, `decision`
         • Migrate default rules in `crates/talos-permission/src/lib.rs` to nature form

Week 2 ── PERM-002 completion + TUI wiring
         • "Always approve" creates scoped rule (write path + host) not tool-wide rule
         • Tests: matcher, extractor, live approval scoping, config migration
         • Wire approval UI to surface the matched rule and the resource

Week 3 ── SESSION-001-B
         • `/new` and `/resume` registered through BuiltinCommand
         • Workspace-scoped resume candidate listing (MEM-004)
         • Consume SessionTransition::prepare(New) and SessionTransition::prepare(Resume)
         • Hydrate durable history + visible history on commit
         • Refusal / cancellation while a model/tool turn is active

Week 4 ── SESSION-001-C + iteration closure
         • `/fork` registered through BuiltinCommand
         • Clone durable history boundary into a distinct child identity
         • Activate child through SessionTransition; source bytes unchanged
         • Workspace verification + 5-agent review + retrospective
```

### Scope

**SESSION-001-B — `/new` and `/resume` Slash Commands**:
- Register `/new` and `/resume` via the existing BuiltinCommand registry (CMD-001)
- `/new` consumes `SessionTransition::prepare(New)`; commits on user confirmation
- `/resume` lists workspace-scoped candidates in deterministic order (most-recent first
  by default; explicit session ID override supported)
- Both commands refuse or queue safely while a model/tool turn is active
- Hydrate durable history and visible history from the selected target
- Preserve the old session on prepare failure

**SESSION-001-C — `/fork` Slash Command**:
- Register `/fork` via the BuiltinCommand registry
- Clone the durable history boundary into a distinct child session identity and persistence target
- Activate the child through `SessionTransition::prepare(Fork)` and hydrate its visible history
- Source session identity and bytes remain unchanged after fork activation
- Same refusal/queue rules as `/new` and `/resume`

**PERM-002 — Operation-Scoped Permission Rules**:
- `PermissionRule` gains `nature: ToolNature` and `resource: Option<String>` fields
- Resource extraction from tool input by nature (Read/Write → `path`; Network → host from `url`; Execute → command string)
- First-match-wins: nature → resource glob/domain
- Config format: `[[rules]] nature = "Write" resource = "src/**" resource_kind = "path" decision = "Allow"`
- Backward compatibility: tool_name-only rules continue to load; inferred nature for legacy rules
- "Always approve" in the approval dialog creates a scoped rule (write path + host)
  instead of a tool-wide rule
- Default rules in `crates/talos-permission/src/lib.rs` migrated to nature form
- Existing config files continue to work without user action

### Non-Goals

- Session deletion, rename, cross-workspace resume, model switching, merge/rebase
- Regex patterns for resources (glob is sufficient for v1)
- Runtime rule editing UI (TUI-008 remains Planned for a future iteration)
- Plugin-level permission rules (still tool-level through `AgentTool::nature()`)
- Per-tool permission metadata overrides (config-only path)

### Acceptance

#### SESSION-001-B
- Given an idle interactive session, when the user runs `/new`, then the next turn
  uses a fresh Agent context and persistence target while process-level configuration
  remains available.
- Given resumable sessions in two workspaces, when the user invokes `/resume`, then only
  active workspace candidates are selectable in deterministic order.
- Given target hydration fails, when resume is attempted, then the original session
  remains active and the user receives a visible error.
- Given a model/tool turn is active, when a lifecycle command is invoked, then the
  documented refusal or confirmed-cancellation policy is applied without racing state
  replacement.

#### SESSION-001-C
- Given a durable source session, when the user runs `/fork`, then Talos activates a
  distinct child id/path containing the intended source history.
- Given the child is active, when subsequent turns complete, then only the child
  persistence target changes and the source session remains byte-for-byte unchanged.
- Given fork preparation fails, when the operation returns, then the source remains
  active and usable.

#### PERM-002
- Given rules `[Write Allow src/**, Write Deny]`, when `write` is called for `Cargo.toml`,
  then decision is `Deny` (catch-all matches).
- Given rules `[Network Allow api.github.com, Network Ask]`, when `http_request` is called
  for `https://api.github.com/repos`, then decision is `Allow`.
- Given user presses `a` on approval for `write src/main.rs`, then a rule
  `Write` + Path `src/main.rs` → `Allow` is added; subsequent `write` calls to
  `src/main.rs` are auto-approved, and `write` calls to other paths still require approval.
- Given old config with `tool_name = "write" decision = "Ask"` (no resource), behavior is
  unchanged (tool-wide Ask).

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- SESSION-001-B: integration tests for `/new` and `/resume` end-to-end; refusal
  while-turn test
- SESSION-001-C: integration test that subsequent writes only touch child target
- PERM-002: matcher tests, extractor tests, live-approval scoping test,
  backward-compat config test
- Real `talos` binary smoke: `/new`, `/resume`, `/fork` and one nature-based
  allow-once-then-auto scenario

### Documentation To Update

- `README.md` — document `/new`, `/resume`, `/fork` and nature-based permission
  rules in Built-In Capabilities and Slash Commands
- Backlog stories: mark SESSION-001-B / SESSION-001-C / PERM-002 as Complete
- `docs/BOARD.md` — move I041 to Review, then Done This Cycle
- `docs/iterations/README.md` — add I041 entry; remove I041 from non-terminal inventory on Complete
- `AGENTS.md` Task Router — add PERM-002 implementation entry if it becomes a recurring route

### Risks And Rollback

- Risk: Resource extraction from `http_request` URL may mis-parse IDN or punycode hosts.
  Rollback: Add normalization test; fall back to `Ask` on extraction failure.
- Risk: `/resume` candidate ordering may produce non-deterministic results when multiple
  sessions share a timestamp.
  Rollback: Tie-break on session ID (lexicographic). Document ordering rule.
- Risk: "Always approve" scoping can over-grant if a user expects tool-wide but the engine
  now creates a per-resource rule.
  Rollback: Document the change in README and release notes. The first rule created by
  the new path covers the exact resource, not the whole tool.
- Risk: Backward-compat path for tool_name-only rules may be mis-classified by `infer_nature()`.
  Rollback: If `nature()` lookup fails, fall back to exact tool_name match (current behavior).
- Risk: Live binary smoke may fail if `/new`/`/resume`/`/fork` race with active streaming.
  Rollback: Document the explicit "turn must be idle" requirement; refuse with a clear message.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-22 | Activation | I040 Complete; SESSION-001-B/C Proposed → Active; PERM-002 Refinement → Active. I041 covers 3 stories over 4 weeks. |

## Verification Evidence

- `cargo check --workspace`:
- `cargo clippy --workspace -- -D warnings`:
- `cargo test --workspace`:
- Runtime evidence: `/new`, `/resume`, `/fork` end-to-end in mock TUI session; one
  nature-based allow-once-then-auto scenario captured

## Variance And Residuals

-

## Retrospective

- Outcome:
- Documentation:
- Lessons:
