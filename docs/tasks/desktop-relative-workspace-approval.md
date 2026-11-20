# Desktop approval for a relative workspace

Document status: Active upon merge of #989.

Desktop accepts a workspace path when opening a live session. With a relative path,
the shell tool can receive a relative working directory while the ordinary invocation
review requires an absolute directory. A controlled Desktop runtime test on 98ffdbdc
requested one human approval for pwd where its configured evaluator allows the call.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Resolve the Desktop live runtime's workspace path for shell review and execution, and cover the relative-path case in the host test |
| Claimed At | 2026-11-20 |
| Source Issue | None |
| Governance Claim PR | #989 |
| Authorization Mode | Independent review |
| Authorization Evidence | Session recovery investigation requested by maintainer. Independent permission review pending. |
| Implementation PR | Not started |
| Last Updated | 2026-11-20 |
| Handoff / Release Condition | Merge after review and CI, with regression evidence and durable session compatibility checked |

## Acceptance

With Auto enabled and a relative workspace, a controlled evaluator allowing pwd
receives the ordinary invocation and the shell uses the selected workspace.
Exercise absolute workspaces, disabled Auto and human fallback in the Desktop suite.
Preserve existing durable session identifiers, permissions, and sandbox checks.
Document relative workspace behavior in crates/talos-desktop/README.md.
Run locked workspace preflight and native CI for the implementation candidate.
