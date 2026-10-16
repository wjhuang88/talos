# SKILL-006: Embedded Skill Progressive Disclosure

**Status**: Complete / Closed; Completion Commit `6d72975e`
**Type**: Product / Runtime API Story
**Parent Epic**: None; related SKILL-002 (CLI explicit activation, Complete)
**Source Issue**: #719
**Iteration**: [I303](../../iterations/I303-embedded-skill-progressive-disclosure.md)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | OpenCode / Claude Opus 5.5 |
| Work Slice | #719 A-D: shared confined Skill Level 1/2 disclosure in talos-skill, load_reference hardening, CLI reuse, RuntimeHandle skill-context methods, opt-in runtime skill_read tool, tests and docs only. |
| Claimed At | 2026-10-15 |
| Source Issue | #719 |
| Governance Claim PR | #720 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer selected full A+B+C+D scope and Single-maintainer merge on 2026-10-15; no distinct maintainer reviewer available. Independent Agent security/API review, exact-head CI, both validators and merge-time CAS are mandatory. |
| Implementation PR | #721 (merged `6d72975e`) |
| Last Updated | 2026-10-15 |
| Handoff / Release Condition | No implementation before this claim and ADR-092 reach main; release only after evidence-bearing closeout or explicit handoff. |

## Identity / Goal / Value

An embedder using only `talos-runtime` (e.g. OBeiBuddy) needs the model to follow a bound Skill's
full `SKILL.md` and reference files, not just its Level 0 name/description, without each host
re-implementing path confinement, budgets and chunking.

## Scope

Per [ADR-092](../../decisions/092-embedded-skill-progressive-disclosure.md):

- A: public confined Level 1/2 disclosure helpers in `talos-skill`; `talos-cli` reuses them;
  `RuntimeHandle` set/clear skill-context methods; root re-exports for single-dependency use.
- B: opt-in `RuntimeBuilder` binding that registers a read-only multi-skill `skill_read` tool.
- C: `SkillManager::load_reference` rejects absolute/prefix/`..`, symlink escape, non-regular and
  oversize files.
- D: rustdoc, Runtime SDK contract and README (en/zh) updates including migration notes.

## Exclusions

No `SessionOp`/`talos-core` protocol change, fuzzy trigger-based auto-activation, startup loading
of all bodies, durable persistence of the activated context, `talos-permission`/sandbox change,
new dependency, TUI/Desktop UI, or release.

## Dependencies / Decision Links And Constraints

ADR-092 (accepted with this claim), ADR-074 prompt authority, ADR-024/052 runtime API boundary,
RUNTIME-006 facade. SKILL-002 CLI behavior and budgets must be preserved.

## Acceptance

- Given a runtime built only through `talos-runtime` public API with a parsed skill, when the
  embedder sets the skill context and submits, then the next provider request contains the
  `SKILL.md` body; clearing removes it; setting during an active turn yields the documented error.
- Given a confined read, when the path is absolute, contains `..`, is a symlink resolving outside
  the skill directory, a directory, oversize or non-UTF-8, then it is rejected; a valid nested
  reference is returned (tests on Unix symlinks; Windows skips symlink creation if unprivileged).
- Given `SkillManager::load_reference`, the same rejections hold (regression tests).
- Given `skill_read` enabled with two bound skills, the model can read each body plus listing and a
  reference with offset continuation within budget; unknown skill names are rejected; without the
  opt-in, or with an empty binding, no `skill_read` tool is registered; a caller tool of the same
  name is kept.
- CLI `/skills activate` and `/skills reference` keep their existing output and budgets.

## Planned Validation / Documentation

Focused locked tests for talos-skill, talos-runtime and talos-cli; `./scripts/release_preflight.sh`
on pinned Rust; both governance validators; exact-head CI; independent Agent security/API review.
Update `RuntimeBuilder::skill_index` rustdoc, `docs/reference/RUNTIME-SDK-CONTRACT.md`,
`README.md` / `README.zh-CN.md` Skills sections, then owner, iteration, backlog and Board.

## Residual Destination

Symlink check/use race and prompt-guidance policy remain documented residuals in ADR-092/I303.
No Complete state without an already-existing implementation SHA.

## Completion — 2026-10-15

Completion Commit: `6d72975e` (merge of #721).
Exact-head CI, independent Agent security/API review APPROVE and merge-time CAS passed. Evidence is owned by
[I303](../../iterations/I303-embedded-skill-progressive-disclosure.md). Residuals stay in ADR-092.
