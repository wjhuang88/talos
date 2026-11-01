# SEARCH-001-D Final Disposition — 2026-11-01

> Gate outcome: **COMPLETE / BLOCKED — zero native providers admitted.**
> Owner: SEARCH-001-D / I294 / #644.
> Parent: SEARCH-001 / #624.
> Completion evidence boundary: `0afeb5b1a0d228d414a53ca2947bfd835c3f598b`.
> This is a research/evidence-gate result, not production provider implementation or legal advice.

SEARCH-001-D asked whether Talos had enough evidence to admit native zero-key web-search routes
before SEARCH-001-E/F. The answer at this checkpoint is **no**. The research gate is complete because
the strongest candidates have explicit evidence-backed dispositions, unresolved items are recorded
as blockers rather than assumptions, and the regional acquisition path is reproducible without
fabricating geography. No route is admitted by completing this gate.

The completion commit above is the latest pre-existing accepted evidence commit (#830). This
closeout record does not use its own status change as completion evidence.

## Accepted 2026-11-01 evidence chain

| PR | Accepted movement | Merge |
|---|---|---|
| #825 | Current Mwmbl/Purili source, API, policy and privacy refresh | `df782c9a6a23bf6e636a0fc1a6461ed84e61b8be` |
| #826 | Independently authored synthetic Purili wire/parser characterization | `7da057536c572e19fa8b9eb192f2db4e643a5985` |
| #827 | Bounded direct-only live/regional observation harness | `099aa63bc8d3c0029a3d22f7d883e1787baf8be9` |
| #828 | Accepted status receipt after #825/#826/#827 | `3ccce7eac00cca65e31750211001fa888c3530a3` |
| #829 | Layered failure-domain / independence classification | `718e566e4ab847b994800036c7fe4dcc808cf37a` |
| #830 | API-use versus returned-result/content-use terms classification | `0afeb5b1a0d228d414a53ca2947bfd835c3f598b` |

Earlier accepted evidence remains part of the provenance chain, including the Mwmbl source/route
packet and synthetic offline wire experiments. The 2026-11-01 sequence narrows the final blockers;
it does not erase the historical evidence register.

## Final candidate disposition

| Candidate | Technical route | Terms / result use | Failure-domain / provenance | Live / regional evidence | Final D disposition |
|---|---|---|---|---|---|
| Mwmbl standard v2 Search API | Strong source-level anonymous/no-key candidate; deterministic synthetic parser evidence exists | API-use permission materially advanced, but current API terms attach CC BY-NC-SA 4.0 plus original-owner rights to returned results; anonymous-plan wording is internally inconsistent | Own native index is supported, but standard search has Wikipedia fan-out; native yield must be provenance-backed; hosted network independence is unproved | #827 provides a bounded acquisition path, but no permitted successful valid-query or genuine controlled mainland-China observation was obtained | **BLOCKED / NOT ADMITTED** |
| Purili core Search API | Strong current public no-account/no-key API candidate; deterministic synthetic parser evidence exists | Current docs strongly invite application/agent/RAG integration, but a current canonical service/result-use terms record was not recovered and historical personal/noncommercial evidence remains unreconciled | Own crawler/index and no-third-party-search-API claims support native-origin research; hosted network independence remains unproved | #827 provides a bounded acquisition path, but no permitted successful valid-query or genuine controlled mainland-China observation was obtained | **BLOCKED / NOT ADMITTED** |
| DuckDuckGo HTML/Lite / Bing family | Existing compatibility/research context only | Prior terms/product-use questions remain unresolved for a new Talos-native default route | Shared/upstream dependence risk means front-end names cannot establish two independent routes | No D admission packet with genuine regional proof | **NOT ADMITTED** |
| Mojeek HTML | Technically researchable | Prior official terms restrict unauthorized automated access/scraping | Not evaluated as a qualifying default native route after the permission blocker | No authorized regional packet | **BLOCKED** |
| Stract hosted route | Historical independent-index reference | Current hosted generic-search service not established | Archived/historical material does not establish a maintained current route | None | **PARKED / NOT ADMITTED** |
| Credentialed/self-hosted routes such as Tavily, Brave API and SearXNG | Useful optional paths only | Auth/operator-specific terms apply | Not candidates for the required default zero-key pair | Not evidence for native zero-key admission | **OUT OF D DEFAULT-ZERO-KEY ADMISSION** |

## Strongest-candidate row closure

The row status below is a gate disposition, not a claim that missing live evidence passed.
`BLOCKED` means the evidence requirement was not satisfied and therefore prevents admission.

### Mwmbl

| Row | Final status | Reason |
|---|---|---|
| M-Q01 valid anonymous request | **BLOCKED** | Published/source anonymous path exists, but no successful bounded valid-query observation was acquired from a working execution network. |
| M-Q02 schema/parser | **Partial evidence; admission blocked** | Independently authored offline characterization exists; no production adapter is authorized. |
| M-Q03 URL/output safety | **Partial evidence; admission blocked** | Synthetic URL/shape behavior is characterized, but no live response acceptance is claimed. |
| M-Q04 provenance/native yield | **BLOCKED** | Source proves Wikipedia fan-out is possible; no live observation proves sufficient native Mwmbl-origin generic-web yield. |
| M-Q05 bounds/malformed/empty handling | **Partial evidence; admission blocked** | Offline bounded experiments exist; they do not substitute for the production contract or live route. |
| M-Q06 transport/failure semantics | **BLOCKED** | #827 supplies a bounded observation mechanism, but no successful provider HTTP exchange was acquired for final qualification. |
| M-Q07 service/privacy/result-use conditions | **BLOCKED** | API use/privacy evidence advanced; default/unrestricted returned-result use remains materially unresolved under the current result clause. |
| M-Q08 independent failure domain / regional evidence | **BLOCKED** | Project/index independence is supported; hosted network independence and genuine paired regional evidence, including mainland China, are absent. |

### Purili

| Row | Final status | Reason |
|---|---|---|
| P-Q01 valid no-key request | **BLOCKED** | Current docs publish the route, but no successful bounded valid-query observation was acquired from a working execution network. |
| P-Q02 schema/parser | **Partial evidence; admission blocked** | Independently authored synthetic wire characterization exists; no production adapter is authorized. |
| P-Q03 URL/output safety | **Partial evidence; admission blocked** | Synthetic invalid/malformed/shape handling is characterized; no live response acceptance is claimed. |
| P-Q04 provenance/native yield | **BLOCKED** | Published own-index/core-Search claims are strong, but no live observation proves useful native yield for Talos acceptance. |
| P-Q05 bounds/malformed/empty handling | **Partial evidence; admission blocked** | Offline experiments narrow parser choices but do not establish a production contract. |
| P-Q06 transport/failure semantics | **BLOCKED** | #827 supplies the bounded acquisition mechanism; no successful provider HTTP exchange completed final transport qualification. |
| P-Q07 service/privacy/result-use conditions | **BLOCKED** | Automated integration intent is strong, but current canonical service/result-use terms remain unreconciled and no blanket returned-content licence was recovered. |
| P-Q08 independent failure domain / regional evidence | **BLOCKED** | Project/index independence is supported; hosted network independence and genuine paired regional evidence, including mainland China, are absent. |

## Regional-evidence disposition

#827 is the accepted reproducible acquisition path. It intentionally records only bounded metadata,
requires operator-supplied coarse region/network class, does not perform public-IP/GeoIP inference,
and never promotes operator labels into certified geography. It can be run from a genuine controlled
network when one is available.

No genuine mainland-China endpoint observation was obtained in this research session. The current
execution environment's DNS failures are environment evidence only, not mainland-China evidence and
not provider-outage evidence. Public measurement networks, foreign-region cloud nodes, VPNs,
platform geolocation labels or CI runners are not silently promoted into genuine mainland-China
acceptance.

Therefore D satisfies the governance requirement to leave regional limits explicit and to provide a
reproducible acquisition path, but **no candidate passes the regional admission gate**.

## E/F authorization decision

**SEARCH-001-E and SEARCH-001-F remain unauthorized.** Parent #624 defines E as depending on C plus
D admission and ultimately requires two validated independent native zero-key routes. This D
closeout provides no admission. A completed research gate with a blocked result is not equivalent to
an admission decision.

No production SearchRouter/provider/default behavior, dependency, lockfile, public tool schema,
startup probe, relay, GeoIP route, paid-provider fallback or provider registration changes as a
result of this closeout.

## What can reopen the gate

Future evidence may justify reopening #644 or starting a separately governed D follow-up. Material
new evidence would include, for example:

1. a current authoritative Purili service/result-use terms record that resolves the historical
   conflict, or equivalent provider clarification;
2. an evidence-backed Mwmbl result-use disposition suitable for the exact intended Talos route;
3. permitted successful valid-query observations for both candidates with native provenance;
4. authoritative hosted failure-domain evidence sufficient to compare correlated dependencies; and
5. paired controlled regional observations from genuine target networks, including mainland China.

Until such evidence is accepted through a new governed change, both strongest candidates remain
outside the default distribution and E/F stay blocked.

## Closeout statement

SEARCH-001-D / I294 is complete as a **research/evidence gate** with a **BLOCKED / ZERO-ADMISSION**
outcome. The parent SEARCH-001 / #624 remains open. The compatibility behavior remains the rollback
boundary, and no user-facing regional availability promise is created by this closeout.
