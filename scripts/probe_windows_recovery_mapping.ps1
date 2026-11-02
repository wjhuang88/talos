# Native evidence only: this probe does not authorize history recovery.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw 'This probe requires native Windows.' }

$root = Join-Path ([IO.Path]::GetTempPath()) ('talos-recovery-mapping-' + [guid]::NewGuid())
[IO.Directory]::CreateDirectory($root) | Out-Null
$path = Join-Path $root 'source.bin'
$stream = $null
$mapping = $null
$view = $null
$lease = $null
try {
    [IO.File]::WriteAllBytes($path, [byte[]](65, 66, 67, 68))
    $stream = [IO.FileStream]::new($path, [IO.FileMode]::Open,
        [IO.FileAccess]::ReadWrite, [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)
    $mapping = [IO.MemoryMappedFiles.MemoryMappedFile]::CreateFromFile(
        $stream, $null, 0, [IO.MemoryMappedFiles.MemoryMappedFileAccess]::ReadWrite,
        [IO.HandleInheritability]::None, $true)
    $view = $mapping.CreateViewAccessor(0, 4, [IO.MemoryMappedFiles.MemoryMappedFileAccess]::ReadWrite)
    $stream.Dispose()
    $stream = $null
    # Only the writable view and mapping remain. Match Rust's READ-only sharing.
    $acquired = $false
    $openError = $null
    try {
        $lease = [IO.FileStream]::new($path, [IO.FileMode]::Open,
            [IO.FileAccess]::Read, [IO.FileShare]::Read, 1, [IO.FileOptions]::None)
        $acquired = $true
    } catch [IO.IOException] {
        $openError = $_.Exception.HResult
        if (($openError -band 0xffff) -ne 32) { throw }
    }
    $observedMutation = $false
    if ($acquired) {
        if ($lease.ReadByte() -ne 65) { throw 'Unexpected initial source byte.' }
        $view.Write(0, [byte]90)
        $view.Flush()
        $lease.Position = 0
        $observedMutation = $lease.ReadByte() -eq 90
        if (-not $observedMutation) { throw 'Mapped write visibility is inconclusive.' }
    }
    [ordered]@{
        platform = [Environment]::OSVersion.VersionString
        read_only_share_lease_acquired_with_existing_mapping = $acquired
        mapped_mutation_visible_through_lease = $observedMutation
        lease_open_error_hresult = $openError
        authorization = 'disabled; diagnostic evidence only'
    } | ConvertTo-Json
} finally {
    if ($null -ne $lease) { $lease.Dispose() }
    if ($null -ne $view) { $view.Dispose() }
    if ($null -ne $mapping) { $mapping.Dispose() }
    if ($null -ne $stream) { $stream.Dispose() }
    if (Test-Path -LiteralPath $root) { Remove-Item -LiteralPath $root -Recurse -Force }
}
