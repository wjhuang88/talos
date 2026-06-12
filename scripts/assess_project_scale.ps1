param(
    [Parameter(Position = 0)]
    [string]$ProjectRoot = "."
)

try {
    $Root = (Resolve-Path -LiteralPath $ProjectRoot -ErrorAction Stop).Path
}
catch {
    Write-Error "project root does not exist: $ProjectRoot"
    exit 2
}

function Get-ProjectFiles {
    param([string[]]$Extensions)
    Get-ChildItem -LiteralPath $Root -Recurse -File -ErrorAction SilentlyContinue |
        Where-Object {
            $_.FullName -notmatch '[\\/](\.git|\.codex|\.opencode|\.sisyphus|node_modules|target|dist|build)[\\/]' -and
            $Extensions -contains $_.Extension.ToLowerInvariant()
        }
}

function Test-AnyPath {
    param([string[]]$Paths)
    foreach ($Path in $Paths) {
        if (Test-Path -LiteralPath (Join-Path $Root $Path)) {
            return $true
        }
    }
    return $false
}

function Test-ProjectText {
    param([string]$Pattern)
    $Extensions = @(".js", ".ts", ".tsx", ".py", ".go", ".rs", ".java", ".rb", ".php", ".cs", ".json", ".toml")
    foreach ($File in Get-ProjectFiles $Extensions) {
        $Text = Get-Content -LiteralPath $File.FullName -Raw -ErrorAction SilentlyContinue
        if ($Text -match $Pattern) {
            return $true
        }
    }
    return $false
}

$SourceExtensions = @(".js", ".ts", ".tsx", ".py", ".go", ".rs", ".java", ".rb", ".php", ".cs")
$SourceFiles = @(Get-ProjectFiles $SourceExtensions).Count
$PackageFiles = @(Get-ChildItem -LiteralPath $Root -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object {
        $_.FullName -notmatch '[\\/](\.git|\.codex|\.opencode|\.sisyphus|node_modules|target|dist|build)[\\/]' -and
        ($_.Name -in @("package.json", "Cargo.toml", "go.mod", "pyproject.toml", "pom.xml") -or
        $_.Name -like "*.csproj")
    }).Count
$CiRoot = Join-Path $Root ".github/workflows"
$CiWorkflows = 0
if (Test-Path -LiteralPath $CiRoot) {
    $CiWorkflows = @(Get-ChildItem -LiteralPath $CiRoot -File -ErrorAction SilentlyContinue).Count
}

$ReleaseSignal = Test-AnyPath @("releases", ".github/workflows/release.yml", "CHANGELOG.md")
try {
    $Tags = & git -C $Root tag --list "v[0-9]*" 2>$null
    if ($Tags) {
        $ReleaseSignal = $true
    }
}
catch {}

$MigrationSignal = Test-AnyPath @("db/migrations", "migrations", "prisma/migrations", "database/migrations")
$AuthSignal = Test-ProjectText "(^|[^A-Za-z])(auth|authentication|authorization|permission|permissions|rbac|oauth|jwt|session|token)([^A-Za-z]|$)|auth[-_]"
$ExternalSignal = Test-ProjectText "webhook|payment|stripe|email|smtp|queue|kafka|sqs|llm|tool executor|third[-_ ]party|api client"

$PlanningDocs = 0
foreach ($Path in @("docs/backlog", "docs/iterations", "docs/roadmap", "docs/decisions", "docs/proposals")) {
    if (Test-Path -LiteralPath (Join-Path $Root $Path)) {
        $PlanningDocs++
    }
}

$WorktreeCount = 0
try {
    $Worktrees = & git -C $Root worktree list 2>$null
    if ($Worktrees) {
        $WorktreeCount = @($Worktrees).Count
    }
}
catch {}

$ProductSignals = 0
if ($SourceFiles -gt 80) { $ProductSignals++ }
if ($PackageFiles -gt 2) { $ProductSignals++ }
if ($CiWorkflows -gt 0) { $ProductSignals++ }
if ($PlanningDocs -ge 2) { $ProductSignals++ }
if ($ReleaseSignal) { $ProductSignals++ }

$HighRiskSignals = 0
if ($MigrationSignal -and $ReleaseSignal) { $HighRiskSignals++ }
if ($AuthSignal) { $HighRiskSignals++ }
if ($ExternalSignal) { $HighRiskSignals++ }

$Profile = "minimal"
if ($ProductSignals -ge 2) { $Profile = "product" }
if ($HighRiskSignals -gt 0) { $Profile = "high-risk" }

$BranchMode = "simple"
if ($ReleaseSignal) { $BranchMode = "release-managed" }

$WorktreeMode = "none"
if ($BranchMode -eq "release-managed" -or $Profile -eq "high-risk") {
    $WorktreeMode = "on-demand"
}
if ($WorktreeCount -gt 1) {
    $WorktreeMode = "required"
}

@"
recommended_profile: $Profile
recommended_branch_mode: $BranchMode
recommended_worktree_mode: $WorktreeMode

signals:
  source_files: $SourceFiles
  package_files: $PackageFiles
  ci_workflows: $CiWorkflows
  release_signal: $($ReleaseSignal.ToString().ToLowerInvariant())
  migration_signal: $($MigrationSignal.ToString().ToLowerInvariant())
  auth_signal: $($AuthSignal.ToString().ToLowerInvariant())
  external_signal: $($ExternalSignal.ToString().ToLowerInvariant())
  planning_doc_groups: $PlanningDocs
  git_worktrees: $WorktreeCount

scores:
  product_signals: $ProductSignals
  high_risk_signals: $HighRiskSignals
"@
