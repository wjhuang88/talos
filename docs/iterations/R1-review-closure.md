# R1: Review Closure

## Status: ACTIVE

## Purpose

Close the review drift before starting the next mainline implementation slice. I008 and I009 have
landed runtime work but still have review/status gaps. This round turns those gaps into explicit
closure work so I010 does not start on top of ambiguous iteration state.

## Scope

- Close I008 Learning Agent review by recording fresh runtime evidence for hook-based learning in
  print/mock and TUI/mock paths.
- Close I009 Extensible Agent review by either implementing the remaining `#I009-S1` TUI consumer
  work or moving it through change control into a numbered follow-up story.
- Pause I011 Open Providers after the completed S1 gateway-root slice; defer S2 provider plugin
  architecture until after I010 R2/R3 or an explicit priority change.
- Preserve I010 R2 as the next mainline iteration entry point after R1 closes.

## Selected Work

- [ ] I008 review closure: runtime evidence and status synchronization.
- [ ] I009 review closure: `#I009-S1` consumer work or formal change-control deferral.
- [ ] Documentation/status sync across iteration index, roadmap, requirement convergence, README,
      board, and governance manifest.

## Acceptance Criteria

- [ ] I008 is either moved to Complete with recorded verification evidence or remains Review with a
      concrete blocker recorded in this file.
- [ ] I009 is either moved to Complete or its remaining TUI provenance marker and `/plugins` work is
      moved into a numbered follow-up through change control.
- [ ] I011 status is no longer Active while R1 is the current operating round.
- [ ] I010 R2 remains Planned and is explicitly identified as the next mainline iteration after R1.
- [ ] `docs/BOARD.md` reflects the current operating state and remains a derived view with owner
      docs for every row.
- [ ] No new extensibility, provider, or portable-tool implementation starts during this round.
- [ ] `cargo test --workspace` is recorded before moving R1 to Review or Complete. If only
      documentation changes land before handoff, a structural documentation consistency check is
      acceptable and must be recorded.

## Non-Goals

- Do not implement `#I010-S7` inside R1.
- Do not start I011 S2 provider plugin architecture.
- Do not start I012 Portable Tools.
- Do not expand I009 beyond closing the already-landed provenance consumer gap.

## Risks

- **Review closure expands into product work**: keep I009 limited to marker rendering and
  `/plugins`, or defer through change control.
- **Status sync drift**: update all public status owners in the same session.
- **False closure**: do not mark I008/I009 Complete without runtime evidence or an explicit
  change-control decision for residual work.

## Verification Notes

Append command output and runtime evidence here during execution.

## Execution Record

### 2026-06-03: R1 opened as current operating round

R1 was opened after inventorying active/review/planned iterations:

- I008 is Review with final evidence/status sync pending.
- I009 is Review with TUI provenance marker and `/plugins` consumer work pending.
- I011 S1 has landed; S2 is deferred so the project has only one active operating round.
- I010 remains Planned and becomes the next mainline iteration after R1 closes.

This is a planning/status synchronization slice only; no Rust code changed in this step.

### 2026-06-03: Board guardrails added

Added `docs/BOARD.md` as a derived operating view with strict limits:

- Four columns only: `Item`, `State`, `Owner Doc`, `Gate`.
- No story details, acceptance criteria, execution evidence, or new requirements.
- Status changes must be made in owner docs first, then reflected on the board.
- `AGENTS.md` session-end checklist now includes board synchronization when active/review/paused/next
  work changes.
