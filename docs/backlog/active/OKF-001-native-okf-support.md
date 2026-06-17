# OKF-001: Native OKF (Open Knowledge Format) Support — Research

| Field | Value |
|-------|-------|
| Story ID | OKF-001 |
| Priority | P3 |
| Status | Research (needs refinement before stories) |
| Type | Spike |
| Depends On | RES-001 (exploration library) for the produce-side mapping; MEM-001 |
| Estimate | Spike (research deliverable, not a feature) |
| Origin | User request 2026-06-17 — add native support for the OKF spec |

## What OKF Is

[Open Knowledge Format v0.1](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
(from Google's `knowledge-catalog`) is a **vendor-neutral** way to represent knowledge as plain
**markdown files with YAML frontmatter**, organized in a directory hierarchy ("bundle"). Highlights:

- Human- and agent-readable; `cat`-able; LLM-ingestible verbatim.
- Version-controllable (bundles live in git), portable (just a directory), lock-in free.
- A small set of required frontmatter keys for interoperability (`type`, `resource`, `tags`,
  `timestamp`) plus arbitrary extensibility.
- Auto-generated `index.md` per directory for **progressive disclosure** navigation.
- **Graph-shaped**, not just tree-shaped: concepts link to each other via normal markdown links.

## Spec Volatility — Always Pull Live (Working Rule)

> `okf/SPEC.md` is currently a **Draft** and changes over time. Treat any snapshot as stale.

- **Canonical location**: <https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md>
  (raw text: `https://raw.githubusercontent.com/GoogleCloudPlatform/knowledge-catalog/main/okf/SPEC.md`).
- **Working rule**: before *any* OKF-related research or development, **re-fetch the live SPEC.md
  text** and work from the current version. Do not rely on a previously-read snapshot, a copied
  excerpt, or this backlog item's summary for spec details.
- **Cite the version**: when recording findings/ADRs, capture the fetched commit SHA + date so the
  decision is reproducible (`https://github.com/GoogleCloudPlatform/knowledge-catalog/commit/<sha>`).
- The README is also volatile; SPEC.md is authoritative for format details.

## Why It Matters for Talos

Talos already has two knowledge-adjacent surfaces where OKF could plug in:

- **RES-001 Exploration Library** — persists research runs, sources, claims, syntheses, caveats.
  OKF is a natural **export/portability format** for an exploration run.
- **MEM-001 Layered Memory** — durable knowledge. OKF could be an **import/context format**
  loaded into agent context the way `AGENTS.md` is today, but richer and graph-shaped.

This is deliberately a **Spike**: the direction must be decided before any implementation story is
written. The output of this research is a decision (ADR) and a phased plan, not code.

## Open Questions (must be resolved)

1. **Direction**: produce-only (export Talos knowledge → OKF bundles), consume-only (load OKF
   bundles into context), or bidirectional?
2. **Concept mapping**: how do OKF concepts (bundle / concept doc / `index.md` / link graph) map to
   Talos's RES-001 (sources / claims / syntheses / claim edges) and MEM-001 layers? Where do they
   not line up?
3. **Progressive disclosure**: how should Talos navigate a large OKF bundle without loading it all
   into context (mirroring OKF's `index.md` per-level design)?
4. **Scope of "native support"**: bundle reader/writer in a new crate? A CLI `talos okf`
   subcommand (export/import/inspect)? TUI viewer? First slice = just export an exploration run?
5. **Dependencies**: OKF is markdown + YAML — confirm a Rust-native path (e.g. existing serde +
   `serde_yaml` / markdown frontmatter crate) with **no new heavy native deps** (respect ADR-010
   dependency discipline).

## Acceptance Criteria (research deliverables)

- [ ] Read OKF `SPEC.md` v0.1 and document: exact required/optional frontmatter keys, `index.md`
      contract, link-graph rules, and bundle directory layout.
- [ ] The SPEC details cited were read from the **live** SPEC.md fetched at the time of work, with
      the commit SHA + date recorded. This item's summary is not a spec source of truth.
- [ ] Decide direction (produce / consume / both) and **record it as an ADR** in
      `docs/decisions/`.
- [ ] Produce a concept-mapping table between OKF and Talos's RES-001 / MEM-001, with mismatches
      called out.
- [ ] Recommend a phased plan (suggested: P0 = export an exploration run to an OKF bundle;
      P1 = ingest an OKF bundle into context with progressive disclosure).
- [ ] Confirm the implementation can stay Rust-native with no C/Python/runtime dependency, or
      record any exception as a Soft-constraint tradeoff in the ADR.

## Required Reads

- https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
- https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/README.md
- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/decisions/017-exploration-library-storage.md` (ADR-017)
- `docs/decisions/016-memory-architecture.md` (ADR-016)
- `docs/decisions/010-...` (ADR-010, dependency discipline)
