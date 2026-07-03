# I090-I093 High-Risk Execution Closeout

**Date**: 2026-07-04
**Track**: Architect-owned four-month high-risk execution
**Task owner record**: `docs/tasks/2026-07-04-architect-owned-four-month-high-risk-execution.md`
**Verdict**: Complete as a direct-owner hardening track; REL-002 remains No-go for `v1.0.0`

## Executive Summary

The direct-owner track completed four high-risk monthly packets in one unattended execution window:

- I090 bounded local document extraction and stabilized ripgrep-backed search.
- I091 added read-only hook diagnostics, hook manifest declaration validation, and optional asset
  distribution policy.
- I092 added bash-only cache-safe compression evidence and the autonomy permission matrix.
- I093 updated self-bootstrap readiness and recorded a non-qualifying REL-002 evidence packet.

No release tag, publish action, GitHub Release, remote install, marketplace behavior, permission
default change, browser/PDF/Office/OCR expansion, or `v1.0.0` claim was made.

## Four-Month Matrix

| Iteration | Focus | Delivered | Verification |
|---|---|---|---|
| I090 | Tool/ingestion permission boundary | Unsupported binary/document formats rejected before text extraction; local search output and input budgets enforced without host `rg`. | Targeted tool tests, full workspace tests, governance validation. |
| I091 | Plugin/hook/distribution boundary | `/hooks` diagnostics, hook manifest declaration validation, and optional runtime asset distribution policy. | Conversation/plugin tests, full workspace tests, governance validation. |
| I092 | Context compression and autonomy gates | Bash compression stable-prefix/export regressions and deny/ask/allow autonomy matrix. | Agent/permission/tool targeted tests, full workspace tests, governance validation. |
| I093 | Runtime/governance/release posture | REL-002 readiness report plus non-qualifying self-bootstrap evidence record. | CLI version smoke, governance validation, and final workspace gates passed. |

## Residual Owners

| Residual | Owner | Next Action |
|---|---|---|
| Real terminal `/connect` walkthrough | I085 / MC-001 | Resume I085 only to run and record the manual TUI walkthrough, then close or move to Review. |
| Talos-primary self-bootstrap loop | REL-002 / RUNTIME-001 / GOV-003 | Build a controlled documentation-only edit loop where Talos plans, edits, validates, and records evidence as primary runtime. |
| Validation execution evidence | RUNTIME-001 / GOV-003 | Add allowlisted validation execution with command, output summary, exit status, and permission decision records. |
| Mutating governance workflow | GOV-003 | Add typed plan/preview/write flow before any self-bootstrap claim. |
| Session SQLite continuity risk | ARCH-030 | Audit or split schema/search/fork paths before continuity becomes self-bootstrap-critical. |
| Git publication boundary | REL-002 / Git tool owners | Decide whether Talos gets permission-gated git/issue publication or REL-002 keeps release-operator actions external. |
| Plugin/package distribution runtime | DIST-001 / PLUGIN-001 | Require follow-up ADR before downloader, installer, marketplace, or remote plugin install behavior. |
| Non-bash compression | MEM-007 | Extend only after stable-prefix/raw-export evidence exists for each tool family. |
| Scheduled direct execution / exec DSL / Guardian auto-approval | SCHED-001 / TOOL-010 / PERM-001 | Keep disabled until deny/ask/allow regressions and ADR gates exist. |

## Release Posture

| Question | Answer |
|---|---|
| Can Talos claim `v1.0.0` readiness? | No. REL-002 remains unsatisfied. |
| Are pre-1.0 hardening releases allowed? | Yes, if release notes keep the pre-1.0 posture explicit and do not imply self-bootstrap completion. |
| Were any publish/tag/release actions performed in this track? | No. |
| Did Codex remain primary executor for I090-I093? | Yes. I093 A14 records that this is non-qualifying for REL-002. |

## Validation Evidence

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

## Recovery

- To resume the direct-owner track, start from the task owner record and this closeout.
- Do not reopen I090-I093 for new objectives; create a new iteration ID for changed acceptance or
  behavior.
- The next highest-value REL-002 packet is a Talos-primary documentation-only edit rehearsal with
  Codex limited to review/fallback.
