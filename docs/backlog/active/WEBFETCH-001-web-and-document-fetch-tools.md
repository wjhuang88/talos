# WEBFETCH-001: Web And Document Fetch Tools

## Outcome

Talos can fetch URLs and local/downloaded documents through permission-aware tools, classify the
response or file type, return token-efficient content to the model, and preserve full structured
metadata for later follow-up fetches.

## Status

Phase 0 (`http_request`) shipped in I039. Phase 0+ content detection and HTML/JSON extraction were
temporarily merged into `http_request` in I040, then corrected on 2026-06-30 after the architecture
review: `fetch_url` is again the unified URL context-ingestion entry and `http_request` is narrowed
to advanced HTTP/API inspection disclosed on demand. `save_url` remains a separate write-capable
download tool. TOOL-012 progressive loading, TOOL-013 hybrid network/write permission boundaries,
and TOOL-014 conditional tool disclosure are complete. Phase 2+ remains the bounded
document-capture feature track: design first, then `document_extract` for local
text/HTML/JSON/CSV/Markdown-like resources, then fetch/save/extract integration. PDF/Office/OCR and
browser automation remain out of scope without a separate dependency gate. PLUGIN-001 WASM remains
the likely delivery path for heavy format handlers.

I090 activated 2026-07-04 to audit the already-present `document_extract`, `fetch_url`/`save_url`,
and permission-boundary implementation before adding more code. The first I090 decision is whether
WEBFETCH Phase 2+ is already satisfied for local text/HTML/JSON/CSV/Markdown/XML resources or
whether a bounded implementation gap remains.

## Priority

P2 research/planning for Phase 2+. Next phases: HTML extraction, link ranking, markdown conversion,
and document extraction design, coordinated with TOOL-004/TOOL-007 before new tool implementation.

## Origin

User requests on 2026-06-19:

- Add tools for grabbing web page information and API responses.
- Evaluate `0xMassi/webclaw` as an embeddable option.
- Consider a lighter design using `rs-trafilatura`, `scraper`, URL normalization, high-value link
  selection, and local full-link storage.
- Handle unknown URLs where the user does not know whether the target is a web page or structured
  API response.
- Consider MarkItDown-like document conversion for PDF, image, binary, Office, and other files.

## Problem

Talos currently has strong local file tools but no first-class network/document ingestion path.
Users may provide a URL that could be:

- an HTML page;
- a JSON/XML/CSV API endpoint;
- a PDF or Office document;
- an image or binary file;
- an error page masquerading as an API response;
- a JavaScript-heavy page that static fetching cannot extract.

The agent should not force users to classify the URL first. The tool layer should fetch, detect,
summarize, and preserve follow-up context in a deterministic way.

## Scope

Design a small Rust-native tool family:

- `fetch_url`: unified URL context-ingestion tool with `mode = auto | raw`.
- `http_request`: advanced HTTP/API request tool for custom methods, headers, bodies, and
  low-level inspection; registered but disclosed on demand.
- `web_fetch`: optional alias or mode for HTML page extraction.
- `document_extract`: local or fetched document-to-Markdown/text extraction, added format by
  format.
- `save_url` or `download_file`: separate write-capable tool that downloads URL bytes to an
  explicit local path and filename.

All network access must be permission-aware and disableable. The first implementation should be
useful without browser automation, anti-bot bypass, proxy pools, hosted APIs, OCR, audio
transcription, or whole-site crawling.

`fetch_url` and `document_extract` are context-ingestion tools: their primary job is to convert a
remote or local resource into bounded, LLM-ready context. They should not silently persist fetched
content to arbitrary files.

Saving remote content is a separate workflow. If the user wants the original response, PDF, image,
archive, or processed text saved locally, the agent should call the dedicated write-capable save
tool with an explicit destination path and filename.

## Detection Pipeline

Default behavior should use `mode = auto`:

```text
input URL
  |
  v
HTTP request with bounded timeout and max body bytes
  |
  v
classify by status, Content-Type, Content-Disposition, URL hints, and body sniffing
  |
  +--> JSON / NDJSON / XML / CSV / text/event-stream -> structured API path
  |
  +--> HTML / XHTML -> page extraction path
  |
  +--> PDF / Office / image / binary -> document/media path or unsupported notice
  |
  +--> unknown -> raw preview + classification evidence
```

Rules:

- Content type is the first signal, but body sniffing can override weak or wrong headers.
- URL shape such as `/api/`, `.json`, `.pdf`, or `.csv` is only a weak hint.
- HTML error pages returned by API endpoints must be reported as HTML/error, not blindly treated as
  JSON.
- Structured API responses should preserve structure; do not convert JSON into Markdown unless the
  user asks for narrative rendering.
- Page responses should extract readable content and avoid dumping full HTML into model context.
- Binary or oversized responses should return metadata and a safe preview only.

## Page Extraction Design

For HTML pages, use a two-track extraction flow:

1. Extract readable main content with a candidate backend such as `rs-trafilatura`.
2. Independently parse the full DOM with `scraper` to collect all `<a>` links.

This prevents readable-content extraction from accidentally dropping navigation, pagination,
documentation, or reference links.

The link pipeline should:

- resolve relative URLs;
- normalize schemes, hosts, paths, query parameters, and fragments;
- remove obvious tracking parameters where safe;
- deduplicate;
- classify links;
- rank high-value links;
- return only a small high-value subset to the model;
- store the full link set locally for later explicit follow-up fetches.

Suggested link classes:

- `same_page_anchor`
- `same_origin_internal`
- `docs_navigation`
- `api_reference`
- `download`
- `external`
- `social`
- `auth_account`
- `asset`
- `tracking_noise`

## Tool Output Policy

The model-facing output should be compact text, not a giant JSON dump. It should include:

- final URL and redirect chain summary;
- status code;
- detected kind;
- content type;
- detection reason when headers and body disagree;
- extracted content or structured preview;
- top high-value links;
- hidden link count;
- local `link_store_id` or result reference for follow-up operations.

Example model-facing shape:

```text
fetched https://example.com/docs
detected: html_page
status: 200
content_type: text/html
title: Example Docs

content:
...

links: 184 total, 18 shown, store_id=links:...
- docs_navigation https://example.com/docs/api
- api_reference https://example.com/openapi.json
```

Internal storage may remain structured JSON or SQLite-backed records.

## Save/Download Tool Boundary

`save_url` or `download_file` should be a separate tool from `fetch_url`.

Purpose:

- download a URL response to a user-specified local file;
- optionally save the processed Markdown/text representation rather than raw bytes;
- return file metadata and path, not inject the full content into model context.

Suggested parameters:

- `url`
- `path` or `directory` + `filename`
- `mode = raw | extracted_text | markdown | auto`
- `overwrite = false` by default
- `max_bytes`
- optional request headers, subject to redaction and policy

Rules:

- This is a write-capable tool and must use the normal file-write permission path.
- The destination path must be explicit; do not infer hidden downloads into the workspace.
- Parent directories may be created only when requested.
- Existing files are not overwritten unless `overwrite = true`.
- The tool should validate extension/content-type mismatches and report them.
- Fetching for context must not automatically save files.
- Saving a file must not automatically inject full content into context; the agent can call
  `read`, `document_extract`, or `fetch_url` separately if it needs context.

## Candidate Dependencies

These are research candidates, not approved dependencies:

| Area | Candidate | Initial Assessment |
| --- | --- | --- |
| HTTP | `reqwest` with `rustls` | Good fit for `http_request`; no browser rendering or anti-bot bypass. |
| Main HTML content | `rs-trafilatura` | Promising; MIT OR Apache-2.0; default feature set is small; `spider` feature should remain off. |
| Link extraction | `scraper` | Mature Rust HTML/CSS parser; ISC license; useful independent DOM scan. |
| HTML-to-Markdown fallback | `html2md` or internal renderer | Candidate fallback if main extraction fails. |
| Full web extraction suite | `0xMassi/webclaw` | Useful reference or optional MCP/CLI integration; not suitable for direct embed due AGPL-3.0 and broad scope. |
| MarkItDown-like all-format conversion | Microsoft MarkItDown | Python tool; not acceptable as default runtime dependency under Rust-first policy. |
| Rust MarkItDown-like crate | `markitdown` | MIT; early version; needs POC before trust. |
| Multi-format Rust conversion | `anytomd` | Apache-2.0; promising but default features include network/Gemini path, so features must be controlled. |
| Heavy dispatcher | `mdkit` | Strong capability but includes Pandoc/PDFium/OCR options; likely optional enhancement only. |
| Office documents | `office_oxide`, `calamine`, `undocx` | Candidate second phase; evaluate per format. |
| PDF | `pdf_oxide`, `spectre_pdf`, `unpdf` | Candidate second phase; start with text PDFs, not scanned/OCR PDFs. |

## Evaluation Notes

`webclaw` can satisfy advanced web extraction, but directly embedding it is not a good first move:

- the repository is AGPL-3.0;
- it includes CLI, MCP server, REST server, hosted API paths, LLM support, PDF handling, proxy
  handling, browser TLS fingerprinting, crawl/map/batch/research workflows, and larger binaries;
- Talos' first need is smaller: permission-aware `http_request`, static HTML extraction, link
  indexing, and typed document expansion.

Microsoft MarkItDown is a useful product reference, but not a Rust dependency candidate. Rust has
partial equivalents, so Talos should define its own conversion boundary and add format backends
incrementally.

## Phasing

### Phase 0: HTTP/API Foundation

- Implement `http_request` with method, headers, query, body, timeout, max bytes, redirect policy,
  and redaction.
- Detect structured responses: JSON, NDJSON, XML, CSV/TSV, text, event stream, binary.
- Return bounded previews and preserve raw response metadata for debugging.
- **Phase 0+ (I040)**: Content-type detection, HTML text extraction via scraper, JSON pretty-print.
  See acceptance criteria below.
- Keep context-fetch behavior separate from saving remote bytes to disk.

### Phase 1: Static HTML Page Fetch

- Implement `fetch_url mode=auto`.
- Use static HTTP fetch only; no browser rendering.
- Extract readable content.
- Extract, normalize, classify, rank, and store links.
- Return compact content plus high-value links.

### Phase 1b: Explicit URL Save Tool

- Implement a write-capable `save_url`/`download_file` only after the context fetch path is clear.
- Require explicit destination path and filename.
- Save raw bytes or selected processed representation without dumping the whole file into history.

### Phase 2: PDF/Text Documents

- Add PDF text extraction for non-scanned PDFs.
- Candidate implementation path: WASM plugin (PLUGIN-001) rather than built-in,
  to avoid embedding heavy PDF parsing dependencies in the core binary.
- Detect scanned/OCR-needed PDFs and return a clear unsupported message.

### Phase 3: Office And Archive Documents

- Add DOCX/XLSX/PPTX text/table extraction.
- Candidate implementation path: WASM plugin (PLUGIN-001), same rationale as PDF.
- Add ZIP dispatch only with recursion, size, and file-count limits.

### Phase 4: Optional Enhancements

- OCR, audio/video transcription, JS rendering, anti-bot bypass, proxy pools,
  hosted extraction, and webclaw integration are optional.
- All heavy format handlers (PDF, Office, image, binary) target PLUGIN-001
  WASM plugin delivery rather than built-in embedding. This keeps the core
  binary lean (see TOOL-008 tree-sitter on-demand analysis).
- Core provides the `http_request` / `fetch_url` fetch + dispatch layer;
  format-specific extraction plugins are loaded on demand.

## Permission And Safety Boundaries

- Network fetches are permission-gated and can be disabled independently from local file reads.
- Private, loopback, link-local, metadata-service, and local-network addresses need explicit policy
  before access is allowed.
- Headers and cookies must be redacted in model-facing output.
- Request bodies and authorization headers must not be persisted unless explicitly allowed.
- Response size, redirect count, timeout, link count, archive expansion, and document page count
  must be bounded.
- Fetching a URL must not automatically crawl additional URLs without a separate tool call.
- Full link sets are stored for later selection; the LLM receives only high-value candidates.
- Fetching for context and saving to disk are separate tools with separate permission surfaces.

## Acceptance Criteria

### Phase 0 (I039 — delivered 2026-06-21)

- [x] `http_request` tool implemented with method/body/header/timeout/max-byte and Network permission gating
- [x] SSRF guard blocking private/reserved IP ranges
- [x] Header sanitization blocking security-sensitive headers and CR/LF injection
- [x] Response size capped at 64KB, redirect limit of 5

### Phase 0+ — Content Type Detection (I040; corrected to `fetch_url` on 2026-06-30)

- Given `fetch_url` with `mode: "auto"` (default) to a URL returning `Content-Type: text/html`
  When the tool executes
  Then response body is HTML-tag-stripped text (via scraper), not raw HTML markup

- Given `fetch_url` with `mode: "auto"` to a URL returning `Content-Type: application/json`
  When the tool executes
  Then response body is pretty-printed JSON

- Given `fetch_url` with `mode: "auto"` to a URL returning `Content-Type: text/plain`
  When the tool executes
  Then response body is returned as-is

- Given `fetch_url` with `mode: "auto"` to a URL returning binary/non-text content
  When the tool executes
  Then response shows content type info and byte count, not raw binary dump

- Given `fetch_url` with `mode: "raw"`
  When the tool executes
  Then response body is returned as-is (preserving current behavior)

### Phase 1+

- [ ] `http_request` requirements define method/body/header/query/timeout/max-byte behavior and
      permission gates.
- [ ] `fetch_url mode=auto` detection rules classify HTML, JSON/NDJSON, XML/feed, CSV/TSV, plain
      text, PDF, Office, image, binary, and unknown responses.
- [ ] HTML page extraction separates readable content extraction from full DOM link extraction.
- [ ] Link normalization, deduplication, classification, ranking, model-facing truncation, and full
      local link storage are specified.
- [ ] A separate write-capable save/download tool is specified for persisting URL content to an
      explicit local destination.
- [ ] Candidate dependencies are evaluated with license, native-code, feature, build, and output
      quality evidence before implementation.
- [x] Hybrid network/write permission behavior is defined through TOOL-013 before adding new
      save/download/extract tools.
- [x] Progressive-loading behavior is aligned with TOOL-012 before adding large document
      extraction outputs.
- [ ] Result-handle behavior is specified before adding large document extraction outputs.
- [ ] webclaw is recorded as reference/optional external integration, not a direct embed candidate.
- [ ] MarkItDown-like functionality is phased by format; no Python runtime dependency is added.
- [ ] RES-001 can use these tools without requiring unattended crawling or hosted services.

## Non-Goals

- Do not add browser automation in the first implementation.
- Do not bypass anti-bot or CAPTCHA systems.
- Do not add web search or autonomous research workflow in this item.
- Do not bundle Python, Node.js, Pandoc, PDFium, OCR engines, or model weights by default.
- Do not make network fetches available without permission and policy controls.
- Do not treat extracted content as trusted instructions.
- Do not mix context fetching with implicit file downloads.

## Required Reads

- `docs/backlog/active/RES-001-exploration-library.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`
- `docs/proposals/builtin-tool-family-design.md`
- `docs/decisions/010-git-search-tool-dependency-boundary.md`
- `docs/decisions/017-exploration-library-storage.md`
- `docs/iterations/I036-research-consolidation.md`
- `https://github.com/0xMassi/webclaw`
- `https://github.com/microsoft/markitdown`

## Residual Work Destination

If Phase 0 and Phase 1 are accepted, create an implementation iteration for the minimal
permission-aware HTTP/web fetch tools. PDF, Office, OCR, JS rendering, webclaw integration, and
MarkItDown-like multi-format conversion remain separate Spikes or optional enhancements.
