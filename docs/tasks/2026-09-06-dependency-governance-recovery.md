# #474 Dependency Governance Recovery And Delivery

Status: Refinement / Unclaimed. This is the recovery ledger, not an effective implementation claim.
Requirement owner: [DEPENDENCY-001](../backlog/active/DEPENDENCY-001-dependency-upgrade-governance.md).

## Verified Checkpoint — 2026-09-06

Remote `main` was fetched and resolved to `e336e438208eebca95576db9fbf245783651d0c1`.
Its DEPENDENCY-001 owner is Intake / Unclaimed, Selected Iteration None.
The open PR inventory returned no PRs. There is no effective #474 claim.

The local `impl/i247-dependency-governance` branch ends at `fa38fff3` and contains
unpublished governance and implementation commits. Preserve it for diagnosis, but do not merge it,
cherry-pick its implementation, or treat its local main ancestry as authorization. The local claim
records at `099e6519` through `37943df3` do not prove target-branch activation. The general request
to finish #474 did not establish the SOP's special Direct commit authorization.

I247's committed Published Baseline excludes scripts and live lookup. Its later local claim widened
that scope without a new iteration. Preserve that baseline on the historical branch; use a new ID
for a materially broader deliverable. I248 is the next proposed ID, subject to fresh collision check.

## Rejected Evidence

- `6830c1be`: Bash emits an empty dependencies array and metadata byte count; this is not an audit.
- `09e508f6`: grep assertions over a static example do not test classification, parsing, rendering,
  multi-resolution preservation, or Bash/PowerShell parity.
- `fa38fff3`: live HTTP responses are discarded. Success proves neither a latest-stable comparison
  nor a fresh dependency report. The tree command also includes workspace roots and is not a
  verified direct-external-dependency collector.
- The baseline contains no actual accepted dependency entries or accepted validation evidence.
- Prior unbound/empty validator outputs are not exact-base validation evidence for a candidate.

No existing dependency manifest or lockfile upgrade was made by these commits. #474 remains open;
none of the above is completion evidence.

## Required Delivery — No Scope Dropped

The outcome is one usable maintainer workflow: accepted baseline -> audit -> candidate proposal ->
existing upgrade governance -> accepted baseline advancement. The Issue's A/B/C/D decomposition is
a suggested staging order, not permission to close the Issue after a schema placeholder.

| Stage | Required result | Evidence before accepting it |
|---|---|---|
| A: deterministic contract | Versioned JSON schema; direct external identity with registry source, aliases, requirements, users and every resolution; independent baseline/upstream drift; policy signals | Executable parser/comparator/renderer tests using manifest, lock/metadata and registry inputs; expected outputs computed independently of production code |
| B: runnable frontends | Bash and PowerShell human/JSON/Markdown reports with equivalent semantics and exit codes | Run both frontends on the same fixtures; compare semantic JSON and ordered human/Markdown goldens; mutation of input must alter results |
| C: permanent SOP | Latest stable including major; isolation/grouping; domain validation; exceptions/blockers/rollback; owner routing and post-merge baseline advancement | Requirement-by-requirement review; routes in existing docs; baseline mutation rejected without accepted merged evidence |
| D: live handoff | Actual direct workspace audit at implementation-time main; generated accepted snapshot with provenance; fresh latest comparison; at least one bounded candidate proposal | Record command, source SHA, timestamp, lookup sources/failures, output and classification; never copy the Issue's historical version table |

The implementation claim must explicitly cover the complete chosen runnable stage, its tests and
user documentation. Prefer one locally converged implementation candidate for tightly coupled
contract/frontends/SOP work; do not submit placeholders as separate remote stages.

## Constraints And Open Technical Readiness

Hard: no dependency upgrades, Cargo manifest/lock edits, workspace dependency centralization,
public API changes, new helper crate/binary, or Python/Node/jq/cargo-outdated requirement. Both
frontends must work with the Issue's permitted host facilities. Use pinned Cargo with `--locked`.
Do not infer direct resolutions from all same-named lockfile packages. Include optional, target,
build and dev declarations; document source identity and unsupported-source handling.

Soft implementation choice: use Cargo metadata as build truth, a bounded POSIX awk JSON parser
on Unix, and ConvertFrom-Json on PowerShell. Before Ready, verify availability and safe handling of
escapes, nested arrays/objects, null, malformed input, aliases and multiple resolutions using
disposable uncommitted experiments. If this cannot be made reliable, record the failure and obtain
an explicit requirement decision; do not silently add a runtime/helper or replace real data with
an empty result.

The test matrix must include current, patch, minor, major, 0.x breaking minor, prerelease and build
metadata, multiple resolutions, baseline requirement/user/version drift, yanked, deprecated,
security, malformed/unknown input and registry-unavailable. Unknown security/deprecation data is
not evidence of absence. An unavailable registry may yield a partial report, never a current result.
No live network check becomes an ordinary compile/test dependency.

## Recovery Order And Resume

1. Preserve the old branch unchanged. Work on `governance/dependency-474-recovery`, based on the
   verified remote main, with no implementation files or invalid Direct commit records.
2. Complete technical readiness and exhaustive current iteration inventory (including legacy
   status formats); read Issue #474 Required Reads. Current header scan identifies I164 as Paused
   and superseded; retain that disposition. Do not mistake Complete headers mentioning historical
   reviews, or stale Claimed metadata in Complete files, for new active implementation authority.
3. Create the new runnable iteration and owner scope; synchronize relevant derived views only
   after the owner. Preserve I247's old baseline and record supersession without transferring claim.
4. Use the SOP's single-maintainer path where allowed: open a Draft governance PR, backfill the
   real PR number, finalize atomic claim/activation, run exact-base validators and exact-head CI,
   then merge-time CAS. Shared-account role limits must be disclosed. Do not invent approval.
5. Only after the claim merges, create a fresh implementation branch from that merge or later main.
   Implement and test locally; no replay of the pre-claim implementation as accepted evidence.
6. Submit one stable candidate, obtain required exact-head evidence/review, and perform merge CAS.
7. Close owner first using an already-merged implementation SHA, synchronize derived views and
   Issue #474, and close the Issue only after all acceptance rows actually pass.

## Validation And Residuals

This recovery document authorizes no implementation. Governance validator results are recorded
after execution, not inferred from the presence of a script or a tool output without completion.
The complete remaining #474 requirement stays owned by DEPENDENCY-001; this ledger is not a
parallel dependency task database. Unrelated branch cleanup and historical owner drift are outside
this correction and must not be used to block a valid future non-overlapping candidate.
