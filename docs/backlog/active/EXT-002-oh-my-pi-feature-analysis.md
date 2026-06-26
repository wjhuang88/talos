# EXT-002: oh-my-pi Feature Analysis

**Status**: Research
**Priority**: P3
**Source**: User request 2026-06-26; analysis of [can1357/oh-my-pi](https://github.com/can1357/oh-my-pi) (14.7k stars, MIT)
**Iteration**: None yet

## Problem

oh-my-pi is the most feature-complete terminal coding agent in OSS. Several of its innovations could improve Talos's edit reliability, tool surface design, and agent steering. We need to evaluate which patterns are worth porting to Rust.

## Scope

Research and assess 5 standout features for Talos adoption.

### Features to evaluate

1. **Hashline: Content-Hash-Anchored Edits** — Diff format where hunks are bound to file-content hashes. Stale anchors are rejected before corruption. Claims 61% fewer output tokens and 10x edit pass rate improvement (6.7% → 68.3%). Language-agnostic, portable to Rust. Directly improves our `edit` tool. **HIGH borrowability.**

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

- [ ] Hashline edit format analyzed for Rust port feasibility; grammar and prompt spec documented.
- [ ] Internal URL scheme trait interface sketched for Talos tool registry.
- [ ] Decision recorded: which features to create backlog stories for, which to defer.

## Dependencies

- TOOL-002 (tool calling architecture) — for URL scheme integration.
- ARCH-006 (prompt cache stability) — for TTSR rule injection.

## Required Reads

- `docs/reference/REFERENCE-PROJECTS.md`
- `docs/backlog/active/TOOL-002-tool-calling-remediation.md`
- [oh-my-pi repo](https://github.com/can1357/oh-my-pi)
