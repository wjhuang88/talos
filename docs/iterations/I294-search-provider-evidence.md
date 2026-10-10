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
| Research PR | #668, #673, #676 and #686 (merged; partial evidence) |
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
| Implementation PR | #668, #673, #676 and #686 (merged; evidence only) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR; exact-head CI, governance validators, remote Issue reconciliation and merge-time CAS required. |
| Last Updated | 2026-10-09 |
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
