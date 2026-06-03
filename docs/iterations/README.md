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
| I004 | Safe Agent | Complete | ✅ Original #I004-S5 runtime-hardening gap was closed by R0/#ARCH-S3; see `R0-remediation-gate.md` and ADR-007 |
| I005 | Smart Agent | Complete | ✅ |
| I006 | Data Agent | Complete | ✅ Session index, fork identity, and search highlight residuals were closed by R0/#ARCH-S5..S7; dead event-loop variant removal remains scoped to I010-S7 |
| I007 | Skilled Agent | Complete | ✅ |
| I008 | Learning Agent | **Review** (impl landed 2026-06-01; TUI made default 2026-06-01) | Re-scoped 2026-06-01: evolution ships as a builtin `HookHandler` (per-Agent registration covers all paths uniformly). Implementation landed 2026-06-01 (509 tests, E2E print + TUI mode verified). TUI is now the default TTY mode (legacy readline REPL retained as `--repl`). Awaiting final review evidence/status sync. See `I008-learning-agent.md` for the new plan + Execution Record. |
| R0 | Remediation Gate | **Complete** (2026-06-01) | All 7 ARCH findings closed; 480 tests pass; I009 unblocked |
| R1 | Review Closure | **Active** (opened 2026-06-03) | Close I008/I009 Review drift, pause I011 S2, and preserve I010 R2 as the next mainline implementation slice. See `R1-review-closure.md`. |
| I009 | Extensible Agent | **Review** (runtime surface landed 2026-06-01) | Backend/runtime surface shipped (S2 hooks, S3 MCP client, S4 MCP server, S5 JSON-RPC, S1 ToolProvenance producers); E2E runtime evidence in `I009-extensible-agent.md`. TUI consumer markers + `/plugins` command remain follow-up work, so I009 is not Complete yet. |
| I010 | Polished Agent | Planned | See `I010-polished-agent.md` |
| I011 | Open Providers | **Paused** (S1 landed 2026-06-02; S2 deferred) | OpenAI-compatible `base_url` override + `OPENAI_COMPAT_API_KEY` env var shipped. S2 provider-plugin architecture is deferred until after R1/I010 or explicit priority change. See `I011-open-providers.md`. |
| I012 | Portable Tools | Planned | Native POSIX-style tool subset plus embeddable tool-pack registration to reduce host environment dependency. See `I012-portable-tools.md`. |

> Update this table whenever an iteration changes state. "Complete" requires runtime
> evidence, not only passing unit tests — see `docs/sop/ITERATION-WORKFLOW.md`.

## Next Execution Rounds

These rounds are the current operating plan for entering the next iterations. They reference
existing backlog stories only; new ideas still go through `docs/proposals/` or requirement intake.

| Round | When | Work Items | Promotion Rule |
|-------|------|------------|----------------|
| R0: Remediation Gate | ✅ Done (2026-06-01) | `R0-remediation-gate.md` | All 7 ARCH stories closed; runtime evidence recorded |
| R1: Review Closure | Active | `R1-review-closure.md`, `I008-learning-agent.md`, `I009-extensible-agent.md` | I008/I009 are Complete or their remaining work is explicitly deferred through change control; I011 S2 stays paused |
| R2: I010 Architecture Slice | After R1 closure | `I010-polished-agent.md` / Slice R2 | AppServerSession seam is for run-path cleanup and Codex-like terminal mode, not I008 evolution wiring |
| R3: I010 Product Polish | After R2 | `I010-polished-agent.md` / Slice R3 | Move I010 to Review when daily-use TUI workflows are verified end-to-end; Guardian/DSL require change control before entering this slice |
| R4: I012 Portable Tools | After I010/R3 or when environment-dependency reduction becomes release-critical | `I012-portable-tools.md` | Native POSIX subset works on a minimal `PATH`; tool-pack interface supports future plugin-provided local tools |
