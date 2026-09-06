$ErrorActionPreference='Stop'
$script=Join-Path $PSScriptRoot 'dependency_candidates.ps1'
$source=Join-Path $PSScriptRoot '../docs/reference/dependency-audit-2026-09-06.json'
$report=(& $script -ObservationPath $source)|ConvertFrom-Json
if($report.candidates.Count -ne 34) { throw 'live observation candidate coverage changed unexpectedly' }
foreach($name in @('rmcp','wasmtime','gix','zstd','libc')) {
    $row=@($report.candidates|Where-Object name -eq $name)
    if($row.Count -ne 1 -or $row[0].disposition -ne 'isolated-proposal' -or $row[0].authorization -ne 'none') { throw "candidate isolation lost: $name" }
}
$path=[System.IO.Path]::GetTempFileName()
try {
    $observation=Get-Content -Raw -LiteralPath $source|ConvertFrom-Json
    foreach($row in $observation.report.dependencies) { $row.classification=@('registry-unavailable');$row.latest_stable=$null }
    $observation|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $path -Encoding utf8NoBOM
    $changed=(& $script -ObservationPath $path)|ConvertFrom-Json
    if($changed.candidates.Count -ne 56 -or @($changed.candidates|Where-Object disposition -ne 'investigate').Count) { throw 'missing registry evidence silently generated upgrade targets' }
    'candidate handoff: PASS (34 candidates, isolation, no authorization, unavailable evidence mutation)'
} finally {Remove-Item -LiteralPath $path}
