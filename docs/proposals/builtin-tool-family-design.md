# Built-in Tool Family Design

## Status

Proposal. Produced by `TOOL-007` on 2026-06-28.

## Problem

Talos's built-in tool set has grown from a small file/shell set into a broader runtime surface that
includes local files, search, AST code intelligence, Git, shell execution, HTTP fetch, URL saving,
and web search. The registry remains simple and functional, but the design now needs explicit
family boundaries so future tools do not increase prompt cost, permission ambiguity, or model
confusion.

## Current Inventory

The shared CLI/TUI/MCP native tool surface is currently 28 tools, plus an MCP-only `status` tool.

| Family | Tools | Notes |
|---|---|---|
| File and directory | `read`, `write`, `edit`, `delete`, `ls`, `tree` | `delete` handles both file and directory deletion under one Write classification. |
| Search and inspection | `grep`, `glob`, `diff`, `stat` | `TOOL-004` selected ripgrep library crates for future grep internals. |
| Code intelligence | `find_symbol`, `find_references`, `list_symbols`, `list_imports` | Distinct from grep because these are AST-aware. |
| Git | `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list`, `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout` | Split tools preserve schema and permission clarity better than a raw `git` subcommand. |
| Network and web | `http_request`, `save_url`, `web_search` | `save_url` is both network and write; current single-nature model cannot express both. |
| Shell | `bash` | Escape hatch. Descriptions should continue steering agents to safer native tools first. |

## Design Principles

1. **Registry is capability truth; presentation is policy.**
   `ToolRegistry` remains the source of executable tools. Progressive loading, prompt grouping, and
   provider tool selection should be a presentation layer over the registry, not separate
   registration.

2. **Tool families should be stable prompt units.**
   Tools should carry family metadata so prompt/tool-definition blocks can be loaded as cacheable
   chunks: always-on, file/search, code intelligence, Git, network/web, shell, and extension tools.

3. **One tool may touch multiple risk surfaces.**
   Tool safety cannot be represented only by `Read | Write | Execute | Network` once a tool both
   downloads and writes, or shells out and mutates a remote. Hybrid tools need multi-resource
   classification before web/document tooling expands further.

4. **Do not collapse tools into raw CLI mirrors to save tokens.**
   Git subtools cost prompt space, but they preserve structured schemas, clearer descriptions, and
   permission gates. Consolidation should happen only when a unified tool remains as safe and
   agent-readable as the split tools.

5. **Context fetch and persistence remain separate workflows.**
   `http_request` / future `fetch_url` are context-ingestion tools. `save_url` / future download
   tools persist bytes. This separation should remain in `WEBFETCH-001` Phase 2+.

6. **Model-facing output is compact text unless a result handle is needed.**
   Current `ToolResult` is text-only. Large web/document/search results should move toward bounded
   previews plus explicit follow-up handles rather than dumping large JSON or raw bytes into the
   model context.

## Orthogonality Map

| Area | Overlap | Decision |
|---|---|---|
| `ls` / `tree` / `glob` | All inspect paths, but answer different questions: directory contents, hierarchy, pattern match. | Keep separate; improve descriptions and family guidance. |
| `stat` / `ls --long` | Metadata overlap. | Keep `stat` for single-path detailed metadata; `ls` remains browsing. |
| `grep` / code intelligence | Both find code-related text. | Keep separate; `grep` for text/string/config, symbol tools for AST structure. |
| Git subtools | Ten tools mirror common Git operations. | Keep split for now; raw `git` subcommand would weaken schemas and permission classification. |
| `bash` / all native tools | `bash` can perform many operations. | Keep as explicit escape hatch; native tools remain preferred and should be always visible enough for the model to choose them. |
| `http_request` / `web_search` / `save_url` / future `fetch_url` | Network tools differ by intent: request, discover, persist, ingest. | Keep distinct; require multi-resource permissions before adding more hybrid fetch/save tools. |

## Recommended Work

1. **`TOOL-012`: Tool family metadata and progressive loading.**
   Add family metadata and a presentation policy that can select prompt/provider tool blocks by
   context without changing which tools are registered.

2. **`TOOL-013`: Multi-resource permission classification.**
   Replace or augment single `ToolNature` with a risk profile that can represent hybrid tools such
   as `save_url`, `git_push`, and `git_pull`.

3. **Use ADR-025 for search direction.**
   `TOOL-011` may implement ripgrep-backed grep before or during progressive-loading work.

4. **Keep `WEBFETCH-001` Phase 2+ inside the tool-family design.**
   Document extraction should not add ad hoc tools until result handles, permission surfaces, and
   progressive-loading behavior are defined.

## Non-Decisions

- This proposal does not implement progressive loading.
- This proposal does not change existing tool permissions.
- This proposal does not remove or rename Git tools.
- This proposal does not approve document/PDF/Office extraction dependencies.
