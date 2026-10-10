# Iteration I294: Native Provider Admission And Regional Evidence

> Document status: Active
> Parent: SEARCH-001-D / #644 / SEARCH-001 / #624
> Objective: qualify native zero-key routes with provenance-backed evidence before E/F authorization.

| Field | Value |
|---|---|
| Story | SEARCH-001-D |
| Source Issue | #644 |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 complete; B adapter boundary available |
| Claim State | Claimed |
| Research PR | #668, #673, #676, #686 and #687 (merged); #693 (Review; partial offline evidence) |
| Completion | Pending |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-D native provider admission and regional evidence only |
| Claimed At | 2026-10-06 |
| Source Issue | #644 |
| Governance Claim PR | #665 |
| Implementation PR | #668, #673, #676, #686 and #687 (merged); #693 (Review; offline evidence only) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR; exact-head CI, governance validators, remote Issue reconciliation and merge-time CAS required. |
| Last Updated | 2026-10-10 |
| Handoff / Release Condition | Claim effective after #665 merge `7019f63d3df3d6eb58473a53a499b9b7e8b98925`; evidence research only, with E/F provider implementation gated by the resulting matrix. |

## Scope

Record accepted, rejected and unresolved candidates with terms/automation review, parser maintainability, endpoint and failure-domain independence, privacy/query exposure, deterministic fixtures, protocol observations and reproducible regional evidence including genuine mainland-China evidence.

## Non-goals

No production behavior change, provider addition, GeoIP routing, CAPTCHA/access-control bypass, default relay, or paid-provider auto-selection.

## Acceptance

The candidate matrix has dated evidence links and honest limitations; two independent native zero-key routes are explicitly admitted or remain blocked; E/F authorization is gated by the result.

## Evidence Register

The working candidate matrix, regional observation schema and admission gate are maintained in
[SEARCH-001-D provider evidence register](../reference/SEARCH-001-D-PROVIDER-EVIDENCE-2026-10-06.md).
It is a research artifact only; no candidate is admitted by the register itself.

## Partial Research Evidence

- #668: evidence register, merged at `08b5bd2188892e3d721d7b5e845fdb35b522af19`.
- #673: dated source review, merged at `2708db4f56362d626d10b4e5c53fae880e7d3b08`.
- DDG/Bing terms follow-up: see the register's 2026-10-08 review. No route admitted.
- #676: terms follow-up, merged at `916bc70bc43dcbae1679ea253eb4f2961c459d39`.

## Independent API Research Stage — 2026-10-09

Base: `c6c545b788c497528133cbab453c0eee1494922e`. The evidence register adds Mwmbl standard
anonymous JSON search, Purili core API and the parked Stract hosted route. D-005 through D-009
link current primary evidence and pin Mwmbl source at
`ff482e225d99c694abbd3337a1b1be93b8c95dd8`. Mwmbl source accepts anonymous standard search;
keyed/combined routes are separate, and external provenance/privacy still need qualification.
Purili developer integration claims must be reconciled with general use terms; Stract's archived
repository/current site do not establish a maintained hosted search route.

The maintainer's technical-first direction (#644 comment `6055213313`) supersedes a blanket
business-correspondence prerequisite. Published interface conditions are reviewed per route;
explicit restrictions remain effective. No outreach, account, key, paid use, live query/response
fixture, production adapter or regional measurement was performed. Inspected upstream tests
were not executed. Source evidence alone admits no route.

Local documentation convergence passed both governance validators (locked Cargo metadata/SQLite
consumer boundary included), public-site and installer checks, the 14-case CI classifier suite,
changed-document relative links and whitespace checks. No new workspace or upstream tests are
claimed for this source-only stage. Exact-head documentation CI and technical review remain
remote gates.

Next: verify Mwmbl's deployed anonymous standard route, query fan-out/provenance and applicable
result-use conditions; then acquire permitted deterministic and controlled regional evidence.
This is partial D research, not C implementation or E/F authorization. D/I294 remains Active /
Claimed, completion pending, and #624 stays open.

Terms applicability, privacy, authorized fixtures, independent failure domains and genuine regional
observations remain incomplete. E/F remain gated; this is partial evidence, not completion.

## Standard Route Trace And Query-Free Diagnostics — 2026-10-09

Previous source stage #686 merged at `fecc9583992ccf2c0678b58e902f4a3bdcf19d2b`, after
exact-head CI3067 / run `37893403091` and independent Agent technical APPROVE. The unchanged
trusted documentation route passed; no new full-workspace test execution was claimed.

[Mwmbl route qualification packet](../reference/SEARCH-001-D-MWMBL-ROUTE-QUALIFICATION.md)
traces the pinned standard endpoint through MMR/LTR, local retrieval, Wikipedia helper,
upstream cache and wire provenance. It records one public-schema GET (HTTP 200) and one
missing-query GET (HTTP 422), with UTC times/body hashes and no credentials, retries,
redirect following, search query or geographic inference. Public schema matches the inspected
field shapes; input validation is not anonymous valid-query/search-success evidence.

Upstream Wikipedia retries/circuit/cache are explicitly distinguished from Talos budgets and
cleanup. Per-hit origin is distinct from backend identity, and source/policy/deployment differences
remain qualification work. M-Q01 through M-Q08 are pending fixture/acceptance rows, not delivered
tests. No result-response fixture, provider admission or production code change.

Local convergence passed both governance validators (offline locked Cargo metadata/SQLite
boundary included), public-site/installers, 14 classifier cases, changed-document links, observed
schema structural checks and whitespace validation. No workspace/upstream tests were rerun.
Exact-head documentation CI and technical review remain remote gates.

Next: independently authored offline parser evidence toward route qualification, plus permitted
anonymous valid-query evidence; hosted privacy/result-use and genuine regional observations
remain pending.
D/I294 remains Active / Claimed and incomplete; E/F gated, #624 open.

## Offline Wire Experiment — 2026-10-10

Base: `6b4bfb8d3f20faa6e711b218d1631603da53b572`. #687 qualification packet merged at
`9e376155ddc7b6c3644bf65f69b4c388956c1b5c` after exact-head CI3072/run37897478071 and Agent
technical APPROVE. New main context-budget work does not change D's source, claim or tool code.
D remains Active/Claimed under effective #665 and its original acceptance.

The disposable implementation is `crates/talos-tools/tests/i294_mwmbl_wire_characterization.rs`,
with independently authored `tests/fixtures/i294_mwmbl_synthetic.json`. It compiles only into a
network-feature integration-test binary; reqwest is used only for URL parsing. No SearchBackend,
public API, provider registration, request, credentials, config or production helper is introduced.
The fixture uses invented text and example.invalid URLs, not captured responses or upstream code.

Nine experiments cover required fields/types and optional nullable usage, mixed/unknown/reference/
curated labels, plain Unicode and inert highlights, wire order independent of score, malformed/
empty/invalid-only outcomes, invalid URLs filtered before truncation, advertised-count mismatch
and explicit body/output bounds. Integer widths, whole-response schema rejection, URL policy and
limits are experimental choices, not accepted production defaults. A byte check on an already
received body does not prove transport buffering, deadlines, cancellation or endpoint/SSRF policy.

No wire label proves hosting independence, a live upstream call or native-route credit. These
experiments supply partial offline M-Q02..M-Q05 evidence only; no row is fully accepted. Hosted
anonymous execution/provenance/use conditions, transport parts of M-Q06 and genuine regional
M-Q08 remain pending. D/I294 remains incomplete; E/F gated, #644/#624 open.

The previous temporary checkout/log was removed before its final preflight result could be read.
The restored candidate is validated afresh; no inferred result is carried forward. Local builds use
pinned Rust 1.97.0, locked dependencies, debug=0 for dev/test, no incremental state and four jobs.
The Oct9 default-debug attempt exhausted disk and was cleaned; these local environment overrides
do not change source profiles, tests, dependencies, release configuration or CI.

### Initial Local Stage Validation

- Focused locked network-feature integration target: nine passed, no skips; targeted Clippy with
  `-D warnings` passed. The locked workspace test run also discovered and passed all nine.
- Standard `./scripts/release_preflight.sh`: public-site/installers, both governance validators
  (zero warnings), text boundary, 14 classifier cases, formatting, locked workspace check and
  Clippy passed. It stopped at the unchanged runtime Unix-socket fixture: 64 passed, one local
  `PermissionDenied` / OS error 1. Full preflight is therefore not claimed passed.
- Under the maintainer's existing local-only deferral, `cargo test --locked --workspace -- --skip
  tests::runtime_evidence_rejects_directory_symlinks_and_socket_entries` passed: successful
  summaries total 3,575 passed, zero failed/ignored and one filtered. No test or CI source changed;
  full unskipped exact-head CI is mandatory before merge.
- Independent SDK fixture validator passed both default and coding runs. Initial offline attempt
  lacked its separately locked `core_detect` index entry; fetching unchanged dependencies resolved
  the cache gap. Neither lockfile changed.
- Latest target check: `71fc8bce8e80c32a7fc9b9b641d482c57dc70544` adds unrelated I297 governance
  only; D claim, acceptance and search code are unchanged.
- Qualification packet's local links and `git diff --check` passed. At this local checkpoint, stage submission and
  exact-head CI/review remain pending; whole D acceptance remains pending.

## #693 Baseline Refresh — 2026-10-10

Initial head `75870a360432405c163fafdd580d16d1f4a04dce` received Agent technical APPROVE
(comment6091964103). CI3088/run38011578886 attempt two passed full unskipped macOS preflight,
optional language Plugin acceptance, offline dependency audit and Desktop validation. The nine
new tests and original socket fixture passed. The first-attempt TUI highlighting failure and
passing focused local retest remain recorded; the precise failure branch is unknown.

Windows attempts one and two were cancelled at the existing 30-minute job cap. Attempt two
reached workspace test execution but did not finish. Format/check/Clippy/Desktop, focused
process/permission/timeout tests and I170 walkthrough passed; subsequent test/governance/audit/
smoke completion is not claimed. These attempts are partial evidence, not a passing CI gate.

The candidate now integrates target `7097f1b503063539a2205e17ae29aed09c7ae32e`, including
#694's existing 45-minute Windows job budget and independently merged provider-default work,
and #688's governance-only I296 activation. Rust remains pinned at 1.97.0; the toolchain
implementation is not part of this slice. No Search claim, acceptance, dependency/lockfile,
production search or test semantics change. Main's production changes are upstream context,
not implementation authored by #693. Local checkout/cache was again absent on continuation;
restored source is verified against the published candidate, and fresh validation is required.

Stage #693 remains in Review; whole D/I294 stays Active/Claimed and incomplete. Fresh exact-head
CI/Agent review and final CAS supersede old-head approval for merge. E/F gated; #644/#624 open.

### Refreshed Local Checkpoint

- Pinned Rust 1.97.0, locked dependencies, local dev/test debug=0, incremental disabled,
  four jobs. Standard preflight passed site/installers, both governance validators, text,
  classifier, formatting, workspace check and Clippy; stopped at the unchanged socket fixture
  (64 runtime tests passed, one PermissionDenied/OS error 1). No full local preflight pass claimed.
- With only the previously authorized local socket deferral, locked workspace tests passed:
  3,576 passed, zero failed/ignored and one filtered; all nine I294 tests included and passed.
- Independent SDK default and coding validator runs passed. No dependency or lockfile changed.
- Old candidate parser/fixture are byte-identical. Fresh target Clippy with `-D warnings` and
  final governance/claim validators passed (zero warnings); changed-file links and diff checks
  passed. Remote exact-head CI/review and final CAS remain mandatory before merge.
