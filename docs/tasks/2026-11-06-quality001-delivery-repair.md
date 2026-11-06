# QUALITY-001: Recent Delivery Quality Repairs

> Document status: Review / Claimed
> Published plan date: 2026-11-06
> Deliverable: reproducible checkout regression corrected, large saved messages restored, usable bounded workspace validation, and accurate T5 Board status.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Repair the four findings from the 2026-11-06 delivery quality assessment: checkout conditional-config regression, host message restoration across large records, fixed validation workload budgets, and T5 derived Board synchronization |
| Claimed At | 2026-11-06 |
| Source Issue | None |
| Governance Claim PR | #864 |
| Authorization Mode | Independent review |
| Authorization Evidence | Session repair instruction; #864 merged as 2e7614bb after independent Agent APPROVE for 4e08f6ba against 35d7264b and CI42200671396 passed; review receipt in PR body |
| Implementation PR | Not started |
| Last Updated | 2026-11-06 |
| Handoff / Release Condition | Local regression and standard preflight pass, independent review and exact-head CI, then implementation evidence and owner closure |

## Baseline and inventory

Assessment baseline: `35d7264b2b2689b26610bee7dda6b04ac4895614`.
Standard preflight passed governance, formatting, check and Clippy, then failed
the conditional-checkout test at checkout.rs:276 (1122 tools tests passed).
The same workspace-built binary fails independently on Linux/Git 2.56.0.
This does not establish an authorization bypass; reconcile the test with the
already implemented target-configuration discovery before changing behavior.
Normal storage writes and legacy reads a 2097153-byte record; paged history
returns RecordTooLarge. Workspace validation uses verbose Cargo tests but caps
each stream at 16 KiB and each command at 30 seconds. T5's owner is Complete
at `322574bc9707492ea6e9d5232794f73e687071d9`, while its Board row is stale.

Inventory of nonterminal iteration headers: I310 Active (Gemini, retain), I317
Active claim merged through #851 (search helpers, retain), I302 Review (Windows
handles, retain), I277 Review (deferred device/human acceptance, retain), I249
Planned (dependency pilot, defer), I164 Paused (superseded layout, retain).
No Blocked header found; all other inspected owners are terminal. Open PRs #838,
#849, #860 and #861 concern Gemini/search and do not own this repair. Recheck
overlap before merging. This independent repair task does not reopen completed
I308/I313/I315 or change their published historical baselines.

## Scope and acceptance

1. Reconcile checkout creation and existing-branch behavior with current target
   configuration admission. Test safe creation and real target-hook refusal;
   preserve exact permission and no-mutation-on-refusal assertions.
2. Restore writer-produced TLOG and JSONL messages larger than a source page
   through the host message scan. Preserve bounded I/O steps, cancellation,
   source-version validation, legacy compatibility and no partial publication.
   Model history indexing/disclosure keeps its separate bounded authority.
   No on-disk migration, format change or whole-file fallback.
3. Make fixed Cargo validation appropriate for real workspaces with finite time
   and retained output limits. Preserve ADR-097 overflow-as-failure, cleanup,
   cancellation and exact admission. Validate actual Cargo execution, not just
   a fake executor. No model-selected arbitrary command or timeout authority.
4. Synchronize the T5 Board row from its existing owner evidence. Record the
   reusable lessons and update user-facing/SDK documentation for changed limits.
5. Focused regression checks and pinned locked standard preflight pass. Obtain
   independent security/API review and exact-head CI before merge; preserve
   unexecuted native platform checks as explicit residuals until CI provides them.

## Constraints, validation and closure ledger

Hard: permission pipeline, bounded process ownership, unchanged storage format,
separate model-disclosure authority, pinned toolchain and locked dependencies.
Soft: fixed validation timeout/output budgets; record chosen values and rationale.
Assumption: the checkout failure is a stale test; verify rather than assume.

Artifacts: Session scan/parser and regressions; validation profiles and real
Cargo fixture; checkout regression if warranted; Board, SDK/CLI documentation,
EVOLUTION and this owner. Preserve unrelated sources and completed owner history.
Run focused feature-enabled tests, both governance validators and
`./scripts/release_preflight.sh`. Windows tests require xwin/MSVC per LOCAL-DEV
and native CI; no Linux result represents native Windows execution.
Residual destination: this owner. Keep delivery Review until actual implementation
commit evidence and required review/CI exist. Roll back only this repair candidate
if acceptance fails; do not suppress tests or widen permission policy.

## Implementation checkpoint — 2026-11-06

Claim #864 is effective at `2e7614bb`. The checkout failure is a stale fixture:
its hook was conditional on `target`, but it expected unrelated `created` to
refuse despite existing admitted target-discovery support. Corrected fixture
verifies successful safe creation and refusal for `blocked` with an actual
matching hook condition, preserving HEAD/index/no-hook/no-new-ref checks. No
production Git behavior or permission change. Focused workspace-built test passed.

Host message restoration accumulates unfinished records across bounded reads;
strict model indexing retains its old bounds. Sixteen focused message-scan tests
passed, including both writer formats, >page records, cancellation and changed
source refusal. No persisted format or writer change. Fixed validation now uses
quiet Cargo tests, 15-minute checks and 2 MiB streams; ADR-097 overflow failures
and smaller evidence summaries remain. These budget choices are Soft policy,
not a promise every project fits. Real Cargo and full preflight remain pending.

T5 Board row now mirrors existing #837 owner evidence. README tool-delivery
wording and execution budgets, SDK restoration semantics and ADR-097 are updated.
All unfinished checks, independent implementation review and remote/native CI
remain owned here; no Complete claim is made.

### Preflight prerequisite fixture correction

First repaired preflight failed an existing Agent pressure fixture: it compared
System instructions from separately prepared turns, including independently
generated unix_seconds timestamps (boundary crossed during the run). This is
fixture-only maintenance within the repair's validation checkpoint: compare the
provider request to its exact prepared request; retain pressure persistence, tool,
budget and call-count assertions. No production Agent behavior or admission change.
Original log: `/tmp/talos-quality-fixed-preflight.log`. The later result does not
erase that failed run. Large-buffer mid-read cancellation/retry regression added
after independent preliminary review; focused message-scan tests now pass 17/17.

The next preflight exposed two old fixture assumptions directly affected by the
repair: the validation deny rule matched the pre-quiet command, and a resume
refusal fixture used a now-supported large record. Updated exact deny matching;
use genuinely malformed history for refusal and add a deadline to avoid hanging
on unintended retirement. The real Actor startup fixture now includes a single
>2 MiB message and verifies exact provider delivery. Interrupted the obsolete
hanging run; `/tmp/talos-quality-final-preflight.log` is not a passing result.

### Local validation checkpoint

- `cargo test --locked -p talos-session --lib message_scan`: 17 passed, including
  cancellation during an already accumulated record and strict model refusal.
- Workspace-feature checkout regression: 1 passed, safe creation and actual
  conditional-hook refusal. Production Git unchanged.
- Conversation validation suite: 25 passed. Real CLI Cargo fixture: 1 passed
  (31.99 seconds), including a deliberately failing second run.
- Real Actor large-message startup and corrupt-resume route-preservation tests:
  1 passed each. The initial malformed-header fixture was excluded by inventory;
  corrected it to a legacy-inventory-visible invalid version, not weaker assertions.
- `cargo clippy --locked -p talos-cli --tests -- -D warnings`: passed.
- `cargo xwin check --locked -p talos-session -p talos-cli --tests --target
  x86_64-pc-windows-msvc`: passed. Tools coding-feature tests also cross-compiled
  with `cargo xwin check --locked -p talos-tools --features coding --tests --target
  x86_64-pc-windows-msvc`. These are compile checks, not native Windows execution.
- MSVC std was missing initially; installed pinned Rust1.99.0 target after official
  archive SHA256 verification. cargo-xwin and SDK/CRT cache were already available.
- A subsequent preflight reached Runtime and found the remaining exact command
  fixture expected pre-quiet argv; synchronized it. Failed/interrupted runs remain
  `/tmp/talos-quality-{fixed,final,converged}-preflight.log`. Final preflight is
  `/tmp/talos-quality-verified-preflight.log`, still running at this checkpoint.

Independent preliminary security/API review found no blockers; its suggestion for
mid-record cancellation coverage is implemented. Final exact-head review, full
preflight and native CI are still required before closure.

## Converged local candidate — 2026-11-06

`./scripts/release_preflight.sh` exited 0, ending `release preflight: passed` in
`/tmp/talos-quality-verified-preflight.log`. Includes both governance validators,
format, locked workspace check/Clippy/tests/doc-tests and both independent SDK
modes. Agent655/655, CLI410/410, Session305/305, Tools1123/1123 passed; no ignored
tests or global serialization added. Final Runtime exact-argv fixture includes
quiet mode. Both xwin test checks and CLI test-target Clippy passed as above.
Native Windows/macOS execution and exact-head CI/review remain remote gates.
This is Review, not Complete; no implementation SHA exists yet for closure.
