# I098-I101 Autonomy, Permission, Runtime Closeout — 2026-07-06

## Verdict

The four-phase autonomy, permission, and runtime hardening task is complete as a product-hardening
track. It is not qualifying REL-002 self-bootstrap evidence.

Talos gained useful internal capabilities for long-running work:

- permission preflight for scoped approval planning;
- structured direct-argv `exec` parallel and pipe workflows;
- project-type detector metadata and internal governance validation routing;
- corrected model setup behavior for standard versus custom providers;
- viewport-windowed model browser rendering for large packaged catalogs;
- continued `gix 0.85.0` fallback tracking.

Codex remained the primary executor for planning, implementation, validation orchestration,
documentation, commit, and push. Talos was the target system and validation subject, not the
primary autonomous development runtime.

## Phase Summary

| Iteration | Result | Notes |
|---|---|---|
| I098 | Complete | Added `talos permissions preflight` as a read-only packet for expected permission decisions and reusable scopes. No permission default was relaxed. |
| I099 | Complete | Added bounded direct-argv `exec` steps, parallel steps, pipe chains, and parallel pipe chains. No shell parser, glob, redirection, or background-job behavior was added. |
| I100 | Complete | Exposed detector descriptor metadata and routed governance mutation validation through the internal validation service instead of shelling out to the compatibility script. |
| I101 | Complete | Closed MODEL-006 residuals, recorded `gix 0.85.0` tracking, and preserved REL-002 No-go posture. |

## I101 Model Evidence

- `target/debug/talos --available-models-browser` was run in a real PTY.
- The browser opened on the packaged catalog at `1/4190` rows and showed provider/group data
  without dumping the whole model list.
- The visible screen rendered only the current terminal viewport. The deterministic large-catalog
  test separately proves an 8-line view over 500 rows excludes non-visible tail rows.
- Search mode was exercised with `/`; a mistyped query reduced the result set to `0/0` without a
  crash, then `Enter` and `q` exited cleanly and restored the terminal.
- No credential entry, provider network request, or config write was performed.
- Standard provider setup no longer asks for a URL when catalog endpoint metadata exists. Custom
  provider setup still requires a URL.

## GIT-001 Tracking

I101 did not update `gix` because the current lockfile already resolves `gix 0.85.0` and no newer
replacement evidence was required for this phase.

Recorded checks:

```sh
cargo tree --invert gix@0.85.0 -e features
rg -n "Command::new\\(\"git\"\\)|git status --porcelain|bash git|git_status|gix =" crates docs/backlog/active/GIT-001-embedded-git-tools.md Cargo.toml Cargo.lock
```

Observed posture:

- Talos still uses the accepted local feature set: `basic`, `status`, `revision`, `blob-diff`,
  `index`, and `sha1`.
- No `gix` network or worktree-mutation feature was enabled.
- Governance dirty-tree status remains internal/gix-backed.
- Write/publication workflows retain structured host-`git` fallbacks until a scoped replacement
  proves behavior equivalence and permission safety.

## REL-002 Classification

This track does not satisfy REL-002 because:

- Codex selected and sequenced the work.
- Codex edited the repository.
- Codex interpreted validation failures and evidence.
- Codex synchronized owner docs.
- Codex performed commits and pushes.

The track does reduce future self-bootstrap blockers by improving validation, execution, model UX,
and permission planning surfaces.

## Release Boundary

No `v1.0.0` claim, release tag, crate publish, GitHub Release, permission-default relaxation,
runtime `catalog.db` resurrection, provider network request, broad bash allow, force push, or
destructive Git cleanup occurred.

## Final Validation

All final closeout gates passed on 2026-07-06:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

Governance validation reported `0 warning(s)`.
