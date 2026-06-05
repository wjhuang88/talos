# I020: Exploration Library

**User can**: Ask Talos to research a question, preserve sources and conclusions locally, and later
reuse the result with provenance.

## Status: Planned

## Scope

This iteration implements the first local research-library slice under ADR-017.

## Selected Stories

- [ ] #I020-S1: define research-run, source-card, claim, edge, and synthesis schema.
- [ ] #I020-S2: implement local library storage on bundled SQLite + FTS5.
- [ ] #I020-S3: implement permission-aware research artifact writer.
- [ ] #I020-S4: add vector/graph storage Spike with benchmark and dependency report.

## Acceptance Criteria

- [ ] Research runs store query, plan, sources, extracted claims, synthesis, caveats, and unresolved questions.
- [ ] Stored conclusions cite source IDs and can be traced back to source chunks.
- [ ] Library search works through SQLite FTS without an external service.
- [ ] Vector/graph DB adoption is not implemented without a follow-up ADR.
- [ ] Network/paper search tools remain permission-aware and can be disabled.
- [ ] `cargo test --workspace` passes.

## Out of Scope

- External database servers.
- Cloud sync.
- Fully autonomous unattended web crawling.

