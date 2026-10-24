# Temporary I300 native control: no Talos launcher, Job implementation or admission path.
$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'This diagnostic requires native Windows.' }
$probeRoot = Join-Path ([IO.Path]::GetTempPath()) ('talos-native-push-' + [guid]::NewGuid())
$processes = [Collections.Generic.List[Diagnostics.Process]]::new()
$gitExe = (Get-Command git -CommandType Application).Source
function Invoke-FixtureGit {
    param([string]$Directory, [string[]]$GitArgs)
    & $gitExe -C $Directory @GitArgs
    if ($LASTEXITCODE -ne 0) { throw "Fixture Git failed: $GitArgs (exit $LASTEXITCODE)" }
}
try {
    New-Item -ItemType Directory -Path $probeRoot | Out-Null
    $sender = Join-Path $probeRoot 'sender'
    New-Item -ItemType Directory -Path $sender | Out-Null
    Invoke-FixtureGit $sender @('init')
    Invoke-FixtureGit $sender @('config', 'user.name', 'Native diagnostic')
    Invoke-FixtureGit $sender @('config', 'user.email', 'test@example.invalid')
    Invoke-FixtureGit $sender @('config', 'core.hooksPath', (Join-Path $probeRoot 'no-hooks'))
    [IO.File]::WriteAllText((Join-Path $sender 'payload'), "native transfer`n")
    Invoke-FixtureGit $sender @('add', 'payload')
    Invoke-FixtureGit $sender @('commit', '-m', 'fixture')
    foreach ($mode in @('default', 'sideband-off')) {
        $receiver = Join-Path $probeRoot ($mode + '.git')
        New-Item -ItemType Directory -Path $receiver | Out-Null
        Invoke-FixtureGit $receiver @('init', '--bare')
    }
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    $port = $listener.LocalEndpoint.Port
    $listener.Stop()
    $daemonLog = Join-Path $probeRoot 'daemon.log'
    $daemonArgs = @('daemon', '--reuseaddr', '--verbose', '--export-all', '--enable=receive-pack', '--listen=127.0.0.1', "--port=$port", ('--base-path="' + $probeRoot.Replace('\', '/') + '"'))
    $daemon = Start-Process -FilePath $gitExe -ArgumentList $daemonArgs -PassThru -NoNewWindow -RedirectStandardError $daemonLog
    $processes.Add($daemon)
    $ready = $false
    for ($attempt = 0; $attempt -lt 50; $attempt++) {
        if ($daemon.HasExited) { throw 'Native daemon exited before readiness.' }
        $client = [Net.Sockets.TcpClient]::new()
        try { $client.Connect('127.0.0.1', $port); $ready = $true; break }
        catch { Start-Sleep -Milliseconds 100 }
        finally { $client.Dispose() }
    }
    if (-not $ready) { throw 'Native daemon readiness timed out.' }
    foreach ($mode in @('default', 'sideband-off')) {
        $log = Join-Path $probeRoot ($mode + '.log')
        $out = Join-Path $probeRoot ($mode + '.out')
        $pushArgs = @('-C', ('"' + $sender + '"'))
        if ($mode -eq 'sideband-off') { $pushArgs += @('-c', 'sendpack.sideband=false') }
        $pushArgs += @('push', '--no-verify', "git://127.0.0.1:$port/$mode.git", 'HEAD:refs/heads/probe')
        $push = Start-Process -FilePath $gitExe -ArgumentList $pushArgs -PassThru -NoNewWindow -RedirectStandardError $log -RedirectStandardOutput $out
        $processes.Add($push)
        $finished = $push.WaitForExit(10000)
        if ($finished) {
            Write-Host "Native Windows Push control: mode=$mode exit=$($push.ExitCode)"
        } else {
            Write-Host "Native Windows Push control: mode=$mode deadline elapsed"
            & taskkill.exe /PID $push.Id /T /F
            if ($LASTEXITCODE -ne 0) { throw 'Native sender tree cleanup failed.' }
            if (-not $push.WaitForExit(5000)) { throw 'Native sender did not exit after tree cleanup.' }
        }
        Get-Content -LiteralPath $log
        Invoke-FixtureGit (Join-Path $probeRoot ($mode + '.git')) @('for-each-ref', '--format=%(refname):%(objectname)')
    }
} finally {
    $cleanupConfirmed = $true
    foreach ($process in $processes) {
        if (-not $process.HasExited) {
            & taskkill.exe /PID $process.Id /T /F
            if ($LASTEXITCODE -ne 0) { $cleanupConfirmed = $false }
            if (-not $process.WaitForExit(5000)) { $cleanupConfirmed = $false }
        }
        $process.Dispose()
    }
    if (Test-Path -LiteralPath $probeRoot) {
        Get-ChildItem -LiteralPath $probeRoot -Filter '*.log' | ForEach-Object {
            Write-Host "Native diagnostic log: $($_.Name)"
            Get-Content -LiteralPath $_.FullName
        }
        if ($cleanupConfirmed) { Remove-Item -LiteralPath $probeRoot -Recurse -Force }
    }
    if (-not $cleanupConfirmed) { throw 'Native diagnostic tree cleanup unconfirmed; fixture retained.' }
}
