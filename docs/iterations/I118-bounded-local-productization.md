# Iteration I118: Bounded Local Productization

> Document status: Planned
> Published plan date: 2026-07-12
> Planned objective: Finish useful local/read-only extension, ingestion, dashboard, and installer
> surfaces without expanding remote, browser, marketplace, or write-capable boundaries.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: local plugin/hook diagnostics, bounded document extraction, read-only dashboard,
> and validated installer entrypoints work together in a release-candidate smoke.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| N120 | PLUGIN-001/CMD-002/HOOK-001 | Mixed/In Progress | ADR-027..032 | Local explicit read-only diagnostics closure |
| N121 | WEBFETCH-001 | Partial | Existing document tools/TOOL-013 | Bounded local extraction and handoff proof |
| N122 | I087/WEB-002/REL-001 | Planned | Existing canonical installers/releases | Validated site installer entrypoints or explicit deferral |
| N123 | WEB-001/GOV-003 | In Progress | ADR-031 | Read-only dashboard diagnostics closure |
| N124 | Month-3 closeout | Planned | N120-N123 | Pre-1.0 release candidate posture |

### Scope

- Local explicit plugin/hook provenance and failure diagnostics.
- Text/HTML/JSON/CSV/Markdown-like extraction with bounded size/type behavior.
- Site installer synchronization/checksum validation.
- Loopback-only read-only dashboard diagnostics and redaction.

### Non-Goals

- Remote plugin install, marketplace, write-capable plugin tools, executable hooks, PDF/Office/OCR,
  remote dashboard, web writes, approvals, browser automation, or WebSocket control.

### Acceptance

- Given malformed/untrusted local extension or document input, when Talos inspects it, then failure
  is bounded, provenance is visible, and no write/network authority is gained.
- Given site installers, when validation compares them with canonical sources/assets, then content,
  checksum, platform naming, and failure behavior agree before docs change.
- Dashboard route tests prove loopback/auth/redaction and absence of write routes.

### Planned Validation

- `./scripts/release_preflight.sh`
- plugin/hook/document/dashboard focused tests
- `scripts/validate_public_site.sh`
- installer dry-run fixtures and publish guard

### Documentation To Update

- README/README.zh-CN install and extension docs only after runtime/site evidence
- WEBFETCH-001, WEB-001, PLUGIN-001, CMD-002, HOOK-001, Board/backlog

### Risks And Rollback

- Risk: a local feature expands into remote or native-parser scope.
- Rollback: retain diagnostics/current text formats and defer the expanded carrier behind ADR.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-12 | Planning | Published as Month 3 shell; activation waits for I117 Complete. |
