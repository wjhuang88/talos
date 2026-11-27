# PROVIDER-TEMPERATURE-001 OpenAI chat sampling temperature

Status: Intake / Unclaimed. Separate claim proposed in #1047; this registration does not activate it. Source [Issue #1046](https://github.com/wjhuang88/talos/issues/1046).

## Scope and acceptance

Expose an opt-in OpenAI-compatible Chat Completions temperature setting for SDK hosts that import published model configurations. Reject non-finite or out-of-range values (inclusive range 0–2, as specified in #1046). Send the configured temperature on ordinary requests. Preserve omission by default and isolated settings for decision and permission-review requests. The slice covers the provider API, wire tests, and API documentation. It requires no new dependencies.

Validate with `cargo test --locked -p talos-provider`, the governance validators, and CI on the review revision. The #1046 submitter reports a local prototype against `088c7f2d` passing provider tests, including ordinary streaming and isolated review HTTP verification; the intake registration does not independently verify this evidence. The separate Codex implementation may proceed only after #1047 establishes an effective claim on main.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #1046 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Intake registration only; separate #1047 claim remains proposed, not effective. |
| Implementation PR | Not started |
| Last Updated | 2026-11-27 |
| Handoff / Release Condition | Effective claim, reviewed implementation, and a compatible published release |
