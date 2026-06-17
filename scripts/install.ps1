# Talos installer for Windows.
#
# Usage:
#   iex (irm https://<your-domain>/install.ps1)
#
# Environment overrides:
#   $env:TALOS_REPO         GitHub <owner>/<repo>   (default: wjhuang88/talos)
#   $env:TALOS_VERSION       release tag or 'latest' (default: latest)
#   $env:TALOS_INSTALL_DIR   install directory       (default: %USERPROFILE%\.talos\bin)
$ErrorActionPreference = 'Stop'

$Repo = if ($env:TALOS_REPO) { $env:TALOS_REPO } else { 'wjhuang88/talos' }
$Version = if ($env:TALOS_VERSION) { $env:TALOS_VERSION } else { 'latest' }

$Arch = switch ($env:PROCESSOR_ARCHITECTURE) {
  'AMD64' { 'x86_64' }
  'ARM64' { 'aarch64' }
  default { throw "unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}
$Target = "$Arch-pc-windows-msvc"
$Archive = "talos-$Target.zip"

$Base = if ($Version -eq 'latest') {
  "https://github.com/$Repo/releases/latest/download"
} else {
  "https://github.com/$Repo/releases/download/$Version"
}

$InstallDir = if ($env:TALOS_INSTALL_DIR) { $env:TALOS_INSTALL_DIR } else { Join-Path $env:USERPROFILE '.talos\bin' }
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$TmpDir = Join-Path $env:TEMP "talos-install-$([System.Guid]::NewGuid())"
New-Item -ItemType Directory -Path $TmpDir | Out-Null

try {
  Write-Host "-> downloading talos $Version ($Target)"
  Invoke-WebRequest -UseBasicParsing -Uri "$Base/$Archive" -OutFile (Join-Path $TmpDir $Archive)
  Expand-Archive -Path (Join-Path $TmpDir $Archive) -DestinationPath $TmpDir -Force
  Move-Item -Path (Join-Path $TmpDir 'talos.exe') -Destination (Join-Path $InstallDir 'talos.exe') -Force
} finally {
  Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}

Write-Host "-> installed talos to $(Join-Path $InstallDir 'talos.exe')"
if (-not (($env:PATH -split ';') -contains $InstallDir)) {
  Write-Host "note: add $InstallDir to your PATH"
}
& (Join-Path $InstallDir 'talos.exe') --version 2>$null
