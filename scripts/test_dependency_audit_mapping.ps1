$ErrorActionPreference = 'Stop'
$audit = Join-Path $PSScriptRoot 'dependency_audit.ps1'
$source = 'registry+https://github.com/rust-lang/crates.io-index'
$fixture = [System.IO.Path]::GetTempFileName()
$registry = [System.IO.Path]::GetTempFileName()
try {
    $inputData = @{
        version = 1
        workspace_members = @('workspace')
        packages = @(
            @{id='workspace'; name='consumer'; source=$null; dependencies=@(
                @{name='example'; source=$source; req='^1'; rename='old-example'; kind=$null; target=$null},
                @{name='example'; source=$source; req='^2'; rename='new-example'; kind='build'; target='cfg(windows)'}
            )},
            @{id='old'; name='example'; source=$source; version='1.9.0'},
            @{id='new'; name='example'; source=$source; version='2.10.0'},
            @{id='transitive'; name='example'; source=$source; version='9.0.0'}
        )
        resolve = @{nodes=@(
            @{id='workspace'; deps=@(
                @{name='old_example'; pkg='old'; dep_kinds=@(@{kind=$null; target=$null})},
                @{name='new_example'; pkg='new'; dep_kinds=@(@{kind='build'; target='cfg(windows)'})}
            )}
        )}
    }
    function Run-Fixture {
        $inputData | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $fixture -Encoding utf8
        (& $audit -Format json -MetadataPath $fixture) | ConvertFrom-Json
    }
    $result = Run-Fixture
    if ($result.dependencies.Count -ne 2) { throw 'declarations were collapsed' }
    $old = @($result.dependencies | Where-Object { $_.manifest_requirements -contains '^1' })[0]
    $new = @($result.dependencies | Where-Object { $_.manifest_requirements -contains '^2' })[0]
    if (($old.resolved_versions -join ',') -ne '1.9.0') { throw 'old alias received wrong resolution' }
    if (($new.resolved_versions -join ',') -ne '2.10.0') { throw 'build/target alias received wrong resolution' }
    $inputData.packages[2].version = '2.11.0'
    $mutated = Run-Fixture
    $new = @($mutated.dependencies | Where-Object { $_.manifest_requirements -contains '^2' })[0]
    if (($new.resolved_versions -join ',') -ne '2.11.0') { throw 'resolution mutation did not reach output' }
    @{example=@{versions=@(
        @{num='1.9.0';yanked=$false}, @{num='2.11.0';yanked=$true},
        @{num='2.12.0';yanked=$false}, @{num='3.0.0-rc.1';yanked=$false}
    )}} | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $registry -Encoding utf8NoBOM
    $powershell=(& $audit -Format json -MetadataPath $fixture -RegistryPath $registry) | ConvertFrom-Json
    # Fixture must be BOM-free for the explicitly UTF-8 JSON contract on Unix.
    $inputData | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $fixture -Encoding utf8NoBOM
    $unixText=& bash (Join-Path $PSScriptRoot 'dependency_audit.sh') --format json --metadata $fixture --registry $registry
    if($LASTEXITCODE -ne 0) { throw 'Unix fixture audit failed' }
    $unix=$unixText|ConvertFrom-Json
    if(($powershell|ConvertTo-Json -Depth 20 -Compress) -cne ($unix|ConvertTo-Json -Depth 20 -Compress)) { throw 'registry-enriched reports differ' }
    $new=@($unix.dependencies | Where-Object { $_.manifest_requirements -contains '^2' })[0]
    if($new.latest_stable -ne '2.12.0' -or ($new.classification -join ',') -ne 'minor,yanked') { throw 'registry classification incorrect' }
    foreach($row in $unix.dependencies) {
        if($row.signals.deprecated -cne 'unknown' -or $row.signals.security -cne 'unknown') {
            throw 'registry version evidence was incorrectly treated as a security/deprecation audit'
        }
    }
    $saved=$inputData|ConvertTo-Json -Depth 20
    foreach($fault in @('resolve','member-node','member-package','duplicate-package','duplicate-node','resolved-package')) {
        $inputData=$saved|ConvertFrom-Json -AsHashtable
        switch($fault) {
            'resolve' { $inputData.resolve=$null }
            'member-node' { $inputData.resolve.nodes=@() }
            'member-package' { $inputData.packages=@($inputData.packages | Where-Object id -ne 'workspace') }
            'duplicate-package' { $inputData.packages+=@($inputData.packages[0]) }
            'duplicate-node' { $inputData.resolve.nodes+=@($inputData.resolve.nodes[0]) }
            'resolved-package' { $inputData.packages=@($inputData.packages | Where-Object id -ne 'old') }
        }
        $rejected=$false
        try { $null=Run-Fixture } catch { $rejected=$true }
        if(-not $rejected) { throw "PowerShell accepted invalid metadata: $fault" }
        $inputData|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $fixture -Encoding utf8NoBOM
        $null=& bash (Join-Path $PSScriptRoot 'dependency_audit.sh') --format json --metadata $fixture 2>$null
        if($LASTEXITCODE -eq 0) { throw "Unix accepted invalid metadata: $fault" }
    }
    $inputData=$saved|ConvertFrom-Json -AsHashtable
    $gitSource='git+https://example.invalid/library#0123456789abcdef'
    $inputData.packages[0].dependencies[0].source=$gitSource
    $inputData.packages[1].source=$gitSource
    $null=Run-Fixture
    $inputData|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $fixture -Encoding utf8NoBOM
    $powershell=(& $audit -Format json -MetadataPath $fixture -RegistryPath $registry)|ConvertFrom-Json
    $unix=(& bash (Join-Path $PSScriptRoot 'dependency_audit.sh') --format json --metadata $fixture --registry $registry)|ConvertFrom-Json
    if($LASTEXITCODE -ne 0 -or ($powershell|ConvertTo-Json -Depth 20 -Compress) -cne ($unix|ConvertTo-Json -Depth 20 -Compress)) { throw 'external Git dependency parity failed' }
    $gitRow=@($unix.dependencies|Where-Object source -eq $gitSource)
    if($gitRow.Count -ne 1 -or $gitRow[0].classification -notcontains 'unknown' -or $unix.status -ne 'partial') { throw 'unsupported registry source was silently dropped or blessed' }
    'dependency declaration mapping: PASS (aliases, kind/target, resolutions, mutation, registry parity, six malformed graphs, Git external source)'
} finally {
    Remove-Item -LiteralPath $fixture
    Remove-Item -LiteralPath $registry
}
