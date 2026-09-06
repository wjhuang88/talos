# Pure version operations: no Cargo calls, registry access or baseline mutation.
function ConvertTo-AuditVersion([string]$Text) {
    if ($Text -cnotmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$') {
        throw "invalid SemVer: $Text"
    }
    $major = [System.Numerics.BigInteger]::Parse($Matches[1])
    $minor = [System.Numerics.BigInteger]::Parse($Matches[2])
    $patch = [System.Numerics.BigInteger]::Parse($Matches[3])
    $pre = $Matches[4]
    foreach ($part in @($pre -split '\.')) {
        if ($part -match '^0[0-9]+$') { throw "invalid numeric prerelease: $Text" }
    }
    [pscustomobject]@{ Text=$Text; Major=$major; Minor=$minor; Patch=$patch; Pre=$pre }
}

function Compare-AuditVersion($A, $B) {
    foreach ($field in @('Major','Minor','Patch')) {
        $c = $A.$field.CompareTo($B.$field)
        if ($c -ne 0) { return $c }
    }
    if (-not $A.Pre -and -not $B.Pre) { return 0 }
    if (-not $A.Pre) { return 1 }
    if (-not $B.Pre) { return -1 }
    $aa = $A.Pre.Split('.'); $bb = $B.Pre.Split('.')
    for ($i=0; $i -lt [Math]::Min($aa.Length,$bb.Length); $i++) {
        $an = $aa[$i] -match '^[0-9]+$'; $bn = $bb[$i] -match '^[0-9]+$'
        if ($an -and $bn) {
            $c = ([System.Numerics.BigInteger]::Parse($aa[$i])).CompareTo([System.Numerics.BigInteger]::Parse($bb[$i]))
        } elseif ($an) { $c=-1 } elseif ($bn) { $c=1 }
        else { $c=[string]::CompareOrdinal($aa[$i],$bb[$i]) }
        if ($c -ne 0) { return $c }
    }
    return $aa.Length.CompareTo($bb.Length)
}

function Get-AuditDistance([string]$Current, [string]$Latest) {
    try { $a=ConvertTo-AuditVersion $Current; $b=ConvertTo-AuditVersion $Latest }
    catch { return 'unknown' }
    if ($a.Pre -or $b.Pre) { return 'prerelease' }
    $order=Compare-AuditVersion $a $b
    if ($order -eq 0) { return 'current' }
    if ($order -gt 0) { return 'ahead-of-stable' }
    if ($a.Major -ne $b.Major) { return 'major' }
    if ($a.Minor -ne $b.Minor) {
        if ($a.Major -eq 0) { return 'pre-1.0-breaking-minor' }
        return 'minor'
    }
    if ($a.Major -eq 0 -and $a.Minor -eq 0) { return 'pre-1.0-breaking-patch' }
    return 'patch'
}

function Get-AuditLatestStable($Response) {
    if ($null -eq $Response.versions -or @($Response.versions).Count -eq 0) {
        throw 'registry response has no version evidence'
    }
    $best=$null
    foreach ($item in $Response.versions) {
        if ($item.yanked -isnot [bool]) { throw 'registry response lacks explicit yanked state' }
        $v=ConvertTo-AuditVersion $item.num
        if ($item.yanked -or $v.Pre) { continue }
        if ($null -eq $best -or (Compare-AuditVersion $v $best) -gt 0) { $best=$v }
    }
    if ($null -eq $best) { return $null }
    return $best.Text
}
