param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$ProjectRoot
)

$ErrorCount = 0
$WarningCount = 0

function Add-WarningMessage {
    param([string]$Message)
    Write-Output "WARNING: $Message"
    $script:WarningCount++
}

function Add-ErrorMessage {
    param([string]$Message)
    Write-Output "ERROR: $Message"
    $script:ErrorCount++
}

function Get-ManifestValue {
    param([string]$Key)
    foreach ($Line in Get-Content -LiteralPath $script:Manifest) {
        $Pattern = "^\s*$([regex]::Escape($Key)):\s*(.*)$"
        if ($Line -match $Pattern) {
            return (($Matches[1] -replace "\s*#.*$", "").Trim().Trim("'`""))
        }
    }
    return ""
}

function Get-SectionMap {
    param([string]$Section)
    $Result = @{}
    $Current = ""
    foreach ($Line in Get-Content -LiteralPath $script:Manifest) {
        if ($Line -match "^([A-Za-z0-9_]+):\s*($|#)") {
            $Current = $Matches[1]
            continue
        }
        if ($Current -eq $Section -and $Line -match "^\s{2}([A-Za-z0-9_]+):\s*([^#\s].*)$") {
            $Key = $Matches[1]
            $Value = (($Matches[2] -replace "\s*#.*$", "").Trim().Trim("'`""))
            $Result[$Key] = $Value
        }
    }
    return $Result
}

function Get-CapabilityValue {
    param([string]$Name)
    if ($script:Capabilities.ContainsKey($Name)) {
        return $script:Capabilities[$Name]
    }
    return ""
}

function Test-CompletionClaim {
    param([string]$Path)
    $Text = Get-Content -LiteralPath $Path -Raw
    return $Text -match "(?im)^\s*[*_>\-\s]*status\b.*\b(complete|completed|done|shipped|delivered)\b" -or
        $Text -cmatch "\b(COMPLETE|COMPLETED|DONE|SHIPPED|DELIVERED)\b"
}

function Test-Evidence {
    param([string]$Path)
    $Text = Get-Content -LiteralPath $Path -Raw
    return $Text -match "(?is)```|\b(test|tests|tested|testing|cargo|npm|pnpm|yarn|pytest|gradle|mvn|make|go test|passed|passing|verified|verify|evidence|exit\s*0|coverage|benchmark|smoke)\b"
}

function ConvertTo-RelativePath {
    param(
        [string]$BasePath,
        [string]$TargetPath
    )
    $BaseFullPath = [System.IO.Path]::GetFullPath($BasePath)
    $TargetFullPath = [System.IO.Path]::GetFullPath($TargetPath)
    if (-not $BaseFullPath.EndsWith([System.IO.Path]::DirectorySeparatorChar.ToString())) {
        $BaseFullPath = $BaseFullPath + [System.IO.Path]::DirectorySeparatorChar
    }
    $BaseUri = New-Object System.Uri($BaseFullPath)
    $TargetUri = New-Object System.Uri($TargetFullPath)
    if ($BaseUri.Scheme -ne $TargetUri.Scheme) {
        return $TargetFullPath
    }
    $RelativeUri = $BaseUri.MakeRelativeUri($TargetUri)
    $RelativePath = [System.Uri]::UnescapeDataString($RelativeUri.ToString())
    if ($TargetUri.Scheme -eq "file") {
        $RelativePath = $RelativePath -replace '/', [System.IO.Path]::DirectorySeparatorChar
    }
    return $RelativePath
}

function Test-CapabilityFile {
    param(
        [string]$Capability,
        [string[]]$RelativePaths
    )
    if ((Get-CapabilityValue $Capability) -ne "conformant") {
        return
    }
    foreach ($RelativePath in $RelativePaths) {
        if (-not (Test-Path -LiteralPath (Join-Path $script:Root $RelativePath))) {
            Add-ErrorMessage "$Capability is conformant but required file is missing: $RelativePath"
        }
        $AgentGuide = Join-Path $script:Root "AGENTS.md"
        if ($Capability -notin @("task_router", "evolution_feedback") -and
            (Test-Path -LiteralPath $AgentGuide) -and
            -not ((Get-Content -LiteralPath $AgentGuide -Raw).Contains($RelativePath))) {
            Add-ErrorMessage "conformant recurring workflow is not routed from AGENTS.md: $Capability -> $RelativePath"
        }
    }
}

try {
    $script:Root = (Resolve-Path -LiteralPath $ProjectRoot -ErrorAction Stop).Path
}
catch {
    Write-Error "project root does not exist: $ProjectRoot"
    exit 2
}

$script:Manifest = Join-Path $script:Root ".agent-governance/manifest.yaml"
if (-not (Test-Path -LiteralPath $script:Manifest)) {
    Add-ErrorMessage "missing .agent-governance/manifest.yaml"
    Write-Output "Governance validation failed: $ErrorCount error(s), $WarningCount warning(s)."
    exit 1
}

$ProjectProfile = Get-ManifestValue "profile"
$ManifestStatus = Get-ManifestValue "status"
$script:Capabilities = Get-SectionMap "capabilities"
$Entrypoints = Get-SectionMap "entrypoints"

foreach ($Name in $Entrypoints.Keys) {
    $Target = $Entrypoints[$Name]
    if (-not (Test-Path -LiteralPath (Join-Path $script:Root $Target))) {
        Add-ErrorMessage "declared entrypoint does not exist: $Name -> $Target"
    }
}

if ($ProjectProfile -in @("product", "high-risk")) {
    foreach ($Capability in @("task_router", "evolution_feedback", "testing_policy", "git_workflow", "requirement_intake", "iteration_workflow", "change_control")) {
        if ((Get-CapabilityValue $Capability) -eq "not_applicable") {
            Add-ErrorMessage "$ProjectProfile profile cannot mark $Capability as not_applicable"
        }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $script:Root "docs/README.md"))) {
        if ($ManifestStatus -eq "conformant") {
            Add-ErrorMessage "product governance is missing documentation map: docs/README.md"
        }
        else {
            Add-WarningMessage "product governance is missing documentation map: docs/README.md"
        }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $script:Root "docs/sop/DOC-CHECK.md"))) {
        if ($ManifestStatus -eq "conformant") {
            Add-ErrorMessage "multi-layer product governance is missing docs/sop/DOC-CHECK.md"
        }
        else {
            Add-WarningMessage "multi-layer product governance is missing docs/sop/DOC-CHECK.md"
        }
    }
}

$IterationRoot = Join-Path $script:Root "docs/iterations"
$IterationRecords = @()
if (Test-Path -LiteralPath $IterationRoot) {
    $IterationRecords = Get-ChildItem -LiteralPath $IterationRoot -Filter "*.md" -File |
        Where-Object { $_.Name.ToLowerInvariant() -ne "readme.md" }
}

if ($IterationRecords.Count -gt 0 -and (Get-CapabilityValue "iteration_workflow") -eq "not_applicable") {
    Add-ErrorMessage "iteration records exist but iteration_workflow is marked not_applicable"
}

if ($ManifestStatus -in @("degraded", "adopting")) {
    Add-WarningMessage "manifest status is '$ManifestStatus': declared capabilities are not fully trustworthy yet; verify they reflect reality before relying on the governance state"
}

foreach ($Record in $IterationRecords) {
    if ((Test-CompletionClaim $Record.FullName) -and -not (Test-Evidence $Record.FullName)) {
        $Relative = ConvertTo-RelativePath $script:Root $Record.FullName
        Add-WarningMessage "iteration claims completion but records no validation evidence (command, test, or recorded result): $Relative"
    }
}

$Board = Join-Path $script:Root "docs/BOARD.md"
if (Test-Path -LiteralPath $Board) {
    $BoardText = Get-Content -LiteralPath $Board -Raw
    if ($BoardText -notmatch "(?i)derived[\s-]+operating[\s-]+view") {
        Add-WarningMessage "docs/BOARD.md exists but is not explicitly marked as a derived operating view"
    }
    if (-not $BoardText.Contains("Owner Doc")) {
        Add-WarningMessage "docs/BOARD.md exists but does not include an Owner Doc column or equivalent label"
    }
    if (-not $BoardText.Contains("Gate")) {
        Add-WarningMessage "docs/BOARD.md exists but does not include a Gate column or equivalent label"
    }
}

Test-CapabilityFile "task_router" @("AGENTS.md")
Test-CapabilityFile "evolution_feedback" @("EVOLUTION.md", "docs/sop/EVOLUTION-FEEDBACK.md")
Test-CapabilityFile "long_running_task" @("docs/sop/LONG-RUNNING-TASK.md")
$AgentGuide = Join-Path $script:Root "AGENTS.md"
if ((Get-CapabilityValue "evolution_feedback") -eq "conformant" -and
    (Test-Path -LiteralPath $AgentGuide) -and
    -not ((Get-Content -LiteralPath $AgentGuide -Raw).Contains("docs/sop/EVOLUTION-FEEDBACK.md"))) {
    Add-ErrorMessage "conformant evolution_feedback is not routed from AGENTS.md: docs/sop/EVOLUTION-FEEDBACK.md"
}
Test-CapabilityFile "testing_policy" @("docs/sop/TESTING.md")
Test-CapabilityFile "git_workflow" @("docs/sop/GIT-WORKFLOW.md")
Test-CapabilityFile "requirement_intake" @("docs/sop/REQUIREMENT-INTAKE.md")
Test-CapabilityFile "iteration_workflow" @("docs/sop/START-ITERATION.md", "docs/sop/ITERATION-WORKFLOW.md")
Test-CapabilityFile "change_control" @("docs/sop/CHANGE-CONTROL.md")
Test-CapabilityFile "decision_records" @("docs/decisions/README.md")
Test-CapabilityFile "release_workflow" @("docs/sop/RELEASE.md")

if ((Get-CapabilityValue "iteration_workflow") -eq "conformant") {
    $IterationTemplate = Join-Path $script:Root "docs/iterations/TEMPLATE.md"
    if (-not (Test-Path -LiteralPath $IterationTemplate)) {
        Add-ErrorMessage "conformant iteration workflow is missing published-baseline template: docs/iterations/TEMPLATE.md"
    }
    elseif (-not (Get-Content -LiteralPath $IterationTemplate -Raw).Contains("Published plan date")) {
        Add-ErrorMessage "iteration template is missing published baseline metadata: Published plan date"
    }

    $AgentGuideText = Get-Content -LiteralPath (Join-Path $script:Root "AGENTS.md") -Raw
    if (-not $AgentGuideText.Contains("published baseline")) {
        Add-ErrorMessage "AGENTS.md does not expose the published iteration baseline rule"
    }

    $StartIterationText = Get-Content -LiteralPath (Join-Path $script:Root "docs/sop/START-ITERATION.md") -Raw
    if (-not $StartIterationText.Contains("Inventory Existing Iterations")) {
        Add-ErrorMessage "START-ITERATION does not require non-terminal iteration inventory"
    }
    if (-not $StartIterationText.Contains("runnable, testable deliverable")) {
        Add-ErrorMessage "START-ITERATION does not require a runnable, testable deliverable"
    }

    $ChangeControlText = Get-Content -LiteralPath (Join-Path $script:Root "docs/sop/CHANGE-CONTROL.md") -Raw
    if (-not $ChangeControlText.Contains("Never overwrite a published iteration baseline")) {
        Add-ErrorMessage "CHANGE-CONTROL does not preserve published iteration baselines"
    }
}

if ((Get-CapabilityValue "requirement_intake") -eq "conformant") {
    $RequirementIntakeText = Get-Content -LiteralPath (Join-Path $script:Root "docs/sop/REQUIREMENT-INTAKE.md") -Raw
    foreach ($RequiredText in @("Given/When/Then", "Required Reads", "Decision links", "user-facing documentation")) {
        if (-not $RequirementIntakeText.Contains($RequiredText)) {
            Add-ErrorMessage "REQUIREMENT-INTAKE is missing current ready-story rule: $RequiredText"
        }
    }
}

if ((Get-CapabilityValue "long_running_task") -eq "conformant") {
    $LongRunningTaskText = Get-Content -LiteralPath (Join-Path $script:Root "docs/sop/LONG-RUNNING-TASK.md") -Raw
    foreach ($RequiredText in @("Startup Contract", "Consolidated Confirmation", "Recovery or resume instruction", "Completion Gate")) {
        if (-not $LongRunningTaskText.Contains($RequiredText)) {
            Add-ErrorMessage "LONG-RUNNING-TASK is missing required contract section: $RequiredText"
        }
    }
    if (-not (Get-Content -LiteralPath (Join-Path $script:Root "AGENTS.md") -Raw).Contains("docs/sop/LONG-RUNNING-TASK.md")) {
        Add-ErrorMessage "conformant long-running task workflow is not routed from AGENTS.md"
    }
}

if ((Test-Path -LiteralPath $AgentGuide) -and $ProjectProfile -in @("product", "high-risk") -and (Get-CapabilityValue "task_router") -eq "conformant") {
    $GuideText = Get-Content -LiteralPath $AgentGuide -Raw
    foreach ($Section in @("Hard Constraints", "Coding Behavior", "Git Rules", "Task Router", "Session End Checklist")) {
        if (-not $GuideText.Contains($Section)) {
            Add-ErrorMessage "AGENTS.md is missing required section: $Section"
        }
    }
    if ($GuideText -notmatch 'type\(scope\):\s+description.*\[model:\s*<model-name>\]') {
        Add-ErrorMessage "AGENTS.md Git Rules must include the Agent commit model tag format: type(scope): description (#story-id) [model:<model-name>]"
    }
    if ($GuideText -notmatch '\[model:\s*<model-name>\].*(required|mandatory)|required.*\[model:\s*<model-name>\]|mandatory.*\[model:\s*<model-name>\]') {
        Add-ErrorMessage "AGENTS.md Git Rules must say the model tag is required for Agent-authored or Agent-assisted commits"
    }
}

$MarkdownFiles = @()
foreach ($Path in @("AGENTS.md", "EVOLUTION.md", "README.md")) {
    $FullPath = Join-Path $script:Root $Path
    if (Test-Path -LiteralPath $FullPath) {
        $MarkdownFiles += Get-Item -LiteralPath $FullPath
    }
}
$DocsRoot = Join-Path $script:Root "docs"
if (Test-Path -LiteralPath $DocsRoot) {
    $MarkdownFiles += Get-ChildItem -LiteralPath $DocsRoot -Recurse -Filter "*.md" -File
}

foreach ($Source in $MarkdownFiles) {
    $Text = Get-Content -LiteralPath $Source.FullName -Raw
    foreach ($Match in [regex]::Matches($Text, "\[[^\]]+\]\(([^)#]+\.md)(?:#[^)]+)?\)")) {
        $Link = $Match.Groups[1].Value
        if ($Link -match "://" -or $Link.Contains("<")) {
            continue
        }
        $Target = Join-Path $Source.DirectoryName $Link
        if (-not (Test-Path -LiteralPath $Target)) {
            $RelativeSource = ConvertTo-RelativePath $script:Root $Source.FullName
            Add-ErrorMessage "broken Markdown link: $RelativeSource -> $Link"
        }
    }
}

$ActiveFiles = @()
if (Test-Path -LiteralPath $AgentGuide) {
    $ActiveFiles += Get-Item -LiteralPath $AgentGuide
}
foreach ($Folder in @("docs/reference", "docs/sop")) {
    $FullFolder = Join-Path $script:Root $Folder
    if (Test-Path -LiteralPath $FullFolder) {
        $ActiveFiles += Get-ChildItem -LiteralPath $FullFolder -Recurse -Filter "*.md" -File
    }
}

foreach ($Source in $ActiveFiles) {
    $Text = Get-Content -LiteralPath $Source.FullName -Raw
    $TextWithoutCode = [regex]::Replace($Text, '(?s)```.*?```', "")
    $Refs = [regex]::Matches($TextWithoutCode, '`(src/[A-Za-z0-9_./@-]+\.[A-Za-z0-9]+)`') |
        ForEach-Object { $_.Groups[1].Value } |
        Sort-Object -Unique
    foreach ($RelativePath in $Refs) {
        if ($RelativePath -match 'MyPage|Example|<|>') {
            continue
        }
        if (-not (Test-Path -LiteralPath (Join-Path $script:Root $RelativePath))) {
            $RelativeSource = ConvertTo-RelativePath $script:Root $Source.FullName
            Add-ErrorMessage "missing explicit source path referenced by active governance: $RelativeSource -> $RelativePath"
        }
    }
}

if ($ErrorCount -gt 0) {
    Write-Output "Governance validation failed: $ErrorCount error(s), $WarningCount warning(s)."
    exit 1
}

Write-Output "Governance validation passed: $WarningCount warning(s)."
