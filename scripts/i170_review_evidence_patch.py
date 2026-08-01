from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text(encoding="utf-8")
    if old in text:
        count = text.count(old)
        if count != 1:
            raise SystemExit(f"{label}: expected one occurrence, found {count}")
        file.write_text(text.replace(old, new, 1), encoding="utf-8")
    elif new not in text:
        raise SystemExit(f"{label}: neither old nor new form found")


IMPLEMENTATION_HEAD = "1ca536159c34437719e4f776db2e02e4afc8510d"
RUN_ID = "30686493121"

# Iteration owner: move to Review and bind the complete automated packet.
replace_once(
    "docs/iterations/I170-windows-workspace-validation-unblocker.md",
    "> Document status: Active",
    "> Document status: Review",
    "I170 status",
)
replace_once(
    "docs/iterations/I170-windows-workspace-validation-unblocker.md",
    "| 2026-08-01 | Validation finding | macOS release preflight and Windows format/check/Clippy plus focused PowerShell tests passed on an earlier Head; the full Windows workspace test job exposed remaining platform assumptions and is not final evidence. |",
    "| 2026-08-01 | Validation correction | Earlier Windows full-workspace failures exposed platform-sorted inventory expectations and document fixtures outside their explicit workspace. Expected inventories now sort independently, successful document fixtures execute inside their own workspace, and external-path rejection remains unchanged. |",
    "I170 validation correction",
)
replace_once(
    "docs/iterations/I170-windows-workspace-validation-unblocker.md",
    "| 2026-08-01 | Draft handoff | Draft PR #126 remains open and unmergeable by policy until the stronger deadline correction, complete Windows workspace tests, rebuilt platform evidence, security acceptance and exact-head review pass. |",
    f"| 2026-08-01 | Automated review gate | Implementation Head `{IMPLEMENTATION_HEAD}` passed CI run `{RUN_ID}`: macOS release preflight; Windows format/check/Clippy, focused PowerShell/permission/deadline tests, complete locked workspace tests, project governance, collaboration claims and rebuilt CLI mock smoke; remote Issue/Owner reconciliation; and the Windows installer fixture. |\n| 2026-08-01 | Review handoff | PR #126 remains Draft until this documentation synchronization Head repeats the same exact-head gates and independent process/security plus maintainer review accepts ADR-057 and the direct-child residual. |",
    "I170 automated review handoff",
)
old_verification = """- Claim-head release preflight, format/check/Clippy/tests, Windows installer fixture and remote Issue/Owner reconciliation passed before activation.
- On a non-final PR #126 Head, macOS release preflight passed; Windows format, workspace check, Clippy and focused PowerShell/environment/continuous-output timeout tests passed.
- The same non-final Windows run failed during full workspace tests. No test is deleted, ignored or weakened; exact failing cases must be repaired and rerun on the final Head.
- Implementation exact-head CI for PR #126 remains pending, and historical recovery validation is not reused as current evidence."""
new_verification = f"""- Implementation Head `{IMPLEMENTATION_HEAD}` passed CI run `{RUN_ID}` with all four jobs green.
- macOS evidence: `git diff --check` plus release preflight, including locked workspace format/check/Clippy/tests and governance validators.
- Windows evidence: format, locked workspace check, Clippy, focused native PowerShell/environment/permission/continuous-output/descendant-held-pipe deadline tests, full locked workspace tests, project governance, collaboration claims, and rebuilt CLI mock smoke.
- Remote evidence: all 26 open Issues reconciled to owner documents and the Windows installer fixture passed.
- Earlier failing Windows inventory and document-boundary fixtures were repaired without deleting, ignoring, weakening, or bypassing external-path checks.
- This Review synchronization commit must repeat the exact-head CI gates before the PR is made ready. Recovery PR #121 evidence is not reused as current proof."""
replace_once(
    "docs/iterations/I170-windows-workspace-validation-unblocker.md",
    old_verification,
    new_verification,
    "I170 verification packet",
)

# Child owners: automated criteria are complete; independent review remains open.
replace_once(
    "docs/backlog/active/TOOL-023-A-bash-timeout-fix.md",
    "**Status**: In Progress — implemented in Draft PR #126; exact-head cross-platform validation pending (2026-08-01)",
    f"**Status**: Review — implementation Head `{IMPLEMENTATION_HEAD}` passed cross-platform automation in CI run `{RUN_ID}`; independent process/security and maintainer review remain pending (2026-08-01)",
    "TOOL-023-A status",
)
replace_once(
    "docs/backlog/active/TOOL-023-A-bash-timeout-fix.md",
    "- Kill and wait for the direct child at expiry, then drain already-produced output.",
    "- At expiry, kill/wait the direct child when still running, preserve output already received, and return without waiting for descendant-held pipe EOF.",
    "TOOL-023-A timeout scope",
)
replace_once(
    "docs/backlog/active/TOOL-023-A-bash-timeout-fix.md",
    """- [ ] Exact final Head passes focused Unix and Windows tests.
- [ ] Exact final Head passes full locked workspace format/check/Clippy/tests on macOS and Windows.
- [ ] Governance, collaboration, release preflight and review gates pass.""",
    """- [x] Implementation Head passes focused Unix and Windows tests.
- [x] Implementation Head passes full locked workspace format/check/Clippy/tests on macOS and Windows.
- [x] Governance, collaboration, release preflight, remote reconciliation and rebuilt Windows smoke pass.
- [ ] Independent process/security and maintainer review accepts the direct-child/process-tree residual.""",
    "TOOL-023-A acceptance",
)
replace_once(
    "docs/backlog/active/TOOL-023-C-windows-powershell.md",
    "**Status**: In Progress — implemented in Draft PR #126; exact-head cross-platform validation and security acceptance pending (2026-08-01)",
    f"**Status**: Review — implementation Head `{IMPLEMENTATION_HEAD}` passed cross-platform automation in CI run `{RUN_ID}`; independent process/security and maintainer acceptance remain pending (2026-08-01)",
    "TOOL-023-C status",
)
replace_once(
    "docs/backlog/active/TOOL-023-C-windows-powershell.md",
    """- [ ] Exact final Head passes Windows and macOS/Unix CI.
- [ ] Rebuilt Windows walkthrough evidence and independent security/maintainer acceptance are recorded.""",
    """- [x] Implementation Head passes Windows and macOS/Unix CI.
- [x] Native PowerShell process/permission/deadline tests and rebuilt Windows CLI mock smoke are recorded in CI run `30686493121`.
- [ ] Independent process/security and maintainer acceptance are recorded.""",
    "TOOL-023-C acceptance",
)
replace_once(
    "docs/backlog/active/TOOL-023-cross-platform-shell-and-timeout.md",
    "**Status**: Partial / Active — TOOL-023-A/C are implemented in Draft PR #126 under I170; TOOL-023-B remains separately unimplemented (2026-08-01)",
    "**Status**: Partial / Review — TOOL-023-A/C passed automated I170 review gates in Draft PR #126; independent review remains pending and TOOL-023-B remains separately unimplemented (2026-08-01)",
    "TOOL-023 epic status",
)

# Security review: close automated findings without self-approving independent review.
security_path = "docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md"
for old, new, label in [
    ("| I170-S1 | High | A second Windows shell registration would bypass the authoritative contribution inventory. | Addressed by reusing `bash_tool_contribution`; exact-head inventory tests pending. |", "| I170-S1 | High | A second Windows shell registration would bypass the authoritative contribution inventory. | Automated evidence complete: one authoritative contribution and platform-sorted product inventories passed on Windows and macOS. |", "S1"),
    ("| I170-S2 | High | A timeout created inside the output loop can be extended indefinitely. | Addressed by one pinned deadline; exact-head Windows/Unix tests pending. |", "| I170-S2 | High | A timeout created inside the output loop can be extended indefinitely. | Automated evidence complete: continuous output and descendant-held-pipe regressions passed under one deadline on Windows and Unix/macOS. |", "S2"),
    ("| I170-S3 | High | Parent-side environment mutation would race concurrent process execution. | Addressed by child-local `env_remove`; child-observation evidence pending. |", "| I170-S3 | High | Parent-side environment mutation would race concurrent process execution. | Implementation and command-builder evidence complete: all canonical names are removed child-locally with no parent mutation; independent reviewer confirmation remains required. |", "S3"),
    ("| I170-S5 | Medium | PowerShell command classification reuses conservative shell heuristics rather than a PowerShell parser. | Acceptable only because unknown/control syntax remains exact; permission regression evidence pending. |", "| I170-S5 | Medium | PowerShell command classification reuses conservative shell heuristics rather than a PowerShell parser. | Automated permission evidence complete: unknown/control, drive/provider and `$`/`~` expansion remain exact resources. |", "S5"),
    ("| I170-S6 | Medium | Platform output normalization could conceal authorization path changes. | Addressed by applying normalization after authorized resolution; focused path tests pending. |", "| I170-S6 | Medium | Platform output normalization could conceal authorization path changes. | Automated evidence complete: normalization occurs after authorized resolution; document fixtures stay inside explicit workspaces while external paths remain rejected. |", "S6"),
    ("| I170-S7 | Medium | Windows CI previously validated only installer fixtures. | Addressed by adding a full Windows Rust workspace job; exact-head result pending. |", f"| I170-S7 | Medium | Windows CI previously validated only installer fixtures. | Automated evidence complete on `{IMPLEMENTATION_HEAD}` / run `{RUN_ID}`: full Windows Rust workspace, governance and rebuilt CLI smoke passed. |", "S7"),
]:
    replace_once(security_path, old, new, f"security finding {label}")
replace_once(
    security_path,
    "**Not yet approved.** PR #126 must remain Draft until the evidence above is complete. Historical Windows workspace success from recovery PR #121 is not current-head evidence.",
    f"**Automated gate passed; independent approval pending.** Implementation Head `{IMPLEMENTATION_HEAD}` passed CI run `{RUN_ID}`. PR #126 remains Draft until the Review synchronization Head repeats the gates and an independent process/security plus maintainer review accepts I170-S3/I170-S4 and ADR-057. Historical recovery PR #121 is provenance only.",
    "security recommendation",
)

# Derived operating views.
replace_once(
    ".agent-governance/manifest.yaml",
    'status_note: "High-risk/release-managed profile confirmed. I170/TOOL-023-A/C is Active in Draft PR #126 under Proposed/Review ADR-057; exact-head Windows/macOS validation and independent security/maintainer acceptance remain pending. TUI-044/I169 is Ready but implementation remains blocked on merged I170. I158 remains Review; I159-I162 remain Blocked. REL-002 remains NO-GO."',
    'status_note: "High-risk/release-managed profile confirmed. I170/TOOL-023-A/C is Review in Draft PR #126 under Proposed/Review ADR-057; automated Windows/macOS, governance, remote reconciliation and rebuilt smoke evidence is green, while independent process/security and maintainer acceptance remain pending. TUI-044/I169 is Ready but implementation remains blocked on merged I170. I158 remains Review; I159-I162 remain Blocked. REL-002 remains NO-GO."',
    "manifest I170 status",
)
replace_once(
    "docs/BOARD.md",
    "| I170 Windows Shell And Portability Recovery | Active — Draft PR #126 (2026-08-01) | [I170](iterations/I170-windows-workspace-validation-unblocker.md) / [TOOL-023-A](backlog/active/TOOL-023-A-bash-timeout-fix.md) / [TOOL-023-C](backlog/active/TOOL-023-C-windows-powershell.md) / [ADR-057](decisions/057-windows-powershell-process-boundary.md) | Keep Draft until exact-head Windows/macOS tests, rebuilt platform evidence, security review and maintainer acceptance pass; never merge archival PR #121. |",
    "| I170 Windows Shell And Portability Recovery | Review — Draft PR #126; automated gates green (2026-08-01) | [I170](iterations/I170-windows-workspace-validation-unblocker.md) / [TOOL-023-A](backlog/active/TOOL-023-A-bash-timeout-fix.md) / [TOOL-023-C](backlog/active/TOOL-023-C-windows-powershell.md) / [ADR-057](decisions/057-windows-powershell-process-boundary.md) | Keep Draft until the Review synchronization Head is green and independent process/security plus maintainer review accepts ADR-057/direct-child residuals; never merge archival PR #121. |",
    "Board I170 row",
)
replace_once(
    "docs/iterations/README.md",
    "| I170 | Windows Workspace Validation Unblocker | Active — Draft PR #126 (2026-08-01) | PowerShell/absolute-timeout/portability implementation exists; exact-head Windows/macOS CI and security/maintainer acceptance remain pending. |",
    f"| I170 | Windows Workspace Validation Unblocker | Review — Draft PR #126 (2026-08-01) | Implementation Head `{IMPLEMENTATION_HEAD}` passed CI run `{RUN_ID}`; final synchronization CI and independent security/maintainer acceptance remain pending. |",
    "Iteration index I170 row",
)
replace_once(
    "docs/backlog/PRODUCT-BACKLOG.md",
    "| 0 | I170 Windows shell and portability recovery | Draft PR #126 implements the still-missing PowerShell process boundary, absolute shell timeout and cross-platform fixtures on current main. Keep it isolated from I169 until exact-head Windows/macOS and security gates pass. |",
    "| 0 | I170 Windows shell and portability recovery | Draft PR #126 is in Review with automated Windows/macOS, governance, remote reconciliation and rebuilt-smoke gates green. Keep it isolated from I169 until final synchronization CI and independent process/security plus maintainer acceptance pass. |",
    "Product Backlog I170 row",
)
