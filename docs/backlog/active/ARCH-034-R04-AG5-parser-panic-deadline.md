# ARCH-034-R04-AG5: Parser Panic And Deadline Containment

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-5 / arborium-tree-sitter execution boundary |
| Status | Refinement — enforceable deadline adapter requires architecture decision |
| Priority | P1 |
| Selected Iteration | None |
| Preserved behavior | Symbol schemas/results/order, language detection, AG-4 traversal notices, direct-file semantics, and TUI plain-text fallback |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Independent native-boundary review required |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Accept an adapter/timeout decision before implementation; a Tokio timeout around uncancellable native work is insufficient. |

## Confirmed Baseline

Symbol-tool parser construction/language/parse work is synchronous inside async
tool execution with no panic boundary or enforceable deadline. TUI highlighting
catches panics, but its elapsed-time check runs only after the native call returns
and cannot interrupt a hang. AG-4 bounds directory admission but explicitly does
not bound pathological parsing, direct-file reads or serialized output size.

## Scope And Acceptance

- Inventory every tools/TUI parser call and choose one dependency-only adapter.
- Catch dependency panics without swallowing Talos logic panics or reusing
  potentially poisoned parser state.
- Select an enforceable deadline strategy. `spawn_blocking` plus dropped timeout
  future alone is not acceptance because native work continues unbounded.
- Preserve tool error and TUI plain-text fallback contracts for panic, timeout,
  unsupported language and malformed input.
- Add malformed/adversarial corpus, injected panic and bounded-duration fixtures
  in both reverse-dependency crates.
- Keep any helper process Rust-owned and ADR-reviewed if isolation is selected.

## Exclusions And Residuals

No traversal-budget change, catch-all panic handler, new parser dependency,
silent result truncation or global worker/event bus. Direct-file byte and symbol
serialization caps require explicit behavior owners if later selected.

## Minimum Validation

Focused `talos-tools`/`talos-tui` tests, locked release preflight, Unix/Windows CI,
dependency/ADR review and independent native-boundary review.
