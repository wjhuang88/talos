# I324 test-only official Git package oracle; never point at an installed Git.
param([ValidateSet('MinGit','PortableGit')][string]$Package = 'MinGit', [switch]$SelectedWriterOnly, [switch]$PeImportsOnly)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'Native Windows PowerShell 7 required.' }
$packages = @{
    MinGit = @{ Name='MinGit-2.36.1-64-bit.zip'; Sha='a7a78c306dea018cc7ca3efe6a0d87c1dd7a43762705ccb0c5e31e3e44349207' }
    PortableGit = @{ Name='PortableGit-2.36.1-64-bit.7z.exe'; Sha='af17a2803c5c6406b9b60dfef2d34f72f218975f9d78df21005a44f6e2f0caf9' }
}
$root = Join-Path ([IO.Path]::GetTempPath()) ('talos-i324-profile-' + [guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($root) | Out-Null
try {
    $receipt = $packages[$Package]
    $archive = Join-Path $root $receipt.Name
    $uri = 'https://github.com/git-for-windows/git/releases/download/v2.36.1.windows.1/' + $receipt.Name
    Invoke-WebRequest -Uri $uri -OutFile $archive -MaximumRedirection 5
    if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -cne $receipt.Sha) {
        throw 'Package checksum mismatch; nothing extracted or executed.'
    }
    $install = Join-Path $root 'package'
    if ($Package -eq 'MinGit') {
        Expand-Archive -LiteralPath $archive -DestinationPath $install
    } else {
        # Extract as data. Do not execute the self-extracting installer.
        $extractor = Join-Path $env:ProgramFiles '7-Zip/7z.exe'
        if (-not [IO.File]::Exists($extractor)) { throw 'Runner 7-Zip unavailable.' }
        & $extractor x -y "-o$install" $archive | Out-Null
        if ($LASTEXITCODE -ne 0) { throw 'PortableGit extraction failed.' }
    }
    if ($PeImportsOnly) {
        if ($SelectedWriterOnly) { throw 'Choose one oracle mode.' }
        # Static comparison only, using the hash-verified extracted archive.
        $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
        if (-not [IO.File]::Exists($vswhere)) { throw 'Runner vswhere unavailable.' }
        $vs = @(& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)
        if ($LASTEXITCODE -ne 0 -or $vs.Count -ne 1) { throw 'Runner MSVC tools unavailable.' }
        $toolVersion = (Get-Content -LiteralPath (Join-Path $vs[0] 'VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt') -Raw).Trim()
        if ($toolVersion -notmatch '^\d+\.\d+\.\d+$') { throw 'Unexpected MSVC tool version.' }
        $dumpbin = Join-Path $vs[0] "VC/Tools/MSVC/$toolVersion/bin/Hostx64/x64/dumpbin.exe"
        if (-not [IO.File]::Exists($dumpbin)) { throw 'Runner dumpbin unavailable.' }
        $source = Join-Path $PSScriptRoot '../crates/talos-tools/src/git_write/pe_import_frontier_tests.rs'
        $harness = Join-Path $root 'pe-import-tests.exe'
        & rustc +1.99.0 --edition=2024 --test $source -o $harness
        if ($LASTEXITCODE -ne 0) { throw 'PE fixture harness build failed.' }
        & $harness
        if ($LASTEXITCODE -ne 0) { throw 'PE synthetic fixture tests failed.' }
        $images = @('mingw64/bin/git.exe')
        if ($Package -eq 'PortableGit') {
            $images += @(
                'mingw64/libexec/git-core/git.exe',
                'mingw64/libexec/git-core/libpcre2-8-0.dll',
                'mingw64/libexec/git-core/libintl-8.dll',
                'mingw64/libexec/git-core/libiconv-2.dll',
                'mingw64/libexec/git-core/libssp-0.dll',
                'mingw64/libexec/git-core/zlib1.dll'
            )
        }
        $previous = [Environment]::GetEnvironmentVariable('TALOS_I324_PE_FIXTURE','Process')
        try {
            foreach ($relative in $images) {
                $image = Join-Path $install $relative
                if (-not [IO.File]::Exists($image)) { throw "Missing pinned fixture: $relative" }
                $hash = (Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash.ToLowerInvariant()
                [Environment]::SetEnvironmentVariable('TALOS_I324_PE_FIXTURE',$image,'Process')
                $candidate = @(& $harness --exact pe_import_frontier_fixture_from_file --ignored --nocapture)
                if ($LASTEXITCODE -ne 0) { throw "PE parser fixture failed: $relative" }
                $oracle = @(& $dumpbin /NOLOGO /IMPORTS $image)
                if ($LASTEXITCODE -ne 0) { throw "Dumpbin fixture failed: $relative" }
                # Compare the union of declared ordinary and delay imports,
                # not resolved paths, dynamic loading, or mapped identity.
                $actual = @($candidate | ForEach-Object {
                    if ($_ -cmatch '^I324_PE_DECLARED kind=(ordinary|delayed) name=([a-z0-9_.-]+\.dll)$') {
                        $Matches[2]
                    }
                } | Sort-Object -Unique)
                $expected = @($oracle | ForEach-Object {
                    if ($_ -match '^\s+([a-z0-9_.-]+\.dll)\s*$') { $Matches[1].ToLowerInvariant() }
                } | Sort-Object -Unique)
                if ($expected.Count -eq 0 -or ($actual -join ',') -cne ($expected -join ',')) {
                    throw "PE import-name differential failed: $relative expected=$($expected -join ',') actual=$($actual -join ',')"
                }
                Write-Output "I324_PE_IMPORT_DIFFERENTIAL package=$Package image=$relative sha256=$hash count=$($actual.Count) names=$($actual -join ',') result=pass static_only=true"
            }
        } finally {
            [Environment]::SetEnvironmentVariable('TALOS_I324_PE_FIXTURE',$previous,'Process')
        }
        return
    }
    $fixture = Join-Path $root 'fixture'
    $isolatedHome = Join-Path $root 'home'
    $xdg = Join-Path $root 'xdg'
    $programData = Join-Path $root 'programdata'
    foreach ($dir in @($fixture,$isolatedHome,$xdg,$programData,(Join-Path $xdg 'git'),(Join-Path $isolatedHome '.config/git'))) {
        [IO.Directory]::CreateDirectory($dir) | Out-Null
    }
    $utf8 = [Text.UTF8Encoding]::new($false)
    function Write-Fixture([string]$Path,[string]$Text) {
        [IO.File]::WriteAllText($Path,$Text,$utf8)
    }
    function Invoke-Oracle {
        param([string]$Executable,[string[]]$Arguments,[hashtable]$Overrides=@{})
        $start = [Diagnostics.ProcessStartInfo]::new()
        $start.FileName = $Executable
        $start.WorkingDirectory = $fixture
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        foreach ($arg in $Arguments) { [void]$start.ArgumentList.Add($arg) }
        foreach ($key in @($start.Environment.Keys)) {
            if ($key -like 'GIT_*') { [void]$start.Environment.Remove($key) }
        }
        $start.Environment['HOME'] = $isolatedHome
        $start.Environment['USERPROFILE'] = $isolatedHome
        $start.Environment['XDG_CONFIG_HOME'] = $xdg
        $start.Environment['PROGRAMDATA'] = $programData
        $start.Environment['ALLUSERSPROFILE'] = $programData
        $start.Environment['GIT_TERMINAL_PROMPT'] = '0'
        foreach ($key in $Overrides.Keys) { $start.Environment[$key] = [string]$Overrides[$key] }
        $proc = [Diagnostics.Process]::new()
        $proc.StartInfo = $start
        try {
            if (-not $proc.Start()) { throw 'Could not start pinned Git fixture.' }
            $stdout = $proc.StandardOutput.ReadToEndAsync()
            $stderr = $proc.StandardError.ReadToEndAsync()
            if (-not $proc.WaitForExit(15000)) {
                try { $proc.Kill($true); $proc.WaitForExit() } catch {}
                throw 'Pinned Git fixture timed out.'
            }
            $out = $stdout.GetAwaiter().GetResult()
            [void]$stderr.GetAwaiter().GetResult()
            if ($proc.ExitCode -ne 0) { throw "Pinned Git query failed exit=$($proc.ExitCode); output withheld." }
            return $out
        } finally { $proc.Dispose() }
    }
    function Assert-Equal([string]$Actual,[string]$Expected,[string]$Case) {
        if ($Actual -cne $Expected) { throw "I324 profile mismatch case=$Case; output withheld." }
    }
    # Observe the actual rebound sender in the same spelling used by
    # prepared_add::request_sender_program. This is a fixture observation,
    # not a complete loader manifest or a production attestation.
    function Observe-SelectedModules([string]$Executable) {
        $start = [Diagnostics.ProcessStartInfo]::new()
        $start.FileName = $Executable
        $start.WorkingDirectory = $fixture
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardInput = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        foreach ($arg in @('cat-file','--batch-check')) { [void]$start.ArgumentList.Add($arg) }
        foreach ($key in @($start.Environment.Keys)) {
            if ($key -like 'GIT_*') { [void]$start.Environment.Remove($key) }
        }
        $start.Environment['HOME'] = $isolatedHome
        $start.Environment['USERPROFILE'] = $isolatedHome
        $start.Environment['XDG_CONFIG_HOME'] = $xdg
        $start.Environment['PROGRAMDATA'] = $programData
        $start.Environment['ALLUSERSPROFILE'] = $programData
        $start.Environment['GIT_CONFIG_NOSYSTEM'] = '1'
        $start.Environment['GIT_CONFIG_GLOBAL'] = Join-Path $root 'missing-global'
        $start.Environment['GIT_TERMINAL_PROMPT'] = '0'
        $proc = [Diagnostics.Process]::new()
        $proc.StartInfo = $start
        $started = $false
        try {
            if (-not $proc.Start()) { throw 'Could not start selected writer fixture.' }
            $started = $true
            $stderr = $proc.StandardError.ReadToEndAsync()
            # A completed read-only request is a handshake; the process then
            # stays alive on stdin. No arbitrary sleep or repository mutation.
            $missing = '0' * 40
            $proc.StandardInput.WriteLine($missing)
            $proc.StandardInput.Flush()
            $line = $proc.StandardOutput.ReadLineAsync()
            if (-not $line.Wait(5000)) { throw 'Selected writer handshake timed out.' }
            Assert-Equal $line.Result "$missing missing" 'selected-writer-handshake'
            $proc.Refresh()
            if ($proc.HasExited -or -not [IO.Path]::GetFullPath($proc.MainModule.FileName).Equals(
                [IO.Path]::GetFullPath($Executable), [StringComparison]::OrdinalIgnoreCase)) {
                throw 'Selected writer process image mismatch.'
            }
            $packagePrefix = [IO.Path]::GetFullPath($install).TrimEnd('\','/') + '\'
            $osPrefix = [IO.Path]::GetFullPath($env:SystemRoot).TrimEnd('\','/') + '\'
            $packageCount = 0
            $osCount = 0
            $otherCount = 0
            foreach ($module in @($proc.Modules)) {
                $name = [IO.Path]::GetFullPath($module.FileName)
                if ($name.StartsWith($packagePrefix, [StringComparison]::OrdinalIgnoreCase)) {
                    $relative = $name.Substring($packagePrefix.Length).Replace('\','/')
                    $hash = (Get-FileHash -LiteralPath $name -Algorithm SHA256).Hash.ToLowerInvariant()
                    Write-Output "I324_SELECTED_MODULE package=$Package image=$relative sha256=$hash"
                    $packageCount++
                } elseif ($name.StartsWith($osPrefix, [StringComparison]::OrdinalIgnoreCase)) {
                    $osCount++
                } else {
                    # Do not disclose arbitrary runner paths. An outside
                    # module is explicitly NOT proven by this receipt.
                    $otherCount++
                }
            }
            if ($packageCount -lt 1) { throw 'Selected writer module inventory empty.' }
            $proc.StandardInput.Close()
            if (-not $proc.WaitForExit(5000)) { throw 'Selected writer did not stop.' }
            [void]$stderr.GetAwaiter().GetResult()
            if ($proc.ExitCode -ne 0) { throw 'Selected writer handshake process failed.' }
            Write-Output "I324_SELECTED_MODULES package=$Package package_count=$packageCount os_count=$osCount other_count=$otherCount snapshot_only=true"
        } finally {
            if ($started -and -not $proc.HasExited) { $proc.Kill($true); $proc.WaitForExit() }
            $proc.Dispose()
        }
    }

    # The older distribution may omit the internal executable entirely.
    # Absence is a classified negative fixture result, NOT a usable profile.
    # Never copy the bin image into libexec or silently substitute it.
    if ($SelectedWriterOnly) {
        $native = Join-Path $install 'mingw64/bin/git.exe'
        if (-not [IO.File]::Exists($native)) { throw 'Required native bin image missing.' }
        $reported = (Invoke-Oracle $native @('--exec-path') @{GIT_CONFIG_NOSYSTEM='1'; GIT_CONFIG_GLOBAL=(Join-Path $root 'missing-global')}).Trim()
        $expected = Join-Path $install 'mingw64/libexec/git-core'
        if (-not [IO.Path]::GetFullPath($reported).Equals([IO.Path]::GetFullPath($expected), [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Unexpected native bin exec-path; no classification.'
        }
        if (-not [IO.File]::Exists((Join-Path $expected 'git.exe'))) {
            Write-Output "I324_SELECTED_IMAGE package=$Package image=mingw64/libexec/git-core/git.exe result=absent"
            Write-Output "I324_SELECTED_PROFILE package=$Package result=unsupported reason=missing-rebound-image"
            # Run only source-completeness observations on the explicitly
            # labeled native bin; do not claim it is the rebound writer.
            $sourceOnly = $true
        } else { $sourceOnly = $false }
    } else { $sourceOnly = $false }

    $roots = @('etc','mingw64/etc')
    $images = if ($sourceOnly) { @('mingw64/bin/git.exe') } elseif ($SelectedWriterOnly) { @('mingw64/libexec/git-core/git.exe') } else { @('cmd/git.exe','mingw64/bin/git.exe') }
    $receiptImages = if ($SelectedWriterOnly -and -not $sourceOnly) { @($images) + @('mingw64/bin/git.exe') } elseif ($sourceOnly) { @($images) } else { $images + @('usr/bin/sh.exe') }
    foreach ($relative in $receiptImages) {
        $exe = Join-Path $install $relative
        if ([IO.File]::Exists($exe)) {
            $hash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
            Write-Output "I324_PACKAGE_IMAGE package=$Package image=$relative sha256=$hash"
        } elseif ($relative -ceq 'usr/bin/sh.exe') {
            Write-Output "I324_PACKAGE_IMAGE package=$Package image=$relative absent=true"
        } else { throw "Required package image missing: $relative" }
    }
    foreach ($image in $images) {
        $exe = Join-Path $install $image
        $isolatedGlobal = @{GIT_CONFIG_GLOBAL=(Join-Path $root 'missing-global')}
        $version = Invoke-Oracle $exe @('--version') @{GIT_CONFIG_NOSYSTEM='1'; GIT_CONFIG_GLOBAL=(Join-Path $root 'missing-global')}
        Assert-Equal ($version.Trim()) 'git version 2.36.1.windows.1' 'version'
        $execPath = (Invoke-Oracle $exe @('--exec-path') @{GIT_CONFIG_NOSYSTEM='1'; GIT_CONFIG_GLOBAL=(Join-Path $root 'missing-global')}).Trim()
        $expectedExec = Join-Path $install 'mingw64/libexec/git-core'
        if (-not [IO.Path]::GetFullPath($execPath).Equals([IO.Path]::GetFullPath($expectedExec),[StringComparison]::OrdinalIgnoreCase)) {
            throw 'Unexpected selected writer exec-path; no profile inferred.'
        }
        # Compare both possible roots ONLY within this disposable release.
        foreach ($candidate in $roots) {
            $dir = Join-Path $install $candidate
            [IO.Directory]::CreateDirectory($dir) | Out-Null
            $marker = if ($candidate -eq 'etc') { 'root' } else { 'mingw' }
            Write-Fixture (Join-Path $dir 'gitconfig') ("[i324]" + [Environment]::NewLine + "    profile = $marker" + [Environment]::NewLine)
            Write-Fixture (Join-Path $dir 'gitattributes') ("i324-fixture.txt filter=$marker" + [Environment]::NewLine)
        }
        [void](Invoke-Oracle $exe @('init','-q') $isolatedGlobal)
        if ($SelectedWriterOnly -and -not $sourceOnly) {
            $selectedHash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
            $binHash = (Get-FileHash -LiteralPath (Join-Path $install 'mingw64/bin/git.exe') -Algorithm SHA256).Hash
            Write-Output "I324_SELECTED_IMAGE package=$Package same_as_bin=$($selectedHash -ceq $binHash)"
            Observe-SelectedModules $exe
        }
        $config = (Invoke-Oracle $exe @('config','--system','--get','i324.profile') $isolatedGlobal).Trim()
        $attr = (Invoke-Oracle $exe @('check-attr','filter','--','i324-fixture.txt') $isolatedGlobal).Trim()
        if ($config -cnotin @('root','mingw')) { throw 'Unknown compiled config root.' }
        if ($attr -ceq 'i324-fixture.txt: filter: root') { $attrRoot = 'root' }
        elseif ($attr -ceq 'i324-fixture.txt: filter: mingw') { $attrRoot = 'mingw' }
        else { throw 'Unknown compiled attribute root.' }
        $configDir = if ($config -ceq 'root') { 'etc' } else { 'mingw64/etc' }
        $attrDir = if ($attrRoot -ceq 'root') { 'etc' } else { 'mingw64/etc' }
        Write-Output "I324_PACKAGE_ROOTS package=$Package image=$image config=$configDir/gitconfig attributes=$attrDir/gitattributes exec=mingw64/libexec/git-core"
        $cfgPath = Join-Path (Join-Path $install $configDir) 'gitconfig'
        $attrPath = Join-Path (Join-Path $install $attrDir) 'gitattributes'
        foreach ($mode in @('empty','missing')) {
            if ($mode -eq 'empty') {
                Write-Fixture $cfgPath ''
                Write-Fixture $attrPath ''
            } else { Remove-Item -LiteralPath $cfgPath,$attrPath -Force }
            $all = Invoke-Oracle $exe @('config','--show-origin','--null','--list','--includes') $isolatedGlobal
            if ($all.Contains('i324.profile')) { throw "Unexpected config fallback case=$mode" }
            $value = (Invoke-Oracle $exe @('check-attr','filter','--','i324-fixture.txt') $isolatedGlobal).Trim()
            Assert-Equal $value 'i324-fixture.txt: filter: unspecified' "system-attributes-$mode"
            Write-Output "I324_PACKAGE_ABSENCE package=$Package image=$image state=$mode result=pass"
        }
        Write-Fixture (Join-Path $xdg 'git/config') ("[i324]" + [Environment]::NewLine + "    profile = xdg")
        Write-Fixture (Join-Path $isolatedHome '.gitconfig') ("[i324]" + [Environment]::NewLine + "    profile = home")
        Write-Fixture (Join-Path $xdg 'git/attributes') 'i324-fixture.txt filter=xdg'
        Write-Fixture (Join-Path $isolatedHome '.config/git/attributes') 'i324-fixture.txt filter=homefallback'
        $order = (Invoke-Oracle $exe @('config','--get-all','i324.profile') @{GIT_CONFIG_NOSYSTEM='1'}).Trim() -split '\r?\n'
        Assert-Equal ($order -join ',') 'xdg,home' 'global-order'
        Assert-Equal ((Invoke-Oracle $exe @('check-attr','filter','--','i324-fixture.txt') @{GIT_ATTR_NOSYSTEM='1';GIT_CONFIG_NOSYSTEM='1'}).Trim()) 'i324-fixture.txt: filter: xdg' 'global-attributes'
        Assert-Equal ((Invoke-Oracle $exe @('check-attr','filter','--','i324-fixture.txt') @{GIT_ATTR_NOSYSTEM='1';GIT_CONFIG_NOSYSTEM='1';XDG_CONFIG_HOME='' }).Trim()) 'i324-fixture.txt: filter: homefallback' 'home-attributes'
        if ($SelectedWriterOnly) {
            # Test both global config entrypoints independently, including
            # empty and absent files; no host config is modified.
            $homeCfg = Join-Path $isolatedHome '.gitconfig'
            $xdgCfg = Join-Path $xdg 'git/config'
            $fallbackCfg = Join-Path $isolatedHome '.config/git/config'
            foreach ($case in @('xdg-empty','xdg-missing','home-empty','home-missing')) {
                Write-Fixture $homeCfg "[i324]`n    profile = home"
                Write-Fixture $xdgCfg "[i324]`n    profile = xdg"
                $path = if ($case.StartsWith('xdg-')) { $xdgCfg } else { $homeCfg }
                if ($case.EndsWith('-empty')) { Write-Fixture $path '' }
                else { Remove-Item -LiteralPath $path -Force }
                $remaining = if ($case.StartsWith('xdg-')) { 'home' } else { 'xdg' }
                $values = (Invoke-Oracle $exe @('config','--get-all','i324.profile') @{GIT_CONFIG_NOSYSTEM='1'}).Trim()
                Assert-Equal $values $remaining $case
                Write-Output "I324_GLOBAL_CONFIG package=$Package image=$image case=$case result=pass"
            }
            Write-Fixture $homeCfg "[i324]`n    profile = home"
            Write-Fixture $xdgCfg "[i324]`n    profile = xdg"
            Write-Fixture $fallbackCfg "[i324]`n    profile = homefallback"
            $fallback = (Invoke-Oracle $exe @('config','--get-all','i324.profile') @{GIT_CONFIG_NOSYSTEM='1';XDG_CONFIG_HOME='' }).Trim() -split '\r?\n'
            Assert-Equal ($fallback -join ',') 'homefallback,home' 'global-config-home-fallback'
            Remove-Item -LiteralPath $fallbackCfg -Force

            # Probe the environment-selected ProgramData candidate. A negative
            # result does not rule out Windows known-folder or other host roots.
            $pd = Join-Path $programData 'Git'
            [IO.Directory]::CreateDirectory($pd) | Out-Null
            $pdCfg = Join-Path $pd 'config'
            Write-Fixture $pdCfg "[i324]`n    programdata = fixture-marker"
            $all = Invoke-Oracle $exe @('config','--show-origin','--null','--list','--includes')
            $observed = $all.Contains("i324.programdata`nfixture-marker")
            Write-Output "I324_PROGRAMDATA package=$Package image=$image env_candidate_observed=$observed known_folder_not_proven=true"
            Remove-Item -LiteralPath $pdCfg -Force
            $all = Invoke-Oracle $exe @('config','--show-origin','--null','--list','--includes')
            if ($all.Contains("i324.programdata`nfixture-marker")) { throw 'Removed ProgramData marker still observed.' }
        }
        Remove-Item -LiteralPath (Join-Path $isolatedHome '.gitconfig'),(Join-Path $xdg 'git/config'),(Join-Path $xdg 'git/attributes'),(Join-Path $isolatedHome '.config/git/attributes') -Force
        Write-Output "I324_PACKAGE_GLOBALS package=$Package image=$image order=xdg,home attributes=pass"
    }
    if ($sourceOnly) {
        Write-Output "I324_PACKAGE_PROFILE package=$Package archive_sha256=$($receipt.Sha) result=source-observed-only rebound=absent"
    } else {
        Write-Output "I324_PACKAGE_PROFILE package=$Package archive_sha256=$($receipt.Sha) result=observed"
    }
} finally {
    $cleanupRoot = [IO.Path]::GetFullPath($root)
    $tempParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\','/')
    if ([IO.Path]::GetDirectoryName($cleanupRoot) -cne $tempParent -or [IO.Path]::GetFileName($cleanupRoot) -notmatch '^talos-i324-profile-[0-9a-f]{32}$') {
        throw 'Refusing cleanup outside generated fixture.'
    }
    if ([IO.Directory]::Exists($cleanupRoot)) { Remove-Item -LiteralPath $cleanupRoot -Recurse -Force }
}
