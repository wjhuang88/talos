# TOOL-026: Non-Interactive Terminal Containment

| Field | Value |
|---|---|
| Story ID | TOOL-026 |
| Type | Tool Process Safety Correction |
| Priority | P0 Emergency |
| Status | Review |
| Source Issue | Maintainer incident report, 2026-08-12 |
| Responsible Actor | @wjhuang88 |
| Selected Iteration | I191 |

## Problem

Foreground `bash` and `exec` tool children inherit Talos's standard input and, on Unix, retain
access to its controlling terminal. Interactive programs and password prompts can therefore race
the TUI for input and inject terminal replies or control bytes into the composer.

## Required Outcome

- Tool children receive EOF on standard input unless `exec` explicitly supplies pipeline input.
- Unix tool children cannot open Talos's controlling terminal through `/dev/tty`.
- Password prompts fail explicitly instead of reading from, or corrupting, the TUI composer.
- Existing permission decisions, output capture and direct-child timeout behavior are unchanged.

## Exclusions

- No interactive PTY support, background jobs, descendant-tree supervision or TOOL-024 activation.
- No permission-policy relaxation and no change to I188/I189.

## Validation

- Focused `talos-tools` tests prove default stdin EOF, explicit `exec` pipe input and Unix
  controlling-terminal detachment.
- Locked format, check, Clippy and workspace tests.
- Independent natural-person security review on the exact implementation head before merge.

## Implementation Evidence

- Implementation commits: `d6d298a4` and warning-only correction `6bbcb568`.
- `./scripts/release_preflight.sh` passed on implementation tree `c597c0bb` after two earlier
  failed attempts exposed and corrected an unused import and a test-only undeclared dependency.
- Merge and independent exact-head security review remain pending; this Story is not Complete.
