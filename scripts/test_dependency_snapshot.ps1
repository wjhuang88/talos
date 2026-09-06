$ErrorActionPreference='Stop'
$metadata=Join-Path $PSScriptRoot '../tests/fixtures/dependency-live-probe.json'
$psAudit=Join-Path $PSScriptRoot 'dependency_audit.ps1'
$unixAudit=Join-Path $PSScriptRoot 'dependency_audit.sh'
$sha='c'*40
$time='2026-09-06T00:00:00Z'
$evidence='Fixture only: "quoted" evidence, slash \ and Unicode 验证'
$ps=(& $psAudit -MetadataPath $metadata -Format json -Snapshot -SourceCommit $sha -AcceptedAt $time -ValidationEvidence $evidence)|ConvertFrom-Json
$unix=(& bash $unixAudit --metadata $metadata --format json --snapshot --source-commit $sha --accepted-at $time --validation-evidence $evidence)|ConvertFrom-Json
if($LASTEXITCODE -ne 0 -or ($ps|ConvertTo-Json -Depth 20 -Compress) -cne ($unix|ConvertTo-Json -Depth 20 -Compress)) { throw 'snapshot generation parity failed' }
if($ps.dependencies.Count -ne 1 -or $ps.dependencies[0].resolved_versions[0] -ne '1.0.150' -or $ps.validation_evidence -cne $evidence) { throw 'snapshot lost data' }
$rejected=$false
try { $null=& $psAudit -MetadataPath $metadata -Format json -Snapshot } catch { $rejected=$true }
if(-not $rejected) { throw 'PowerShell generated provenance-free baseline' }
$null=& bash $unixAudit --metadata $metadata --format json --snapshot 2>$null
if($LASTEXITCODE -eq 0) { throw 'Unix generated provenance-free baseline' }
$rejected=$false
try { $null=& $psAudit -MetadataPath $metadata -Format json -Snapshot -Live } catch { $rejected=$true }
if(-not $rejected) { throw 'PowerShell allowed live lookup to bless a baseline' }
$null=& bash $unixAudit --metadata $metadata --format json --snapshot --live 2>$null
if($LASTEXITCODE -eq 0) { throw 'Unix allowed live lookup to bless a baseline' }
'snapshot generation: PASS (full parity, provenance escaping, missing provenance rejection, no live acceptance)'
