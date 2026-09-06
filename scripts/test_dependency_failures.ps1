$ErrorActionPreference='Stop'
$audit=Join-Path $PSScriptRoot 'dependency_audit.ps1'
$unixAudit=Join-Path $PSScriptRoot 'dependency_audit.sh'
$metadata=Join-Path $PSScriptRoot '../tests/fixtures/dependency-live-probe.json'
$path=[System.IO.Path]::GetTempFileName()
try {
    $cases=@(
        @{name='missing';response=@{};expected='registry-unavailable';status='partial'},
        @{name='transport';response=@{serde_json=$null};expected='registry-unavailable';status='partial'},
        @{name='empty';response=@{serde_json=@{}};expected='unknown';status='partial'},
        @{name='malformed-version';response=@{serde_json=@{versions=@(@{num='1.0';yanked=$false})}};expected='unknown';status='partial'},
        @{name='malformed-yanked';response=@{serde_json=@{versions=@(@{num='1.0.151';yanked='false'})}};expected='unknown';status='partial'},
        @{name='prerelease-only';response=@{serde_json=@{versions=@(@{num='2.0.0-rc.1';yanked=$false})}};expected='unknown';status='partial'},
        @{name='yanked-only';response=@{serde_json=@{versions=@(@{num='1.0.151';yanked=$true})}};expected='unknown';status='partial'},
        @{name='current';response=@{serde_json=@{versions=@(@{num='1.0.150';yanked=$false})}};expected='current';status='audited'},
        @{name='patch';response=@{serde_json=@{versions=@(@{num='1.0.151';yanked=$false})}};expected='patch';status='audited'},
        @{name='major';response=@{serde_json=@{versions=@(@{num='2.0.0';yanked=$false})}};expected='major';status='audited'}
    )
    foreach($case in $cases) {
        $case.response|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $path -Encoding utf8NoBOM
        $ps=(& $audit -MetadataPath $metadata -RegistryPath $path -Format json)|ConvertFrom-Json
        $unix=(& bash $unixAudit --metadata $metadata --registry $path --format json)|ConvertFrom-Json
        if($LASTEXITCODE -ne 0) { throw "Unix failed: $($case.name)" }
        if(($ps|ConvertTo-Json -Depth 20 -Compress) -cne ($unix|ConvertTo-Json -Depth 20 -Compress)) { throw "failure parity: $($case.name)" }
        if(($ps.dependencies[0].classification -join ',') -cne $case.expected -or $ps.status -cne $case.status) { throw "wrong classification: $($case.name)" }
        if($ps.dependencies[0].signals.security -cne 'unknown') { throw 'version lookup incorrectly attests security' }
        foreach($format in @('table','markdown')) {
            $a=(& $audit -MetadataPath $metadata -RegistryPath $path -Format $format) -join "`n"
            $b=(& bash $unixAudit --metadata $metadata --registry $path --format $format) -join "`n"
            if($LASTEXITCODE -ne 0 -or $a -cne $b) { throw "failure rendering parity: $($case.name)/$format" }
        }
    }
    'registry entrypoint failure parity: PASS (10 scenarios, JSON/table/Markdown, explicit unknown security)'
} finally { Remove-Item -LiteralPath $path }
