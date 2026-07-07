# Programmer Handoff: Four-Month Developer Operating Plan

Use this prompt when assigning the plan to a developer or another agent.

```text
You are taking over Talos implementation work in /home/playground/workplace/talos.

Objective:
Execute the planned four-month developer operating package in
docs/tasks/2026-07-07-four-month-developer-operating-plan.md. Work only inside the active monthly
iteration selected from I102-I105. Start with I102 unless the maintainer explicitly activates a
later iteration.

Required reads before editing:
- AGENTS.md
- docs/tasks/2026-07-07-four-month-developer-operating-plan.md
- docs/iterations/I102-provider-runtime-reliability-gate.md
- docs/sop/LONG-RUNNING-TASK.md
- docs/sop/ITERATION-WORKFLOW.md
- docs/sop/GIT-WORKFLOW.md
- docs/sop/DOC-CHECK.md
- docs/BOARD.md
- docs/backlog/PRODUCT-BACKLOG.md
- every owner doc listed under the selected task's Required Reads

Hard boundaries:
- Do not tag, push, publish crates, create GitHub Releases, deploy, invite external trial users, or
  run destructive cleanup unless the maintainer separately authorizes it.
- Do not change permission deny/allow precedence, sandbox behavior, process hardening, provider
  credential persistence, session storage defaults, or release gates without senior review and the
  required ADR/owner-doc update.
- Do not add new dependencies unless the active task explicitly requires review and the review is
  complete.
- Do not resurrect runtime catalog.db behavior.

Execution rules:
1. Run git status --short and record whether the worktree is clean.
2. Read the active iteration and pick the lowest-numbered Planned task whose dependencies are met.
3. Before coding, state the exact files you expect to change and the validation commands you expect
   to run.
4. Implement the smallest change that satisfies the task acceptance.
5. Run targeted tests first, then the monthly gates when closing the iteration:
   cargo fmt --all -- --check
   cargo check --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   scripts/validate_project_governance.sh .
   git diff --check
6. Record runtime evidence for behavior-facing changes in the iteration file.
7. Update owner docs before derived docs such as docs/BOARD.md.
8. Record residuals in the iteration or backlog owner doc. Do not leave residuals only in chat.

Stop and ask for review if a task requires a forbidden change, if owner docs conflict in a way that
changes scope, or if three materially different implementation attempts fail.

Current starting point:
I102 is Planned. Its first task is D100, the start-gate inventory and current regression check.
```
