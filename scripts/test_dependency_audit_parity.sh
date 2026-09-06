#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
bash_json=$("$root/scripts/dependency_audit.sh" --format json)
ps_json=$(pwsh -NoProfile -File "$root/scripts/dependency_audit.ps1" -Format json)
export bash_json ps_json
pwsh -NoProfile -Command '
$ErrorActionPreference="Stop"
$b=$env:bash_json|ConvertFrom-Json
$p=$env:ps_json|ConvertFrom-Json
if($b.schema -cne $p.schema -or $b.status -cne $p.status -or $b.registry -cne $p.registry){throw "envelope mismatch"}
function Records($report) {
  @($report.dependencies | ForEach-Object {
    # Compare every emitted field: a projection silently ignores new contract fields.
    $_ | ConvertTo-Json -Depth 20 -Compress
  } | Sort-Object)
}
$br=Records $b;$pr=Records $p
if(-not $br.Count -or -not $pr.Count){throw "empty report"}
$diff=@(Compare-Object $br $pr)
if($diff.Count){$diff|Format-List;throw "dependency record mismatch"}
"dependency audit collection parity: PASS ($($br.Count) complete records)"
'
