# MEM-004: Workspace-Scoped Session Topology

| Field | Value |
|-------|-------|
| Story ID | MEM-004 |
| Priority | P2 |
| Status | Planned |
| Depends On | MEM-002/I024 closeout; ADR-016 memory boundaries |
| Blocks | Same-workspace multi-session awareness; future cross-session recall scoped by workspace |
| Origin | Follow-up correction from `-c` resume behavior on 2026-06-13 |

## Problem

Session storage currently treats the workspace relationship as a directory-name convention. This
is enough to avoid global `--continue` mistakes when filtered carefully, but it is not a durable
data model:

- Two workspaces with the same final path segment can collide.
- Session metadata exposes `project`, while the product behavior is workspace-scoped.
- Future same-workspace multi-session awareness needs a first-class parent boundary.
- Cross-session recall must not accidentally mix unrelated workspaces.

## Target Model

Introduce an explicit workspace/session topology:

```text
Workspace
  id: stable workspace identity
  root_path: canonical workspace root when available
  display_name: human-readable basename
  sessions: [Session]

Session
  id: UUID
  workspace_id: parent workspace identity
  created_at / updated_at
  JSONL entries
```

The workspace boundary is the parent of multiple sessions. Future same-workspace recall and
session discovery must query through that parent instead of scanning all session files globally.

## Acceptance Criteria

- Session creation records a stable workspace identity instead of relying only on basename.
- `--continue` and `--resume` list only sessions attached to the active workspace.
- Existing basename-based session directories are migrated or read through a compatibility layer.
- SQLite index schema exposes workspace identity for filtering and future same-workspace recall.
- Same-basename workspaces do not collide in new storage.
- Cross-workspace session recall remains impossible unless a future explicit feature enables it.
- `cargo test --workspace` passes and includes a regression for two workspaces with the same
  display name.

## Non-Goals

- No semantic memory consolidation.
- No vector or graph retrieval.
- No automatic cross-session prompt injection.
- No deletion or rewrite of existing JSONL history without a migration plan.

## Current Stopgap

I024 now scopes implicit `--continue`/`--resume` candidates to the active workspace name and
hydrates visible TUI history after the first viewport draw. That fixes the immediate `-c` UX bug,
but it is still a compatibility layer over the old directory convention.

## Required Reads

- `crates/talos-session/src/lib.rs`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-cli/src/main.rs`
- `docs/backlog/active/MEM-002-conversation-context-continuity.md`
- `docs/iterations/I024-conversation-context.md`
- `docs/decisions/016-layered-memory-architecture.md`
