# Iterations

## Purpose

Track iteration plans, execution progress, and retrospectives.

## Naming Convention

```
docs/iterations/
├── README.md           (this file)
├── R0-<slug>.md        (remediation gate / execution round)
├── I001-<slug>.md      (iteration plan + execution record)
├── I002-<slug>.md
└── ...
```

## Lifecycle

1. **Planned** — Iteration created with scope, selected stories, and acceptance criteria.
2. **Active** — Work in progress. Update story status as work proceeds.
3. **Review** — All stories implemented. Run verification checklist.
4. **Complete** — Verification passed, retrospective written.

## Rules

- Each iteration has a unique ID (`I001`, `I002`, ...).
- Published iteration baselines must not be silently overwritten by later execution.
- Start a new iteration only after inventorying all existing active, review, planned, and blocked iterations.
- Record execution results by appending to the plan, not replacing it.

## Current Iterations

| ID | Codename | State | Deliverable verified end-to-end? |
|----|----------|-------|----------------------------------|
| I001 | Project Scaffold | Complete | ✅ |
| I002 | Hello Agent | Complete | ✅ |
| I003 | Tool User | Complete | ✅ |
| I004 | Safe Agent | Complete | ⚠️ #I004-S5 process hardening built+tested but **unwired** (no runtime effect); see `I004-safe-agent.md` → #ARCH-S3, ADR-007 |
| I005 | Smart Agent | Complete | ✅ |
| I006 | Data Agent | Complete | ⚠️ ADR-004 event-loop variants partly unused (dead code); session index/fork residuals tracked by #ARCH-S5/#ARCH-S6/#ARCH-S7 |
| I007 | Skilled Agent | Complete | ✅ |
| I008 | Learning Agent | **Review** | ⚠️ print-mode runtime wired; TUI/interactive paths pending (see `I008-learning-agent.md`) |
| R0 | Remediation Gate | **Complete** (2026-06-01) | All 7 ARCH findings closed; 480 tests pass; I009 unblocked |
| I009 | Extensible Agent | **Active** (2026-06-01) | See `I009-extensible-agent.md`; S2 → S3 → S4 → S5 → S1 order |
| I010 | Polished Agent | Planned | See `I010-polished-agent.md` |

> Update this table whenever an iteration changes state. "Complete" requires runtime
> evidence, not only passing unit tests — see `docs/sop/ITERATION-WORKFLOW.md`.

## Next Execution Rounds

These rounds are the current operating plan for entering the next iterations. They reference
existing backlog stories only; new ideas still go through `docs/proposals/` or requirement intake.

| Round | When | Work Items | Promotion Rule |
|-------|------|------------|----------------|
| R0: Remediation Gate | ✅ Done (2026-06-01) | `R0-remediation-gate.md` | All 7 ARCH stories closed; runtime evidence recorded |
| R1: I009 Extensibility | Now | `I009-extensible-agent.md` | Move I009 to Review when hook/MCP/RPC paths work end-to-end with permission gates |
| R2: I010 Architecture Slice | After I009 Review, or earlier only if needed to unblock I008 | `I010-polished-agent.md` / Slice R2 | I008 can become Complete only after TUI/interactive evolution wiring attaches at the shared AppServerSession seam |
| R3: I010 Product Polish | After R2 | `I010-polished-agent.md` / Slice R3 | Move I010 to Review when daily-use TUI workflows are verified end-to-end |
