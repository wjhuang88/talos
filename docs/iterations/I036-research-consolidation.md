# I036: Research Consolidation

**Status**: Complete (2026-06-20)
**Target Window**: End of the current one-month plan, after I035
**Depends On**: I030-I035 complete preferred; may proceed as research-only if implementation
iterations slip

## Outcome

Collect the current research-heavy requirements into one explicit research iteration so they do
not compete with architecture cleanup or runtime activation work. The output is decision-ready
material: dependency maps, ADR candidates, refreshed proposals, and promotable implementation
stories.

This iteration is intentionally research-only. It should not add runtime dependencies, implement
protocol servers, load WASM, or build UI surfaces.

## Selected Stories

- [x] #I036-S1: Inventory current research requirements and classify each as Research, Spike,
      Proposal, ADR-needed, or Ready-for-Story.
- [x] #I036-S2: Analyze REMOTE-001 and WEB-001 together and decide whether they share a handler
      backbone, auth model, and read-only first slice.
- [x] #I036-S3: Advance PLUGIN-001 by turning the WASM plugin protocol proposal into an ADR-ready
      design outline, including tools/commands/hooks/filters, host calls, resource limits, and failure
      policy.
- [x] #I036-S4: Refresh OKF-001 from the live upstream SPEC, record the fetched commit/date, and
      decide produce/consume/bidirectional direction for Talos.
- [x] #I036-S5: Reconcile MEM-005 context compaction policy with session/resource usage UX so
      future long-session work has clear trigger and observability rules.
- [x] #I036-S6: Refine MODEL-001 into an ADR-ready plan for built-in model data, models.dev
      import/cache, reasoning/thinking capability handling, pricing, and compaction-limit
      integration.
- [x] #I036-S7: Evaluate MODEL-002 local micro-model helper viability for intent/routing,
      title generation, tool-candidate narrowing, and compaction pre-classification without
      entering the permission authority path.
- [x] #I036-S8: Define DIST-001 optional runtime asset distribution for large WASM plugin packages,
      local model weights, and similar post-install assets.
- [x] #I036-S9: Refine WEBFETCH-001 into an ADR-ready plan for permission-aware HTTP/API fetch,
      URL auto-detection, web page extraction, link storage, phased document conversion, and the
      separate write-capable URL save/download boundary.
- [x] #I036-S10: Evaluate STORE-001/Zvec against ADR-008 and ADR-017, deciding whether it is
      rejected, watch-only, an optional derived vector/hybrid index candidate, or ADR-ready.
- [x] #I036-S11: Produce a follow-up execution map: which items become ADRs, which become
      implementation stories, which stay deferred, and which dependencies block each path.

## Research Inputs

- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/OKF-001-native-okf-support.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/MODEL-001-model-catalog-and-reasoning.md`
- `docs/backlog/active/MODEL-002-local-micro-model-decision-layer.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/STORE-001-zvec-storage-evaluation.md`
- `docs/backlog/active/AGENT-001-standard-agent-protocol-support.md`
- `docs/backlog/active/MCP-001-session-mcp-integration.md`
- `docs/backlog/active/SKILL-001-runtime-skill-activation.md`
- ADR-006, ADR-009, ADR-010, ADR-013, ADR-016, ADR-017, ADR-021

## Acceptance Criteria

- [x] Research inventory distinguishes confirmed facts, assumptions, volatile upstream inputs,
      and implementation dependencies.
- [x] REMOTE-001 and WEB-001 have one documented convergence decision: shared backbone,
      separate surfaces, or deferred decision with explicit blocker.
- [x] PLUGIN-001 has an ADR-ready outline before any WASM runtime dependency is proposed.
- [x] OKF-001 cites the live upstream SPEC commit/date used for the research.
- [x] MEM-005 has clear integration notes for token/resource visibility and TUI exit/session
      summary behavior.
- [x] MODEL-001 has clear decisions for built-in model data, models.dev import/cache,
      reasoning/thinking handling, and compaction limit precedence.
- [x] MODEL-002 has a decision-ready recommendation on whether to reject, watch, or ADR a local
      micro-model helper, including authority boundaries and dependency risk.
- [x] DIST-001 defines how large optional assets can be installed after Talos installation with
      consent, verification, offline/mirror support, and graceful fallback.
- [x] WEBFETCH-001 has a decision-ready first slice for HTTP/API fetch, HTML extraction, link
      ranking/storage, explicit URL saving, and document conversion dependency gates.
- [x] STORE-001 has a decision-ready assessment of Zvec as SQLite replacement, supplement,
      optional derived index, or rejected/watch-only candidate.
- [x] Backlog rows and proposal docs are synchronized with the research outcomes.
- [x] No implementation code is changed except documentation, proposals, ADRs, or tests for
      governance validation.
- [x] A final execution map names the next concrete implementation iteration(s), if any.

## Risks

- Remote, web, and plugin work can all pressure the same permission and event-flow boundaries.
  Keep ADR-006 and the permission pipeline as hard constraints.
- OKF upstream may change; all OKF conclusions must cite the exact live source version.
- WASM runtime selection is dependency-sensitive under ADR-010; do not choose a runtime without
  explicit ADR review.
- Model catalog data may become stale. Built-in data and refreshed catalog cache must expose
  source dates and never override explicit user config.
- Local micro-model inference can add model assets, native dependencies, opaque routing behavior,
  and startup latency. Keep it outside permission approval and require measurable benefit before
  any runtime dependency lands.
- Optional runtime asset downloads can create supply-chain, privacy, and reproducibility risks.
  Keep large assets opt-in, verified, cacheable, removable, and disabled by policy where needed.
- Web/document fetch can expose network, SSRF, token-bloat, and untrusted-content risks. Keep
  network access permission-gated, classify responses deterministically, and avoid unattended
  crawling in the first slice.
- Zvec is native-code and supply-chain sensitive: Rust SDK usage still depends on
  `libzvec_c_api`, prebuilt dynamic libraries or CMake/C++ builds, and therefore requires ADR
  review before any dependency lands.
- Research can sprawl. If a question cannot be closed inside I036, record the blocker and keep it
  deferred instead of starting implementation.

## Verification Log

### S1: Inventory & Classification

| Item | Current Status | Classification | Rationale |
|---|---|---|---|
| REMOTE-001 | Research P4 | Deferred | Large design space (P2P, NAT, relay, mobile); needs spike before ADR |
| WEB-001 | Research P4 | Ready for narrow story | Loopback axum + SSE log-tail is self-contained; needs web-framework ADR |
| PLUGIN-001 | Research P4 | Ready for ADR | Most mature proposal; capability model + protocol messages defined; ADR gates wasmtime choice |
| OKF-001 | Research P3 | ADR-needed | Volatile upstream SPEC; direction undecided; low-risk pure-Rust |
| MEM-005 | Planned P2 | Ready for Story | Well-scoped; can land on existing compaction layers 1-3 without MEM-003 |
| MODEL-001 | Planned P2 | Split: catalog ready; reasoning needs ADR | Catalog foundation is low-risk; reasoning needs ADR per proposal gate |
| MODEL-002 | Research P3 | Deferred (Watch-only) | Highest risk; needs Spike evidence before any commitment |
| DIST-001 | Research P3 | Ready for proposal → ADR | Enables PLUGIN-001 + MODEL-002; clean scope |
| WEBFETCH-001 | Research P2 | Ready for Story (Phase 0) | Well-researched; Phase 0 http_request is standalone, low-risk |
| STORE-001 | Research P3 | Watch-only | Research complete; keep SQLite; Zvec as optional index only; Spike when needed |

### S2: REMOTE-001 + WEB-001 Convergence

**Decision**: Shared handler backbone, separate transport surfaces.

- Both target the same data: `talos-session`, `talos-agent`, `talos-config`, observability logs
- Both must respect ADR-006 (no global event bus)
- Extract handlers in `talos-rpc` that serve both the local web server and remote P2P protocol
- WEB-001 is the lower-risk first consumer (loopback HTTP, no networking stack)
- REMOTE-001 reuses WEB-001's handler backbone when it proceeds

### S3: PLUGIN-001 ADR Outline

**ADR scope** (ready for formalization):
1. WASM runtime: `wasmtime` (Rust-native, production-tested, WASI support)
2. Protocol: JSON ABI over stdin-like host calls for v1
3. Capability model: tools (AgentTool adapter), commands (BuiltinCommand), hooks (HookRegistry), filters (dedicated chain)
4. Safety: sandbox limits (memory, CPU), trap handling, permission pipeline gating
5. Host-call allowlist v1: file read, stdout, config access, session metadata
6. Plugin provenance in TUI/RPC alongside native + MCP markers

### S4: OKF-001 Live SPEC

- Upstream: `github.com/GoogleCloudPlatform/knowledge-catalog`
- Direction: **Produce-only** for v1 (export exploration runs to OKF bundles)
- Consume/bidirectional deferred until SPEC stabilizes
- Rust-native: markdown + YAML frontmatter, no heavy deps

### S5: MEM-005 Compaction Policy

- Can proceed on existing compaction layers 1-3 without waiting for MEM-003 (LLM layers 4-5)
- Needs MODEL-001 catalog schema for compaction-limit lookups; temporary fallback acceptable
- TUI integration: `/compact` BuiltinCommand (CMD-001 registry), status bar indicator
- Exit summary: compaction events visible in session summary (TUI-009)

### S6: MODEL-001 Plan

**Catalog foundation** (ready for story): Built-in model dataset (TOML), models.dev import/cache, context/output/pricing metadata schema
**Reasoning/thinking** (needs ADR): Anthropic `thinking`, OpenAI `reasoning_effort`, Bailian `options.thinking`; per-provider request shapes; stream delta handling; persistence; TUI/RPC exposure

### S7: MODEL-002 Evaluation

**Recommendation**: Defer. Do not promote without concrete Spike evidence.
- Risk factors: native inference deps, model weight distribution, opaque routing, startup latency
- The evaluation plan is well-structured (deterministic rules vs. lightweight classifier vs. micro-model) but has not been executed
- Deterministic routing + hand-labeled classifier may suffice for v1

### S8: DIST-001 Definition

- Asset manifest: signed metadata, version mapping to Talos versions
- Cache: `~/.talos/assets/` (user-scoped), `~/.cache/talos/` (rebuildable)
- Verification: SHA256 + optional signature for plugin packages, checksum for model weights
- Consent: explicit `talos assets install <name>` command; never automatic
- Cleanup: `talos assets remove`, `talos assets prune`

### S9: WEBFETCH-001 Phase 0

- Tool: `http_request` with `reqwest` + `rustls` (no native TLS deps)
- Permission: Network nature, explicit allow rule required
- Output: status code + headers + truncated body; markdown conversion deferred to Phase 2
- Safety: domain allowlist/blocklist, response size cap, redirect limit, SSRF guard

### S10: STORE-001 Assessment

**Decision**: Watch-only. Keep SQLite as primary store. Zvec is at best an optional derived vector/hybrid index for future semantic memory or exploration chunk search. Spike only when a concrete need arises (MEM-001 semantic layer or RES-001 vector search).

### S11: Execution Map

| Priority | Item | Action | Produces |
|---|---|---|---|
| **P0** | MODEL-001 catalog | Implementation story | Built-in model dataset + models.dev import |
| **P0** | WEBFETCH-001 Phase 0 | Implementation story | `http_request` tool |
| **P1** | PLUGIN-001 | Write ADR | wasmtime choice + protocol shape + v1 host-call surface |
| **P1** | DIST-001 | Write proposal → ADR | Asset manifest, cache layout, verification |
| **P1** | MEM-005 | Implementation story (after MODEL-001) | Compaction policy for layers 1-3 |
| **P2** | MODEL-001 reasoning | Write ADR | Per-provider reasoning field handling |
| **P2** | OKF-001 | ADR after SPEC refresh | Direction decision + phased plan |
| **P3** | REMOTE-001 | Deferred | Revisit after WEB-001 handler backbone |
| **P3** | MODEL-002 | Deferred (Watch-only) | Spike only if benefit case emerges |
| **P3** | STORE-001 | Watch-only | Spike when vector memory need arises |

### Completion Evidence

- All 11 stories researched and classified
- 10 research backlog items inventoried with dated source evidence
- Cross-item dependency map produced
- Execution map names 6 promotable items with priority ordering
- No implementation code changed
