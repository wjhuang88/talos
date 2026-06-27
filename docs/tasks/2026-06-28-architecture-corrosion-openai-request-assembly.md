# 2026-06-28 Architecture Corrosion: OpenAI Request Assembly Decomposition

**Status**: Complete
**Parent task**: `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
**Iteration**: I073
**Backlog story**: ARCH-028

## Requested Outcome

Reduce `talos-provider/src/openai.rs` responsibility by moving Chat Completions request DTOs and
request body assembly into a focused module without changing provider behavior.

## Artifacts To Change

- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/openai_request.rs`
- `crates/talos-provider/src/lib.rs`
- `docs/backlog/active/ARCH-028-openai-request-assembly-decomposition.md`
- `docs/iterations/I073-openai-request-assembly-decomposition.md`
- `docs/tasks/2026-06-27-two-month-architecture-optimization-plan.md`
- `docs/BOARD.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/iterations/README.md`

## Success Criteria

- OpenAI request DTOs and body assembly live in a focused module.
- Request body shape and secret redaction remain covered by existing tests.
- `openai.rs` drops materially below the 1001-line baseline.
- Provider targeted tests and workspace validation pass.

## Checkpoints

| Date | State | Evidence | Next |
|---|---|---|---|
| 2026-06-28 | Started | Selected request assembly extraction because it is behavior-preserving and already covered by request-body serialization tests. | Extract module and run validation. |
| 2026-06-28 | Complete | `talos-provider/src/openai.rs` 1001→848 lines; `openai_request.rs` owns request DTOs/body assembly/redaction; `cargo test -p talos-provider --quiet`, `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --quiet`, `scripts/validate_project_governance.sh .`, and `git diff --check` passed. | Continue M9 exploration/tools/storage. |
