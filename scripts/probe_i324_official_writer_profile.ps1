# I324 test-only official Git package oracle; never point at an installed Git.
param([ValidateSet('MinGit','PortableGit')][string]$Package = 'MinGit')
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
    $roots = @('etc','mingw64/etc')
    $images = @('cmd/git.exe','mingw64/bin/git.exe')
    foreach ($relative in $images + @('usr/bin/sh.exe')) {
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
        Remove-Item -LiteralPath (Join-Path $isolatedHome '.gitconfig'),(Join-Path $xdg 'git/config'),(Join-Path $xdg 'git/attributes'),(Join-Path $isolatedHome '.config/git/attributes') -Force
        Write-Output "I324_PACKAGE_GLOBALS package=$Package image=$image order=xdg,home attributes=pass"
    }
    Write-Output "I324_PACKAGE_PROFILE package=$Package archive_sha256=$($receipt.Sha) result=observed"
} finally {
    $cleanupRoot = [IO.Path]::GetFullPath($root)
    $tempParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\','/')
    if ([IO.Path]::GetDirectoryName($cleanupRoot) -cne $tempParent -or [IO.Path]::GetFileName($cleanupRoot) -notmatch '^talos-i324-profile-[0-9a-f]{32}$') {
        throw 'Refusing cleanup outside generated fixture.'
    }
    if ([IO.Directory]::Exists($cleanupRoot)) { Remove-Item -LiteralPath $cleanupRoot -Recurse -Force }
}
