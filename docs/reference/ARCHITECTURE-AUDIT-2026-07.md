# Architecture Audit — 2026-07

> **Iteration**: I144 / ARCH-034-A
> **Date**: 2026-07-20 (revision 4)
> **Scope**: Evidence-and-boundary audit of all 21 production crates. No production code was modified.
> **Baseline**: v0.4.0 workspace, main branch, commit db1ccf9 (local).
> **Previous audit**: [ARCHITECTURE-AUDIT-2026-06-18](ARCHITECTURE-AUDIT-2026-06-18.md) (16 crates).
> **LOC method**: Item-scoped `#[cfg(test)]` subtraction. See Section 1 for the reproducible procedure and its limits.

## Executive Summary

The workspace has grown from 16 to 21 crates since the June 2026 audit. The dependency graph remains acyclic with `talos-core` as the pure foundation (zero internal deps). Two extension points show elevated coupling cost (tool registration at 5-7 touch points; permission facet construction with 31 independent implementations across 16 files). Five native or panic-capable integration boundaries need an explicit containment assessment, including two in the agent tool-execution path (`gix` and `arborium` in `talos-tools`), which is a P1 gap against AGENTS.md hard constraint #9. The normal production Clippy gate is clean; `clippy --all-targets` emits 806 lint warnings, predominantly test-target lint families, but this audit does not claim an exact warning-by-source classification. Four `unsafe` blocks exist in production code, all ADR-007-governed `libc::setrlimit`/`env::remove_var` calls in `talos-sandbox/src/hardening.rs`. No P0 correctness or safety issues were found.

---

## 1. Crate Inventory (21 crates)

### Dependency Direction and Fan-Out

| Crate | Internal Deps | Fan-In | Role | Public Items |
|-------|--------------|--------|------|-------------|
| `talos-core` | 0 | 17 | Foundation traits, message/protocol/tool types | 61 |
| `talos-config` | core | 3 | User config schema, catalog, env substitution | 40 |
| `talos-permission` | core | 6 | Permission rules and approval decisions | 17 |
| `talos-session` | core | 4 | Session persistence, TLOG, durable sessions | 79 |
| `talos-skill` | 0 | 2 | Skill discovery and loading | 15 |
| `talos-sandbox` | core | 3 | Process sandbox and hardening | 9 |
| `talos-memory` | 0 | 2 | Semantic memory consolidation and retrieval | 34 |
| `talos-plugin` | core, permission | 4 | Hook events, WASM plugin runtime | 39 |
| `talos-provider` | config, core | 2 | Anthropic, OpenAI-compatible, mock adapters | 13 |
| `talos-tools` | core, sandbox, permission | 3 | File, search, git, exec, symbol tools | 110 |
| `talos-conversation` | core, plugin | 2 | TUI conversation state, typed UI events | 80 |
| `talos-mcp` | core, permission, plugin, tools | 1 | MCP client transport and tool integration | 27 |
| `talos-rpc` | core, plugin | 1 | JSON-RPC API layer | 24 |
| `talos-evolution` | core, plugin | 1 | Runtime learning hook, evolution store | 20 |
| `talos-exploration` | 0 | 1 | Research library: SQLite/FTS5, citation | 18 |
| `talos-dashboard` | 0 | 1 | Loopback HTTP dashboard server | 3 |
| `talos-models` | core | 1 | Quarantined catalog.db compatibility shim | 9 |
| `talos-agent` | 8 crates | 2 | Turn loop, prompt, compaction, scheduler | 45 |
| `talos-tui` | conversation, core, permission | 1 | Interactive TUI: ratatui, panels, status bar | 23 |
| `talos-runtime` | 9 crates | 0 | Embeddable runtime facade | 11 |
| `talos-cli` | 17 (all except dashboard, exploration, models) | 0 | Composition root, CLI modes, TUI bridge | 11 |

**Dependency cycle check**: `cargo metadata` + graph traversal confirms **zero cycles**.

### LOC Distribution

`Non-test LOC` is the number of lines in `src/**/*.rs` that are not part of an item directly guarded by `#[cfg(test)]`. This is not a claim about every possible Cargo feature combination. It deliberately differs from the rejected “everything after the first `#[cfg(test)]`” shortcut: `#[cfg(test)]` may guard a single function, import, static, inline module, or external `mod tests;` declaration while later items remain production code.

**Reproducible procedure** (run from repo root; Python 3 standard library only):
```bash
python3 - <<'PY'
from pathlib import Path
import re
ATTR=re.compile(r'^\s*#\[cfg\(test\)\]\s*$'); PATH=re.compile(r'^\s*#\[path\s*=\s*"([^"]+)"\]\s*$'); MOD=re.compile(r'\bmod\s+(\w+)\s*;')
def delta(s):
 q=None; esc=False; n=0; i=0
 while i<len(s):
  c=s[i]; d=s[i+1] if i+1<len(s) else ''
  if q:
   if esc: esc=False
   elif c=='\\': esc=True
   elif c==q: q=None
  elif c=='/' and d=='/': break
  elif c in "\"'": q=c
  elif c=='{': n+=1
  elif c=='}': n-=1
  i+=1
 return n
def item_end(lines, start):
 bal=0; opened=False
 for i in range(start,len(lines)):
  d=delta(lines[i]); bal+=d; opened |= d>0
  if opened and bal<=0: return i+1
  if not opened and ';' in lines[i]: return i+1
 return len(lines)
def ranges(path, lines):
 out=[]; extern=set(); i=0
 while i<len(lines):
  if not ATTR.match(lines[i]): i+=1; continue
  start=i; j=i+1; override=None
  while j<len(lines) and lines[j].lstrip().startswith('#['):
   m=PATH.match(lines[j]); override=m.group(1) if m else override; j+=1
  end=item_end(lines,j); out.append((start,end)); m=MOD.search(''.join(lines[j:end]))
  if m: extern.add(path.parent/(override or f'{m.group(1)}.rs'))
  i=max(end,i+1)
 return out,extern
root=Path('.'); crates=sorted(root.glob('crates/talos-*'))
for crate in crates:
 files=sorted((crate/'src').rglob('*.rs')); external=set()
 for f in files: external |= ranges(f,f.read_text().splitlines())[1]
 total=prod=0
 for f in files:
  lines=f.read_text().splitlines(); total+=len(lines)
  if f not in external:
   excluded={k for a,b in ranges(f,lines)[0] for k in range(a,b)}; prod+=len(lines)-len(excluded)
 print(f'{crate.name} total={total} non_test={prod} cfg_test={total-prod}')
PY
```

The scanner follows Rust item delimiters and the `#[path = "..."] mod tests;` form used in this workspace. It does not parse macros or evaluate arbitrary non-test `cfg` expressions; therefore the label is **non-test LOC under this defined method**, not a universal compiler-configuration LOC claim. Spot checks: `talos-tui/src/app.rs` is 1,305 non-test lines (the test-only `for_test` method is 29 lines), and `scrollback.rs` is 1,104 (four test-only re-exports are 8 lines).

| Crate | Total | Non-test | `cfg(test)` | Test% |
|-------|-------|-----------|------|-------|
| talos-agent | 15,996 | 9,086 | 6,910 | 43.2% |
| talos-cli | 15,919 | 11,028 | 4,891 | 30.7% |
| talos-tools | 13,422 | 10,108 | 3,314 | 24.7% |
| talos-tui | 13,894 | 10,005 | 3,889 | 28.0% |
| talos-session | 9,625 | 5,711 | 3,914 | 40.7% |
| talos-provider | 5,480 | 3,926 | 1,554 | 28.4% |
| talos-conversation | 6,681 | 3,704 | 2,977 | 44.6% |
| talos-config | 4,093 | 2,083 | 2,010 | 49.1% |
| talos-memory | 4,132 | 2,029 | 2,103 | 50.9% |
| talos-permission | 3,056 | 1,323 | 1,733 | 56.7% |
| talos-skill | 1,761 | 628 | 1,133 | 64.3% |
| talos-evolution | 2,663 | 1,483 | 1,180 | 44.3% |
| talos-mcp | 1,976 | 1,720 | 256 | 13.0% |
| talos-plugin | 1,810 | 1,201 | 609 | 33.6% |
| talos-core | 2,730 | 1,903 | 827 | 30.3% |
| talos-exploration | 1,867 | 1,151 | 716 | 38.4% |
| talos-sandbox | 1,234 | 982 | 252 | 20.4% |
| talos-models | 1,585 | 1,031 | 554 | 34.9% |
| talos-runtime | 1,624 | 602 | 1,022 | 62.9% |
| talos-dashboard | 1,146 | 483 | 663 | 57.9% |
| talos-rpc | 682 | 682 | 0 | 0.0% |
| **TOTAL** | **111,376** | **70,869** | **40,507** | **36.4%** |

---

## 2. Large and Hot Production Files

The non-test line counts use the item-scoped method in Section 1.

| File | Total | Non-test | Responsibility | Changes (last 100) |
|------|-------|------------|---------------|-------------------|
| `talos-agent/src/scheduler.rs` | 3,230 | 993 | Scheduled follow-up lifecycle | 9 |
| `talos-session/src/todo.rs` | 2,353 | 1,653 | Session todo CRUD | 0 |
| `talos-provider/src/openai_sse.rs` | 2,197 | 1,787 | OpenAI SSE stream parser | 0 |
| `talos-cli/src/registry.rs` | 1,350 | 938 | Tool registry + approval | 3 |
| `talos-tui/src/app.rs` | 1,334 | 1,305 | TUI event loop + frame | 4 |
| `talos-tools/src/exec_tool.rs` | 1,298 | 916 | Structured exec | 0 |
| `talos-conversation/src/engine.rs` | 1,267 | 1,267 | Conversation engine | 4 |
| `talos-core/src/tool.rs` | 1,264 | 916 | AgentTool trait, ToolNature | 0 |
| `talos-tools/src/bash_tool.rs` | 1,236 | 660 | Bash escape-hatch | 0 |
| `talos-conversation/src/validation.rs` | 1,130 | 851 | Governance validation | 0 |
| `talos-tui/src/scrollback.rs` | 1,112 | 1,104 | Scrollback rendering | 3 |

All changes are proportional to stated responsibilities. LOC alone does not condemn.

---

## 3. Extension Scenario Traces

| Scenario | Touch Points | Crates | Coupling |
|----------|-------------|--------|----------|
| 1: New Provider (novel protocol) | 5-6 | config, cli, provider | Medium |
| 1: New Provider (OpenAI-compatible) | 0 code | 0 | None (config only) |
| 2: New Tool | 5-7 | tools, cli (optionally tui) | **High** (3 registry builders) |
| 3: New Permission Facet | 5-6 | core, permission, runtime | **High** (exhaustive enum match) |
| 4: New TUI Slash Command | 3-4 | conversation, tui | Medium |
| 5: New Session Backend | 3-4 | session | **Low** (trait-based) |
| 6: New Plugin Carrier | 5-7 | plugin, cli | **High** (parallel runtime) |
| 7: New Runtime Consumer | 0 workspace | 0 (downstream) | **Low** |

---

## 4. Duplication Inventory

| Pattern | Count | Classification |
|---------|-------|---------------|
| Permission facet construction | 31 impls / 16 files | Same domain logic |
| JSON parsing | 248 / 88 files | Textual similarity |
| Tracing/logging | 41 / 17 files | Textual similarity |
| Workspace root threading | 1,049 / 109 files | Textual similarity |
| Token/context access | 237 / 37 files | Textual similarity |
| TLOG format | 412 / 26 files | Shared domain policy |
| Error mapping | 1 match | No duplication |

---

## 5. State/Data Flow, Concurrency, Persistence, Native Containment

**State**: `ConversationEngine` owns conversation state; `TuiState` owns UI state. No shared mutable state across boundaries.

**Concurrency**: `CancellationToken` for shutdown; `tokio::mpsc` for streaming. 9 `await_holding_lock` warnings — all in test code.

**Persistence**: SQLite/bundled (ADR-008); TLOG + zstd archival (ADR-036/037); config TOML (ADR-023).

### Production `unsafe` sites

Four blocks in `talos-sandbox/src/hardening.rs`, all ADR-007-governed:

| Line | Expression | Purpose |
|------|-----------|---------|
| 258 | `env::remove_var(var)` | Remove dangerous env vars from child |
| 289 | `libc::setrlimit(RLIMIT_CORE)` | Disable core dumps |
| 305 | `libc::setrlimit(RLIMIT_CPU)` | CPU time limit |
| 321 | `libc::setrlimit(RLIMIT_AS)` | Address space limit |

ADR-018 (TUI `libc::raise(SIGTSTP)`): source file removed during TUI refactoring. ADR should be marked superseded.

### Native panic containment matrix

| Dep | Crate | Site | Wrapped? | Failure Model |
|-----|-------|------|----------|--------------|
| gix | talos-tui | `scrollback_status_git.rs:37` | Yes | — |
| gix | **talos-tools** | `git.rs:36-59` | **No** | Tool-path panic containment not documented |
| wasmtime | talos-plugin | `wasm.rs:110`, `registry.rs:109` | Yes | — |
| arborium | talos-tui | `highlight.rs:36`, `scrollback_markdown.rs:42` | Yes | — |
| arborium | **talos-tools** | `symbol.rs:314,441,541,662` | **No** | Native-backed tool-path containment not documented |
| grep-searcher | talos-tools | `search_engine.rs:370` | Yes | — |
| rusqlite | 5 crates | All SQLite calls | **No** | Boundary policy not documented |
| zstd | 2 crates | Compress/decompress | **No** | Boundary policy not documented |
| libc | talos-sandbox | `hardening.rs:289,305,321` | **No** | Return-code path exists; ADR-007 policy reconciliation required |
| agent session | talos-agent | `session.rs:214` | Yes | — |

**Covered**: 5 boundaries. **Uncovered**: 5 boundaries. The two talos-tools gaps (gix + arborium) are highest risk (agent tool-execution path).

---

## 6. ARCH-011/022/023/030 Reconciliation

| Story | Item | June 2026 | July 2026 | Disposition |
|-------|------|----------|----------|-------------|
| ARCH-011 | prompt.rs | 1,232 | **65** | Resolved (ARCH-020) |
| ARCH-011 | tests.rs (agent) | large | 3,503 | Watch |
| ARCH-011 | scrollback.rs | ~756 | 1,112 | Watch (within tolerance) |
| ARCH-022 | mode_runners.rs | 1,778 | **741** | **Resolved** (-58%) |
| ARCH-023 | app.rs | 1,118 | 1,334 | Deferred (>1,500 trigger) |
| ARCH-030 | sqlite.rs | 986 | 986 | Deferred |
| ARCH-030 | exploration lib.rs | 958 | 958 | Deferred |
| ARCH-030 | ingestion.rs | 799 | 799 | Deferred |

---

## 7. No-Change Conclusions

| Claim | Evidence |
|-------|---------|
| No dependency cycles | Graph traversal: 0 cycles |
| talos-core purity | 0 internal deps; 1,903 non-test LOC; 61 public items |
| Production clippy clean | `clippy --workspace --locked -- -D warnings` exits 0 |
| Session store clean trait | 7-method object-safe; 2 impls |
| Provider config-only | `protocol = "openai-chat"` = zero code |
| Runtime facade downstream | 0 workspace files changed by consumer |

---

## 8. Validation Evidence

| Command | Result |
|---------|--------|
| `cargo metadata --locked --no-deps --format-version 1` | 21 packages, v0.4.0 |
| `cargo fmt --all -- --check` | Clean |
| `cargo check --workspace --locked` | Clean |
| `cargo clippy --workspace --locked -- -D warnings` | Clean (production) |
| `cargo clippy --workspace --all-targets --locked` | Exit 0; 806 lint warnings |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Exit 101 (expected; warnings become errors, compilation halts) |
| `cargo test --workspace --locked` | 62 suites pass |
| `scripts/validate_project_governance.sh .` | 0 warnings |
| `git diff --check` | Clean |

### clippy methodology

```bash
# Warning count (compilation succeeds):
cargo clippy --workspace --all-targets --locked 2>&1 | grep -c "^warning: "
# Result: 806

# Per-crate breakdown:
cargo clippy --workspace --all-targets --locked 2>&1 | grep "^ *-->" | sed 's|.*crates/||;s|/.*||' | sort | uniq -c | sort -rn
# Top: talos-memory 198, talos-tools 146, talos-cli 145, talos-agent 101

# With -D warnings: exit code 101 (expected). No global error total is
# meaningful because compilation halts at the first warning-as-error.
```

---

## 9. Findings Summary

| ID | Sev | Title | Disposition |
|----|-----|-------|-------------|
| F01 | P1 | Tool registration 5-7 touch points | Proposed (R01) |
| F02 | P1 | Permission facet 31 impls / 16 files | Proposed (R02) |
| F03 | P2 | scheduler.rs 3,230 lines | Deferred (watch) |
| F04 | P2 | clippy --all-targets 806 warnings (predominantly test-target) | Deferred |
| F05 | P2 | app.rs 1,005 to 1,334 regression | Deferred |
| F06 | P2 | todo.rs 2,353 lines | Deferred |
| F07 | P2 | openai_sse.rs 2,197 lines | Deferred |
| F08 | P3 | 9 await_holding_lock in tests | Deferred |
| F09 | P3 | agent tests.rs 3,503 lines | Deferred |
| F10 | P3 | TUI churn 37/100 | Closed |
| F11 | info | No dependency cycles | Closed |
| F12 | info | talos-core purity (1,903 non-test LOC) | Closed |
| F13 | P1 | native/panic-boundary containment: 5 uncovered assessments | Proposed (R03a/R03b/R03c) |
| F14 | info | Session store clean extension | Closed |
| F15 | info | Provider config-only | Closed |
| F16 | info | Runtime facade clean | Closed |
| F17 | info | ARCH-022 resolved | Closed |
| F18 | info | ARCH-011 prompt.rs resolved | Closed |
| F19 | P2 | 4 production unsafe (ADR-007); ADR-018 removed | Deferred |
| F20 | info | scrollback.rs stable | Closed |

---

## 10. Proposed Stories

### ARCH-034-R01: Tool Registration Consolidation
Declarative registration macro or ToolRegistryBuilder. Effort M. **Proposed, not Ready.**

### ARCH-034-R02: Permission Facet Builder
ToolPermissionFacetBuilder in talos-core. Effort S. **Proposed, not Ready.**

### ARCH-034-R03: Native/Panic Boundary Containment

**R03a: talos-tools native boundaries (gix + arborium)**
- Gap: gix in git.rs:36-59; arborium in symbol.rs:314,441,541,662
- Failure model: these calls sit on the agent tool path and may propagate a dependency panic across the process boundary. `arborium` also crosses a native tree-sitter binding. Existing `Result` handling covers ordinary errors but not an unwind.
- Strategy: Wrap discover_repo/git_status_lines and Parser::new/parse in catch_unwind. Return tool errors (GitToolError::Internal, symbol-search error) instead of crashing.
- catch_unwind limitation: catches Rust panics from the wrapper layer; does NOT catch C-level SIGABRT/SIGSEGV. The gix/arborium Rust wrappers return Result for most errors; catch_unwind is for the rare panic path.
- Effort: S (1 day). **Proposed, not Ready.**

**R03b: SQLite and Zstd cross-crate boundaries**
- Gap: rusqlite in 5 crates; zstd in 2 crates
- Failure model: SQLite and zstd expose ordinary failures as `Result`, but their native-backed integration boundaries also need an explicit panic-containment assessment. C-level aborts (SIGABRT/SIGSEGV) are NOT catchable by `catch_unwind`.
- Unified strategy: wrap at the session/transaction boundary in talos-session/sqlite.rs and at compress/decompress in talos-config and talos-session. Other crates (memory, evolution, exploration, models) access SQLite through their own connections and need separate wraps.
- Per-crate: session (highest risk), config (startup-critical), memory/evolution/exploration/models (lower risk).
- catch_unwind limitation: it does not handle C-level aborts. Error-returning APIs remain the primary normal-failure path; the remediation must define the fallback returned after a Rust unwind without claiming to contain process aborts.
- Effort: M (2-3 days). **Proposed, not Ready.**

**R03c: talos-sandbox/libc FFI**
- Gap: 3 setrlimit calls in hardening.rs:289,305,321
- Failure model: `setrlimit` normally reports failure through its return code, which the current code propagates. It is nonetheless an FFI integration boundary; `catch_unwind` cannot contain a C-level abort and is not, by itself, a sufficient compliance argument.
- Required decision: ADR-007 must either specify a compliant boundary policy for this non-panicking FFI (including the existing error path and safe fallback) or record a justified exception to AGENTS.md #9. The audit does not decide that exception.
- Security review: Required before any sandbox hardening change or ADR-007 policy amendment.
- Effort: S decision/review, then implementation only if the reviewed policy requires it. **Proposed, not Ready.**

---

## Declaration

No production code, Cargo metadata, CI, permission policy, or sandbox behavior was modified by this audit. ARCH-034-B/C remain unauthorized.
