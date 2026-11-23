# I324 / #919: opt-in, output-redacted selected Git for Windows install-root differential.
# No production authorization changes. This does not prove that a GIT_INSTALL_ROOT
# allowlist would be safe: only a small set of read-only child commands is sampled.
param(
    [Parameter(Mandatory = $true)]
    [string]$GitExecutable
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'Windows PowerShell 7 is required.' }
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'PowerShell 7+ is required.' }

# Never print root values, Git config/path contents, or the selected executable.
$installRoot = [Environment]::GetEnvironmentVariable('GIT_INSTALL_ROOT', 'Process')
if ([string]::IsNullOrWhiteSpace($installRoot)) {
    throw 'GIT_INSTALL_ROOT is not present in this caller; this differential is not applicable.'
}
$gitPath = [IO.Path]::GetFullPath($GitExecutable)
if (-not [IO.File]::Exists($gitPath)) { throw 'Selected Git binary is absent.' }

$root = Join-Path ([IO.Path]::GetTempPath()) ('talos-i324-' + [guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($root) | Out-Null
try {
    $fixture = Join-Path $root 'fixture'
    $isolatedHome = Join-Path $root 'home'
    $xdg = Join-Path $root 'xdg'
    foreach ($dir in @($fixture, $isolatedHome, $xdg)) {
        [IO.Directory]::CreateDirectory($dir) | Out-Null
    }

    function Invoke-I324SelectedGit {
        param(
            [string[]]$GitArguments,
            [bool]$InheritInstallRoot
        )
        $start = [Diagnostics.ProcessStartInfo]::new()
        $start.FileName = $gitPath
        $start.WorkingDirectory = $fixture
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        foreach ($arg in $GitArguments) { [void]$start.ArgumentList.Add($arg) }

        # The two cases differ ONLY by GIT_INSTALL_ROOT. Existing Git-specific
        # overrides are removed from both. HOME/XDG are identical, isolated
        # and never changed in the parent PowerShell process.
        foreach ($key in @($start.Environment.Keys)) {
            if ($key -like 'GIT_*') { [void]$start.Environment.Remove($key) }
        }
        $start.Environment['HOME'] = $isolatedHome
        $start.Environment['XDG_CONFIG_HOME'] = $xdg
        if ($InheritInstallRoot) {
            $start.Environment['GIT_INSTALL_ROOT'] = $installRoot
        }
        $proc = [Diagnostics.Process]::new()
        $proc.StartInfo = $start
        try {
            if (-not $proc.Start()) { throw 'Selected Git failed to start.' }
            $stdoutTask = $proc.StandardOutput.ReadToEndAsync()
            $stderrTask = $proc.StandardError.ReadToEndAsync()
            if (-not $proc.WaitForExit(15000)) {
                try { $proc.Kill($true); $proc.WaitForExit() } catch {}
                throw 'Selected Git timed out during a read-only diagnostic.'
            }
            return [pscustomobject]@{
                ExitCode = $proc.ExitCode
                Stdout = $stdoutTask.GetAwaiter().GetResult()
                Stderr = $stderrTask.GetAwaiter().GetResult()
            }
        } finally {
            $proc.Dispose()
        }
    }

    $version = Invoke-I324SelectedGit -GitArguments @('--version') -InheritInstallRoot $false
    if ($version.ExitCode -ne 0) { throw 'Selected Git version probe failed.' }
    Write-Output "I324_INSTALL_ROOT_DIFF version=$($version.Stdout.Trim()) caller_root_present=true isolated_home=true"

    # A brand-new repo avoids touching any user worktree. This sole write step
    # is contained inside the script-owned TEMP fixture (no user Git mutation).
    $created = Invoke-I324SelectedGit -GitArguments @('init', '-q') -InheritInstallRoot $false
    if ($created.ExitCode -ne 0) { throw 'Isolated Git repository setup failed.' }
    [IO.File]::WriteAllText(
        (Join-Path $fixture 'probe.txt'),
        'I324 fixture',
        [Text.UTF8Encoding]::new($false)
    )

    $queries = @(
        @{ Name = 'exec-path'; Args = @('--exec-path') },
        @{ Name = 'config-list'; Args = @('config', '--show-origin', '--null', '--list', '--includes') },
        @{ Name = 'system-config-list'; Args = @('config', '--system', '--show-origin', '--null', '--list') },
        @{ Name = 'global-config-list'; Args = @('config', '--global', '--show-origin', '--null', '--list') },
        @{ Name = 'attributes-all'; Args = @('check-attr', '--all', '--', 'probe.txt') }
    )
    $differences = 0
    $errors = 0
    foreach ($query in $queries) {
        $without = Invoke-I324SelectedGit -GitArguments $query.Args -InheritInstallRoot $false
        $with = Invoke-I324SelectedGit -GitArguments $query.Args -InheritInstallRoot $true
        $sameExit = $without.ExitCode -eq $with.ExitCode
        $sameStdout = $without.Stdout -ceq $with.Stdout
        $sameStderr = $without.Stderr -ceq $with.Stderr
        if (-not ($sameExit -and $sameStdout -and $sameStderr)) { $differences++ }
        if ($without.ExitCode -ne 0 -or $with.ExitCode -ne 0) { $errors++ }
        # Do not disclose Git output, configured paths, environment values,
        # credential-bearing keys, hashes or lengths.
        Write-Output "I324_INSTALL_ROOT_QUERY name=$($query.Name) same_exit=$sameExit same_stdout=$sameStdout same_stderr=$sameStderr without_exit=$($without.ExitCode) with_exit=$($with.ExitCode)"
    }
    Write-Output "I324_INSTALL_ROOT_DIFF_RESULT queries=$($queries.Count) differences=$differences nonzero_exit_pairs=$errors diagnostic_only=true"
    Write-Output 'I324_INSTALL_ROOT_NOTE equal_observations_do_not_authorize_env_exception=true'
} finally {
    if ([IO.Directory]::Exists($root)) {
        # Git object files can be read-only on Windows. Remove only our unique
        # direct child of system TEMP, never a caller-provided directory.
        $resolved = [IO.Path]::GetFullPath($root)
        $parent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\', '/')
        if ([IO.Path]::GetDirectoryName($resolved) -cne $parent -or
            [IO.Path]::GetFileName($resolved) -notmatch '^talos-i324-[0-9a-f]{32}$') {
            throw 'Refusing cleanup outside generated I324 temporary fixture.'
        }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
