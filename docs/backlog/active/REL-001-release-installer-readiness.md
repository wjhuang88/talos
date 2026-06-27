# REL-001: Release And Installer Readiness

| Field | Value |
|-------|-------|
| Story ID | REL-001 |
| Priority | P1 |
| Status | Planned |
| Depends On | I046 release follow-up record |
| Estimate | M |
| Origin | 2026-06-25 handoff audit: existing `v0.1.1` Release kept old binaries and old archive names; next stable release should be `v0.1.2` |

## Problem

Talos has a GitHub Release and installer scripts, but the release surface drifted after the Linux
target switch and archive-name simplification:

- Existing `v0.1.1` Release assets are old `gnu` Linux artifacts with old target-qualified names.
- Remote `v0.1.1` tag still exists and points at the old release build, so moving or recreating it
  would confuse release history.
- Installers must match the simplified `talos-{arch}-{os}` archive names before the next release.
- Release validation needs to prove local packaging, installer URL construction, and release notes
  agree before a tag is pushed.

## Proposed

Prepare and ship the next stable release as `v0.1.2`, leaving the existing `v0.1.1` Release and
tag untouched.

## Target Matrix

| Platform | Target | Archive |
|---|---|---|
| Linux x86_64 | `x86_64-unknown-linux-musl` | `talos-x86_64-linux.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-musl` | `talos-aarch64-linux.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `talos-x86_64-darwin.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `talos-aarch64-darwin.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `talos-x86_64-windows.zip` |
| Windows ARM64 | Deferred | Not published until the TLS/`ring` boundary is changed and validated |

## Validation Plan

1. Audit archive-name construction in `build.sh`, `.github/workflows/release.yml`,
   `install/install.sh`, and `install/install.ps1`.
2. Run a local packaging smoke or explicitly record why a target must be validated only in CI.
3. Verify `dist/` contains no stale artifacts before checksum generation.
4. Verify installer URL construction without creating or moving a tag.
5. Confirm release notes and README installation instructions match the generated artifact names.
6. Push `v0.1.2` only after local validation evidence is recorded in the iteration.
7. After CI publishes the release, run an install smoke from published assets where feasible.

## Acceptance Criteria

- [ ] `v0.1.2` release plan explicitly preserves `v0.1.1` history and does not move the old tag.
- [ ] `build.sh`, `.github/workflows/release.yml`, `install/install.sh`, and `install/install.ps1`
      agree on archive names.
- [ ] Linux release artifacts are built from the `musl` targets and published as
      `talos-x86_64-linux.tar.gz` and `talos-aarch64-linux.tar.gz`.
- [ ] Windows x86_64 installer uses `talos-x86_64-windows.zip`; Windows ARM64 either remains
      unsupported with a clear message or has a separately validated target.
- [ ] Local packaging smoke verifies the expected files and `checksum.sha256`.
- [ ] Installer dry-run or mocked-download validation proves URL construction before tagging.
- [ ] Release notes and README installation instructions match the actual asset names.
- [ ] Release workflow succeeds from a fresh `v0.1.2` tag.

## Required Reads

- `docs/iterations/I046-architecture-structure-governance-repair.md`
- `build.sh`
- `.github/workflows/release.yml`
- `install/install.sh`
- `install/install.ps1`
- `README.md`
- `README.zh-CN.md`
- `EVOLUTION.md` lesson #31

## Non-Goals

- Do not move, delete, or overwrite `v0.1.1` unless the user explicitly changes the release
  strategy.
- Do not re-enable `aarch64-pc-windows-msvc` until the `ring`/TLS dependency boundary changes.
- Do not migrate `reqwest` to `native-tls` in this story; track that as a future dependency
  strategy item.

## Failure Handling

- If local packaging fails for a target, do not tag. Record the failing target, command, and
  smallest follow-up fix.
- If CI release upload fails after a tag push, preserve the tag and repair via a new commit/tag
  only after the release state is audited.
- If an installer resolves the wrong archive name, treat it as release-blocking even if the binary
  artifacts are otherwise valid.
