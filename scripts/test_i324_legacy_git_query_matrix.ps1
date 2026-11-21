# I324 / #919 read-only legacy Git command API oracle (PowerShell 7, Windows).
# The selected binary is executed only in a fresh temporary Git fixture.
# No PATH, process/user HOME, Git installation, or caller environment is changed.
# Git stdout/stderr are redacted; only names, exit codes and result classes print.
param(
    [Alias('PinnedMinGit')]
    [switch]$UsePinnedMinGit,
    [string]$GitExecutable
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $IsWindows) { throw 'This I324 test needs native Windows PowerShell 7.' }
if ($UsePinnedMinGit.IsPresent -eq (-not [string]::IsNullOrWhiteSpace($GitExecutable))) {
    throw 'Specify exactly one: -UsePinnedMinGit or -GitExecutable <absolute path>.'
}

$root = Join-Path ([IO.Path]::GetTempPath()) ('talos-i324-' + [guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($root) | Out-Null
try {
    if ($UsePinnedMinGit) {
        $uri = 'https://github.com/git-for-windows/git/releases/download/v2.36.1.windows.1/MinGit-2.36.1-64-bit.zip'
        $expectedSha = 'a7a78c306dea018cc7ca3efe6a0d87c1dd7a43762705ccb0c5e31e3e44349207'
        $zip = Join-Path $root 'MinGit-2.36.1-64-bit.zip'
        Invoke-WebRequest -Uri $uri -OutFile $zip -MaximumRedirection 5
        $sha = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($sha -cne $expectedSha) {
            throw 'Pinned Git 2.36.1 SHA-256 mismatch: archive was not executed.'
        }
        $extracted = Join-Path $root 'mingit'
        Expand-Archive -LiteralPath $zip -DestinationPath $extracted
        $GitExecutable = Join-Path $extracted 'cmd/git.exe'
    }

    $selectedGit = [IO.Path]::GetFullPath($GitExecutable)
    if (-not [IO.File]::Exists($selectedGit)) { throw 'Selected Git executable not found.' }
    $fixture = Join-Path $root 'fixture'
    $isolatedHome = Join-Path $root 'home'
    $isolatedXdg = Join-Path $root 'xdg'
    $hookDir = Join-Path $root 'empty-hooks'
    foreach ($dir in @($fixture, $isolatedHome, $isolatedXdg, $hookDir)) {
        [IO.Directory]::CreateDirectory($dir) | Out-Null
    }

    function Invoke-I324Git {
        param([string[]]$GitArguments)
        $start = [Diagnostics.ProcessStartInfo]::new()
        $start.FileName = $selectedGit
        $start.WorkingDirectory = $fixture
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        foreach ($arg in $GitArguments) { [void]$start.ArgumentList.Add($arg) }
        # Preserve PATH for bundled DLL loading but remove Git-specific ambient
        # overrides. Do not use this isolated result as original Scoop evidence.
        foreach ($key in @($start.Environment.Keys)) {
            if ($key -like 'GIT_*') { [void]$start.Environment.Remove($key) }
        }
        $start.Environment['HOME'] = $isolatedHome
        $start.Environment['XDG_CONFIG_HOME'] = $isolatedXdg
        $start.Environment['GIT_CONFIG_NOSYSTEM'] = '1'
        $start.Environment['GIT_ATTR_NOSYSTEM'] = '1'
        $start.Environment['GIT_CONFIG_GLOBAL'] = (Join-Path $root 'missing-global')
        $start.Environment['GIT_TERMINAL_PROMPT'] = '0'

        $proc = [Diagnostics.Process]::new()
        $proc.StartInfo = $start
        try {
            if (-not $proc.Start()) { throw 'Could not start selected Git.' }
            $stdout = $proc.StandardOutput.ReadToEndAsync()
            $stderr = $proc.StandardError.ReadToEndAsync()
            if (-not $proc.WaitForExit(15000)) {
                try { $proc.Kill($true); $proc.WaitForExit() } catch {}
                throw 'Selected Git timed out during isolated query.'
            }
            return [pscustomobject]@{
                ExitCode = $proc.ExitCode
                Stdout = $stdout.GetAwaiter().GetResult()
                Stderr = $stderr.GetAwaiter().GetResult()
            }
        } finally {
            $proc.Dispose()
        }
    }

    function Assert-I324Git {
        param([string[]]$GitArguments)
        $result = Invoke-I324Git -GitArguments $GitArguments
        if ($result.ExitCode -ne 0) {
            throw "Isolated Git fixture failed exit=$($result.ExitCode); output withheld."
        }
        return $result
    }

    $version = Assert-I324Git -GitArguments @('--version')
    $versionString = $version.Stdout.Trim()
    if ($UsePinnedMinGit -and $versionString -cne 'git version 2.36.1.windows.1') {
        throw 'Pinned Git 2.36.1 binary version mismatch.'
    }
    Write-Output "I324_LEGACY_FIXTURE version=$versionString pinned=$($UsePinnedMinGit.IsPresent)"
    Write-Output 'I324_LEGACY_ENV mode=isolated host_environment_unchanged=true'

    $unexpected = 0
    foreach ($name in @(
        'GIT_ATTR_SYSTEM', 'GIT_ATTR_GLOBAL',
        'GIT_CONFIG_SYSTEM', 'GIT_CONFIG_GLOBAL', 'GIT_SHELL_PATH'
    )) {
        $result = Invoke-I324Git -GitArguments @('-C', $fixture, '--no-pager', 'var', $name)
        $combined = $result.Stdout + [Environment]::NewLine + $result.Stderr
        if ($result.ExitCode -eq 0) {
            $kind = 'supported'
        } elseif ($combined -match '(?i)usage:\s*git\s+var') {
            $kind = 'unsupported-variable'
        } elseif ($result.ExitCode -eq 1 -and
                [string]::IsNullOrWhiteSpace($result.Stdout) -and
                [string]::IsNullOrWhiteSpace($result.Stderr)) {
            $kind = 'supported-but-unset'
        } else {
            $kind = 'unexpected-error'
            $unexpected++
        }
        Write-Output "I324_LEGACY_VAR name=$name exit=$($result.ExitCode) classification=$kind"
    }

    [void](Assert-I324Git -GitArguments @('-C', $fixture, 'init', '-q'))
    $utf8 = [Text.UTF8Encoding]::new($false)
    $attrs = Join-Path $fixture '.gitattributes'
    [IO.File]::WriteAllText($attrs, ('*.txt filter=committed' + [Environment]::NewLine), $utf8)
    [IO.File]::WriteAllText((Join-Path $fixture 'tracked.txt'), 'fixture', $utf8)
    [void](Assert-I324Git -GitArguments @(
        '-C', $fixture, '-c', "core.hooksPath=$hookDir", 'add', '--', '.gitattributes', 'tracked.txt'
    ))
    [void](Assert-I324Git -GitArguments @(
        '-C', $fixture, '-c', "core.hooksPath=$hookDir",
        '-c', 'user.name=I324 Test', '-c', 'user.email=i324@example.invalid',
        'commit', '-q', '-m', 'fixture'
    ))
    [IO.File]::WriteAllText($attrs, ('*.txt filter=index' + [Environment]::NewLine), $utf8)
    [void](Assert-I324Git -GitArguments @('-C', $fixture, 'add', '--', '.gitattributes'))
    [IO.File]::WriteAllText($attrs, ('*.txt filter=worktree' + [Environment]::NewLine), $utf8)

    $current = Invoke-I324Git -GitArguments @(
        '-C', $fixture, 'check-attr', 'filter', '--', 'tracked.txt'
    )
    $cached = Invoke-I324Git -GitArguments @(
        '-C', $fixture, 'check-attr', '--cached', 'filter', '--', 'tracked.txt'
    )
    if ($current.ExitCode -ne 0 -or
        -not $current.Stdout.Trim().EndsWith('filter: worktree') -or
        $cached.ExitCode -ne 0 -or
        -not $cached.Stdout.Trim().EndsWith('filter: index')) {
        throw 'Ordinary or --cached attribute semantics differ from the fixture.'
    }
    $source = Invoke-I324Git -GitArguments @(
        '-C', $fixture, 'check-attr', '--source=HEAD', 'filter', '--', 'tracked.txt'
    )
    $sourceOutput = $source.Stdout + [Environment]::NewLine + $source.Stderr
    if ($source.ExitCode -eq 0 -and $source.Stdout.Trim().EndsWith('filter: committed')) {
        $sourceKind = 'supported'
    } elseif ($sourceOutput -match '(?i)usage:\s*git\s+check-attr|unknown option') {
        $sourceKind = 'unsupported-option'
    } else {
        $sourceKind = 'unexpected-error'
        $unexpected++
    }
    Write-Output "I324_LEGACY_CHECK_ATTR worktree=ok index=ok source=$sourceKind exit=$($source.ExitCode)"
    Write-Output "I324_LEGACY_QUERY_MATRIX unexpected=$unexpected status=$(if ($unexpected -eq 0) {'complete'} else {'investigate'})"
    if ($unexpected -ne 0) {
        throw 'Unexpected Git command outcome; preserve classified results for I324 investigation.'
    }
} finally {
    if ([IO.Directory]::Exists($root)) {
        [IO.Directory]::Delete($root, $true)
    }
}
