# SEARCH-001-BROWSER-SPIKE: Browser SERP feasibility intake

> Document status: Proposed / Unclaimed; research dependency blocked
> Source Issue: #891

This is a reconciliation-only intake of the existing Issue, not a Collaboration
Claim, activation, implementation authorization or completion record.

The Issue's 2026-11-09 offline decision rejects direct mapping of
`BrowserPageRecord.selected_links` to search results: identical exposed fields can
represent a result page or navigation-only page. Its evidence is recorded in
[Issue #891](https://github.com/wjhuang88/talos/issues/891#issuecomment-6735371810).
That evidence was reported by the Issue author; this intake does not independently
certify the experiment or native browser behavior.

Parent: SEARCH-001 / #624. Native browser dependency: WEB-007 / #452.
Reconsider only after a reviewed real browser read-only connector supplies structured
result identity and independent authorization, followed by a separately claimed
bounded proposal. Default search behavior, browser permissions and the three open
implementation/research PRs are unchanged. No implementation work is claimed here.

This intake and the global matrix entry repair the newly opened Issue's missing
registration discovered during WORKTREE-001 documentation closeout. Responsible
maintainer for subsequent intake/ownership reconciliation: @wjhuang88. Delivery
remains Proposed / Unclaimed; no completion evidence exists or is asserted.
