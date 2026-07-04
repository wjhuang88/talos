# I097/B9 Controlled Self-Bootstrap Rehearsal Evidence

> Date: 2026-07-04
> Iteration: I097 Controlled Self-Bootstrap Rehearsal
> REL-002 verdict: Does not qualify

## Objective

Attempt one documentation-only self-bootstrap rehearsal using the Talos capabilities added in I095
and I096, then record the primary-executor boundary honestly.

## Talos-Executed Steps

Talos executed the allowlisted governance validation profile:

```sh
cargo run -p talos-cli -- validate run --profile governance --json
```

Actual evidence:

- `authority`: `allowlisted validation execution; no arbitrary commands accepted`
- profile: `governance`
- command: `scripts/validate_project_governance.sh .`
- permission decision: `allowlisted validation profile: governance`
- status: `passed`
- exit status: `0`
- stdout summary: `Governance validation passed: 0 warning(s).`
- stderr summary: `<empty>`

Talos then executed the bounded governance mutation gate:

```sh
cargo run -p talos-cli -- governance iteration-record write \
  --iteration I097 \
  --date 2026-07-04 \
  --record-type execution \
  --record "Controlled self-bootstrap rehearsal attempted with Talos validation and governance mutation commands. Codex remained primary for planning, evidence interpretation, docs editing, validation orchestration, commit, and push; therefore the record is non-qualifying for REL-002." \
  --confirm-preview
```

Actual evidence:

- preview printed the owner doc: `docs/iterations/I097-controlled-self-bootstrap-rehearsal.md`;
- preview printed the post-write validator: `scripts/validate_project_governance.sh .`;
- preview printed the exact row to append;
- write reported `Write: applied`;
- post-write validation reported `Validation: passed`.

## Primary Executor Boundary

This rehearsal does not qualify for REL-002 because Codex remained the primary executor for:

- selecting and interpreting the rehearsal;
- deciding the qualification result;
- editing this evidence packet and the cross-document closeout;
- orchestrating validation beyond the two Talos commands above;
- preparing the commit and push.

Talos materially contributed validation evidence and one bounded owner-doc mutation, but it did not
perform the full development loop as the primary runtime.

## Result

REL-002 remains No-go for `v1.0.0`. The minimum next packet is a real Talos-primary session where
Talos owns planning, owner-doc edits, validation orchestration, residual reporting, and handoff,
with Codex limited to labeled review or fallback.
