# STORE-001: Zvec Storage Evaluation

## Outcome

Talos has a decision-ready evaluation of Alibaba Zvec as a possible SQLite replacement or
supplement before any storage dependency, migration, or architecture change is proposed.

## Status

Research. Selected as an input to I036.

## Priority

P3.

## Required Reads

- `docs/decisions/008-sqlite-bundled-storage.md`
- `docs/decisions/017-exploration-library-storage.md`
- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-evolution/src/store.rs`
- Zvec repository: <https://github.com/alibaba/zvec>
- Zvec Rust SDK: <https://github.com/zvec-ai/zvec-rust>
- Zvec benchmarks: <https://zvec.org/en/docs/db/benchmarks/>

## Evaluation Summary

Zvec is promising for vector and hybrid retrieval, but it is not currently a drop-in replacement
for SQLite in Talos.

Confirmed upstream facts as of 2026-06-19:

- `alibaba/zvec` describes itself as an open-source, in-process vector database embedded directly
  into applications.
- Zvec v0.5.0, published 2026-06-12, added native full-text search, hybrid retrieval,
  DiskANN, and official Go/Rust SDKs.
- The main repository is primarily C++ with C/SWIG/CMake components, licensed Apache-2.0.
- The official Rust SDK is a Rust wrapper over `libzvec_c_api`, not a pure-Rust storage engine.
- The Rust SDK build script can download prebuilt native dynamic libraries from GitHub Releases,
  or clone/build the C++ project with CMake when no library is available.
- Current Rust SDK release assets cover macOS ARM64, Linux x86_64/aarch64, and Windows x86_64
  MSVC; there is no confirmed macOS x86_64 prebuilt asset in the inspected v0.5.0 release.
- Zvec benchmark documentation focuses on vector database performance using VectorDBBench over
  Cohere 1M and 10M 768-dimensional datasets.

Talos-local facts:

- `talos-session` uses bundled SQLite as a derived session index with FTS5 and metadata tables;
  JSONL remains the session source of truth.
- `talos-evolution` uses SQLite for structured observation and pattern persistence.
- ADR-008 approves `rusqlite/bundled` as a tightly scoped native dependency exception because
  Talos needs local FTS5, transactional metadata, structured local queries, and no system SQLite
  runtime dependency.
- ADR-017 already treats vector stores as optional accelerators behind Spike evidence, not as the
  first exploration-library storage substrate.

## Fit Assessment

| Talos Need | SQLite Today | Zvec Fit | Assessment |
| --- | --- | --- | --- |
| Session source of truth | JSONL, not SQLite | Not relevant | No replacement needed. |
| Session metadata index | SQLite tables | Scalar fields/filtering exist, but relational/migration parity unproven | Not ready as replacement. |
| Session full-text search | SQLite FTS5 | Zvec FTS exists in v0.5.0 | Candidate for benchmark only. |
| Evolution store | Structured tables and queries | Vector DB schema is not a relational store | Poor fit as replacement. |
| Research library first slice | SQLite tables + FTS5 + provenance | Hybrid search is attractive, but provenance schema still needs relational shape | Supplement candidate. |
| Semantic/vector memory | Not implemented yet | Strong candidate | Evaluate as optional vector/hybrid index. |
| Self-contained Rust build | `rusqlite/bundled` ADR exception | Native C++ dynamic library / CMake / downloaded binaries | Requires new ADR and supply-chain review. |

## Recommendation

Do not replace SQLite with Zvec now.

Treat Zvec as a candidate optional retrieval index for future memory and exploration work:

1. Keep JSONL as the session source of truth.
2. Keep bundled SQLite as the current structured metadata, FTS, and evolution store.
3. Evaluate Zvec only behind a storage/index trait introduced by a concrete second implementation
   need, not preemptively.
4. Require a dependency ADR before adding `zvec`, `zvec-sys`, downloaded prebuilt libraries, or
   any CMake-built Zvec artifact to the workspace.
5. If adopted later, use Zvec as a derived vector/hybrid index whose data can be rebuilt from
   JSONL, SQLite tables, or research-library source chunks.

## Required Spike

Before any implementation dependency lands, run a Spike that answers these questions with code,
benchmarks, and failure evidence:

- Can Talos build reproducibly on the release matrix without network access during Cargo build?
- Can the SDK be used without automatic GitHub binary download in normal developer and CI builds?
- Does the native library link strategy work for macOS ARM64, Linux x86_64/aarch64, and Windows
  x86_64 under the current release pipeline?
- What happens on unsupported targets such as macOS x86_64 or future Linux musl targets?
- Can Zvec persist, reopen, query, delete, and recover after simulated process interruption?
- Does Zvec FTS ranking and query syntax meet or exceed SQLite FTS5 for session search?
- Does hybrid retrieval improve answer quality for Talos memory/research use cases over FTS-only
  ranking?
- Can Talos attach enough metadata filters for workspace, session, branch, role, timestamp,
  source, claim status, and provenance without duplicating relational query logic?
- Can all Zvec indexes be treated as rebuildable derived data rather than authoritative state?
- What are the storage size, memory usage, index build time, and query latency on representative
  Talos data sizes: small local project, long session history, and research-library corpus?
- What are the dependency tree, license, NOTICE, CVE, and native-code audit obligations?

## POC Shape

Use a standalone prototype outside the main runtime first:

1. Export a fixed corpus from existing JSONL sessions plus synthetic research chunks.
2. Build two indexes:
   - SQLite FTS5 baseline using current schema/query behavior.
   - Zvec collection with string fields, FTS index, optional dense vector field, and metadata.
3. Run keyword-only, vector-only, and hybrid queries.
4. Compare recall, ranking quality, snippet/debuggability, latency, memory, and disk footprint.
5. Kill the process during insert/flush, reopen, and verify index consistency.
6. Repeat on the release target matrix or document unsupported targets explicitly.

## Acceptance Criteria

- [ ] Spike report compares Zvec against current SQLite FTS5 for session search and planned
      RES-001 source-chunk search.
- [ ] Report separates confirmed upstream behavior, local benchmark evidence, inference, and
      unknowns.
- [ ] Build and release impact is documented, including auto-download behavior, dynamic library
      handling, CMake fallback, and offline build strategy.
- [ ] Security/dependency review covers native code, third-party submodules, licenses, and
      whether a new ADR exception is needed under AGENTS.md hard constraint #1.
- [ ] Recommendation chooses one of: reject, keep watching, optional derived index, or begin ADR
      for replacement/supplement.
- [ ] No production code depends on Zvec until the Spike and follow-up ADR are accepted.

## Non-Goals

- Do not replace `rusqlite` in `talos-session` or `talos-evolution` as part of this item.
- Do not move session source-of-truth data out of JSONL.
- Do not add Zvec, Zvec Rust SDK, CMake, downloaded binaries, or a vector embedding model to the
  workspace during this research item.
- Do not create a generic storage abstraction until there is a concrete accepted second
  implementation.

## Residual Work Destination

If the Spike is positive, create a follow-up ADR and an implementation story for a rebuildable
derived retrieval index. If the Spike is negative or inconclusive, keep ADR-008 unchanged and
record Zvec as watch-only in the research inventory.
