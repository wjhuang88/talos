# ADR-078: Read-Only Browser Provider Connector

**Status:** Accepted (BROWSER-001 / Issue #508; maintainer acceptance 2026-09-15)

## Decision (proposed)

The first Browser Provider is read-only and exposes only the existing `fetch_url` browser-page
context contract. Its carrier may be MCP or a confined helper process selected by implementation
review; it must not require WASM or native dynamic-library loading. Discovery/loading is owned by
CAP-001 resolver boundaries, while TOOL-014 remains the sole model disclosure policy.

The connector must enforce origin/TTL/revisit policy, redact cookies, credentials, storage and
secrets, and return bounded typed failures for unavailable, malformed, timed-out or cancelled
requests. It cannot click, fill, upload, download, checkout, mutate pages, or bypass the permission
pipeline. Startup remains network-independent and missing providers degrade to the existing
read-only fallback.

## Evidence required before acceptance

Connector contract and carrier choice; offline/mock and timeout fixtures; redaction and origin/TTL
tests; permission/disclosure conformance; independent security review for credential extraction,
SSRF and process escape; exact changed-file inventory proving no Dashboard overlap.

This proposal authorizes no implementation or network behavior until accepted and claimed.
