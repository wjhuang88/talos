# 2026-07-03 Four-Month Product Hardening Plan

**Status**: Planned
**Owner area**: Post-`v0.2.2` product hardening, distribution, and self-bootstrap readiness.
**Created**: 2026-07-03
**Timebox**: 16 weeks / roughly 4 months
**Primary release marker**: Continue pre-1.0 releases while keeping `REL-002` honest.
**Supersedes**: The unexecuted remainder of
`docs/tasks/2026-07-02-frontline-four-month-development-plan.md` after I080/I084 reprioritization.

## Objective

Turn the current post-release state into a disciplined four-month queue: fix the model catalog and
session-history quality issues first, then improve distribution, extension discipline, ingestion,
and release evidence without claiming v1.0 readiness.

This is a planning and execution handoff. It does not authorize crate publishing, branch pushes,
remote dashboard access, browser automation, write-capable plugin tools, marketplace behavior, or
permission-default changes.

**Programmer handoff**:
[I085 Model Catalog Programmer Handoff](2026-07-03-programmer-handoff-i085-model-catalog.md)

## Current State

- I084 is complete and release-facing reliability work is closed.
- `v0.2.2` is the current patch-release target for the 2026-07-03 closeout.
- I085 is the next product candidate: `talos-models`, correct models.dev parsing, `/model`
  selection, and `/connect` provider setup with optional custom endpoint.
- The install scripts can be exposed through `talos.hwj.zone`, but that should be implemented as a
  verified distribution entrypoint rather than an untracked copy of the scripts.

## Install Script Site Discussion

Hosting installer entrypoints under `talos.hwj.zone` is feasible and desirable for product polish:
users could run a stable command such as `curl -fsSL https://talos.hwj.zone/install.sh | sh` instead
of a raw GitHub URL. The safer design is:

- keep GitHub Releases as the binary asset source of truth;
- add site-hosted script entrypoints that download release assets from GitHub and verify checksums;
- keep repository `install/install.sh` and `install/install.ps1` as canonical source files, with a
  build or validation step proving the site copies match;
- document failure behavior when GitHub Releases are unavailable;
- do not change the default README install command until the site entrypoints are deployed and
  validated.

## Four-Month Execution Matrix

| ID | Week | Iteration | Track | Deliverable | Validation | Status |
|---|---:|---|---|---|---|---|
| H100 | 1 | I085 | Model catalog | `talos-models` crate and SQLite-backed catalog cache. | crate tests; migration tests | Planned |
| H101 | 1 | I085 | Model catalog | Correct models.dev object-format import, provider/model identity, and refresh metadata. | talos-config/model tests | Planned |
| H102 | 2 | I085 | Model UX | `/model` is the only selection command; no `/models` alias is added. | TUI/command tests | Planned |
| H103 | 2 | I085 | Connect UX | `/connect` configures provider credentials and optional custom endpoint/base URL. | config and command tests | Planned |
| H104 | 3 | I086 | Experience polish | Retry attempt status events are surfaced through conversation/TUI events instead of tracing-only logs. | provider/conversation/TUI tests | Planned |
| H105 | 3 | I086 | Experience polish | Thinking preview refinements: keep transient display bounded and document replay/compaction policy. | TUI/conversation tests | Planned |
| H106 | 4 | I086 | Release quality | Month-1 closeout: catalog + experience polish evidence, docs, and residuals. | workspace tests; governance | Planned |
| H110 | 5 | I087 | Distribution | Decide site-hosted install entrypoint shape for `talos.hwj.zone/install.sh` and PowerShell equivalent. | design note; site validation | Planned |
| H111 | 5 | I087 | Distribution | Implement generated or synchronized site installer files without diverging from `install/`. | script diff check; site validator | Planned |
| H112 | 6 | I087 | Distribution | Update README/site install commands only after published entrypoint validation. | installer dry-run/mock | Planned |
| H113 | 7 | I087 | Distribution | Release docs matrix: assets, checksums, install commands, no crates.io claim. | publish guard; docs validation | Planned |
| H114 | 8 | I087 | Release quality | Month-2 closeout and patch-release posture. | workspace tests; governance | Planned |
| H120 | 9 | I088 | Extension | Local plugin diagnostics and hook listing without remote install or write tools. | plugin/command tests | Planned |
| H121 | 10 | I088 | Extension | Optional asset distribution policy: manifests, cache, checksum, offline/mirror behavior. | proposal/ADR if needed | Planned |
| H122 | 11 | I088 | Ingestion | Bounded document/HTML extraction slices that avoid PDF/Office/OCR/browser scope creep. | tools tests; permission tests | Planned |
| H123 | 12 | I088 | Release quality | Month-3 closeout for extension and ingestion risk. | workspace tests; governance | Planned |
| H130 | 13 | I089 | Ecosystem | Opt-in shared Skills policy for `~/.agents/skills`; Talos-owned config precedence. | ADR/policy tests | Planned |
| H131 | 14 | I089 | Self-bootstrap | REL-002 rehearsal packet with exact primary-executor boundary. | evidence doc | Planned |
| H132 | 15 | I089 | Docs | Command/help/docs sweep for `/model`, `/connect`, `/agile`, `/plugins`, `/hooks`, install. | README/site validators | Planned |
| H133 | 16 | I089 | Closeout | Final four-month matrix, residual owners, release posture, and next handoff. | workspace tests; clippy; governance | Planned |

## Milestones

| Milestone | Target Week | Exit Criteria |
|---|---:|---|
| M1 Model and experience correctness | 4 | Users can choose/connect models from a real catalog, and retry/thinking behavior is visible without history pollution. |
| M2 Distribution entrypoints disciplined | 8 | `talos.hwj.zone` installer entrypoints are either validated and documented or explicitly deferred. |
| M3 Extension and ingestion risk bounded | 12 | Plugin/hook/assets/document slices improve utility without expanding permission or browser boundaries. |
| M4 Next release posture known | 16 | REL-002 status, release docs, residuals, and handoff are current and test-backed. |

## Non-Authorizations

- No crate publish or `publish = false` removal.
- No branch push unless separately requested; release tasks may push a tag only when explicitly
  authorized.
- No remote dashboard, browser automation, web write route, remote plugin install, marketplace, or
  permission-default change.
- No v1.0 readiness claim until `REL-002` evidence is complete.

## Recovery Instructions

1. Run `git status --short`.
2. Read this file, `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and the current iteration
   shell.
3. Continue from the lowest-numbered planned H-task unless the maintainer explicitly changes
   priority.
4. Update owner docs before derived board/backlog views.
5. Run `scripts/validate_project_governance.sh .` after governance changes.

## Execution Log

### I085 Delegation Prep (2026-07-03)

Created `docs/tasks/2026-07-03-programmer-handoff-i085-model-catalog.md` for Stage 1 I085
assignment. The handoff limits immediate delegation to H100-H101 / MC100-MC103: shared catalog
types, `talos-models`, SQLite store/migrations, models.dev import parsing, gated built-in refresh,
and catalog-aware resolver precedence. Stage 2 `/model` and `/connect` work remains gated until the
resolver precedence tests pass.
