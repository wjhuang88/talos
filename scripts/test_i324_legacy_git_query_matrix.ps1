# I324 / #919: test-only, value-redacted Git 2.36.1 command API oracle.
# Never modifies PATH, the selected host Git, Talos authorization or production.
# Pinned bytes: official git-for-windows/git v2.36.1.windows.1 release notes.
param(
    [switch]$UsePinnedMinGit,
    [string]$GitExecutable
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw 'I324 legacy Git probe requires native Windows.' }
if ($UsePinnedMinGit -eq (-not [string]::IsNullOrWhiteSpace($GitExecutable))) {
    throw 'Choose exactly one of -UsePinnedMinGit or -GitExecutable.'
}

$root = Join-Path ([IO.Path]::GetTempPath()) ('talos-i324-git236-' + [guid]::NewGuid())
[IO.Directory]::CreateDirectory($root) | Out-Null
$envNames = @('HOME', 'XDG_CONFIG_HOME', 'GIT_CONFIG_NOSYSTEM',
    'GIT_ATTR_NOSYSTEM', 'GIT_CONFIG_GLOBAL', 'GIT_CONFIG_COUNT',
    'GIT_INSTALL_ROOT', 'GIT_PAGER', 'GIT_TERMINAL_PROMPT')
$previous = @{}
foreach ($name in $envNames) {
    $previous[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
}
try {
    if ($UsePinnedMinGit) {
        $uri = 'https://github.com/git-for-windows/git/releases/download/v2.36.1.windows.1/MinGit-2.36.1-64-bit.zip'
        $expectedSha = 'a7a78c306dea018cc7ca3efe6a0d87c1dd7a43762705ccb0c5e31e3e44349207'
        $zip = Join-Path $root 'MinGit-2.36.1-64-bit.zip'
        Invoke-WebRequest -Uri $uri -OutFile $zip -MaximumRedirection 5
        $actualSha = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actualSha -cne $expectedSha) {
            throw 'Pinned Git for Windows archive SHA-256 mismatch; refusing execution.'
        }
        $extract = Join-Path $root 'mingit'
        Expand-Archive -LiteralPath $zip -DestinationPath $extract
        $GitExecutable = Join-Path $extract 'cmd/git.exe'
    }
    $gitPath = [IO.Path]::GetFullPath($GitExecutable)
    if (-not (Test-Path -LiteralPath $gitPath -PathType Leaf)) {
        throw 'Selected legacy Git executable is absent.'
    }
    $version = (& $gitPath --version 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Selected Git version query failed.' }
    if ($UsePinnedMinGit -and $version -cne 'git version 2.36.1.windows.1') {
        throw 'Pinned Git archive executable reports an unexpected version.'
    }
    Write-Host "I324_LEGACY_FIXTURE version=$version pinned=$($UsePinnedMinGit.IsPresent)"

    $fixture = Join-Path $root 'fixture'
    $isolatedHome = Join-Path $root 'isolated-home'
    $xdg = Join-Path $root 'isolated-xdg'
    foreach ($dir in @($fixture, $isolatedHome, $xdg)) {
        [IO.Directory]::CreateDirectory($dir) | Out-Null
    }
    [Environment]::SetEnvironmentVariable('HOME', $isolatedHome, 'Process')
    [Environment]::SetEnvironmentVariable('XDG_CONFIG_HOME', $xdg, 'Process')
    [Environment]::SetEnvironmentVariable('GIT_CONFIG_NOSYSTEM', '1', 'Process')
    [Environment]::SetEnvironmentVariable('GIT_ATTR_NOSYSTEM', '1', 'Process')
    [Environment]::SetEnvironmentVariable('GIT_CONFIG_GLOBAL', (Join-Path $root 'nonexistent-global'), 'Process')
    [Environment]::SetEnvironmentVariable('GIT_CONFIG_COUNT', $null, 'Process')
    [Environment]::SetEnvironmentVariable('GIT_INSTALL_ROOT', $null, 'Process')
    [Environment]::SetEnvironmentVariable('GIT_PAGER', 'cat', 'Process')
    [Environment]::SetEnvironmentVariable('GIT_TERMINAL_PROMPT', '0', 'Process')

    $variables = @('GIT_ATTR_SYSTEM', 'GIT_ATTR_GLOBAL',
        'GIT_CONFIG_SYSTEM', 'GIT_CONFIG_GLOBAL', 'GIT_SHELL_PATH')
    foreach ($variable in $variables) {
        # Collect but NEVER print stdout/stderr: values can reveal paths/config.
        $response = (& $gitPath -C $fixture --no-pager var $variable 2>&1 | Out-String)
        $exitCode = $LASTEXITCODE
        $kind = if ($exitCode -eq 0) {
            'supported'
        } elseif ($exitCode -eq 1 -and [string]::IsNullOrWhiteSpace($response)) {
            'unset'
        } elseif ($response -match '(?i)usage:\s*git\s+var') {
            'unsupported-variable'
        } else {
            throw "Unexpected legacy git var failure variable=$variable exit=$exitCode response_bytes=$($response.Length)"
        }
        Write-Host "I324_LEGACY_VAR name=$variable exit=$exitCode classification=$kind"
    }

    function Assert-FixtureGit {
        param([string[]]$CliArguments)
        $response = (& $gitPath -C $fixture @CliArguments 2>&1 | Out-String)
        $exitCode = $LASTEXITCODE
        if ($exitCode -ne 0) {
            throw "Isolated Git fixture operation failed exit=$exitCode response_bytes=$($response.Length)"
        }
    }
    $encoding = [Text.UTF8Encoding]::new($false)
    Assert-FixtureGit -CliArguments @('init', '-q')
    $attrs = Join-Path $fixture '.gitattributes'
    [IO.File]::WriteAllText($attrs, "*.txt filter=committed" + [Environment]::NewLine, $encoding)
    [IO.File]::WriteAllText((Join-Path $fixture 'tracked.txt'), 'fixture', $encoding)
    Assert-FixtureGit -CliArguments @('add', '--', '.gitattributes', 'tracked.txt')
    Assert-FixtureGit -CliArguments @('-c', 'user.name=I324 Fixture',
        '-c', 'user.email=i324@example.invalid', 'commit', '-q', '-m', 'fixture')
    [IO.File]::WriteAllText($attrs, "*.txt filter=index" + [Environment]::NewLine, $encoding)
    Assert-FixtureGit -CliArguments @('add', '--', '.gitattributes')
    [IO.File]::WriteAllText($attrs, "*.txt filter=worktree" + [Environment]::NewLine, $encoding)

    $worktree = (& $gitPath -C $fixture check-attr filter -- tracked.txt 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $worktree.EndsWith('filter: worktree')) {
        throw 'Legacy ordinary check-attr failed isolated worktree semantics.'
    }
    $cached = (& $gitPath -C $fixture check-attr --cached filter -- tracked.txt 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $cached.EndsWith('filter: index')) {
        throw 'Legacy cached check-attr failed isolated index semantics.'
    }
    $source = (& $gitPath -C $fixture check-attr --source=HEAD filter -- tracked.txt 2>&1 | Out-String)
    $sourceExit = $LASTEXITCODE
    if ($sourceExit -eq 0) {
        if (-not $source.Trim().EndsWith('filter: committed')) {
            throw 'Supported target-tree check-attr returned wrong commit semantics.'
        }
        $sourceKind = 'supported'
    } elseif ($source -match '(?i)usage:\s*git\s+check-attr|unknown option') {
        $sourceKind = 'unsupported-option'
    } else {
        throw "Unexpected check-attr source failure exit=$sourceExit response_bytes=$($source.Length)"
    }
    Write-Host "I324_LEGACY_CHECK_ATTR worktree=ok index=ok source=$sourceKind exit=$sourceExit"
    Write-Host 'I324_LEGACY_QUERY_MATRIX result=complete authorization=diagnostic-only'
} finally {
    foreach ($name in $envNames) {
        [Environment]::SetEnvironmentVariable($name, $previous[$name], 'Process')
    }
    if (Test-Path -LiteralPath $root) {
        Remove-Item -LiteralPath $root -Recurse -Force
    }
}
