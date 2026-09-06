param([Parameter(Mandatory=$true)][string]$ObservationPath)
$ErrorActionPreference='Stop'
$observation=Get-Content -Raw -LiteralPath $ObservationPath|ConvertFrom-Json
if($observation.schema -cne 'talos.dependency-observation/v1' -or $observation.source_commit -cnotmatch '^[0-9a-f]{40}$' -or $observation.report.schema -cne 'talos.dependency-audit/v1') { throw 'invalid audit observation' }
$candidates=@()
foreach($group in ($observation.report.dependencies|Group-Object name|Sort-Object Name)) {
    $rows=@($group.Group)
    $classes=@($rows.classification|Sort-Object -Unique)
    if($classes.Count -eq 1 -and $classes[0] -eq 'current') { continue }
    $versions=@($rows.resolved_versions|Sort-Object -Unique)
    $targets=@($rows.latest_stable|Where-Object {$_}|Sort-Object -Unique)
    $consumers=@($rows.used_by|Sort-Object -Unique)
    $sources=@($rows.source|Sort-Object -Unique)
    $unknown=($classes -contains 'unknown' -or $classes -contains 'registry-unavailable' -or $targets.Count -ne 1 -or $sources.Count -ne 1)
    $isolated=($unknown -or @($classes|Where-Object {$_ -in @('major','pre-1.0-breaking-minor','pre-1.0-breaking-patch','prerelease','ahead-of-stable','yanked','baseline-drift')}).Count -gt 0)
    $candidates+=[pscustomobject][ordered]@{
        name=$group.Name;source=$sources;current_versions=$versions;target_versions=$targets
        classifications=$classes;direct_consumers=$consumers
        disposition=$(if($unknown){'investigate'}elseif($isolated){'isolated-proposal'}else{'domain-review-required'})
        authorization='none'
        required_next_step='Inspect actual consumer/API/native/security/MSRV impact; assign existing Story/iteration/claim before Cargo changes.'
        validation='Locked preflight plus consumer-domain and feature-matrix tests; exact-head authorization-specific review.'
        rollback='Revert complete candidate manifests, lock and migrations; assess data compatibility before implementation.'
    }
}
[pscustomobject][ordered]@{
    schema='talos.dependency-candidates/v1';source_commit=$observation.source_commit
    observed_date_utc=$observation.observed_date_utc
    policy='Latest stable, including majors. Compatible version distance is not a risk clearance. No candidate is implementation authority.'
    candidates=$candidates
}|ConvertTo-Json -Depth 20
