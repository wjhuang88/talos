# Reference

## Purpose

Stable technical facts about the project. Not procedures or status — those live in `sop/` and
`iterations/` respectively.

## Contents

| Document | Content |
|----------|---------|
| `ARCHITECTURE.md` | System design, crate structure, data flow, core traits |
| `REFERENCE-PROJECTS.md` | Analysis of projects that influenced Talos design |
| `CRATE-PUBLICATION-MATRIX.md` | crates.io publish readiness, dry-run evidence, install gate |
| `RUNTIME-SDK-CONTRACT.md` | talos-runtime pre-1.0 embedding support boundary and caveats |
| `DOCS-SYNC-CHECKLIST.md` | Surfaces that must stay in sync when behavior/install/tools change |
| `RELEASE-NOTES-DRAFT-2026-07-02.md` | Draft post-v0.2.0 release notes and known gaps |
| `SELF-BOOTSTRAP-EVIDENCE-TEMPLATE.md` | Template for Talos-on-Talos rehearsal evidence records |
| `config.reference.toml` | Full configuration schema with examples for all providers |

## Rules

- Reference docs describe what IS, not what SHOULD BE.
- Update when architecture changes, not when work is planned.
- Procedures belong in `docs/sop/`.
- Moving status belongs in `docs/iterations/` or `docs/backlog/`.
