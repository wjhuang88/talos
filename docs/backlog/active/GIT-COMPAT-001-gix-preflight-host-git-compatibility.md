# GIT-COMPAT-001: gix-Native Prepared Preflight (Git 2.36 and Newer)

**Status:** In Progress / Claimed proposed by #920, ineffective until its merge.

**Type:** Git Tools / Security-Preserving Compatibility Repair

**Source Issue:** [#919](https://github.com/wjhuang88/talos/issues/919)

**Source design:** [gix-native system-Git preflight proposal](../../proposals/gix-preflight-system-git-compatibility.md)

**Related authority:** ADR-089, I300, completed I309 (gix 0.90.0), GIT-SNAPSHOT-001 (separate Review owner), Windows native report of 2026-11-10.

## Outcome

Replace Git CLI preflight discovery/interpretation with auditable gix-native analysis, enabling structured Git Checkout/Commit preparation on Windows with system Git 2.36.1 and newer versions, without reducing the exact prepared permission frontier, immutable source evidence, execution-program binding or post-approval revalidation. Retain actual system-Git execution; do not silently bypass unsupported configuration or attribute discovery.

## Bounded scope

1. Audit the *specific* selected gix 0.90.0 path/config/attribute APIs for hidden PATH discovery, subprocesses, ambient environment, process-global caches and panic/IO. **gix/Talos-native computation is the preflight implementation**, not a fallback applied only to older Git. Use pure parsing/planning over the admitted file snapshots, not APIs that load unauthorized files directly.
2. Replace preflight `git var` and `git config` subprocesses for config/include/includeIf/attribute/path/effect discovery. Preserve the staged exact-Read frontier, including absent/empty sources. **No Git CLI preflight probes**, no hidden gix-launched Git process, no guessed installation roots and no unauthorized file-body reads. If exact Git-installation defaults are unknowable from approved sources, fail closed and record an explicit unsupported case.
3. Verify the exact effect of Windows/Scoop `GIT_INSTALL_ROOT` without adding OS/version-specific path or command fallbacks. Narrowly admit it only if proven semantically inert for the bound execution. Keep dangerous `GIT_*` redirection/config injection/trace output rejected; `GIT_PAGER` is already allowed.
4. Model hooks/filters, URL rewrite and tracking semantics with gix/Talos-native analysis of admitted configuration, or explicitly refuse when unrepresentable; do not claim a fix for all Windows Git failures from one preflight patch.
5. Prove source path/executable/config/include changes invalidate prior admission; test mixed Git versions, HOME/XDG, includes, attribute paths, Unicode/spaces, multiple installations and >16MiB source checkout/commit.
6. Update user Git support documentation, exact-head test record and independent review evidence.

## Exclusions and dependency gates

- Preserve I300's completed acceptance and ADR-089 security invariants. **ADR-089 currently names fixed Git CLI discovery commands**, so replacing those commands needs an explicit reviewed amendment/maintainer architecture decision *before* production implementation; no unreviewed permission change is authorized.
- Does not own GIT-SNAPSHOT-001's 16MiB/digest/auxiliary budget fixes, TUI-APPROVAL-001, PERM-007, process-hardening, sandbox, search, provider, Server or other Git operation expansion.
- No replacing system Git with gix; no new dependency/version bump/unsafe; no automatic fallback from missing admission.
- Implementation may start only after effective governance claim and activation on main, recorded no-overlap CAS and any necessary ADR decision.

## Acceptance and verification

- Instrumented Git executable/test sentinel proves **zero Git CLI command invocations during gix-native preflight**, even on versions without `GIT_ATTR_*`; ordinary final write still uses the selected bound system Git. Authorized sources only; missing/unverifiable installation roots fail closed.
- Real Git 2.36.1.windows.1: native Git tools tests and large checkout/commit pass in original Scoop environment, with any remaining failures independently classified; newer Git regression matrix passes.
- Changes after plan creation (config, missing source created, executable or symlink path, relevant environment/source) refuse before mutation. Malicious/unknown config, executable hooks/filters and budget failures remain fail closed.
- `cargo fmt --all -- --check`, locked tool tests, workspace preflight, both governance validators, Windows native CI, exact-head checks and independent Git authorization/security/API review pass. No `Complete` without existing implementation SHA.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | GIT-COMPAT-001 / #919 I324 only: safe preflight discovery of config/attribute paths and controlled environment compatibility on Git 2.36 using audited gix 0.90.0 without Git CLI preflight subprocesses; preserve prepared Read/executable/facet/revalidation and system-Git execution; no snapshot digest/aux budget or TUI/other operations |
| Claimed At | 2026-11-11 |
| Source Issue | #919 |
| Governance Claim PR | #920 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested implementation of the 2026-11-10 gix design on 2026-11-11 (conversation). Single-maintainer fallback is proposed for documentation-only claim because no separate reviewing maintainer is available; exact-head CI, both governance validators and no-overlap merge-time CAS are mandatory. No source implementation or security acceptance is claimed by this PR. Subsequent implementation requires independent Git authorization/security/API review and native Windows evidence. |
| Implementation PR | Not started |
| Last Updated | 2026-11-12 |
| Handoff / Release Condition | Independent security/API review and native Windows acceptance before delivery; exact-head CI and governance CAS for claim/implementation. |

## Delivery evidence

2026-11-11: Issue #919 records the maintainer's request to implement the existing proposal. This #920 governance baseline is documentation only. No production code, platform test pass, activated claim or completed compatibility behavior is asserted.

## Architecture clarification — 2026-11-12

Maintainer explicitly clarified that **gix must replace Git CLI for preflight**, not that Talos should add hardcoded version/installation compatibility branches. This supersedes the original proposal's suggestion to use host probes. The final system Git writer and Talos admission/stale-evidence checks remain unchanged. No CLI preflight fallback is approved. Any `gix::config_path` or related convenience API that internally invokes Git or captures global ambient state is disallowed in this plan unless reworked into a pure, per-invocation, externally admitted interface. Open decision: how to prove compiled installation default attribute paths match the selected system Git without a host subprocess; if unprovable, stop safely instead of guessing. See Issue #919.
