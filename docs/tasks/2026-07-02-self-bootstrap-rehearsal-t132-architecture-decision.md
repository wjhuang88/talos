# 2026-07-02 Self-Bootstrap Rehearsal: Associative Memory Policy Decision

**Rehearsal number**: 3
**Plan item**: T132
**Session date**: 2026-07-02
**Runtime**: Talos validation planner on macOS; Codex remained the primary development executor
**Change type**: architecture-sensitive decision slice
**Target**: autonomous validation/self-bootstrap coverage greater than 60%
**Result**: target missed; useful gap evidence only

## Objective

Use an architecture-sensitive slice to test whether Talos can support its own release-readiness
workflow beyond read-only validation planning. The concrete slice was T131: ADR-033 associative
memory injection policy.

## Scope

- **In scope**: Talos `validate plan` on `workspace` and `governance` profiles; T131 ADR evidence;
  REL-002 gap assessment.
- **Out of scope**: claiming a REL-002 qualifying Talos-on-Talos session; automatic validation
  execution; code editing through Talos; release, tag, or publish actions.

## Environment

- Workspace: `/Users/GHuang/WorkSpace/RustProjects/talos`
- T131 commit: `e43dd3c docs(memory): decide associative injection policy (#T131) [model:gpt-5]`
- Talos command surface used: `./target/debug/talos validate plan`

## Execution Record

| Step | Tool(s) used | Outcome | Notes |
|---|---|---|---|
| 1 | Codex + repo docs | success | Read MEM-008, T31/T50/T51 evidence, ADR-016, memory prompt/runtime implementation, and I051/I057 acceptance records. |
| 2 | Codex | success | Authored ADR-033 and synchronized MEM-008, I079, replan, and Board. |
| 3 | `./target/debug/talos validate plan --profile workspace` | success | Talos listed required workspace checks and prerequisites; it did not execute commands. |
| 4 | `./target/debug/talos validate plan --profile workspace --json` | success | Talos produced structured read-only validation metadata for fmt/check/test/governance. |
| 5 | `./target/debug/talos validate plan --profile governance` | success | Talos listed governance validation as required; it did not execute the script. |
| 6 | Codex terminal/tools | success | Codex ran governance validation for the T132 evidence update. |

## Talos-Generated Validation Plan

Workspace profile checks listed by Talos:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

Talos findings:

- Cargo workspace manifest found.
- Governance validator found.
- Read-only plan only: commands are listed but not executed.

External validation for this evidence record:

- `scripts/validate_project_governance.sh .` passed with 0 warnings.

## External Assistance

| Step | What was needed | Who provided it | Why Talos could not |
|---|---|---|---|
| Architecture decision synthesis | Read cross-document constraints and author ADR-033. | Codex | Talos is not currently running as the primary repository-editing agent in this environment. |
| Validation execution | Run governance validation and inspect result. | Codex | T108 intentionally implemented Phase 1 planning only; Phase 2 execution is not implemented. |
| Git operations | Commit and push T131. | Codex | No Talos-owned self-bootstrap git/issue workflow is available. |
| Governance sync | Update replan, I079, Board, and MEM-008. | Codex | Talos has read-only validation planning but no autonomous owner-doc update loop. |

## Coverage Assessment

| Capability | Required For REL-002 | Talos Participation | Assessment |
|---|---|---|---|
| Work selection and context reading | Yes | None beyond Codex invoking Talos validation planner after the decision. | External |
| Architecture-risk classification | Yes | None. ADR-033 was reasoned and written by Codex. | External |
| Validation planning | Yes | Talos produced workspace/governance validation matrices. | Autonomous |
| Validation execution | Yes | None. Talos explicitly reported read-only authority. | External |
| Documentation edits | Yes | None. | External |
| Commit/push/issue sync | Yes | None. | External |

Estimated self-bootstrap coverage: **20%**. Talos contributed useful validation planning, but did
not plan, implement, verify, document, commit, push, or synchronize status as the primary runtime.

## Assessment

- **Would this rehearsal satisfy REL-002?** No.
- **Did it meet T132's >60% autonomous validation target?** No. The target remains unmet because
  Talos still cannot execute validation or capture command evidence.
- **Was the rehearsal useful?** Yes. It confirms the validation planner is stable on an
  architecture-sensitive doc slice and precisely identifies the Phase 2 execution/evidence gap.

## Gaps Exposed

| Gap | Severity | Blocking REL-002? | Recommended fix |
|---|---|---|---|
| Validation planner cannot execute allowlisted checks or store command evidence. | High | Yes | Implement Phase 2 explicit validation execution with no-hidden-pass evidence records. |
| Talos cannot yet author or edit governance/ADR files as the primary runtime. | High | Yes | Add a Talos-owned self-bootstrap editing workflow or record a separate runtime integration path. |
| Architecture-risk classification is not automated or exposed as a Talos command/tool. | Medium | Yes for architecture-sensitive qualifying sessions | Add a read-only architecture-risk review/report surface before any autonomous high-risk execution claim. |
| GitHub issue sync and git push still rely on external tooling. | Medium | No for local-only development; yes for unattended governance parity | Define a permission-gated issue-sync/git-publication workflow or keep it explicitly external. |

## Recovery

- Next work item: T133 publish gate packet.
- REL-002 remains Planned and unsatisfied.
- A future qualifying rehearsal should not be attempted until Phase 2 validation execution exists or
  the rehearsal explicitly scopes itself to gap evidence again.
