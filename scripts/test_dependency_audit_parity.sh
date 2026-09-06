#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
bash_json=$("$root/scripts/dependency_audit.sh" --format json)
ps_json=$(pwsh -NoProfile -File "$root/scripts/dependency_audit.ps1" -Format json)
export bash_json ps_json
pwsh -NoProfile -Command '$b=$env:bash_json|ConvertFrom-Json;$p=$env:ps_json|ConvertFrom-Json;if($b.schema -ne $p.schema){throw "schema mismatch"};if($b.status -ne $p.status){throw "status mismatch"};$bn=@($b.dependencies|% name|Sort-Object -Unique);$pn=@($p.dependencies|% name|Sort-Object -Unique);if((Compare-Object $bn $pn)){throw "dependency identity mismatch"};"dependency audit parity: PASS ($($bn.Count) names)"'
