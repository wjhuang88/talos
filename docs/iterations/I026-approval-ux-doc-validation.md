# I026: Approval UX Fix + Documentation Validation

**Status**: Active
**Started**: 2026-06-17
**Depends On**: TOOL-002 (Approval bug documented), ARCH-002 (Phase 1 planned)

## Outcome

Fix the approval-tool call ordering bug so users can clearly see which tool call requires
approval. Then validate all project documentation against actual implementation to prepare
for ARCH-002 Phase 2 (architecture audit).

## Stories

### S1: Approval-Tool Call Ordering Fix

| Field | Value |
|-------|-------|
| Source | TOOL-002 bug section |
| Est | ~100 LOC |
| Priority | P1 |

**Problem**: Approval prompts appear after tool call lines with no visual association. In
multi-tool batches the user cannot tell which tool call the approval is for.

**Fix**: Push approval as inline scrollback lines directly after the tool call line, before
the tool executes. Each approval block shows:
```
→ write, path: poem.txt
  ⚠ Requires approval — press y/n/a
  ✓ approved
  ✓ wrote 921 bytes
```

**Scope**:
- `TuiPermissionAwareTool::execute()`: when approval is needed, send tool call display first,
  then approval request via the existing channel
- TUI: render approval as scrollback lines tied to the preceding tool call
- Approval response: push result line inline before tool execution proceeds
- Auto-allowed tools: no approval lines shown (unchanged behavior)

**Acceptance**:
- [ ] Approval prompt appears immediately after the tool call line
- [ ] Visual association between tool call and its approval is clear
- [ ] Multi-tool batches show approval per-tool, not batched at the end
- [ ] Approved/denied result shown inline before next tool call
- [ ] No regression for auto-allowed tools

### S2: ARCH-002 Phase 1 — Documentation Validation

| Field | Value |
|-------|-------|
| Source | ARCH-002 Phase 1 |
| Est | Research (no code changes) |
| Priority | P1 |

**Problem**: Project documentation (AGENTS.md, ARCHITECTURE.md, ADRs, backlog stories,
iteration records) may have drifted from actual implementation after rapid TOOL-003/I025
development.

**Scope**:
- Verify AGENTS.md hard constraints still match actual code constraints
- Verify ARCHITECTURE.md crate dependency graph matches Cargo.toml
- Verify ADRs referenced in AGENTS.md still exist and are accurate
- Verify backlog story acceptance criteria match actual implementation evidence
- Verify iteration records match actual commit history
- Verify README.md "What Works" section matches actual tool inventory
- Verify BOARD.md status entries match owner docs
- Document all discrepancies found

**Acceptance**:
- [ ] Every AGENTS.md hard constraint verified against code
- [ ] Architecture dependency graph validated
- [ ] ADR cross-references valid
- [ ] Discrepancy list produced and filed as residual work or fixed

## Exit Criteria

- [ ] S1 and S2 complete
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] BOARD.md updated with final state
