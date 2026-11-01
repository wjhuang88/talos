# SEARCH-001-D Status Receipt — 2026-11-01

> Status: Historical accepted checkpoint; superseded for current status by the final disposition.
> Owner: SEARCH-001-D / I294 / #644.
> Parent: SEARCH-001 / #624.
> Scope: research and evidence only; no provider admission or SEARCH-001-E/F authorization.

This receipt records the accepted target-branch state after the initial 2026-11-01 evidence slices.
It is retained to remove ambiguity from older in-file wording that still described #825 as a draft
candidate while preserving the historical research record unchanged. Current status is defined by
[SEARCH-001-D Final Disposition — 2026-11-01](SEARCH-001-D-FINAL-DISPOSITION-2026-11-01.md).

## Accepted merge receipts

| PR | Purpose | Merge |
|---|---|---|
| #825 | Current Mwmbl/Purili policy, source and privacy refresh | `df782c9a6a23bf6e636a0fc1a6461ed84e61b8be` |
| #826 | Synthetic offline Purili Search wire characterization | `7da057536c572e19fa8b9eb192f2db4e643a5985` |
| #827 | Controlled live/regional observation harness | `099aa63bc8d3c0029a3d22f7d883e1787baf8be9` |

#827 exact head `d1aaf28f860bbc0d4af97a47553d41e0708293b5` passed CI3701 / run
`41295028143` with all jobs successful. Independent exact-head re-review recorded no
code/evidence/governance-boundary blocker and confirmed eight offline harness tests without network
access. An accidental generated `.pyc` commit pushed by the review agent was explicitly rejected and
removed before the authoritative gate; it is not part of the accepted history.

## Evidence movement

The accepted refresh materially strengthens Mwmbl and Purili as research candidates, but admits
neither route. Mwmbl has current anonymous/no-key API and privacy evidence plus pinned source-path
inspection, while result-use licensing, deployed quota behavior, live valid-query success and
regional/failure-domain qualification remain unresolved. Purili has current no-account/no-key
integration documentation for applications, agents and RAG plus a bounded privacy policy and an
independently authored offline wire characterization; current applicable service/result-use terms,
live success, transport behavior and regional/failure-domain qualification remain unresolved.

#827 adds the reproducible acquisition path for the remaining live M-Q01/P-Q01 and regional
M-Q08/P-Q08 work. The harness is deliberately direct-only and bounded: one fixed-provider HTTPS
request per invocation, query from non-interactive stdin, no redirects or semantic retries, no
ambient proxy inheritance, no credentials/cookies/Auth header, system TLS verification, bounded
query/body/timeout handling, and evidence output limited to digests and summarized metadata. It does
not certify geography, infer public IP/GeoIP, prove failure-domain independence or admit a provider.

The current source/published-service independence classification is maintained in
[SEARCH-001-D Failure-Domain Checkpoint — 2026-11-01](SEARCH-001-D-FAILURE-DOMAIN-2026-11-01.md).
It supports distinct project/implementation and native-index origins while explicitly leaving
runtime hosting/network independence and regional reachability unproved.

The current terms split is maintained in
[SEARCH-001-D Terms And Result-Use Checkpoint — 2026-11-01](SEARCH-001-D-TERMS-RESULT-USE-2026-11-01.md).
It distinguishes permission to automate/call an API from rights to reuse returned result/page
material; both providers advance on interface evidence but neither closes the default result-use gate.

## Blockers at this checkpoint

At this checkpoint SEARCH-001-D / I294 remained incomplete. The shortest remaining path was:

1. obtain permitted live valid-query observations for Mwmbl and Purili from networks with working
   DNS/HTTPS while retaining only bounded metadata and provenance;
2. demonstrate usable native-origin yield rather than treating Wikipedia/reference or generic
   labels as a second independent route;
3. compare actual endpoint/downstream failure domains rather than inferring independence from
   different hostnames or documentation;
4. close the remaining result-use/service-terms disposition documented in the terms checkpoint;
5. collect paired controlled regional observations, including genuine mainland-China provenance.

At that checkpoint #644 and #624 remained open, Mwmbl/Purili were unadmitted, and
SEARCH-001-E/F remained unauthorized.

## Interpretation rule

Where an older SEARCH-001-D/I294 section on `main` still calls #825 a draft or candidate, this
receipt is the newer accepted status for that intermediate 2026-11-01 sequence. Historical details
remain useful for provenance, but they must not be read as overriding later accepted evidence.

## Supersession

The [final disposition](SEARCH-001-D-FINAL-DISPOSITION-2026-11-01.md) supersedes this receipt for
current status. It closes the research/evidence gate with a BLOCKED / ZERO-ADMISSION outcome while
preserving all blockers above as reasons that neither Mwmbl nor Purili is admitted. Parent #624
remains open and SEARCH-001-E/F remain unauthorized because D admission did not occur.
