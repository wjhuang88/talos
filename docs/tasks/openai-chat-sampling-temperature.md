# PROVIDER-TEMPERATURE-001 OpenAI chat sampling temperature

Status: Active upon merge. Source [Issue #1046](https://github.com/wjhuang88/talos/issues/1046).

## Scope and acceptance

Expose an opt-in OpenAI-compatible Chat Completions temperature setting for SDK hosts that import published model configurations. Reject non-finite or out-of-range values (inclusive range 0–2, as specified in #1046). Send the configured temperature on ordinary requests. Preserve omission by default and isolated settings for decision and permission-review requests. The slice covers the provider API, wire tests, and API documentation. It requires no new dependencies.

Validate with `cargo test --locked -p talos-provider`, the governance validators, and CI on the review revision. The #1046 submitter reports a local prototype against `088c7f2d` passing provider tests, including ordinary streaming and isolated review HTTP verification. This claim does not independently verify that evidence. The separate Codex implementation may proceed only after #1047 establishes an effective claim on main.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | PROVIDER-TEMPERATURE-001 OpenAI chat sampling configuration |
| Claimed At | 2026-11-27 |
| Source Issue | #1046 |
| Governance Claim PR | #1047 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized submitting the tested upstream change. Review and CI are pending. |
| Implementation PR | Not started |
| Last Updated | 2026-11-28 |
| Handoff / Release Condition | Effective claim, reviewed implementation, and a compatible published release |
