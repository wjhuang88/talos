# Read-only comparison. Baseline generation/acceptance is a separate governed action.
function Get-BaselineIdentity($Row) {
    # JSON tuple avoids collisions in source URLs, target expressions and requirements.
    ConvertTo-Json -InputObject @($Row.name,$Row.source,$Row.kind,$Row.target,@($Row.manifest_requirements | Sort-Object -Unique)) -Depth 8 -Compress
}
function Get-BaselineSet($Values) {
    ConvertTo-Json -InputObject @($Values | Sort-Object -Unique) -Compress
}
function Compare-AuditBaseline($Report, $Baseline) {
    if ($Baseline.schema -cne 'talos.dependency-baseline/v1' -or
        $Baseline.source_commit -cnotmatch '^[0-9a-f]{40}$' -or
        $Baseline.accepted_at -notmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$' -or
        -not $Baseline.validation_evidence -or $null -eq $Baseline.dependencies) {
        throw 'invalid accepted baseline provenance or schema'
    }
    $accepted=@{}
    foreach ($row in $Baseline.dependencies) {
        if (-not $row.name -or -not $row.source -or
            $null -eq $row.manifest_requirements -or $null -eq $row.resolved_versions -or
            $null -eq $row.used_by) { throw 'invalid baseline dependency' }
        $key=Get-BaselineIdentity $row
        if($accepted.ContainsKey($key)) { throw 'duplicate baseline identity' }
        $accepted[$key]=$row
    }
    $seen=@{}
    foreach ($row in $Report.dependencies) {
        $key=Get-BaselineIdentity $row
        $old=$accepted[$key]
        $drift=($null -eq $old)
        if($null -ne $old) {
            $drift=((Get-BaselineSet $row.resolved_versions) -cne (Get-BaselineSet $old.resolved_versions)) -or
                   ((Get-BaselineSet $row.used_by) -cne (Get-BaselineSet $old.used_by))
        }
        $row | Add-Member -NotePropertyName accepted_versions -NotePropertyValue @($(if($old){$old.resolved_versions})) -Force
        $row | Add-Member -NotePropertyName baseline_drift -NotePropertyValue $drift -Force
        if($drift) { $row.classification=@($row.classification)+@('baseline-drift') }
        $seen[$key]=$true
    }
    # Preserve the accepted snapshot's order in both frontends.
    $removed=@($Baseline.dependencies | Where-Object { -not $seen.ContainsKey((Get-BaselineIdentity $_)) })
    $Report.baseline=[ordered]@{
        source_commit=$Baseline.source_commit
        accepted_at=$Baseline.accepted_at
        validation_evidence=$Baseline.validation_evidence
        removed_dependencies=$removed
        drift=($removed.Count -gt 0 -or @($Report.dependencies | Where-Object baseline_drift).Count -gt 0)
    }
}
