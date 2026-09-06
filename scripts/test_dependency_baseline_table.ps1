$ErrorActionPreference='Stop'
$unix=(& bash (Join-Path $PSScriptRoot 'dependency_baseline_table.sh')) -join "`n"
if($LASTEXITCODE -ne 0) { throw 'Unix baseline rendering failed' }
$ps=(& (Join-Path $PSScriptRoot 'dependency_baseline_table.ps1')) -join "`n"
if($ps -cne $unix) { throw 'baseline Markdown frontend mismatch' }
$document=Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot '../docs/reference/DEPENDENCY-BASELINE.md')
$blocks=[regex]::Matches($document,'(?s)<!-- dependency-baseline:begin -->.*?<!-- dependency-baseline:end -->')
if($blocks.Count -ne 1 -or $blocks[0].Value.Replace("`r`n","`n") -cne $ps) { throw 'accepted Markdown table drifted from generated snapshot' }
'baseline Markdown: PASS (frontend parity and committed table equality)'
