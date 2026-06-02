# Talos Documentation

## Structure

| Directory | Purpose | Key Files |
|-----------|---------|-----------|
| `backlog/` | Planned work and story tracking | `PRODUCT-BACKLOG.md` |
| `iterations/` | Iteration plans, progress, and retrospectives | `README.md` |
| `decisions/` | Architecture Decision Records (ADRs) | `README.md` |
| `roadmap/` | Phased implementation plan and requirement traceability | `IMPLEMENTATION-ROADMAP.md`, `REQUIREMENT-CONVERGENCE.md` |
| `proposals/` | Uncommitted ideas for future consideration | `README.md` |
| `reference/` | Stable facts: architecture, contracts, config | `ARCHITECTURE.md`, `REFERENCE-PROJECTS.md` |
| `sop/` | Standard Operating Procedures | See below |

## SOP Index

| SOP | When to Use |
|-----|-------------|
| `REQUIREMENT-INTAKE.md` | When a new feature or change is requested |
| `START-ITERATION.md` | When beginning a new iteration from the backlog |
| `ITERATION-WORKFLOW.md` | During active iteration work |
| `CHANGE-CONTROL.md` | When requirements change mid-iteration |
| `LOCAL-DEV.md` | Setting up local development environment |
| `NEW-FEATURE.md` | Implementing a new feature during an iteration |
| `TESTING.md` | Writing and running tests |
| `GIT-WORKFLOW.md` | Committing, branching, and PR workflow |
| `DOC-CHECK.md` | Keeping documentation synchronized with code reality |

## Quick Reference

- **Agent rules**: See `AGENTS.md` at project root
- **Requirement closure**: See `docs/roadmap/REQUIREMENT-CONVERGENCE.md`
- **Lessons learned**: See `EVOLUTION.md` at project root
- **Governance state**: See `.agent-governance/manifest.yaml`
