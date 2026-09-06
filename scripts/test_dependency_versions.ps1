$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'dependency_versions.ps1')
$shared=Join-Path $PSScriptRoot '../tests/fixtures/dependency-version-cases.tsv'
foreach ($line in Get-Content -LiteralPath $shared) {
    $row=$line -split "`t"
    if ($row.Count -ne 3) { throw 'bad shared version fixture' }
    $got=Get-AuditDistance $row[0] $row[1]
    if ($got -cne $row[2]) { throw "shared version fixture: $line, got $got" }
}
$cases=@(
    @('1.2.3','1.2.3+build.9','current'),
    @('1.2.3','1.2.4','patch'),
    @('1.9.0','1.10.0','minor'),
    @('1.2.3','2.0.0','major'),
    @('0.22.1','0.23.0','pre-1.0-breaking-minor'),
    @('0.0.1','0.0.2','pre-1.0-breaking-patch'),
    @('2.0.0','1.9.0','ahead-of-stable'),
    @('1.0.0-alpha.2','1.0.0','prerelease'),
    @('01.0.0','2.0.0','unknown'),
    @('1.0.0-alpha.01','2.0.0','unknown'),
    @('1.2','2.0.0','unknown'),
    @('999999999999999999999.0.0','1000000000000000000000.0.0','major')
)
foreach ($row in $cases) {
    $actual=Get-AuditDistance $row[0] $row[1]
    if ($actual -cne $row[2]) { throw "$($row[0]) -> $($row[1]): $actual, expected $($row[2])" }
}
$response=[pscustomobject]@{versions=@(
    @{num='1.9.0';yanked=$false}, @{num='1.10.0';yanked=$false},
    @{num='2.0.0-rc.1';yanked=$false}, @{num='3.0.0';yanked=$true}
)}
if ((Get-AuditLatestStable $response) -ne '1.10.0') { throw 'latest stable selection failed' }
$response.versions += @{num='1.11.0';yanked=$false}
if ((Get-AuditLatestStable $response) -ne '1.11.0') { throw 'upstream mutation ignored' }
if ((Compare-AuditVersion (ConvertTo-AuditVersion '1.0.0-rc.9') (ConvertTo-AuditVersion '1.0.0-rc.10')) -ge 0) { throw 'numeric prerelease comparison failed' }
$rejected=$false
try { $null=Get-AuditLatestStable @{versions=@(@{num='1.0.0'})} } catch { $rejected=$true }
if (-not $rejected) { throw 'missing yanked state silently accepted' }
'dependency versions: PASS (12 distance cases, stable filtering, mutation, prerelease order, malformed registry)'
