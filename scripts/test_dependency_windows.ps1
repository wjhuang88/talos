param([switch]$Workspace)
$ErrorActionPreference = 'Stop'
$audit = Join-Path $PSScriptRoot 'dependency_audit.ps1'
$metadata = Join-Path $PSScriptRoot '../tests/fixtures/dependency-live-probe.json'
$temporary = Join-Path ([System.IO.Path]::GetTempPath()) ('talos-dependency-' + [guid]::NewGuid())
$null = New-Item -ItemType Directory -Path $temporary
try {
    & (Join-Path $PSScriptRoot 'test_dependency_versions.ps1')
    $registry = Join-Path $temporary 'registry.json'
    $baseline = Join-Path $temporary 'baseline.json'
    $initial = (& $audit -MetadataPath $metadata -Format json) | ConvertFrom-Json
    if ($initial.dependencies.Count -ne 1 -or
        $initial.dependencies[0].name -cne 'serde_json' -or
        $initial.dependencies[0].resolved_versions[0] -cne '1.0.150' -or
        $initial.registry -cne 'not-queried') { throw 'fixture collection failed' }
    & $audit -MetadataPath $metadata -Snapshot -Format json -SourceCommit ('a' * 40) `
        -AcceptedAt '2026-09-07T00:00:00Z' -ValidationEvidence 'synthetic test only' |
        Set-Content -LiteralPath $baseline -Encoding utf8
    $before = (Get-FileHash -LiteralPath $baseline).Hash
    @{serde_json=@{versions=@(@{num='1.0.151';yanked=$false},
        @{num='2.0.0';yanked=$true}, @{num='3.0.0-rc.1';yanked=$false})}} |
        ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $registry -Encoding utf8
    $report = (& $audit -MetadataPath $metadata -RegistryPath $registry -BaselinePath $baseline -Format json) | ConvertFrom-Json
    if ($report.dependencies[0].latest_stable -cne '1.0.151' -or
        $report.dependencies[0].classification -notcontains 'patch' -or $report.baseline.drift) {
        throw 'latest stable / baseline comparison failed'
    }
    foreach ($format in @('table','markdown')) {
        $rendered = (& $audit -MetadataPath $metadata -RegistryPath $registry -Format $format) -join "`n"
        if (-not $rendered.Contains('serde_json') -or -not $rendered.Contains('1.0.151') -or
            -not $rendered.Contains('patch')) { throw "$format rendering lost dependency facts" }
    }
    '{}' | Set-Content -LiteralPath $registry -Encoding utf8
    $unavailable = (& $audit -MetadataPath $metadata -RegistryPath $registry -Format json) | ConvertFrom-Json
    if ($unavailable.status -cne 'partial' -or
        $unavailable.dependencies[0].classification -notcontains 'registry-unavailable') {
        throw 'missing registry evidence was not reported as unavailable'
    }
    if ((Get-FileHash -LiteralPath $baseline).Hash -cne $before) { throw 'audit mutated accepted baseline' }
    if ($Workspace) {
        # CI executes after locked workspace tests have populated Cargo's cache.
        $workspaceReport = (& $audit -Format json) | ConvertFrom-Json
        if ($workspaceReport.schema -cne 'talos.dependency-audit/v1' -or
            $workspaceReport.dependencies.Count -eq 0 -or
            @($workspaceReport.dependencies | Where-Object { $_.used_by.Count -eq 0 }).Count -ne 0) {
            throw 'real locked workspace collection failed'
        }
        "Windows workspace audit: $($workspaceReport.dependencies.Count) dependency identities"
    }
    'PowerShell dependency frontend: PASS (collection, stable filtering, baseline, rendering, unavailable registry, immutability)'
} finally {
    Remove-Item -LiteralPath $temporary -Recurse -Force
}
