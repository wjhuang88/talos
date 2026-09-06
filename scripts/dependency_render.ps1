function Format-AuditCell($Value, $Format) {
    $text=([string]$Value).Replace("`r",'\r').Replace("`n",'\n').Replace("`t",'\t')
    if($Format -eq 'markdown') { $text=$text.Replace('|','\|') }
    return $text
}
function Write-AuditReport($Report, $Format) {
    $separator=if($Format -eq 'markdown'){' | '}else{"`t"}
    'status: '+(Format-AuditCell $Report.status $Format)+'; registry: '+(Format-AuditCell $Report.registry $Format)
    if($Report.baseline) {
        'baseline: '+(Format-AuditCell $Report.baseline.source_commit $Format)+'; accepted-at: '+(Format-AuditCell $Report.baseline.accepted_at $Format)
    }
    ''
    @('name','source','kind','target','manifest','used-by','resolved','accepted','latest-stable','classification','deprecated','security') -join $separator
    if($Format -eq 'markdown') { (@('---')*12) -join $separator }
    foreach($row in $Report.dependencies) {
        $cells=@()
        foreach($field in @('name','source','kind','target','manifest_requirements','used_by','resolved_versions','accepted_versions','latest_stable','classification')) {
            $cells+=(@($row.$field | ForEach-Object { Format-AuditCell $_ $Format }) -join ',')
        }
        $cells+=(Format-AuditCell $row.signals.deprecated $Format)
        $cells+=(Format-AuditCell $row.signals.security $Format)
        $cells -join $separator
    }
}
