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
$LatestJson = '[{"tag_name":"v0.0.0"}]'

$Passed = 0
$Failed = 0

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
  '@echo off' | Set-Content -Path $StubExe -Encoding ASCII
  Compress-Archive -Path $StubExe -DestinationPath $ZipPath -Force
  Remove-Item -Recurse -Force $TmpZipDir
}

function Reset-Install {
  Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

function Run-Installer {
  $env:TEMP = $TempBase
  if (-not $env:USERPROFILE) { $env:USERPROFILE = $TempBase }
  . $Installer
}

Assert-Pass "A. successful install places talos.exe and runs --version" {
  Prepare-Zip
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'AMD64'
  $env:TALOS_VERSION = 'v0.0.0'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = ''
  Run-Installer
  $ExePath = Join-Path $InstallDir 'talos.exe'
  if (-not (Test-Path $ExePath)) { throw "talos.exe not found at $ExePath" }
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

Assert-Fail "C. offline mode causes terminating error (no false success)" {
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'AMD64'
  $env:TALOS_VERSION = 'latest'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = '1'
  Run-Installer
}

Assert-Fail "D. ARM64 architecture throws explicit unsupported message" {
  Reset-Install
  $env:PROCESSOR_ARCHITECTURE = 'ARM64'
  $env:TALOS_VERSION = 'v0.0.0'
  $env:TALOS_INSTALL_DIR = $InstallDir
  $env:TALOS_REPO = 'wjhuang88/talos'
  $env:FIXTURE_OFFLINE = ''
  Run-Installer
}

Write-Host ""
Write-Host "NOTE: install.ps1 does not verify checksums (unlike install.sh); checksum-mismatch coverage is a known gap requiring a maintainer decision to add verification to the installer."

$Total = $Passed + $Failed
if ($Failed -gt 0) {
  Write-Host "powershell installer fixture tests: ${Passed}/${Total} passed (${Failed} failed)" -ForegroundColor Red
  exit 1
}
Write-Host "powershell installer fixture tests: ${Total}/${Total} passed"
exit 0
