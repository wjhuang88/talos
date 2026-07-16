# EXT-002: oh-my-pi Feature Analysis

**Status**: Research
**Priority**: P3
**Source**: User request 2026-06-26; analysis of [can1357/oh-my-pi](https://github.com/can1357/oh-my-pi) (14.7k stars, MIT); user-highlighted omp.sh web control surface reference for WEB-001
**Iteration**: None yet

## Problem

oh-my-pi is the most feature-complete terminal coding agent in OSS. Several of its innovations could improve Talos's edit reliability, tool surface design, and agent steering. We need to evaluate which patterns are worth porting to Rust.

## Scope

Research and assess 5 standout features for Talos adoption.

### Features to evaluate

0. **Browser/Web Control Surface** — omp.sh demonstrates an agent product direction where a browser
   surface complements the terminal. Evaluate which interaction patterns map to Talos' loopback-only
   WEB-001 model: status, project/governance views, history, approvals, logs, and config. **HIGH
   strategic relevance.**

1. **Hashline: Content-Hash-Anchored Edits** — Diff format where hunks are bound to file-content
   hashes. Stale anchors are rejected before corruption. Claims 61% fewer output tokens and 10x
   edit pass rate improvement (6.7% → 68.3%). Language-agnostic, portable to Rust. Directly
   improves our `edit` tool. **HIGH borrowability.** The 2026-07-16 Talos assessment is now recorded
   in `docs/proposals/model-private-snapshot-anchored-file-edits.md` and child owner TOOL-022. Talos
   adopts the principle, not an external protocol: exactly two model-visible hex digits are only a
   compact check code; a full Runtime-memory file revision is authoritative; snapshot mechanics
   remain absent from TUI/hooks/history/TLOG; automatic reapply is excluded from Phase 1. I134 and
   TOOL-022 completed this child slice on 2026-07-16.

2. **Internal URL Scheme System** — 12 protocols (`pr://`, `issue://`, `memory://`, `skill://`, `mcp://`, `conflict://`, etc.) that resolve transparently inside every filesystem-shaped tool. `read pr://1428` returns the same shape as `read src/foo.rs`. Unifies the tool surface — model learns one interface. Maps to Rust traits. **HIGH borrowability.**

3. **TTSR: Time-Traveling Stream Rules** — Rules that sit dormant until the model goes off-script. Regex or AST-condition match aborts the stream mid-token, injects a system reminder, and retries. Rules survive compaction. Requires streaming abort support in our provider layer first. **MEDIUM-HIGH borrowability.**

4. **Advisor: Second Model Watching Every Turn** — Optional reviewer model with read-only tools that injects advice at 3 severity levels (nit/concern/blocker). Has its own `WATCHDOG.md`. Hard-isolated ToolSession. **MEDIUM borrowability.**

5. **Role-Based Model Routing** — `default`/`smol`/`slow`/`plan`/`commit` roles with per-role provider fallback chains and round-robin credential rotation. Cheap subagents on small models, deep reasoning on slow models. Simple config, high impact. **MEDIUM borrowability.**

### Non-goals

- No TypeScript/N-API architecture migration (Talos is pure Rust).
- No Bun runtime, Python kernels, or Chromium browser automation.
- No vouch-only contribution model.
- No DAP debugger integration (separate concern).

## Acceptance

- [x] Hashline edit format analyzed for Rust port feasibility; compact model protocol, hidden
      projection boundary, collision/concurrency risks, dependency options, phased delivery, and
      child owner TOOL-022 documented. This closes only the Hashline research sub-item; EXT-002
      remains Research until its other acceptance items are decided.
- [ ] omp.sh/browser control surface patterns mapped to WEB-001 MVP/non-goals.
- [ ] Internal URL scheme trait interface sketched for Talos tool registry.
- [ ] Decision recorded: which features to create backlog stories for, which to defer.

## Dependencies

- TOOL-002 (tool calling architecture) — for URL scheme integration.
- ARCH-006 (prompt cache stability) — for TTSR rule injection.

## Required Reads

- `docs/reference/REFERENCE-PROJECTS.md`
- `docs/backlog/active/TOOL-002-tool-calling-remediation.md`
- `docs/proposals/model-private-snapshot-anchored-file-edits.md`
- `docs/backlog/active/TOOL-022-model-private-snapshot-anchored-edits.md`
- [oh-my-pi repo](https://github.com/can1357/oh-my-pi)
