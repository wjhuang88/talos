# Iteration I294: Native Provider Admission And Regional Evidence

> Document status: Complete / Closed — research gate completed with BLOCKED / ZERO-ADMISSION outcome
> Parent: SEARCH-001-D / #644 / SEARCH-001 / #624
> Objective: qualify native zero-key routes with provenance-backed evidence before E/F authorization.

Completion Commit: `0afeb5b1a0d228d414a53ca2947bfd835c3f598b`

| Field | Value |
|---|---|
| Story | SEARCH-001-D |
| Source Issue | #644 |
| Depends on | Accepted ADR-085; SEARCH-001-A / #625 complete; B adapter boundary available |
| Claim State | Closed |
| Research PR | #668, #673, #676, #686, #687, #693, #825, #826, #827, #828, #829 and #830 (merged) |
| Completion | Complete — blocked disposition; zero providers admitted |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-D native provider admission and regional evidence only |
| Claimed At | 2026-10-06 |
| Source Issue | #644 |
| Governance Claim PR | #665 |
| Implementation PR | #668, #673, #676, #686, #687, #693, #825, #826, #827, #828, #829 and #830 (merged) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR; exact-head CI, governance validators, remote Issue reconciliation and merge-time CAS required. |
| Last Updated | 2026-11-01 |
| Handoff / Release Condition | Research gate closed with zero providers admitted. Mwmbl/Purili remain outside the default distribution; E/F remain unauthorized because D admission did not occur; #624 remains open. Material new evidence requires reopening #644 or a separately governed D follow-up. |

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
The current bounded refresh candidate is
[SEARCH-001-D Provider Evidence Refresh — 2026-11-01](../reference/SEARCH-001-D-PROVIDER-REFRESH-2026-11-01.md).
The accepted closeout is
[SEARCH-001-D Final Disposition — 2026-11-01](../reference/SEARCH-001-D-FINAL-DISPOSITION-2026-11-01.md).

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

The final #693 candidate integrated target `7097f1b503063539a2205e17ae29aed09c7ae32e`, including
#694's existing 45-minute Windows job budget and independently merged provider-default work,
and #688's governance-only I296 activation. Rust remained pinned at 1.97.0; the toolchain
implementation was not part of this slice. No Search claim, acceptance, dependency/lockfile,
production search or test semantics change.

### Retained pre-merge local checkpoint

Before the final remote gate, pinned Rust 1.97.0 and locked dependencies were used with local
`dev/test debug=0`, incremental disabled and four jobs. Standard preflight passed site/installers,
both governance validators, text boundary, classifier, formatting, workspace check and Clippy,
then stopped at the unchanged runtime Unix-socket fixture after 64 runtime tests passed and one
local `PermissionDenied` / OS error 1; no full local preflight pass was claimed. With only the
previously authorized local socket deferral, locked workspace tests passed 3,576 tests with zero
failures/ignored and one filtered, including all nine I294 tests. Independent SDK default/coding
validator runs, target Clippy `-D warnings`, final governance/claim validators, changed-file links
and diff checks passed. The earlier Windows cap history above remains part of the evidence record.

#693 then merged as `e63e0ee3793e2e8fd914f676224f982c6cae5593`. Exact head
`e8683d99bb04e4a823e547358e9ba0dbb68625ff` passed full unskipped CI3097 / run
`38024072972`; independent Agent review `6093757461` approved the candidate and final CAS
`6094343503` was recorded. The nine tests remain partial offline M-Q02..M-Q05 evidence only;
no hosted search success, native admission, independence or regional qualification was proved.
D/I294 therefore remains Active / Claimed and incomplete; E/F remain gated, #644/#624 open.

## Provider Evidence Refresh Candidate — 2026-11-01

Draft #825 records a fresh source/policy checkpoint against current Mwmbl and Purili material.
This is still evidence-only and remains ineffective as target-branch evidence until its exact head
passes the applicable documentation/governance CI, independent review and merge-time CAS.

### Mwmbl movement

Official Mwmbl API pages now explicitly describe an anonymous no-key tier. API privacy says search
queries are not logged/retained and request IPs are transient rather than persisted. API terms
permit lawful/commercial API use but separately place returned search results under CC BY-NC-SA
4.0 plus original-content rights, leaving result-use/licence applicability as an explicit default-
distribution gate. Current upstream main observed at
`86376eb363fb458f660223cb1111db8ea8f8bf6a`; the v2 source still permits the no-key branch, while
standard `search_setup.py` still enables Wikipedia alongside Mwmbl's own index. Published anonymous
quota wording and the inspected handler are not fully aligned, so deployed enforcement remains a
qualification item. M-Q07 is materially advanced but not closed; M-Q01/M-Q06/M-Q08 remain pending.

### Purili movement

Current Purili developer/API/integration documentation explicitly exposes no-account/no-key/no-
fixed-quota Search and Context APIs for applications, AI agents, retrieval pipelines and RAG, and
describes Search as output from its crawler-built independent index. Its privacy policy records no
user-linked search history/profiling and bounds standard IP/path/timestamp access logs to seven days.
The previous evidence register cited a general terms page with personal/noncommercial conditions;
a current authoritative general terms/use page was not independently recovered during this refresh.
That discrepancy remains an admission gate; integration documentation alone does not silently grant
all distributed-client/result-use rights.

### Attempted live observation and remaining path

One bounded non-sensitive valid-query attempt was prepared for each provider with TLS verification,
a 15-second deadline, no credentials, no retries and no bypass. Both stopped before HTTP/TLS because
the current execution environment could not resolve the provider hostname. The query text and raw
results are not retained; the evidence packet records only the query digest and DNS outcome. Since
two unrelated provider names failed at the same local DNS boundary, this is environment evidence,
not provider outage and not regional evidence.

The shortest remaining closure path is now explicit: reconcile Purili current use/result conditions;
build an independently authored Purili offline parser/fixture packet; acquire permitted valid-query
observations for Mwmbl and Purili from a working DNS/HTTPS environment; compare native provenance
and failure domains; then collect paired controlled regional observations including genuine
mainland-China evidence. Only after those steps can D decide whether two routes are admitted or
blocked and whether E/F may be activated. #644/#624 remain open.

## Final Closeout — 2026-11-01

The accepted [SEARCH-001-D final disposition](../reference/SEARCH-001-D-FINAL-DISPOSITION-2026-11-01.md)
closes I294 as a research/evidence gate with a **BLOCKED / ZERO-ADMISSION** result. Mwmbl and Purili
remain explicitly not admitted. Missing successful live valid-query evidence, hosted failure-domain
independence, genuine mainland-China regional evidence and material result-use/service-term closure
are blockers, not accepted assumptions. #827 supplies the reproducible bounded acquisition path
without turning missing measurements into positive regional evidence.

The completion evidence boundary is pre-existing merge
`0afeb5b1a0d228d414a53ca2947bfd835c3f598b`; this closeout status change is not its own evidence.
Parent #624 remains open, and SEARCH-001-E/F remain unauthorized because D admission did not occur.
Future material evidence requires reopening #644 or a separately governed D follow-up.
