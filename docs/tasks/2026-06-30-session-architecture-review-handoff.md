# 2026-06-30 Session Architecture Review Handoff

> Status: Reviewed; implementation correction complete
> Originator: Sisyphus session 2026-06-30
> Reviewer: Architecture (Oracle)
> Purpose: Consolidate this session's architecture-relevant work and escalate one unresolved design conflict for judgment.

## 1. Session Summary

Work completed this session, in order. All governance-validated (0 warnings); all code-change
slices pass `cargo test --workspace`.

### 1.1 WEB-002 — GitHub Pages Product Site → Complete
- Maintainer completed DNS CNAME + Pages source + custom domain (`talos.hwj.zone`).
- Status reconciled across owner doc / task / PRODUCT-BACKLOG / BOARD. README public-site row now
  points at the live URL instead of "publishing pending". No code/runtime change.

### 1.2 Plugin Encapsulation Architecture — new owner draft + blocked cluster
- Owner declared the target extensibility model on 2026-06-30:
  - **Atomic components (config-introduced):** Skill, MCP, Hook.
  - **Encapsulation layer:** Plugin = a packaging format wrapping any subset of {skill, mcp, hook}
    **plus additional tool definitions**, carried by an external artifact.
- Draft written: `docs/proposals/plugin-encapsulation-format.md` (DRAFT, awaiting decision).
  Supersedes/expands `wasm-runtime-plugin-protocol.md` (which was WASM-only).
- **Carrier strategy settled by owner:** WASM first-class; Lua optional; **dylib rejected**
  (unsandboxable, conflicts irreconcilably with Hard Constraint #1 / safety-first).
- New blocked items, all gated on the proposal's ADRs:
  - **PLUGIN-001** repositioned from "WASM-only protocol" to "plugin encapsulation system".
  - **CMD-002** (new) — command taxonomy realignment (`/plugins` currently shows MCP status; must
    become `/mcp`, with `/plugins` repurposed and `/hooks` added).
  - **HOOK-001** (new) — promote hook from code-only-registered to a config-introduced peer of
    skill/mcp.
  - **TOOL-008 Phase 3** — WASM parser loading blocked on the plugin runtime.
  - **DIST-001** — plugin-package distribution slice noted as blocked on the format.
- Key correction surfaced: `ToolProvenance` (ADR-009) has only `Native | McpRemote`; a `Plugin`
  variant is needed (additive, enum already `#[non_exhaustive]`).

### 1.3 Zombie-state closures (status drift only, no code change)
- **EXT-001** → Complete (delivered via I014: provenance markers + `/plugins` + degradation + tests).
- **PROV-001** → Complete (owner doc was already Complete; BOARD/backlog were stale).
- **I011** → Complete (S1 shipped; S2 superseded by I015).

### 1.4 MODEL-004 M3 → Complete (code change, verified)
- Root cause: `model_lifecycle.rs` resolved pricing from `builtin_models()` only; builtin
  `models.toml` has no pricing, so `StatusSnapshot` pricing was always `None` and the exit summary
  fabricated a `$3/$15` fallback.
- Fix: pricing now resolved from `model_config.all_models()` (full merge); fabricated fallback
  removed (no-pricing case omits the cost line honestly); dead `calculate_cost` deleted. M1/M2
  baseline (limit resolution + 128k fallback + compactor) preserved. Builtin `models.toml`
  intentionally left pricing-free (no fabricated data). 56/56 test binaries green.

### 1.5 TOOL-014 — conflict discovered, NOT resolved (see §2)
- While scoping the "fetch_url backend migration", discovered `fetch_url` does not exist in source
  (merged into `http_request` in I040), while three design docs name `fetch_url` as the unified
  model-visible read entry. Escalated below; no code or acceptance change made.

## 2. Escalation: fetch_url vs http_request — Unified Read Entry Ownership

This is the primary item requiring architecture judgment.

### 2.1 The Conflict

- **Code today (I040 execution):** `fetch_url.rs` was deleted; its capabilities (`extract_links`,
  content-type detection, HTML text extraction) were merged INTO `http_request`. So `http_request`
  is the de-facto unified HTTP/read tool (`mode: "auto" | "raw"`), `save_url` is the separate
  write-capable download tool, and **`fetch_url` does not exist in source** (it appears only as a
  mock named `"fetch_url"` in the TOOL-014 framework tests).
- **Design docs (three, consistent):**
  - `TOOL-014` — *"`fetch_url` remains the unified read-context tool … `http_request` …
    conditionally disclosed when custom method, headers, body, or low-level HTTP inspection is
    clearly needed."*
  - `WEBFETCH-001` — scopes `fetch_url mode=auto` as the unified fetcher; `http_request` as the
    general HTTP/API tool; `save_url` separate.
  - `WEB-005` — architecture diagram shows `fetch_url` as the model-visible unified entry with
    backends `http` / `document` / `browser_page`; `http_request` as advanced conditional tool;
    `save_url` as the explicit write tool.

Net: **design intent (3 docs) names `fetch_url` as the unified entry; one execution decision (I040)
gave that role to `http_request`.** The two are incompatible.

### 2.2 Why It Matters

- TOOL-014's remaining acceptance is literally *"document how WEBFETCH-001 and WEB-005 use
  `fetch_url` as the unified model-visible read entry"* — unwritable while the name is unresolved.
- WEB-005's `browser_page` conditional backend is specified to hang off `fetch_url`. Hanging it off
  a tool named `http_request` is semantically incongruous.
- The conditional-backend framework (landed 2026-06-30: `ToolBackend`, `ToolBackendDisclosure`,
  `ToolContinuation`, execution gate, continuation handler) is complete and proven by tests, but no
  production tool uses it yet — the first production consumer is precisely this unified read tool.

### 2.3 Option A — Accept I040 (`http_request` is the unified entry)
- Action: rewrite the three design docs to re-point the "unified entry" from `fetch_url` to
  `http_request`; close TOOL-014 acceptance as documentation. Zero code change.
- Pros: fastest; no model-visible surface change; no prompt-cache impact; no reversal of shipped
  behavior.
- Cons: name/semantic mismatch persists (`http_request` is a poor name for "read
  URL/page/document/browser-page"); three docs' narrative must be rewritten; `browser_page` backend
  hangs off a tool called `http_request`; contradicts the "few model-visible tools + conditional
  backends" principle (http_request would be both the facade and the advanced tool).

### 2.4 Option B — Reverse I040 (`fetch_url` unified entry; `http_request` narrowed)
- Action: a **clean split along the existing `mode` axis**, not a blunt reversal. Move the
  `auto` mode + content detection + HTML extraction + future backend hooks into a new `fetch_url`
  tool that implements the four `AgentTool` backend hooks
  (`conditional_backends`, `backend_for_input`, `description_for_backends`,
  `parameters_for_backends`) and returns `ToolContinuation`; keep `http_request` as the
  `raw`/low-level tool, conditionally disclosed. The three design docs need no change.
- Pros: aligns with all three design docs; correct name semantics; gives WEB-005's `browser_page`
  backend a proper home; `fetch_url` is the natural facade name.
- Cons: changes the model-visible tool surface (add `fetch_url`, narrow `http_request`) →
  **prompt-cache prefix impact, must verify stability (ARCH-006)**; larger effort (new tool +
  registration in 3 builders + permission profile + tests + docs); reverses a recorded execution
  decision, requiring a correction note appended to I040.

### 2.5 Sisyphus Recommendation (non-binding, for reviewer)
Option B, executed as a single bounded iteration with an ARCH-006 prompt-cache stability
regression. Rationale: 3 design docs vs 1 execution decision; `fetch_url` is the semantically
correct facade name; the conditional-backend framework's first production consumer should be the
tool the docs already specify. But this is the owner/architecture call.

## 3. Other Architecture-Relevant Decisions to Sanity-Check

(Owner already decided these; flagging for reviewer awareness, not re-litigation.)

- **Plugin carrier strategy:** WASM first-class, Lua optional, **dylib rejected**. Rejection rests
  on "dylib is unsandboxable → conflicts with Hard Constraint #1 + safety-first." Is the rejection
  correctly grounded, or should dylib survive as an explicit trust-escape-hatch behind a separate
  high-friction ADR (the draft's original position before the owner cancelled it)?
- **`ToolProvenance::Plugin` variant:** additive extension to ADR-009. Any reason this should be a
  richer type than `{ name, version, carrier }`?
- **CMD-002 transition policy:** whether `/plugins` should be reserved (no-op with notice) or alias
  `/mcp` during the transition until plugins ship.

## 4. Questions for the Architecture Reviewer

1. **Primary:** For the fetch_url/http_request conflict, do you judge Option A, Option B, or a
   third option (e.g., `fetch_url` as an alias, or a different split axis) to be correct? Give
   reasoning, including prompt-cache and permission-profile implications.
2. If Option B: what is the minimal safe split — should `fetch_url` carry the `browser_page`
   backend declaration from day one (even though the connector is ADR-gated and unimplemented), or
   start backend-less and add backends incrementally?
3. Are there session decisions in §1 or §3 that create hidden architecture coupling or contradict
   an existing ADR?

## 5. Artifacts

- Draft proposal: `docs/proposals/plugin-encapsulation-format.md`
- At-handoff blocked items: `PLUGIN-001`, `CMD-002`, `HOOK-001` (new), `TOOL-008` Phase 3.
  Follow-up on 2026-06-30 accepted ADR-027/028/029/030, so these are no longer blocked on missing
  architecture decisions; they remain gated by their next implementation/security reviews.
- Conflict sources: `TOOL-014`, `WEBFETCH-001`, `WEB-005` backlog docs; `I040` iteration record;
  `crates/talos-tools/src/http_request.rs`, `save_url.rs`; TOOL-014 framework in
  `crates/talos-core/src/tool.rs` and tests in `crates/talos-agent/src/tests.rs`
- Verified code change: MODEL-004 M3 (`model_lifecycle.rs`, `app_summary.rs`,
  `scrollback_status.rs`)

## 6. Validation State at Handoff

- `cargo test --workspace`: 56/56 test binaries green, 0 failures.
- `sh scripts/validate_project_governance.sh .`: 0 warnings.
- Uncommitted changes present (this session's doc + MODEL-004 M3 code); no commit/tag/push
  performed.

## 7. Architecture Review Outcome (Oracle, 2026-06-30)

Oracle completed a read-only review of this handoff and the primary sources. Verdicts below are
recorded to close the loop. Confidence is High unless noted.

### 7.1 fetch_url vs http_request → Option B (reverse I040; clean split along `mode` axis)

- `fetch_url` becomes the unified read-context entry: owns the `auto` path (content-type detection,
  HTML text extraction, link extraction) plus the four `AgentTool` backend hooks; returns
  `ToolContinuation` when a backend switch is warranted.
- `http_request` narrows to the raw/advanced HTTP tool (method/body/header); remove `mode` and
  `extract_links` from `HttpRequestInput`; conditionally disclosed by `fetch_url` when advanced HTTP
  shaping is needed.
- **`fetch_url` starts backend-less on day one.** `browser_page` is added to
  `conditional_backends()` only when the browser connector ADR is accepted and the connector trait
  is implemented. Declaring it now creates a dead backend that only errors — zero present benefit.
- Register `FetchUrlTool` in all three builders; keep `HttpRequestTool` registered in all three.
- Prompt-cache impact is **manageable**: ARCH-006 is Complete (system prompt prefix computed once per
  session). Adding `fetch_url` / narrowing `http_request` is a one-time session-start surface change,
  not a mid-session mutation — it does not violate the stability guarantee. Still run an ARCH-006
  regression as part of the implementing iteration.
- `fetch_url` base permission facet: `network_read`. The future `browser_page` backend adds
  `browser_page_read` (per WEB-005), relevant only when that backend is disclosed.
- Append a **correction note to I040** (do not rewrite the original decision — the orthogonality
  merge was reasonable at the time; the reversal is a refinement identified by architecture review).
- Effort estimate: short (1–4h) — the split is along an existing axis, extraction code already
  exists, backend hooks are default methods.

Reasoning: 3 design docs vs 1 execution decision; `http_request` is a poor model-visible name for
"read URL/page/document/browser context"; `browser_page` belongs on `fetch_url`; the
conditional-backend framework's first production consumer should be the tool the docs already
specify (and the framework tests already mock `fetch_url`).

### 7.2 MODEL-004 M3 risk → Low

- `builtin_models()` → `all_models()` for pricing is correct; user-config / models.dev pricing being
  respected is the *intended* behavior, not a regression. Previous always-`None` pricing was a bug.
- Pricing resolution and context-limit fallback are independent code paths — no interaction risk.
- No-pricing case omitting the cost line is correct (no fabrication). Edge case verified: a model
  with no pricing anywhere still yields `None` and omits the line.

### 7.3 Session decisions sanity-check

- **Plugin 4-entity model + dylib rejection:** coherent; dylib rejection correctly grounded in Hard
  Constraint #1/#5 — "an escape hatch that cannot be sandboxed is not an escape hatch, it is a
  backdoor." No hidden coupling; maps cleanly to `talos-skill` / `talos-mcp` / `talos-plugin`.
- **`ToolProvenance::Plugin { name, version, carrier }`:** correct v1 shape. `carrier` as `String`
  (not enum) is pragmatic. Recommended: the ADR-009 extension should record that plugin tools go
  through an adapter layer enforcing carrier-specific timeout/memory/trap handling.
- **CMD-002 `/plugins` → `/mcp` transition:** **no-op with notice, NOT an alias** (Medium
  confidence). An alias creates hidden coupling — users who learn `/plugins` for MCP will be
  confused when it later means plugin packages. A notice-bearing no-op is the clean deprecation
  path.

### 7.4 Verdict Summary

| Item | Judgment | Confidence |
|---|---|---|
| fetch_url vs http_request | Option B — split along `mode`; `fetch_url` starts backend-less | High |
| MODEL-004 M3 risk | Low; fix is correct | High |
| Plugin 4-entity model + dylib rejection | Coherent; correctly grounded | High |
| `ToolProvenance::Plugin` shape | `{ name, version, carrier }` correct for v1 | High |
| CMD-002 transition policy | No-op with notice, not alias | Medium |

### 7.5 Resulting Next Actions (awaiting owner authorization)

1. Implement Option B as a single bounded iteration with an ARCH-006 prompt-cache stability
   regression; append the I040 correction note; close the remaining TOOL-014 acceptance item.
2. Apply the CMD-002 "no-op with notice" transition policy when CMD-002 is unblocked.
3. Record the `ToolProvenance::Plugin` adapter-layer note in the ADR-009 extension when drafted.

Oracle disagreed with the Sisyphus recommendation on no point of substance; it confirmed Option B
and added precision on (a) day-one backend-less `fetch_url`, (b) CMD-002 notice-vs-alias, and
(c) M3 low-risk confirmation.

## 8. Implementation Follow-up (2026-06-30)

The owner accepted that architecture-sensitive pieces should not be delegated. The correction was
implemented directly:

- `fetch_url` restored as the unified URL context-ingestion tool.
- `http_request` narrowed to advanced raw HTTP/API inspection and moved to the `AdvancedNetwork`
  presentation family.
- Secured agent construction now uses `ToolPresentationPolicy::runtime_default()` instead of
  `full()`, so advanced tools remain registered but hidden until disclosed.
- `ToolContinuation` can disclose either an individual tool (`http_request`) or a narrow backend.
- Browser-page backend remains undeclared until its ADR and connector trait exist.
- GitHub issue #5 was included in the same iteration: `RuntimeBuilder::custom_prompt(...)` and
  `RuntimeBuilder::append_prompt(...)` now expose existing Agent prompt customization to embedders.
- Validation: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`,
  and `scripts/validate_project_governance.sh .` passed.
