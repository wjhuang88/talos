param([ValidateSet('table','json','markdown')][string]$Format = 'table', [switch]$Live,
      [string]$MetadataPath, [string]$RegistryPath, [string]$BaselinePath,
      [switch]$Snapshot, [string]$SourceCommit, [string]$AcceptedAt, [string]$ValidationEvidence)
$ErrorActionPreference = 'Stop'
if($Snapshot -and ($Live -or $RegistryPath -or $BaselinePath -or $Format -ne 'json')) { throw 'snapshot requires JSON format and local collection only' }
if(-not $Snapshot -and ($SourceCommit -or $AcceptedAt -or $ValidationEvidence)) { throw 'snapshot provenance requires -Snapshot' }
. (Join-Path $PSScriptRoot 'dependency_versions.ps1')
. (Join-Path $PSScriptRoot 'dependency_baseline.ps1')
. (Join-Path $PSScriptRoot 'dependency_render.ps1')
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
if ($MetadataPath) {
  $metadataText = Get-Content -Raw -LiteralPath $MetadataPath
} else {
  $metadataText = & cargo metadata --locked --offline --all-features --format-version 1 --manifest-path (Join-Path $root 'Cargo.toml')
  if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed (exit $LASTEXITCODE)" }
}
function Read-JsonText([string]$Text) { return ($Text | ConvertFrom-Json) }
$metadata = Read-JsonText $metadataText
if ($metadata.version -ne 1 -or $null -eq $metadata.resolve) { throw 'unsupported or unresolved Cargo metadata' }
if ($metadata.packages -isnot [array] -or $metadata.workspace_members -isnot [array] -or
    $metadata.resolve.nodes -isnot [array]) { throw 'missing metadata collections' }
$memberIds = @($metadata.workspace_members)
$packages = @($metadata.packages | Where-Object { $_.id -in $memberIds })
$byId = @{}; foreach ($p in $metadata.packages) {
  if (-not $p.id -or $byId.ContainsKey($p.id)) { throw 'missing or duplicate package identity' }
  $byId[$p.id] = $p
}
$nodeById = @{}; foreach ($n in $metadata.resolve.nodes) {
  if (-not $n.id -or $nodeById.ContainsKey($n.id)) { throw 'missing or duplicate resolve identity' }
  $nodeById[$n.id] = $n
}
foreach ($id in $memberIds) {
  if (-not $byId.ContainsKey($id) -or -not $nodeById.ContainsKey($id)) { throw 'member package or resolve node absent' }
}
$rows = @{}
foreach ($p in $packages) {
  foreach ($d in $p.dependencies) {
    if ($null -ne $d.source) {
      if ($d.source -isnot [string]) { throw 'invalid dependency source' }
      $key = "$($d.source)|$($d.name)|$($d.req)|$($d.kind)|$($d.target)"
      if (-not $rows.ContainsKey($key)) { $rows[$key] = [ordered]@{name=$d.name; source=$d.source; manifest_requirements=@($d.req); kind=$d.kind; target=$d.target; used_by=@(); resolved_versions=@(); accepted_version=$null; latest_stable=$null; classification=@('registry-unavailable'); signals=[ordered]@{deprecated='unknown';security='unknown'} } }
      $rows[$key].used_by += $p.name
      $alias = if ($null -ne $d.rename) { $d.rename } else { $d.name }
      $alias = $alias.Replace('-', '_')
      foreach ($edge in $nodeById[$p.id].deps) {
        if ($edge.name -ne $alias) { continue }
        $resolved = $byId[$edge.pkg]
        if ($null -eq $resolved) { throw 'resolved package absent' }
        if ($resolved.name -ne $d.name -or $resolved.source -ne $d.source) { continue }
        $matchingKinds = @($edge.dep_kinds | Where-Object {
          $_.kind -eq $d.kind -and $_.target -eq $d.target
        })
        if ($matchingKinds.Count -gt 0) { $rows[$key].resolved_versions += $resolved.version }
      }
    }
  }
}
foreach ($r in $rows.Values) {
  $r.used_by = @($r.used_by | Sort-Object -Unique)
  $r.resolved_versions = @($r.resolved_versions | Sort-Object -Unique)
}
# Sort explicit objects, not OrderedDictionary adapters, whose property lookup
# otherwise leaves the hashtable enumeration order visible in serialized JSON.
# Use ordinal byte-like ordering so empty kind/target fields sort identically to
# the Bash frontend (culture-aware Sort-Object reverses `dev` and empty kinds).
$sortedRows=[System.Collections.Generic.SortedDictionary[string,object]]::new([System.StringComparer]::Ordinal)
foreach($key in $rows.Keys) { $sortedRows[$key]=[pscustomobject]$rows[$key] }
$out = [ordered]@{schema='talos.dependency-audit/v1'; status='local-only'; registry=($(if($Live){'queried'}else{'not-queried'})); dependencies=@($sortedRows.Values)}
if($Snapshot) {
  if($SourceCommit -cnotmatch '^[0-9a-f]{40}$' -or $AcceptedAt -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$' -or [string]::IsNullOrWhiteSpace($ValidationEvidence)) { throw 'snapshot requires source SHA, UTC timestamp and validation evidence' }
  # This is generation, not validation or authorization to advance the baseline.
  [pscustomobject][ordered]@{schema='talos.dependency-baseline/v1';source_commit=$SourceCommit;accepted_at=$AcceptedAt;validation_evidence=$ValidationEvidence;dependencies=$out.dependencies}|ConvertTo-Json -Depth 20 -Compress
  return
}
if ($Live -or $RegistryPath) {
  if ($Live -and $RegistryPath) { throw 'Live and RegistryPath are mutually exclusive' }
  $out.registry=if($RegistryPath){'fixture'}else{'queried'}
  $out.status='audited'
  $registryFixture=if($RegistryPath){Read-JsonText (Get-Content -Raw -LiteralPath $RegistryPath)}else{$null}
  $registryCache=@{}
  foreach ($r in $out.dependencies) {
    if ($r.source -ne 'registry+https://github.com/rust-lang/crates.io-index') {
      $r.classification=@('unknown'); $out.status='partial'; continue
    }
    try {
      if (-not $registryCache.ContainsKey($r.name)) {
        try {
          if($RegistryPath) { $registryCache[$r.name]=$registryFixture.($r.name) }
          else { $registryCache[$r.name] = Invoke-RestMethod -Uri "https://crates.io/api/v1/crates/$($r.name)" -TimeoutSec 10 }
        }
        catch { $registryCache[$r.name]=$null }
      }
      $response=$registryCache[$r.name]
      if ($null -eq $response) { $r.classification=@('registry-unavailable'); $out.status='partial'; continue }
      $latest=Get-AuditLatestStable $response
      $r.latest_stable=$latest
      if (-not $latest -or $r.resolved_versions.Count -eq 0) { $r.classification=@('unknown'); $out.status='partial'; continue }
      $r.classification=@($r.resolved_versions | ForEach-Object { Get-AuditDistance $_ $latest } | Sort-Object -Unique)
      if($r.classification -contains 'unknown') { $out.status='partial' }
      if (@($response.versions | Where-Object { $_.yanked -and $_.num -in $r.resolved_versions }).Count -gt 0) {
        $r.classification += 'yanked'
      }
    } catch { $r.classification=@('unknown'); $out.status='partial' }
  }
}
if ($BaselinePath) {
  # Preserve the provenance timestamp as JSON text, not a locale-formatted DateTime.
  # ConvertFrom-Json on Windows PowerShell 5.1 has no DateKind switch; restore the
  # exact ISO token from the source text for both supported PowerShell generations.
  $baselineText=Get-Content -LiteralPath $BaselinePath -Raw
  $baseline=$baselineText | ConvertFrom-Json
  $timestampMatch=[regex]::Match($baselineText,'"accepted_at"\s*:\s*"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z)"')
  if($timestampMatch.Success) { $baseline.accepted_at=$timestampMatch.Groups[1].Value }
  Compare-AuditBaseline $out $baseline
}
if ($Format -eq 'json') { [pscustomobject]$out | ConvertTo-Json -Compress -Depth 12 } else { Write-AuditReport $out $Format }
