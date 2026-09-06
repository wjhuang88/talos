param([string]$SnapshotPath=(Join-Path $PSScriptRoot '../docs/reference/dependency-baseline.json'))
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'dependency_render.ps1')
$snapshot=Get-Content -Raw -LiteralPath $SnapshotPath|ConvertFrom-Json
if($snapshot.schema -cne 'talos.dependency-baseline/v1' -or $snapshot.dependencies -isnot [array]) { throw 'expected accepted baseline snapshot' }
'<!-- dependency-baseline:begin -->'
'| Name | Manifest | Accepted resolutions | Direct consumers | Kind | Target |'
'|---|---|---|---|---|---|'
foreach($row in $snapshot.dependencies) {
    $cells=@((Format-AuditCell $row.name 'markdown'))
    foreach($field in @('manifest_requirements','resolved_versions','used_by')) {
        $cells+=(@($row.$field|ForEach-Object {Format-AuditCell $_ 'markdown'}) -join ', ')
    }
    $cells+=$(if($row.kind){Format-AuditCell $row.kind 'markdown'}else{'normal'})
    $cells+=$(if($row.target){Format-AuditCell $row.target 'markdown'}else{'all'})
    '| '+($cells -join ' | ')+' |'
}
'<!-- dependency-baseline:end -->'
