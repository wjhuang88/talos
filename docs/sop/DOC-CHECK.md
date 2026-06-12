# SOP: Documentation Sync Check

## Purpose

Keep documentation honest. Documentation drift — where docs claim a state the code does not
have — is a correctness defect, not cosmetic. This SOP defines the checks that keep the
governance and user-facing docs synchronized with the actual codebase.

> Originating lesson (I008): the iteration doc, README, and roadmap all marked the
> self-evolution engine COMPLETE while the feature was never wired into the binary. Status
> claims must trace to runtime reality.

## When to Run

- Before marking any story or iteration Done.
- At the Session End Checklist (see `AGENTS.md`).
- During a governance audit.

## Authoritative Status Sources

There is exactly one source of truth for each fact. All other mentions must agree with it:

| Fact | Source of truth |
|------|-----------------|
| Iteration state (Planned/Active/Review/Complete) | `docs/iterations/README.md` table + the iteration file |
| What the user can do per iteration | `docs/iterations/IXXX-*.md` (with runtime evidence) |
| Test count / overall status | `cargo test --workspace` output |
| Governance capability state | `.agent-governance/manifest.yaml` |
| Governance skill refresh state | `.agent-governance/manifest.yaml` `governance.skill_version` and `governance.last_refresh` |
| Governance profile / branch / worktree recommendation | `scripts/assess_project_scale.sh .` output |
| Public-facing status | `README.md` / `README.zh-CN.md` |

## Checklist

- [ ] `README.md` "Status" line agrees with the iterations table (no I004-vs-I008 contradiction).
- [ ] `README.md` and `README.zh-CN.md` say the same thing (bilingual parity).
- [ ] Every iteration marked Complete has recorded **runtime** evidence, not only unit tests
      (see `ITERATION-WORKFLOW.md` §3a).
- [ ] `docs/iterations/README.md` "Current Iterations" table reflects every existing iteration.
- [ ] `.agent-governance/manifest.yaml` `status`, `last_audited_at`, `governance.skill_version`, `governance.last_refresh`, and `next_actions` are current.
- [ ] If the governing skill was updated, the affected capability was compared with the current `agent-project-governance` references before trusting existing SOPs.
- [ ] `scripts/assess_project_scale.sh .` still supports the manifest profile, branch mode, and worktree mode assumptions.
- [ ] Test counts cited in docs match actual `cargo test --workspace` output.
- [ ] No doc claims a feature works that produces a `never used` / `never constructed` warning
      on its core type.
- [ ] ADRs referenced from docs exist under `docs/decisions/`.

## Validation

Run the project-local governance validator before trusting the doc state:

```bash
scripts/validate_project_governance.sh .
```

When profile, branch mode, worktree mode, or governance depth may have changed, also run:

```bash
scripts/assess_project_scale.sh .
```

A failing check, a stale status owner, or an unregistered residual gap means documentation is
**not** in sync — fix before claiming completion.
