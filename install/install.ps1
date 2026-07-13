# Talos release installer for Windows.
#
# Usage:
#   iex (irm https://raw.githubusercontent.com/wjhuang88/talos/main/install/install.ps1)
#
# Environment overrides:
#   $env:TALOS_REPO         GitHub <owner>/<repo>   (default: wjhuang88/talos)
#   $env:TALOS_VERSION      release tag or 'latest' (default: latest)
#   $env:TALOS_INSTALL_DIR  install directory       (default: %USERPROFILE%\.talos\bin)
$ErrorActionPreference = 'Stop'

$Repo = if ($env:TALOS_REPO) { $env:TALOS_REPO } else { 'wjhuang88/talos' }
$Version = if ($env:TALOS_VERSION) { $env:TALOS_VERSION } else { 'latest' }

$Arch = switch ($env:PROCESSOR_ARCHITECTURE) {
  'AMD64' { 'x86_64' }
  'ARM64' { throw 'Windows ARM64 release artifacts are not published yet. Use the x86_64 installer from an x64 PowerShell session or install manually.' }
  default { throw "unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}
$Target = "$Arch-windows"
$Archive = "talos-$Target.zip"

# GitHub's /releases/latest excludes prereleases, so for a prerelease-only
# project the "latest/download" shortcut 404s. Resolve the newest release tag
# (prereleases included) via the API instead.
if ($Version -eq 'latest') {
  $rel = @(Invoke-RestMethod -UseBasicParsing -Uri "https://api.github.com/repos/$Repo/releases?per_page=1")
  if (-not $rel -or -not $rel[0].tag_name) {
    throw "unable to resolve latest release tag for $Repo"
  }
  $Version = $rel[0].tag_name
}
$Base = "https://github.com/$Repo/releases/download/$Version"

$InstallDir = if ($env:TALOS_INSTALL_DIR) { $env:TALOS_INSTALL_DIR } else { Join-Path $env:USERPROFILE '.talos\bin' }
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$TmpDir = Join-Path $env:TEMP "talos-install-$([System.Guid]::NewGuid())"
New-Item -ItemType Directory -Path $TmpDir | Out-Null

try {
  Write-Host "-> downloading talos $Version ($Target)"
  $ArchivePath = Join-Path $TmpDir $Archive
  Invoke-WebRequest -UseBasicParsing -Uri "$Base/$Archive" -OutFile $ArchivePath

  # best-effort checksum verification (mirrors install.sh against the release checksum.sha256)
  $ChecksumPath = Join-Path $TmpDir 'checksum.sha256'
  try {
    Invoke-WebRequest -UseBasicParsing -Uri "$Base/checksum.sha256" -OutFile $ChecksumPath -ErrorAction Stop
  } catch {
    $ChecksumPath = $null
  }
  if ($ChecksumPath -and (Test-Path $ChecksumPath)) {
    $Expected = $null
    foreach ($line in (Get-Content $ChecksumPath)) {
      $line = $line.Trim()
      if (-not $line) { continue }
      $parts = $line -split '\s+'
      if ($parts.Count -ge 2) {
        $name = $parts[1]
        if ($name.StartsWith('*')) { $name = $name.Substring(1) }
        if ($name -eq $Archive) { $Expected = $parts[0]; break }
      }
    }
    if ($Expected) {
      $Actual = (Get-FileHash -Algorithm SHA256 -Path $ArchivePath).Hash.ToLower()
      if ($Actual -ne $Expected.ToLower()) {
        throw "checksum mismatch for $Archive (expected $Expected, got $Actual)"
      }
      Write-Host "-> checksum verified"
    }
  } else {
    Write-Host "note: checksum.sha256 not available; skipping checksum verification"
  }

  Expand-Archive -Path $ArchivePath -DestinationPath $TmpDir -Force
  Move-Item -Path (Join-Path $TmpDir 'talos.exe') -Destination (Join-Path $InstallDir 'talos.exe') -Force
} finally {
  Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}

Write-Host "-> installed talos to $(Join-Path $InstallDir 'talos.exe')"
if (-not (($env:PATH -split ';') -contains $InstallDir)) {
  Write-Host "note: add $InstallDir to your PATH"
}
& (Join-Path $InstallDir 'talos.exe') --version 2>$null
