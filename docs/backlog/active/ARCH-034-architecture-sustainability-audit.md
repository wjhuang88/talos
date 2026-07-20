# ARCH-034: Workspace Architecture Sustainability Audit And Remediation Program

| Field | Value |
|---|---|
| Type | Architecture Epic |
| Status | Audit complete — remediation gated |
| Priority | P1 |
| Selected Iteration | I144 audit only (Complete 2026-07-20) |
| Maintainer Value | Keep future delivery predictable as the workspace grows |

## Goal

Audit Talos as a whole against high-cohesion/low-coupling boundaries, focused
module responsibilities, consistent Rust style, semantic duplication, and real
extension scenarios; then convert findings into bounded remediation stories and
fitness checks that prevent uncontrolled code accumulation.

## Current Evidence Snapshot

- 21 workspace crates and roughly 111k raw Rust lines including tests.
- Largest raw roots include `scheduler.rs` (3230), `todo.rs` (2353),
  `openai_sse.rs` (2197), `talos-runtime/lib.rs` (1624), CLI registry (1350),
  TUI app (1334), conversation engine (1267), and core tool types (1264).
- Raw LOC is a locator, not a verdict: several roots contain large inline tests or
  intentionally cohesive state machines.
- `talos-cli` has the highest internal dependency fan-out (17). Existing ARCH-022,
  ARCH-023 and ARCH-030 measurements/assumptions need refresh.

## Child Stories And Gates

1. [ARCH-034-A](ARCH-034-A-evidence-and-boundary-audit.md) — Ready audit and
   report; no production refactor.
2. [ARCH-034-B](ARCH-034-B-finding-remediation-program.md) — Refinement,
   blocked on accepted A findings; one behavior-preserving story per root/seam.
3. [ARCH-034-C](ARCH-034-C-architecture-fitness-gates.md) — Refinement,
   introduces only evidence-backed prevention checks after A/B.

## Audit Dimensions

- Crate dependency direction, public API ownership, DTO leakage and composition roots.
- Module responsibility, change reasons, production/test separation, file and function size.
- Semantic and textual duplication, repeated error/validation/config/permission flows.
- Style consistency, error handling, visibility, naming, feature flags and lints.
- Extension scenarios: add a provider, tool, permission facet, TUI panel/command,
  session backend, plugin carrier and embedded-runtime consumer.
- State ownership, data flow, concurrency/cancellation, persistence and security seams.

## Hard Guardrails

- No line-count-only split, speculative trait, “shared utils” dumping ground, global bus,
  circular dependency, unsafe code, or behavior change hidden inside restructuring.
- Tests and generated/catalog data are classified separately from production complexity.
- DRY is applied to duplicated policy/semantics, not coincidentally similar syntax.
- Public API breaks require ADR, semver impact and migration plan.
- Permission/sandbox changes require independent security review.
- Each remediation has before/after responsibility maps, focused tests, rollback and
  full locked validation.

## Completion

Audit report and machine-readable finding register are accepted; every finding is
Closed, Deferred with trigger/owner, or represented by a bounded backlog story;
P0/P1 findings are remediated and re-audited; agreed fitness gates run in CI/local
validation; `ARCHITECTURE.md`, ARCH-030 and publication boundaries match code.

## Required Reads

- `docs/reference/ARCHITECTURE.md`
- `docs/reference/ARCHITECTURE-AUDIT-2026-06-18.md`
- `docs/backlog/active/ARCH-011-architecture-watchlist.md`
- `docs/backlog/active/ARCH-022-cli-mode-runner-residual-decomposition.md`
- `docs/backlog/active/ARCH-023-tui-app-residual-decomposition.md`
- `docs/backlog/active/ARCH-030-remaining-production-root-residual-register.md`
- ADR-006, ADR-021, ADR-024, ADR-026, ADR-027, ADR-039
- `Cargo.toml`, all `crates/*/Cargo.toml`, `rust-toolchain.toml`
