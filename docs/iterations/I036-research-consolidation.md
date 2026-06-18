# I036: Research Consolidation

**Status**: Planned
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

- [ ] #I036-S1: Inventory current research requirements and classify each as Research, Spike,
      Proposal, ADR-needed, or Ready-for-Story.
- [ ] #I036-S2: Analyze REMOTE-001 and WEB-001 together and decide whether they share a handler
      backbone, auth model, and read-only first slice.
- [ ] #I036-S3: Advance PLUGIN-001 by turning the WASM plugin protocol proposal into an ADR-ready
      design outline, including tools/hooks/filters, host calls, resource limits, and failure
      policy.
- [ ] #I036-S4: Refresh OKF-001 from the live upstream SPEC, record the fetched commit/date, and
      decide produce/consume/bidirectional direction for Talos.
- [ ] #I036-S5: Reconcile MEM-005 context compaction policy with session/resource usage UX so
      future long-session work has clear trigger and observability rules.
- [ ] #I036-S6: Produce a follow-up execution map: which items become ADRs, which become
      implementation stories, which stay deferred, and which dependencies block each path.

## Research Inputs

- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/OKF-001-native-okf-support.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/AGENT-001-standard-agent-protocol-support.md`
- `docs/backlog/active/MCP-001-session-mcp-integration.md`
- `docs/backlog/active/SKILL-001-runtime-skill-activation.md`
- ADR-006, ADR-009, ADR-010, ADR-013, ADR-016, ADR-017, ADR-021

## Acceptance Criteria

- [ ] Research inventory distinguishes confirmed facts, assumptions, volatile upstream inputs,
      and implementation dependencies.
- [ ] REMOTE-001 and WEB-001 have one documented convergence decision: shared backbone,
      separate surfaces, or deferred decision with explicit blocker.
- [ ] PLUGIN-001 has an ADR-ready outline before any WASM runtime dependency is proposed.
- [ ] OKF-001 cites the live upstream SPEC commit/date used for the research.
- [ ] MEM-005 has clear integration notes for token/resource visibility and TUI exit/session
      summary behavior.
- [ ] Backlog rows and proposal docs are synchronized with the research outcomes.
- [ ] No implementation code is changed except documentation, proposals, ADRs, or tests for
      governance validation.
- [ ] A final execution map names the next concrete implementation iteration(s), if any.

## Risks

- Remote, web, and plugin work can all pressure the same permission and event-flow boundaries.
  Keep ADR-006 and the permission pipeline as hard constraints.
- OKF upstream may change; all OKF conclusions must cite the exact live source version.
- WASM runtime selection is dependency-sensitive under ADR-010; do not choose a runtime without
  explicit ADR review.
- Research can sprawl. If a question cannot be closed inside I036, record the blocker and keep it
  deferred instead of starting implementation.

## Verification Log

(to be filled as stories land)
