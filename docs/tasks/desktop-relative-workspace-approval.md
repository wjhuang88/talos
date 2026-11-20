# Desktop approval for a relative workspace

> Document status: Complete.

Completion Commit: 4462a35b1a18ef05ee3f3da8150fac66216623e5

Desktop accepts a workspace path when opening a live session. With a relative path,
the shell tool can receive a relative working directory while the ordinary invocation
review requires an absolute directory. A controlled Desktop runtime test on 98ffdbdc
requested one human approval for pwd where its configured evaluator allows the call.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Resolve the Desktop live runtime's workspace path for shell review and execution, and cover the relative-path case in the host test |
| Claimed At | 2026-11-20 |
| Source Issue | None |
| Governance Claim PR | #989 |
| Authorization Mode | Independent review |
| Authorization Evidence | Session recovery investigation requested by maintainer. Independent reviewer approved candidate 30146940. The maintainer authorized merge of #990. |
| Implementation PR | #990 |
| Last Updated | 2026-11-21 |
| Handoff / Release Condition | Merge after review and CI, with regression evidence and durable session compatibility checked |

## Acceptance

With Auto enabled and a relative workspace, a controlled evaluator allowing pwd
receives the ordinary invocation and the shell uses the selected workspace.
Exercise absolute workspaces, disabled Auto and human fallback in the Desktop suite.
Preserve existing durable session identifiers, permissions, and sandbox checks.
Document relative workspace behavior in crates/talos-desktop/README.md.
Run locked workspace preflight and native CI for the implementation candidate.

## Delivery

PR #990 merged into main at 7d7e6170. The merged Desktop files match the reviewed
candidate. The Linux runtime test executed pwd in the selected relative workspace
with zero manual approvals. The Linux Desktop suite passed 117 tests and native
Windows Desktop passed 111, including the relative workspace regression.
Workspace release preflight passed. Independent review confirmed the existing
durable session identifiers and permission checks are preserved.

CI run 44666599627 succeeded on its second attempt. The global Issue reconciliation
job reported index drift for #991 and #642. The first Windows workspace job was
cancelled during cache saving after its test steps passed. Its rerun succeeded.
