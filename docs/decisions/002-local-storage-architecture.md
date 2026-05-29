# 002: Local Storage Architecture

## Context

Talos needs persistent local storage for multiple data domains: session history, evolution
observations, learned patterns, user configuration, skills, and permission rules. The storage
architecture must serve a single-user CLI tool built in Rust, prioritizing reliability, minimal
dependency surface, and progressive complexity introduction.

### Data Domains

| Domain | Write Pattern | Read Pattern | Scale | Query Need |
|--------|--------------|-------------|-------|------------|
| **Session messages** | Append-heavy (every turn) | Sequential replay, random search | ~50K turns/year, ~500MB | FTS, metadata lookup |
| **Evolution observations** | Append (every turn) | Aggregation by time/type | ~50K obs/year, ~50MB | GROUP BY, time-range |
| **Evolution patterns** | Rare writes (session end) | Every turn (inject into prompt) | ~100 patterns, ~100KB | Key lookup |
| **Configuration** | Rare (user edits) | Every turn (load into memory) | ~10KB | Layered merge |
| **Skills** | Rare (user creates) | Discovery + on-demand load | ~20 files | File discovery |
| **Permission rules** | Rare (user edits) | Every tool call | ~50 rules | Pattern match |

### Alternatives Evaluated

A comprehensive survey of 14 embedded databases was conducted across four categories:

#### Production-Ready Options

| Database | SQL | FTS | OLTP | Pure Rust | Binary Size | Maturity | Verdict |
|----------|-----|-----|------|-----------|-------------|----------|---------|
| **SQLite** (rusqlite) | ✅ | ✅ FTS5 | ✅ | ❌ C dep | ~1.6MB | 30+ years | **Baseline** |
| **DuckDB** | ✅ Excellent | ⚠️ Manual rebuild | ❌ 10-50x slower | ❌ C++ | 50-80MB | Stable | Wrong workload |

#### Beta/Promising Options

| Database | SQL | FTS | OLTP | Pure Rust | Binary Size | Maturity | Verdict |
|----------|-----|-----|------|-----------|-------------|----------|---------|
| **Turso** (ex-Limbo) | ✅ SQLite compat | ✅ Tantivy | ✅ | ✅ | ~6MB | Beta | **Watch** — v1.0 viable |
| **Stoolap** | ✅ Full | ❌ HNSW only | ⚠️ OLAP-biased | ✅ | Moderate | Beta | Interesting, no FTS |
| **native_db** | ❌ KV only | ❌ | ⚠️ 3-9x slower | ✅ | Small | Beta | Lacks all query needs |

#### Rejected Options

| Database | Rejection Reason |
|----------|-----------------|
| **SurrealDB** | Binary too large, v3.0 performance regression (1ms→2s), excessive CPU |
| **BonsaiDB** | Stalled since Oct 2023, alpha quality, known performance bug |
| **GlueSQL** | No FTS, incomplete SQL, niche adoption |
| **ContextDB** | Over-specialized for graph+vector; we need basic SQL+FTS |
| **MenteDB** | Cognitive engine, not a database; application-layer concern |
| **KiteSQL** | Requires RocksDB (C++ dep), no FTS |
| **TiKV** | Distributed KV, requires cluster deployment |
| **moteDB** | Multimodal storage, alpha stage |

#### Key Finding

**No alternative satisfies all hard requirements** (FTS + OLTP + SQL + production maturity).
SQLite is the only production-ready option. Turso Database (pure Rust SQLite rewrite) shows the
most promise for future migration but is explicitly beta with known data integrity issues.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|---|---|---|---|
| All crates are pure Rust, no FFI | Hard | AGENTS.md | No (but rusqlite uses bundled C, not FFI) |
| No secrets in persistent storage | Hard | AGENTS.md | No |
| Single-user CLI, no concurrent access | Soft | Product scope | Yes |
| Cross-platform (macOS/Linux/Windows) | Soft | Product scope | Yes |
| SQLite is the most reliable embedded DB | Assumption | 30 years of production use | Unlikely to change |
| Turso will reach production readiness | Assumption | Well-funded company, 19K stars, active dev | Maybe (12-18 months) |
| Storage engine should be swappable | Assumption | Future-proofing | Yes |

### Note on "Pure Rust" Constraint

AGENTS.md states "no C/C++ bindings, no Python FFI, no Node.js runtime." rusqlite uses the
`bundled` feature to compile SQLite from C source via the `cc` crate. This is not an FFI binding
to an external library — it compiles SQLite into the binary. This is acceptable because:

1. SQLite is the most tested C code in existence (fuzz-tested to 100% branch coverage).
2. The `bundled` feature requires zero system dependencies.
3. No alternative provides SQLite's feature set in pure Rust today.
4. Turso (pure Rust) is the planned migration path when production-ready.

## Reasoning

### From First Principles

The fundamental question is: **when does each data domain actually need database capabilities?**

- **I001-I003** (MVP → Safe Agent): Sessions are append-only JSONL. Config is TOML. No queries
  needed. Zero database dependencies.
- **I004** (Smart Agent): Session resume (`-c`, `-r`) requires metadata lookup. Session search
  requires FTS. JSONL alone is insufficient. **SQLite introduced here.**
- **I005** (Learning Agent): Evolution observations need aggregation queries. Patterns need
  confidence tracking. **SQLite extended here.**

This progressive introduction follows the agile principle: each iteration adds only the complexity
its features require.

### Storage Engine Abstraction

SQLite is used directly via rusqlite calls. No trait abstraction is introduced until a concrete
second storage engine is production-ready and we have real migration needs. This follows the
Pi design principle: **abstractions emerge from implementation, not from prediction.**

If Turso (or another engine) becomes viable in the future, the migration path is:
1. Identify all rusqlite call sites
2. Extract a `SessionStore` / `EvolutionStore` trait from the existing implementation
3. Implement the trait for the new engine
4. Swap implementations behind a config flag

This "extract when needed" approach avoids speculative abstraction while keeping migration feasible.

### Data Domain → Storage Mapping

```
Phase 1 (I001-I003): Pure Files
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Sessions    → JSONL files  (~/.talos/sessions/<project>/<id>.jsonl)
Config      → TOML files   (~/.talos/config.toml, .talos/config.toml)
Skills      → Not yet (I005)
Rules       → Not yet (I003 uses inline config)

Phase 2 (I004): SQLite Introduction
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Sessions    → JSONL files (unchanged) + SQLite index (~/.talos/index.db)
               SQLite tables: sessions, messages_metadata
               SQLite FTS5: sessions_fts (full-text search)
Config      → TOML files (unchanged)

Phase 3 (I005): SQLite Extension
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Sessions    → Same as Phase 2
Evolution   → Same SQLite DB, new tables:
               observations, patterns, signals
Config      → TOML files (unchanged)
Skills      → File discovery (~/.talos/skills/, .talos/skills/)
Rules       → TOML/DSL files (.talos/rules/)

Future (post-I007): Possible Turso Migration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Engine      → Replace rusqlite with turso crate
               Same traits, different implementation
               SQLite file format compatible (zero migration)
```

### SQLite Schema (Phase 2, introduced in I004)

```sql
-- Session metadata index
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_hash TEXT NOT NULL,
    started_at TEXT NOT NULL,
    last_active_at TEXT NOT NULL,
    model TEXT,
    turn_count INTEGER DEFAULT 0,
    token_total INTEGER DEFAULT 0,
    summary TEXT
);

-- FTS5 virtual table for session content search
CREATE VIRTUAL TABLE sessions_fts USING fts5(
    session_id,
    content,
    tokenize='porter unicode61'
);
```

### SQLite Schema (Phase 3, extended in I005)

```sql
-- Evolution observations
CREATE TABLE observations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    turn_index INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    outcome TEXT NOT NULL,        -- JSON: TurnOutcome
    signals TEXT NOT NULL,        -- JSON: Vec<Signal>
    tools_used TEXT NOT NULL,     -- JSON: Vec<ToolUsage>
    duration_ms INTEGER NOT NULL
);

-- Learned patterns with cognitive feedback (ADR-001)
CREATE TABLE patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,          -- JSON
    confidence REAL NOT NULL DEFAULT 0.5,
    evidence_count INTEGER NOT NULL DEFAULT 0,
    contradicting_count INTEGER NOT NULL DEFAULT 0,
    last_reinforced TEXT NOT NULL,
    source_sessions TEXT NOT NULL, -- JSON: Vec<Uuid>
    created_at TEXT NOT NULL,
    UNIQUE(pattern_type, key)
);

-- Pattern conflict log
CREATE TABLE pattern_conflicts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    existing_pattern_id INTEGER REFERENCES patterns(id),
    new_value TEXT NOT NULL,
    resolution TEXT NOT NULL,      -- ConflictResolution variant
    resolved_at TEXT NOT NULL
);

CREATE INDEX idx_observations_session ON observations(session_id);
CREATE INDEX idx_observations_timestamp ON observations(timestamp);
CREATE INDEX idx_patterns_type ON patterns(pattern_type);
CREATE INDEX idx_patterns_confidence ON patterns(confidence DESC);
```

### Directory Layout (Final State)

```
~/.talos/
├── config.toml                    # Global configuration (TOML)
├── index.db                       # SQLite database (session index + evolution)
├── AGENTS.md                      # Global agent context
├── sessions/                      # Session JSONL files
│   ├── <project_hash>/
│   │   ├── <uuid-1>.jsonl
│   │   └── <uuid-2>.jsonl
│   └── ...
├── skills/                        # Global skills
│   └── <skill-name>/
│       └── SKILL.md
└── evolution/                     # (handled within index.db tables)

<project>/.talos/
├── config.toml                    # Project-level config overlay
├── AGENTS.md                      # Project agent context
├── skills/                        # Project-level skills
│   └── <skill-name>/
│       └── SKILL.md
└── rules/                         # Project-level permission rules
    └── *.rules
```

## Decision

1. **Storage is introduced progressively**, matching each iteration's actual needs:
   - I001-I003: Pure files (zero database dependency)
   - I004: SQLite via rusqlite (session index + FTS5)
   - I005: SQLite extended (evolution tables with cognitive feedback schema)

2. **SQLite (rusqlite, bundled)** is the storage engine. It is the only option that satisfies
   all hard requirements (FTS5, OLTP, SQL, JSON, production maturity, cross-platform).

3. **All storage operations are abstracted behind traits** (`SessionStore`, `EvolutionStore`)
   to enable future engine migration (e.g., Turso when production-ready) without changing
   calling code.

4. **Session data uses JSONL files as the primary store** (append-friendly, human-readable,
   crash-safe) with SQLite as the metadata index and search engine. Session messages are never
   stored solely in SQLite — JSONL is the source of truth, SQLite is the index.

5. **Evolution data lives entirely in SQLite** (observations, patterns, conflicts). This data
   is structured, queryable, and benefits from SQL's aggregation capabilities.

6. **Config, skills, and rules remain file-based** (TOML/Markdown/DSL). These must be
   human-editable and benefit from git-friendliness.

7. **Future Turso migration is a planned possibility**, not a commitment. The trait abstraction
   ensures migration cost is bounded to implementing new storage backends, not rewriting
   application logic.

## Reversal Trigger

Revisit this decision if:
- rusqlite `bundled` compilation causes unacceptable build times on target platforms.
- SQLite WAL mode proves insufficient for write throughput (unlikely for single-user CLI).
- Turso Database reaches v1.0 with FTS5 compatibility and stable data integrity (evaluate migration).
- A pure-Rust embedded database emerges with FTS + SQL + production maturity (none exists today).
- Session data volume exceeds SQLite's practical limits (>10GB, extremely unlikely for CLI use).
