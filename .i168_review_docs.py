from pathlib import Path

OLD_FULL = "2eac5b0523f6d8006318456b631c72cdb5bf9bed"
NEW_FULL = "86262d0290d821b7e3518a0e6371f0b2d3185e95"
OLD_SHORT = "2eac5b05"
NEW_SHORT = "86262d02"


def replace_once(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match in {path}, found {count}")
    file.write_text(text.replace(old, new, 1))


def append_once(path: str, marker: str, content: str) -> None:
    file = Path(path)
    text = file.read_text()
    if marker in text:
        return
    file.write_text(text.rstrip() + "\n\n" + content.rstrip() + "\n")


def replace_all_in(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"missing {old!r} in {path}")
    file.write_text(text.replace(old, new))


# I168 owner evidence.
i168 = "docs/iterations/I168-provider-terminal-outcome-integrity.md"
replace_once(
    i168,
    "| 2026-07-30 | Completion | Red-first print-mode proof failed with exit 101 before `dda2170f`; deterministic fixtures and the complete locked matrix passed at validation harness commit `62ae098d`. Completion Commit is the already-existing implementation/fixture commit `2eac5b05`. |",
    "| 2026-07-30 | Initial closeout candidate | Red-first print-mode proof failed with exit 101 before `dda2170f`; the initial deterministic fixture matrix passed at `62ae098d`, but review later found known protocol terminal values were incorrectly used as unknown fixtures and merged-fd ordering was not asserted. |\n| 2026-07-30 | Review correction | Review required explicit policies for OpenAI `content_filter`/legacy `function_call` and Anthropic `stop_sequence`/`pause_turn`/`refusal`, a synthetic unknown reason, stdout-newline-before-stderr ordering, and negative fixture assertions. Red evidence at run `30557879070` recorded provider exit 101 and fixture exit 1 before correction. |\n| 2026-07-30 | Final completion | Corrected implementation `c570991b` and clean Completion Commit `86262d02` passed the expanded fixture matrix and standard Release preflight. |",
    "I168 execution records",
)
replace_once(
    i168,
    "| `cargo test --locked -p talos-provider terminal` | 0 | 8 | 0 | 0 | 1 |",
    "| `cargo test --locked -p talos-provider terminal` | 0 | 13 | 0 | 0 | 1 |",
    "provider terminal count",
)
replace_once(
    i168,
    "| `cargo test --locked -p talos-cli terminal` | 0 | 6 | 0 | 0 | 1 |",
    "| `cargo test --locked -p talos-cli terminal` | 0 | 7 | 0 | 0 | 1 |",
    "CLI terminal count",
)
replace_once(
    i168,
    "| `cargo test --locked -p talos-tui terminal` | 0 | 28 | 0 | 0 | 0 |",
    "| `cargo test --locked -p talos-tui terminal` | 0 | 29 | 0 | 0 | 0 |",
    "TUI terminal count",
)
replace_once(
    i168,
    "| `cargo test --workspace --locked` | 0 | 2673 | 0 | 0 | 1 |",
    "| `cargo test --workspace --locked` | 0 | 2681 | 0 | 0 | 1 |",
    "workspace test count",
)
replace_once(
    i168,
    f"- Completion Commit: `{OLD_FULL}`. This implementation/fixture commit existed before this status update and contains `dda2170f` as an ancestor.",
    f"- Completion Commit: `{NEW_FULL}`. This clean implementation state existed before the review-evidence synchronization and contains corrected implementation `c570991b` as an ancestor.",
    "I168 completion commit",
)
replace_once(
    i168,
    "- Rebuilt-binary acceptance: PASS for both compatible protocols in workflow run `30552762936`; normal, truncation, unknown, EOF, decode, transport, and tool-continuation outcomes matched the owner acceptance matrix.",
    "- Rebuilt-binary acceptance: PASS in review-correction run `30557879070`, job `90922974648`. OpenAI-compatible cases cover normal, MaxTokens, `content_filter`, legacy `function_call`, synthetic unknown, EOF, decode, transport, and tool-continuation. Anthropic-compatible cases cover normal, `stop_sequence`, MaxTokens, `pause_turn`, `refusal`, synthetic unknown, EOF, decode, transport, and tool-continuation. Separate-fd assertions and merged `2>&1` assertions prove partial stdout is newline-terminated before the single MaxTokens warning; all non-MaxTokens paths reject stale truncation text.",
    "I168 rebuilt acceptance",
)
replace_once(
    i168,
    "- Governance acceptance: owner and derived documents synchronized; I164 remains Paused, I158-I162 remain Blocked, ADR-053 remains Proposed, and OBS-002 remains Refinement.",
    "- Final workspace acceptance: CI run `30558599777`, rerun job `90926266628`, checked the PR merge containing clean HEAD `86262d02`; Release preflight passed with 2681 tests and zero failures. Windows fixture job `90926267367` passed.\n- Governance acceptance: owner and derived documents synchronized; I164 remains Paused, I158-I162 remain Blocked, ADR-053 remains Proposed, and OBS-002 remains Refinement.",
    "I168 final CI acceptance",
)
append_once(
    i168,
    "## Review Correction Evidence",
    """## Review Correction Evidence

- Review source: PR #67 top-level review on 2026-07-30.
- Protocol policy:
  - OpenAI `content_filter` is a known filtered terminal error; deprecated `function_call` is a known legacy terminal error that directs compatible gateways to `tool_calls`; only synthetic/unrecognized values use `UnsupportedReason`.
  - Anthropic `stop_sequence` is explicit normal completion; `pause_turn` and `refusal` are known bounded non-success outcomes; only synthetic/unrecognized values use `UnsupportedReason`.
- Red-first correction: workflow run `30557879070`, job `90922974648`, recorded `REVIEW_RED_PROVIDER_EXIT=101` and `REVIEW_RED_FIXTURE_EXIT=1`. The merged output was exactly the rejected shape `fixture partialWarning...` before the ordering correction.
- Focused green evidence in the same job: I168 provider matrix 12 passed; provider `terminal` filter 13 passed; session diagnostic filter 4 passed plus the six-test I168 diagnostic binary in workspace preflight; CLI terminal filter 7 passed; TUI terminal filter 29 passed; package Clippy with `-D warnings` and rebuilt-binary fixture passed.
- Final clean implementation: `c570991b` followed by artifact cleanup `86262d02`; no temporary workflow, generated bytecode, or red log is present in the PR diff.
- Standard clean-HEAD evidence: CI run `30558599777`, Release preflight job `90926266628` and Windows fixture job `90926267367`, both PASS; 2681 workspace tests, zero failures.
""",
)

# Story owner.
story = "docs/backlog/active/RUNTIME-003-provider-terminal-outcome-integrity.md"
replace_once(
    story,
    f"- Completion Commit: `{OLD_FULL}`.",
    f"- Completion Commit: `{NEW_FULL}`.",
    "story completion commit",
)
replace_once(
    story,
    "- Final evidence: workflow run `30552762936`, job `90905349923`, `OVERALL_EXIT=0`; 2673 workspace tests passed with zero failures; focused counts are recorded in I168.",
    "- Initial closeout evidence: workflow run `30552762936`, job `90905349923`, exposed the first implementation packet. PR #67 review then required known-protocol policy separation and merged-output ordering evidence.\n- Final review-correction evidence: workflow run `30557879070`, job `90922974648`, passed the expanded parser/session/CLI/TUI and rebuilt-binary matrix; clean-HEAD CI run `30558599777`, job `90926266628`, passed Release preflight with 2681 workspace tests and zero failures.",
    "story final validation",
)
replace_once(
    story,
    "- Rebuilt `target/debug/talos` passed the OpenAI-compatible and Anthropic-compatible matrix for explicit completion, MaxTokens, unknown reason, EOF, invalid UTF-8, transport failure, and tool-success-then-continuation-failure.",
    "- Rebuilt `target/debug/talos` passed explicit normal completion, OpenAI `content_filter`, legacy `function_call`, Anthropic `stop_sequence`, `pause_turn`, `refusal`, synthetic unknown reasons, MaxTokens, EOF, invalid UTF-8, transport failure, and tool-success-then-continuation-failure. The fixture asserts empty normal stderr, exactly one MaxTokens warning, absence of stale truncation text on errors, and correct merged-fd line ordering.",
    "story fixture expansion",
)
append_once(
    story,
    "### Review-Correction Policy",
    """### Review-Correction Policy

- Known provider terminal values are never described as unknown.
- OpenAI `content_filter` and deprecated `function_call` are explicit bounded provider-error policies.
- Anthropic `stop_sequence` maps to explicit normal completion; `pause_turn` and `refusal` are explicit bounded provider-error policies because Talos does not implement automatic pause continuation in this Story.
- Only unrecognized values such as the deterministic `fixture_unknown_reason` use `UnsupportedReason`.
- MaxTokens output is newline-terminated and flushed on stdout before the single stderr warning, including when descriptors are merged.
""",
)

# Derived current-state documents.
replace_once(
    "docs/iterations/README.md",
    "| I168 | Provider Terminal Outcome Integrity | Complete (2026-07-30) | ✅ Completion Commit `2eac5b05`; 2673 workspace tests and deterministic rebuilt-binary OpenAI/Anthropic terminal-outcome fixtures pass. |",
    "| I168 | Provider Terminal Outcome Integrity | Complete (2026-07-30) | ✅ Completion Commit `86262d02`; 2681 workspace tests and reviewed rebuilt-binary policy/ordering fixtures pass. |",
    "iteration index",
)
replace_once(
    "docs/BOARD.md",
    "| I168 Provider Terminal Outcome Integrity | Complete (2026-07-30) | [I168](iterations/I168-provider-terminal-outcome-integrity.md) / [RUNTIME-003](backlog/active/RUNTIME-003-provider-terminal-outcome-integrity.md) | Completion Commit `2eac5b05`; explicit terminal classification, bounded non-transcript diagnostics, print/TUI truncation display, 2673 workspace tests, and rebuilt-binary dual-protocol fixtures pass. |",
    "| I168 Provider Terminal Outcome Integrity | Complete (2026-07-30) | [I168](iterations/I168-provider-terminal-outcome-integrity.md) / [RUNTIME-003](backlog/active/RUNTIME-003-provider-terminal-outcome-integrity.md) | Completion Commit `86262d02`; reviewed known-terminal policies, bounded diagnostics, stdout/stderr ordering, 2681 workspace tests, and negative dual-protocol fixtures pass. |",
    "board I168 row",
)

for path in [
    "docs/backlog/PRODUCT-BACKLOG.md",
    "docs/backlog/active/OBS-002-turn-pipeline-boundary-observability.md",
    "docs/tasks/2026-07-26-v0.6-runtime-productization-program.md",
    "docs/tasks/2026-07-28-four-month-v06-execution-package.md",
]:
    text = Path(path).read_text()
    text = text.replace(OLD_SHORT, NEW_SHORT).replace("2673", "2681")
    Path(path).write_text(text)

# Stable reference documentation.
reference = "docs/reference/PROVIDER-TERMINAL-OUTCOMES.md"
replace_once(
    reference,
    "| Unsupported finish/stop reason | Bounded provider failure | Partial stdout may remain visible; stderr names the bounded unsupported reason | Failure |",
    "| Known policy terminal (`content_filter`, legacy `function_call`, `pause_turn`, `refusal`) | Bounded, named provider failure | Partial stdout is newline-terminated; stderr names the exact known policy | Failure |\n| Truly unknown finish/stop reason | Bounded provider failure | Partial stdout is newline-terminated; stderr names the bounded synthetic/unrecognized reason | Failure |\n| Anthropic `stop_sequence` | Explicit normal completion | Response completes quietly | Success |",
    "reference policy rows",
)
replace_once(
    reference,
    "The fixture covers OpenAI-compatible and Anthropic-compatible normal completion, MaxTokens,\nunsupported reasons, terminal-frame-less EOF, invalid UTF-8, transport failure, and a successful\ntool result followed by continuation EOF.",
    "The fixture covers OpenAI-compatible normal completion, MaxTokens, `content_filter`, legacy\n`function_call`, synthetic unknown, terminal-frame-less EOF, invalid UTF-8, transport failure, and\ntool continuation. Anthropic-compatible coverage adds `stop_sequence`, `pause_turn`, and `refusal`.\nIt asserts exact stdout/stderr contents, exactly one truncation warning, negative absence of stale\ntruncation/error labels, completed tool-result forwarding, and merged-fd line ordering.",
    "reference fixture coverage",
)
replace_once(
    reference,
    "The I168 completion packet used workflow run `30552762936` at validation harness commit `62ae098d`.\nAll focused commands, source scan, governance validation, build, and the rebuilt-binary fixture\nexited 0; `cargo test --workspace --locked` recorded 2673 passed and zero failed. The governance\ncloseout mutation then passed in workflow run `30554255195`. Completion Commit `2eac5b05` predates\nboth evidence-recording steps and owns the final implementation plus deterministic fixture scripts.",
    "The reviewed completion packet uses clean Completion Commit `86262d02`. Review-correction run\n`30557879070` records red parser/fixture evidence before the fix and a green expanded dual-protocol\nfixture afterward. Clean-HEAD CI run `30558599777` passed Release preflight and Windows fixture;\n`cargo test --workspace --locked` recorded 2681 passed and zero failed.",
    "reference final packet",
)

# Reusable lesson.
replace_once(
    "EVOLUTION.md",
    "| 46 | Provider / Runtime | transport EOF 不是成功；每个消费者都必须投影明确 terminal outcome | I168/RUNTIME-003 |",
    "| 46 | Provider / Runtime | transport EOF 不是成功；已知协议终止值必须有独立 policy，不能冒充 unknown；每个消费者都必须投影明确 outcome | I168/RUNTIME-003 |",
    "evolution index",
)
replace_once(
    "EVOLUTION.md",
    "- Prevention: every streaming adapter and every consumer must have a matrix for normal completion, truncation, unknown reason, EOF, decode/transport failure, and tool-continuation failure. A shared projection test is not end-to-end evidence.\n- Promoted to rule/check: I168/RUNTIME-003 Completion Commit `2eac5b05`, `scripts/verify_i168_provider_terminal.sh`, and `docs/reference/PROVIDER-TERMINAL-OUTCOMES.md`.",
    "- Prevention: every streaming adapter and every consumer must distinguish known policy terminals from truly unknown values and cover normal completion, truncation, known filtering/pause/refusal, unknown reason, EOF, decode/transport failure, and tool-continuation failure. Headless tests must assert separate streams and merged-fd ordering; a shared projection test is not end-to-end evidence.\n- Promoted to rule/check: I168/RUNTIME-003 Completion Commit `86262d02`, `scripts/verify_i168_provider_terminal.sh`, and `docs/reference/PROVIDER-TERMINAL-OUTCOMES.md`.",
    "evolution prevention",
)
