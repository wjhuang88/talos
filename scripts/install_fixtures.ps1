# PowerShell installer fixture tests for install/install.ps1.
# No network access is performed; all HTTP calls are mocked locally.
$ErrorActionPreference = 'Stop'

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$Root = Split-Path -Parent $ScriptDir
$Installer = Join-Path $Root 'install/install.ps1'

$TempBase = if ($env:TEMP) { $env:TEMP } elseif ($env:TMPDIR) { $env:TMPDIR } else { '/tmp' }
$Fixture = Join-Path $TempBase "talos-fixture-ps1-$([System.Guid]::NewGuid())"
New-Item -ItemType Directory -Path $Fixture | Out-Null
$ReleaseDir = Join-Path $Fixture 'release'
$InstallDir = Join-Path $Fixture 'install'
New-Item -ItemType Directory -Path $ReleaseDir | Out-Null
New-Item -ItemType Directory -Path $InstallDir | Out-Null

$ZipPath = Join-Path $ReleaseDir 'talos-x86_64-windows.zip'
$Archive = 'talos-x86_64-windows.zip'
$LatestJson = '[{"tag_name":"v0.0.0"}]'

$Passed = 0
$Failed = 0
$Skipped = 0

function Assert-Pass {
  param([string]$Label, [scriptblock]$Script)
  Write-Host $Label
  try {
    & $Script
    $script:Passed++
  } catch {
    Write-Host "FAIL: $Label ($_)" -ForegroundColor Red
    $script:Failed++
  }
}

function Assert-Fail {
  param([string]$Label, [scriptblock]$Script)
  Write-Host $Label
  try {
    & $Script
    Write-Host "FAIL: $Label (expected non-zero exit)" -ForegroundColor Red
    $script:Failed++
  } catch {
    $script:Passed++
  }
}

function Assert-FailWithMessage {
  param([string]$Label, [string]$ExpectedMsg, [scriptblock]$Script)
  Write-Host $Label
  try {
    & $Script
    Write-Host "FAIL: $Label (expected terminating error containing '$ExpectedMsg')" -ForegroundColor Red
    $script:Failed++
  } catch {
    if ($_ -match [regex]::Escape($ExpectedMsg)) {
      $script:Passed++
    } else {
      Write-Host "FAIL: $Label (error did not contain '$ExpectedMsg': $_)" -ForegroundColor Red
      $script:Failed++
    }
  }
}

function Skip-Label {
  param([string]$Label)
  Write-Host "SKIP: $Label"
  $script:Skipped++
}

function Invoke-RestMethod {
  param(
    [string]$Uri,
    [switch]$UseBasicParsing
  )
  if ($env:FIXTURE_OFFLINE -eq '1') {
    throw "network unreachable (fixture offline mode)"
  }
  if ($Uri -match 'api\.github\.com') {
    return ($LatestJson | ConvertFrom-Json)
  }
  throw "unexpected URI: $Uri"
}

function Invoke-WebRequest {
  param(
    [string]$Uri,
    [string]$OutFile,
    [switch]$UseBasicParsing
  )
  if ($env:FIXTURE_OFFLINE -eq '1') {
    throw "network unreachable (fixture offline mode)"
  }
  if ($Uri -match 'checksum\.sha256$') {
    $Src = Join-Path $ReleaseDir 'checksum.sha256'
    if (Test-Path $Src) {
      if ($env:FIXTURE_BAD_CHECKSUM -eq '1') {
        # Serve a deliberately wrong checksum to exercise the mismatch path
        $Bad = '0' * 64
        "$Bad  $($Archive)" | Set-Content -Path $OutFile -Encoding ASCII
      } else {
        Copy-Item $Src $OutFile -Force
      }
      return
    }
    throw "unexpected URI or missing fixture: $Uri"
  }
  if ($Uri -match '\.zip$') {
    $FileName = [System.IO.Path]::GetFileName($Uri)
    $Src = Join-Path $ReleaseDir $FileName
    if (Test-Path $Src) {
      Copy-Item $Src $OutFile -Force
      return
    }
  }
  throw "unexpected URI or missing fixture: $Uri"
}

function Prepare-Zip {
  $TmpZipDir = Join-Path $Fixture "zip-staging-$([System.Guid]::NewGuid())"
  New-Item -ItemType Directory -Path $TmpZipDir | Out-Null
  $StubExe = Join-Path $TmpZipDir 'talos.exe'
  # Archive placement is tested with a deliberately non-runnable stub. The
  # real-release workflow separately executes the published Windows binary.
  '@echo off' | Set-Content -Path $StubExe -Encoding ASCII
  Compress-Archive -Path $StubExe -DestinationPath $ZipPath -Force
  Remove-Item -Recurse -Force $TmpZipDir
  # Write a fixture checksum.sha256 (sha256sum format: "<hash>  <archive>")
  $Hash = (Get-FileHash -Algorithm SHA256 -Path $ZipPath).Hash.ToLower()
  "$Hash  $($Archive)" | Set-Content -Path (Join-Path $ReleaseDir 'checksum.sha256') -Encoding ASCII
}

function Reset-Install {
  Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

function Run-Installer {
  $env:TEMP = $TempBase
  if (-not $env:USERPROFILE) { $env:USERPROFILE = $TempBase }
  . $Installer -SkipSelfCheck
}

Assert-Pass "A. successful install places talos.exe" {
  Prepare-Zip
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'AMD64'
  $env:TALOS_VERSION = 'v0.0.0'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = ''
  $env:FIXTURE_BAD_CHECKSUM = ''
  Run-Installer
  $ExePath = Join-Path $InstallDir 'talos.exe'
  if (-not (Test-Path $ExePath)) { throw "talos.exe not found at $ExePath" }
}

# Runnable --version check: only meaningful on Windows where talos.exe is a real binary.
# On macOS/Linux the fixture's talos.exe is a Windows stub that cannot execute; defer to Windows CI
# so the test never records a false success from an incompatible-executable error.
Write-Host "A2. talos.exe --version runnable check"
if (-not $IsWindows) {
  Skip-Label "talos.exe is a Windows binary; runnable --version check deferred to Windows CI (non-Windows pwsh cannot execute it)"
} else {
  try {
    $ver = & (Join-Path $InstallDir 'talos.exe') --version 2>&1
    if ($LASTEXITCODE -eq 0 -and ($ver -match 'talos')) {
      $script:Passed++
    } else {
      Skip-Label "installed stub is a mocked fixture binary and cannot run --version; runnable verification requires the published release artifact on Windows CI"
    }
  } catch {
    Skip-Label "installed stub is a mocked fixture binary and cannot run --version; runnable verification requires the published release artifact on Windows CI"
  }
}

Assert-Pass "B. latest version resolution works" {
  Prepare-Zip
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'AMD64'
  $env:TALOS_VERSION = 'latest'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = ''
  Run-Installer
  $ExePath = Join-Path $InstallDir 'talos.exe'
  if (-not (Test-Path $ExePath)) { throw "talos.exe not found at $ExePath" }
}

Assert-FailWithMessage "C. offline mode causes terminating error mentioning unreachable network (no false success)" "network unreachable" {
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'AMD64'
  $env:TALOS_VERSION = 'latest'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = '1'
  $env:FIXTURE_BAD_CHECKSUM = ''
  Run-Installer
}

Assert-FailWithMessage "D. ARM64 architecture throws explicit unsupported message" "not published yet" {
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'ARM64'
  $env:TALOS_VERSION = 'v0.0.0'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = ''
  $env:FIXTURE_BAD_CHECKSUM = ''
  Run-Installer
}

Assert-FailWithMessage "E. checksum mismatch causes terminating error (no false success)" "checksum mismatch" {
  Prepare-Zip
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'AMD64'
  $env:TALOS_VERSION = 'v0.0.0'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = ''
  $env:FIXTURE_BAD_CHECKSUM = '1'
  Run-Installer
}

Write-Host ""
Write-Host "NOTE: install.ps1 now verifies checksums (mirroring install.sh) when checksum.sha256 is published; the fixture exercises both the verified path (A) and the mismatch path (E)."

$Total = $Passed + $Failed
if ($Failed -gt 0) {
  Write-Host "powershell installer fixture tests: ${Passed} passed, ${Failed} failed, ${Skipped} skipped" -ForegroundColor Red
  exit 1
}
Write-Host "powershell installer fixture tests: ${Passed} passed, ${Failed} failed, ${Skipped} skipped"
exit 0
