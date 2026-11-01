# SEARCH-001-D Controlled Provider Observation Harness

> Status: Research procedure only; no provider admission or regional claim.
> Owner: SEARCH-001-D / I294 / #644.
> Baseline: `7da057536c572e19fa8b9eb192f2db4e643a5985` (#826 merge).
> Evidence date: 2026-11-01.

This document defines the first executable manual harness for the remaining SEARCH-001-D live
observation gates. It does not add a Talos provider, production request path, model-facing option,
runtime fallback, dependency or deployment behavior. A successful observation is evidence for
review, not automatic admission, geography certification or SEARCH-001-E/F authorization.

## Scope

The helper is `scripts/search_provider_observation.py`. It uses only Python's standard library and
is intentionally separate from Talos runtime/search code. One invocation selects exactly one fixed
provider route:

- Mwmbl: `https://api.mwmbl.org/api/v2/search/`;
- Purili: `https://puri.li/api/search`.

These targets reflect the current published developer surfaces reviewed on 2026-11-01. The harness
does not accept an arbitrary host/path, combined/paid Mwmbl route, Purili Context/infocard/images,
submit-site/challenge flow, result URL fetch or redirect target.

## Privacy And Network Contract

Each run must satisfy all of the following:

1. The operator supplies a full Talos commit SHA, coarse region, network class and opaque pair ID.
   The harness records those values as operator context only and always records that it did not
   certify geography.
2. Query bytes enter only through non-interactive stdin. The evidence contains only SHA-256 and byte
   length. Query text is never printed or stored by the helper.
3. The script performs one HTTPS GET and no semantic retry. `http.client` is used directly, so
   ambient `HTTP_PROXY`, `HTTPS_PROXY` and `ALL_PROXY` variables are not consulted. Redirects are
   not followed.
4. Normal system TLS certificate and hostname verification stays enabled. No credential, cookie or
   Authorization header is sent.
5. Query size is capped at 512 UTF-8 bytes, timeout at 30 seconds maximum and response read at
   1 MiB plus one overflow byte. These are harness safeguards, not proposed production defaults.
6. The output does not include raw body, titles, snippets, result URLs, provider error text, public
   IP or proxy values. Unknown provenance strings are folded into `other` rather than copied.
7. A body larger than the bound is not fully hashed or parsed; the evidence marks it incomplete
   instead of pretending a truncated hash represents the complete response.

This first slice is **direct-only**. User-configured-proxy acquisition may be added as a separately
reviewed mode later; it must not persist proxy URLs or credentials and must remain distinguishable
from direct observations.

## Evidence Output

The JSON record contains:

- schema version and research-only marker;
- Talos commit and UTC observation time;
- opaque pair ID;
- operator-supplied coarse region and network class;
- `proxy_mode: direct` and `geography_certified_by_harness: false`;
- fixed endpoint scheme/host/path without query;
- query SHA-256 and byte length;
- timeout/query/body limits, zero semantic retries and redirect-following false;
- network classification (`http-response`, `dns`, `tls`, `timeout`, `network`, or
  `body-too-large`);
- HTTP status and bounded media type when a response exists;
- complete-body byte count/hash only when the bounded body was fully read;
- parse class, wire-result count, admissible credential-free HTTP(S) URL count and bounded
  provenance-label counts.

For Mwmbl, known `engine` labels are counted separately for `mwmbl`, `wikipedia`, `google`, `user`
and `eusp`; all other values become `other`. For Purili, only documented `crawled` is retained as a
named count and every other source label becomes `other`. These counts are provenance observations,
not hosting/failure-domain proof.

## Paired Run Procedure

Use one opaque operator-generated pair ID for two observations taken from the same controlled
network environment. Run the providers separately so each process can make only one provider
request. Example shape (use a deliberately non-sensitive query and do not put it in shell history):

```text
printf '%s' "$QUERY_FROM_SAFE_SOURCE" | python3 scripts/search_provider_observation.py \
  --provider mwmbl \
  --talos-commit <full-sha> \
  --region '<coarse-region>' \
  --network-class residential \
  --pair-id <opaque-pair-id> > mwmbl-observation.json

printf '%s' "$QUERY_FROM_SAFE_SOURCE" | python3 scripts/search_provider_observation.py \
  --provider purili \
  --talos-commit <full-sha> \
  --region '<same-coarse-region>' \
  --network-class residential \
  --pair-id <same-opaque-pair-id> > purili-observation.json
```

Do not use a personal name, IP address, SSID, provider account identifier or location more precise
than the reviewed coarse-region evidence in `pair_id` or `--region`.

A reviewer must independently determine whether the environment provenance is sufficient. Locale,
DNS resolver geography, cloud-region name, proxy exit, foreign runner or operator-entered label can
never be promoted by this harness into genuine mainland-China evidence. No GeoIP/public-IP lookup is
implemented.

## Interpretation

A valid HTTP/JSON result can advance M-Q01/P-Q01 only as a bounded anonymous route observation. It
does not resolve terms/result-use questions, downstream independence, production transport policy or
regional availability by itself.

Paired observations can contribute to M-Q08/P-Q08 only after environment provenance review and
failure-domain analysis. Mwmbl Wikipedia/reference-only yield must remain separate from Mwmbl-native
yield. Purili `crawled` remains a wire label and not proof of independent hosting.

SEARCH-001-D remains Active / Claimed after this harness. Mwmbl and Purili remain unadmitted, #644
and #624 remain open, and E/F remain unauthorized until the full D admission gate is explicitly
resolved.

## Offline Validation

`scripts/test_search_provider_observation.py` exercises only pure parsing/redaction helpers. It does
not open a socket. The tests verify query redaction, fixed provenance bucketing, empty/error/malformed
classification, credential/non-HTTP URL rejection, metadata validation, direct-mode evidence and
bounded media-type handling.
