$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'dependency_baseline.ps1')
function Make-Row($Versions) {
  [pscustomobject]@{name='example';source='registry+fixture';kind=$null;target=$null;manifest_requirements=@('^1');used_by=@('a');resolved_versions=@($Versions);classification=@('minor')}
}
$baseline=[pscustomobject]@{
  schema='talos.dependency-baseline/v1';source_commit=('a'*40);accepted_at='2026-09-06T00:00:00Z'
  validation_evidence='fixture only; not a repository acceptance'
  dependencies=@((Make-Row @('1.0.0','1.1.0')))
}
$original=$baseline|ConvertTo-Json -Depth 12 -Compress
$report=@{dependencies=@((Make-Row @('1.1.0','1.0.0')))}
Compare-AuditBaseline $report $baseline
if($report.baseline.drift) { throw 'ordering alone caused baseline drift' }
if($report.dependencies[0].accepted_versions.Count -ne 2) { throw 'accepted resolutions collapsed' }
$report=@{dependencies=@((Make-Row @('1.2.0')))}
Compare-AuditBaseline $report $baseline
if(-not $report.baseline.drift -or $report.dependencies[0].classification -notcontains 'minor' -or $report.dependencies[0].classification -notcontains 'baseline-drift') { throw 'baseline and upstream drift not independent' }
$report=@{dependencies=@()}
Compare-AuditBaseline $report $baseline
if($report.baseline.removed_dependencies.Count -ne 1) { throw 'removed dependency lost' }
$changed=Make-Row @('1.0.0','1.1.0'); $changed.manifest_requirements=@('>=1')
$report=@{dependencies=@($changed)}
Compare-AuditBaseline $report $baseline
if(-not $report.baseline.drift -or $report.baseline.removed_dependencies.Count -ne 1) { throw 'requirement drift missed' }
$changed=Make-Row @('1.0.0','1.1.0'); $changed.used_by=@('b')
$report=@{dependencies=@($changed)}
Compare-AuditBaseline $report $baseline
if(-not $report.baseline.drift) { throw 'consumer drift missed' }
if(($baseline|ConvertTo-Json -Depth 12 -Compress) -cne $original) { throw 'audit mutated accepted baseline' }
$baseline.dependencies += $baseline.dependencies[0]
$rejected=$false
try { Compare-AuditBaseline @{dependencies=@()} $baseline } catch { $rejected=$true }
if(-not $rejected) { throw 'duplicate baseline accepted' }
'baseline comparison: PASS (order, multiple resolutions, independent drift, deletion, requirement, users, immutability, duplicates)'

# Exercise both actual entrypoints, rather than only the comparison function.
$path=[System.IO.Path]::GetTempFileName()
try {
  $metadata=Join-Path $PSScriptRoot '../tests/fixtures/dependency-live-probe.json'
  $audit=Join-Path $PSScriptRoot 'dependency_audit.ps1'
  $bashAudit=Join-Path $PSScriptRoot 'dependency_audit.sh'
  $initial=(& $audit -MetadataPath $metadata -Format json)|ConvertFrom-Json
  $snapshot=[pscustomobject]@{
    schema='talos.dependency-baseline/v1';source_commit=('b'*40);accepted_at='2026-09-06T00:00:00Z'
    validation_evidence='synthetic entrypoint parity test';dependencies=$initial.dependencies
  }
  foreach($scenario in @('unchanged','version','requirement','removed','multiple-removed')) {
    if($scenario -eq 'version') { $snapshot.dependencies[0].resolved_versions=@('1.0.149','1.0.148') }
    if($scenario -eq 'requirement') { $snapshot.dependencies[0].manifest_requirements=@('>=1') }
    if($scenario -eq 'removed') { $snapshot.dependencies[0].name='no-longer-used' }
    if($scenario -eq 'multiple-removed') {
      $extra=$snapshot.dependencies[0]|ConvertTo-Json -Depth 12|ConvertFrom-Json
      $extra.name='a-removed-dependency'
      $snapshot.dependencies=@($snapshot.dependencies)+@($extra)
    }
    $snapshot|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $path -Encoding utf8NoBOM
    $before=Get-FileHash -LiteralPath $path
    $ps=(& $audit -MetadataPath $metadata -BaselinePath $path -Format json)|ConvertFrom-Json
    $raw=& bash $bashAudit --metadata $metadata --baseline $path --format json
    if($LASTEXITCODE -ne 0) { throw "Unix baseline audit failed: $scenario" }
    $unix=$raw|ConvertFrom-Json
    if(($ps|ConvertTo-Json -Depth 20 -Compress) -cne ($unix|ConvertTo-Json -Depth 20 -Compress)) { throw "baseline parity failed: $scenario" }
    foreach($format in @('table','markdown')) {
      $psText=(& $audit -MetadataPath $metadata -BaselinePath $path -Format $format) -join "`n"
      $unixText=(& bash $bashAudit --metadata $metadata --baseline $path --format $format) -join "`n"
      if($LASTEXITCODE -ne 0 -or $psText -cne $unixText) { throw "render parity failed: $scenario/$format" }
    }
    if((Get-FileHash -LiteralPath $path).Hash -ne $before.Hash) { throw 'entrypoint mutated baseline' }
  }
  'baseline entrypoint parity: PASS (unchanged, versions, requirement, removed, multiple removed, immutable file)'
} finally { Remove-Item -LiteralPath $path }
