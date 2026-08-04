from pathlib import Path


def replace_exact(path: str, old: str, new: str, expected: int = 1) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{path}: expected {expected} matches, found {count}")
    file.write_text(text.replace(old, new))


def append_once(path: str, marker: str, section: str) -> None:
    file = Path(path)
    text = file.read_text()
    if marker in text:
        raise SystemExit(f"{path}: marker already exists: {marker}")
    file.write_text(text.rstrip() + "\n\n" + section.strip() + "\n")


transition = "crates/talos-cli/src/session_transition.rs"
replace_exact(
    transition,
    '''    /// Durable authoritative generation assigned to the replacement Actor.
    pub generation: u64,
''',
    "",
)
replace_exact(
    transition,
    '''            new_handle: prepared.handle,
            generation: next_generation,
''',
    '''            new_handle: prepared.handle,
''',
)

owner_section = '''## Independent-review remediation handoff (2026-08-04)

Lifecycle remains unchanged while PR #131 awaits a new independent review: **TUI-044 / I169 are Active, ADR-056 is Proposed, and Issue #119 is Open**.

The latest implementation evidence tightens same-Session generation replacement into one acknowledged ownership handoff:

- SQLite admission and generation advance share one immediate transaction. Generation G cannot advance while any `accepted_pending`, `running`, or `paused_pending` custody remains.
- After the durable G → G+1 fence, fresh generation-G submissions are rejected as `WrongGeneration` without creating journal custody; historical same-ID reconciliation remains observable.
- The old generation-bound Bridge route is revoked, the old Scheduler is cancelled and joined, and reliable Actor `Shutdown` is queued and joined before the G+1 Actor and Scheduler are spawned and published.
- Race and reconstruction evidence covers concurrent admission versus fencing, full Actor queues, old-Scheduler cancellation, Actor receiver closure, durable generation 1+ reopen, stale-command rejection, journal state, receipt generation, and Provider call counts.

This evidence is a review handoff only. It does not mark the Story, Iteration, ADR, Issue, or PR as Complete, Accepted, Approved, or merge-ready; exact-head CI and independent approval remain mandatory gates.'''

append_once(
    "docs/backlog/active/TUI-044-transactional-batched-steering-turn.md",
    "## Independent-review remediation handoff (2026-08-04)",
    owner_section,
)
append_once(
    "docs/iterations/I169-batched-steering-turn.md",
    "## Independent-review remediation handoff (2026-08-04)",
    owner_section,
)
append_once(
    "docs/decisions/056-transactional-steering-submission-boundary.md",
    "## Independent-review remediation handoff (2026-08-04)",
    owner_section,
)

summary_section = '''## I169 review synchronization (2026-08-04)

- TUI-044 / I169 remain **Active**; ADR-056 remains **Proposed**; Issue #119 remains **Open**.
- PR #131 now carries an atomic durable generation fence plus awaited old Scheduler/Actor retirement before G+1 publication, with production-path race, reconstruction, journal, Bridge, receipt-generation, stale-command, and Provider-call evidence.
- This synchronization records implementation and review evidence only. It does not claim Complete, Accepted, Approved, merge-ready, or merged status; exact-head CI and a new independent review remain required.'''

for path in [
    "docs/BOARD.md",
    "docs/backlog/PRODUCT-BACKLOG.md",
    "docs/iterations/README.md",
    "docs/reference/ISSUE-DOC-CODE-STATUS-2026-08-01.md",
]:
    append_once(path, "## I169 review synchronization (2026-08-04)", summary_section)
