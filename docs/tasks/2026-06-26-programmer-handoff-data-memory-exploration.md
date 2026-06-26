# Programmer Handoff: DATA-001 -> I019 -> I020 Execution Sequence

> Status: Ready for assignment
> Created: 2026-06-26
> Applies to: I048-I056 two-month sequence
> Primary plan: [DATA-001 -> I019 -> I020 Two-Month Execution Plan](2026-06-26-data-memory-exploration-two-month-plan.md)

## Purpose

This handoff tells implementation programmers how to take work from the two-month plan without
breaking Talos storage, memory, permission, or governance boundaries.

The work is intentionally ordered:

1. Finish local data lifecycle controls.
2. Enable memory writes and retrieval only after storage lifecycle is visible and controllable.
3. Build exploration-library storage only after memory provenance is reliable.
4. Close with release-readiness evidence, not an automatic tag.

Do not skip ahead. Later iterations depend on earlier validation evidence, not just code existing.

## Current Baseline

- `v0.1.2` tag has been pushed.
- `main` includes DATA-001 planning and storage lifecycle foundation APIs.
- I047 is still in Review until release workflow evidence is recorded.
- I048-I056 are Planned.
- DATA-001 foundation APIs landed in `71b0392`.
- The two-month planning baseline landed in `2217c36`.

Before starting any assigned slice, pull latest `main`, read this handoff, then read the assigned
iteration document and every Required Read listed by the owning backlog item.

## Non-Negotiable Rules

- Never force-push `main`.
- Never move or recreate `v0.1.2`.
- Never tag or publish a release without explicit architect approval.
- Never delete user data automatically.
- Cleanup commands must support dry-run and must be explicit for apply.
- Memory writes must be ADD-only and evidence-backed.
- Raw session JSONL remains the durable source of truth.
- Prompt injection must never expose hidden tool result content.
- Procedural memory is advisory only; it must not approve permissions or bypass sandbox rules.
- No vector, graph, external NLP, daemon, or paid/network dependency may be added without Spike
  evidence and an ADR.
- Governance owner docs must be updated before `docs/BOARD.md`.

## Required Gates For Every Implementation Slice

Run these before asking for review or handing off:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
```

If a slice only changes planning/governance docs, run at minimum:

```sh
scripts/validate_project_governance.sh .
git diff --check
```

Record actual command results in the owning iteration and, for long-running work, append a
checkpoint to `docs/tasks/2026-06-26-data-memory-exploration-two-month-plan.md`.

## Assignment Map

| Assignment | Owner Iteration | Programmer Profile | Main Deliverable | Must Not Start Until |
|---|---|---|---|---|
| A1 | I049 | CLI + storage | `talos storage status` and cleanup CLI | I047 release evidence disposition and I048 foundation review |
| A2 | I050 | storage/memory | Episodic-to-semantic consolidation pipeline | DATA-001 user-facing lifecycle controls complete or exception recorded |
| A3 | I051 | agent/prompt | Bounded memory prompt injection | I050 consolidation evidence exists |
| A4 | I052 | memory/code intelligence | Procedural memory and entity linking | I051 hidden-output guard passes |
| A5 | I053 | reliability/release | Memory status, retention dry-run, quality gates | I052 passes permission-boundary tests |
| A6 | I054 | storage/research | Exploration library SQLite/FTS foundation | I053 closes I019 quality gate |
| A7 | I055 | workflow/integration | Ingestion and citation workflow | I054 storage foundation complete |
| A8 | I056 | release/governance | Closeout and `v0.2.0` readiness | I055 complete or blockers recorded |

One programmer may take multiple adjacent assignments, but do not parallelize dependent work unless
the upstream API contract is already merged and verified.

## A1: I049 Storage Status And Cleanup CLI

Read first:

- `docs/iterations/I049-storage-status-and-cleanup-cli.md`
- `docs/backlog/active/DATA-001-local-data-lifecycle-storage-hygiene.md`
- `crates/talos-session/src/manager.rs`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-memory/src/lib.rs`
- ADR-002, ADR-008, ADR-016

Expected result:

- `talos storage status` reports local Talos storage without writing files.
- `talos storage cleanup --dry-run` reports candidates without deleting files.
- `talos storage cleanup --apply` deletes only explicitly selected non-active candidates and keeps
  session JSONL/index/fork rows synchronized.
- Maintenance commands call explicit checkpoint/vacuum APIs.

Key risks:

- Active session deletion.
- Cleaning JSONL but leaving stale index/fork rows.
- Treating missing `~/.talos` as an error.
- Reporting hidden content instead of sizes/counts/metadata.

Required tests:

- Temp-home missing, partial, and populated storage status.
- Cleanup dry-run no deletion.
- Cleanup apply deletes JSONL plus index/fork rows.
- Active session protection.
- WAL/index/memory DB size and maintenance surfaces.

## A2: I050 Memory Consolidation Pipeline

Read first:

- `docs/iterations/I050-memory-consolidation-pipeline.md`
- `docs/iterations/I019-layered-memory-foundation.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `crates/talos-memory/src/lib.rs`
- `crates/talos-session/src/`
- ADR-016

Expected result:

- A bounded consolidation path reads episodic session history and writes semantic memory records
  with evidence links.
- Consolidation can run in deterministic tests without live provider credentials.
- Automatic trigger, if added, is conservative and disable-able.

Key risks:

- Treating generated semantic memory as source of truth.
- Overwriting conflicts instead of preserving ADD-only records.
- Provider-dependent tests.
- Writing memory before DATA-001 completion.

Required tests:

- Session JSONL to semantic memory records.
- Evidence link creation.
- Exact dedup by content hash.
- Conflict preservation.
- Malformed/empty session graceful handling.

## A3: I051 Bounded Memory Prompt Injection

Read first:

- `docs/iterations/I051-bounded-memory-prompt-injection.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `crates/talos-agent/src/prompt.rs`
- `crates/talos-agent/src/compaction.rs`
- `crates/talos-agent/prompts/`
- ADR-016

Expected result:

- Retrieved memory can appear in a bounded prompt section with provenance and confidence/freshness
  metadata.
- Memory prompt injection can be disabled.
- Hidden tool result content is never injected.

Key risks:

- Prompt bloat.
- Hidden-output leak.
- Silent contradiction resolution.
- Memory becoming stronger than current user/session context.

Required tests:

- Enabled/disabled prompt injection snapshots.
- Count/token budget truncation.
- Hidden tool-result fixture regression.
- Contradiction marker rendering.
- Mock provider request-preview runtime evidence.

## A4: I052 Procedural Memory And Entity Linking

Read first:

- `docs/iterations/I052-procedural-memory-and-entity-linking.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `crates/talos-tools/src/symbol.rs` if entity extraction uses symbol tooling
- `crates/talos-permission/src/` for permission boundary checks
- ADR-016, ADR-020

Expected result:

- Procedural memories are stored and retrieved as advisory context.
- Entity links improve retrieval for files, symbols, URLs, and simple concepts.
- No external NLP dependency is introduced.

Key risks:

- Procedural memory approving actions.
- Fragile entity extraction that panics on malformed inputs.
- Pulling in large/native dependencies without ADR.
- Creating a second memory model outside `talos-memory`.

Required tests:

- File/path/url/symbol entity extraction.
- Entity overlap retrieval boost.
- Procedural memory ADD-only storage/retrieval.
- Permission regression proving no auto-allow path.

## A5: I053 Memory Quality And Release Hardening

Read first:

- `docs/iterations/I053-memory-quality-and-release-hardening.md`
- `docs/iterations/I019-layered-memory-foundation.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/backlog/active/DATA-001-local-data-lifecycle-storage-hygiene.md`

Expected result:

- Memory status and retention dry-run are visible and safe.
- I019 can move to Review only if acceptance evidence is complete.
- Memory startup degrades safely on missing/corrupt DB cases.

Key risks:

- Marking I019 complete with missing evidence.
- Dry-run accidentally deleting or mutating memory.
- Exposing memory content where only counts/status should be shown.
- Treating ranking/retention as semantic overwrite.

Required tests:

- Memory status counts/sizes.
- Retention dry-run no deletion.
- Corrupt/missing DB graceful behavior.
- End-to-end mock runtime with memory enabled.

## A6: I054 Exploration Library Storage Foundation

Read first:

- `docs/iterations/I054-exploration-library-storage-foundation.md`
- `docs/iterations/I020-exploration-library.md`
- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/decisions/017-exploration-library-storage.md`
- ADR-008

Expected result:

- Exploration storage persists research runs, sources, chunks, claims, claim edges, syntheses,
  caveats, and unresolved questions.
- FTS5 source/chunk search works offline.
- Citation targets are validated.

Key risks:

- Starting network ingestion before storage integrity exists.
- Adding vector/graph dependencies.
- Allowing syntheses without traceable source IDs.

Required tests:

- Schema migration.
- Source/chunk/claim/synthesis round trips.
- FTS search.
- Citation integrity failure.

## A7: I055 Exploration Ingestion And Citation Workflow

Read first:

- `docs/iterations/I055-exploration-ingestion-and-citation-workflow.md`
- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- Existing fetch/save_url/http_request permission paths

Expected result:

- Local text ingestion creates searchable chunks and claims.
- Permission-aware fetched content can be ingested with source provenance.
- Syntheses distinguish cited evidence from inference.

Key risks:

- Bypassing network permissions.
- Building a crawler.
- Creating uncited conclusions.
- Requiring paid APIs for tests.

Required tests:

- Local ingestion.
- Mock fetch ingestion.
- Citation-preserving synthesis.
- Network-disabled behavior.

## A8: I056 Closeout And v0.2.0 Readiness

Read first:

- `docs/iterations/I056-two-month-closeout-and-v020-readiness.md`
- All changed owner iterations and backlog items from I049-I055
- Release workflow and installer docs

Expected result:

- DATA-001, I019, and I020 statuses are honest and synchronized.
- README/user docs match shipped behavior.
- Full workspace gates pass.
- Architect receives a release/no-release decision package.

Key risks:

- Tagging without approval.
- Closing iterations with missing evidence.
- Leaving Board inconsistent with owner docs.
- Hiding residual work in prose instead of backlog/iteration owners.

Required checks:

- Full workspace gates.
- Runtime smoke tests for storage, memory, and exploration paths.
- Governance validation.
- Release checklist review.

## Programmer Work Protocol

### Start Of Assignment

1. Pull latest `main`.
2. Confirm `git status --short` is clean.
3. Read this handoff.
4. Read assigned iteration doc.
5. Read all Required Reads from the relevant backlog owner.
6. Append an activation record to the assigned iteration before implementation.
7. Update `docs/BOARD.md` only after owner docs are updated.

### During Implementation

- Keep commits small and logical.
- Prefer existing crate boundaries and local patterns.
- Add tests near the changed behavior.
- Use temp dirs for cleanup/storage tests.
- Do not broaden scope to adjacent iterations.
- Record deviations immediately in the owning iteration.

### End Of Assignment

Before handoff or review, update:

- assigned iteration execution table;
- relevant backlog acceptance/progress notes;
- long task checkpoint;
- README/user docs if behavior changed;
- Board after owner docs.

Then run required gates and commit with the project commit format:

```text
<type>(<scope>): <description> (#<story-or-iteration>) [model:<model-name>]
```

Example:

```text
feat(cli): add storage status command (#DATA-001) [model:gpt-5]
```

## Handoff Note Template

Use this when passing work to the next programmer:

```markdown
## Handoff: <iteration / assignment>

Status:
Commit(s):

Completed:
- <item>

Changed files:
- <path>

Validation:
- `cargo fmt --all -- --check`: <result>
- `cargo check --workspace`: <result>
- `cargo clippy --workspace -- -D warnings`: <result>
- `cargo test --workspace`: <result>
- `scripts/validate_project_governance.sh .`: <result>

Important decisions:
- <decision>

Known residuals:
- <residual>

Next step:
- <next action>

Recovery:
- branch/ref:
- owner docs:
- commands to rerun:
```

## Escalation Rules

Stop and ask the architect before proceeding if any of these happen:

- A task requires deleting real user data.
- A release tag, GitHub Release, or version bump is needed.
- A new native, vector, graph, NLP, daemon, or paid/network dependency appears necessary.
- A memory feature would influence permissions, sandboxing, or write approval.
- A gate repeatedly fails after two concrete fixes.
- Requirements conflict with ADR-002, ADR-008, ADR-016, ADR-017, or project `AGENTS.md`.

For ordinary uncertainty, choose the safer reversible option and record the assumption in the
iteration checkpoint.
