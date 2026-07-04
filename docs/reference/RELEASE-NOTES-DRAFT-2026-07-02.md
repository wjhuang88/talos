# Release Notes Draft: Post-v0.2.0 Hardening

**Status**: Draft only
**Created**: 2026-07-02
**Scope**: Work completed after the published `v0.2.0` line and before any future release tag.

This is not a release announcement. It is a consolidation artifact for T134/T135/T136 so the next
release owner can prepare a real changelog without mining every iteration record.

## Highlights

- Provider usage accounting now requests and parses OpenAI-compatible streaming usage chunks.
- TUI context display, million-token formatting, write/edit result previews, tool output hierarchy,
  slash command auto-execution, session todos, and transient thinking previews were hardened.
- Session todo storage, tools, read-only slash/TUI views, and bounded prompt integration landed.
- Direct `exec` is implemented with explicit permission facets, argv-only execution, sensitive env
  denial, bounded output, and timeout behavior.
- WASM plugin work moved from adapter-only to read-only `AgentTool` integration behind the `wasm`
  feature, with provenance and permission coverage.
- `talos validate plan` provides a read-only validation matrix for governance/workspace profiles;
  `talos validate run` now executes built-in allowlisted profiles and emits command/status/output
  evidence.
- `talos governance iteration-record preview/write` provides a narrow owner-doc mutation gate with
  explicit preview, `--confirm-preview`, post-write governance validation, and rollback on
  validation failure.
- Release/readiness work now includes publish guard checks, a publish gate packet, and explicit
  REL-002 self-bootstrap gap evidence.

## Fixes And Hardening

- Removed the remaining active ignored source test by replacing sleep-based session timing with
  event-queue synchronization.
- Suppressed intentional `talos-runtime` example-helper dead-code warning noise.
- Added `talos-dashboard` to the product-only publish guard and publication matrix.
- Kept automatic associative memory injection disabled and recorded ADR-033 for the policy.

## Known Gaps

- REL-002 remains unsatisfied: Talos can execute allowlisted validation profiles with evidence, but
  cannot yet edit repo files, commit, push, or sync issues as the primary runtime.
- `talos-runtime` cannot be published until dependency closure is safe or decoupled; its current
  dry-run is blocked by unpublished `talos-agent`.
- `talos-cli` crates.io install remains blocked by `publish = false`; release archives and source
  builds remain the supported install paths.
- Real publish, tags, GitHub Releases, crate name reservation, browser automation, remote dashboard
  access, and permission default changes remain unapproved.

## Validation References

- `docs/reference/PUBLISH-GATE-PACKET-2026-07-02.md`
- `docs/tasks/2026-07-02-t130-tool-reliability-sweep.md`
- `docs/tasks/2026-07-02-self-bootstrap-rehearsal-t132-architecture-decision.md`
- `docs/iterations/I079-month4-release-readiness-handoff.md`
