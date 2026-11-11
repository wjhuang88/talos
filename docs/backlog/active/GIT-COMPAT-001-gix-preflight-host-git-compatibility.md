# GIT-COMPAT-001: Git 2.36 Prepared Preflight Compatibility

**Status:** Intake / Unclaimed; proposed I324 governance claim is ineffective until its merge.

**Type:** Git Tools / Security-Preserving Compatibility Repair

**Source Issue:** [#919](https://github.com/wjhuang88/talos/issues/919)

**Source design:** [gix-assisted system-Git preflight proposal](../../proposals/gix-preflight-system-git-compatibility.md)

**Related authority:** ADR-089, I300, completed I309 (gix 0.90.0), GIT-SNAPSHOT-001 (separate Review owner), Windows native report of 2026-11-10.

## Outcome

Deliver reproducibly functional structured Git Checkout/Commit preparation on Windows with system Git 2.36.1, without reducing the exact prepared permission frontier, immutable source evidence, execution-program binding or post-approval revalidation. Retain actual system-Git execution; do not silently bypass unsupported configuration or attribute discovery.

## Bounded scope

1. Audit the *specific* selected gix 0.90.0 path/config APIs for hidden PATH discovery, subprocesses, environment snapshot mismatch, process-global cache and panic/IO. Adopt only audited primitives with per-invocation evidence.
2. Provide Git 2.36-compatible configuration and system/global attributes path discovery (including missing/empty sources). Any host probe must run through the existing bounded, admitted Git executable with frozen cwd/env. No guessed installation paths or unauthorized file-body reads.
3. Verify the exact effect of the Windows/Scoop `GIT_INSTALL_ROOT` environment variable before any narrow compatibility admission. Keep dangerous `GIT_*` redirection/config injection/trace output rejected; `GIT_PAGER` is already allowed.
4. Preserve hooks/filters, URL rewrite and tracking interpretation or explicitly refuse when their semantics cannot be represented; do not claim a fix for all Windows Git failures from one preflight patch.
5. Prove source path/executable/config/include changes invalidate prior admission; test mixed Git versions, HOME/XDG, includes, attribute paths, Unicode/spaces, multiple installations and >16MiB source checkout/commit.
6. Update user Git support documentation, exact-head test record and independent review evidence.

## Exclusions and dependency gates

- Does not re-open or change I300's completed contract; preserve ADR-089. If the approved discovery commands or named capability need changing, obtain explicit ADR-089 amendment/maintainer architecture decision *before* modifying production behavior.
- Does not own GIT-SNAPSHOT-001's 16MiB/digest/auxiliary budget fixes, TUI-APPROVAL-001, PERM-007, process-hardening, sandbox, search, provider, Server or other Git operation expansion.
- No replacing system Git with gix; no new dependency/version bump/unsafe; no automatic fallback from missing admission.
- Implementation may start only after effective governance claim and activation on main, recorded no-overlap CAS and any necessary ADR decision.

## Acceptance and verification

- Negative tests reproduce Git 2.36 `git var GIT_ATTR_*` unsupported results; positive tests show the exact approved preflight path succeeds without probing unapproved content.
- Real Git 2.36.1.windows.1: native Git tools tests and large checkout/commit pass in original Scoop environment, with any remaining failures independently classified; newer Git regression matrix passes.
- Changes after plan creation (config, missing source created, executable or symlink path, relevant environment/source) refuse before mutation. Malicious/unknown config, executable hooks/filters and budget failures remain fail closed.
- `cargo fmt --all -- --check`, locked tool tests, workspace preflight, both governance validators, Windows native CI, exact-head checks and independent Git authorization/security/API review pass. No `Complete` without existing implementation SHA.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | GIT-COMPAT-001 / #919 I324 only: safe preflight discovery of config/attribute paths and controlled environment compatibility on Git 2.36 using audited gix 0.90.0 and bound host probes; preserve prepared Read/executable/facet/revalidation and system-Git execution; no snapshot digest/aux budget or TUI/other operations |
| Claimed At | 2026-11-11 |
| Source Issue | #919 |
| Governance Claim PR | #920 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested implementation of the 2026-11-10 gix design on 2026-11-11 (conversation). Single-maintainer fallback is proposed for documentation-only claim because no separate reviewing maintainer is available; exact-head CI, both governance validators and no-overlap merge-time CAS are mandatory. No source implementation or security acceptance is claimed by this PR. Subsequent implementation requires independent Git authorization/security/API review and native Windows evidence. |
| Implementation PR | Not started |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Independent security/API review and native Windows acceptance before delivery; exact-head CI and governance CAS for claim/implementation. |

## Delivery evidence

2026-11-11: Issue #919 records the maintainer's request to implement the existing proposal. This #920 governance baseline is documentation only. No production code, platform test pass, activated claim or completed compatibility behavior is asserted.
