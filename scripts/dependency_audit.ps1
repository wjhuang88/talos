param([ValidateSet('table','json','markdown')][string]$Format = 'table', [switch]$Live)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$metadata = (& cargo metadata --locked --offline --all-features --format-version 1 --manifest-path (Join-Path $root 'Cargo.toml')) | ConvertFrom-Json
$memberIds = @($metadata.workspace_members)
$packages = @($metadata.packages | Where-Object { $_.id -in $memberIds })
$byId = @{}; foreach ($p in $metadata.packages) { $byId[$p.id] = $p }
$rows = @{}
foreach ($p in $packages) {
  foreach ($d in $p.dependencies) {
    if ($d.source -like 'registry+*') {
      $key = "$($d.name)|$($d.req)|$($d.kind)|$($d.target)"
      if (-not $rows.ContainsKey($key)) { $rows[$key] = [ordered]@{name=$d.name; manifest_requirements=@($d.req); used_by=@(); resolved_versions=@(); accepted_version=$null; latest_stable=$null; classification=@('registry-unavailable')} }
      $rows[$key].used_by += $p.name
    }
  }
}
foreach ($node in @($metadata.resolve.nodes | Where-Object { $_.id -in $memberIds })) { foreach ($dep in $node.deps) { if ($byId.ContainsKey($dep.pkg) -and $byId[$dep.pkg].source -like 'registry+*') { $name=$byId[$dep.pkg].name; foreach ($r in $rows.Values) { if ($r.name -eq $name -and $r.resolved_versions -notcontains $byId[$dep.pkg].version) { $r.resolved_versions += $byId[$dep.pkg].version } } } } }
$out = [ordered]@{schema='talos.dependency-audit/v1'; status='local-only'; registry=($(if($Live){'queried'}else{'not-queried'})); dependencies=@($rows.Values | Sort-Object name)}
if ($Live) { foreach ($r in $out.dependencies) { try { Invoke-RestMethod -Uri "https://crates.io/api/v1/crates/$($r.name)" -TimeoutSec 10 | Out-Null } catch { Write-Error "registry-unavailable: $($r.name)"; exit 3 } } }
if ($Format -eq 'json') { [pscustomobject]$out | ConvertTo-Json -Compress -Depth 8 } else { 'name`tmanifest`tused-by`tresolved`tclassification'; $out.dependencies | ForEach-Object { "$($_.name)`t$($_.manifest_requirements -join ',')`t$($_.used_by -join ',')`t$($_.resolved_versions -join ',')`t$($_.classification -join ',')" } }
