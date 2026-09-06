function table_cell(s) {
    gsub(/\r/,"\\r",s); gsub(/\n/,"\\n",s); gsub(/\t/,"\\t",s)
    gsub(/\|/,"\\|",s)
    return s
}
function table_list(id,    c,s) {
    for(c=jf[id];c;c=jn[c]) s=s (c==jf[id] ? "" : ", ") table_cell(jv[c])
    return s
}
END {
    if(jfailed) exit 2
    if(ev(jroot,"schema")!="talos.dependency-baseline/v1" || jt[ef(jroot,"dependencies")]!="array") jfail("expected accepted baseline snapshot")
    print "<!-- dependency-baseline:begin -->"
    print "| Name | Manifest | Accepted resolutions | Direct consumers | Kind | Target |"
    print "|---|---|---|---|---|---|"
    for(r=jf[ef(jroot,"dependencies")];r;r=jn[r]) {
        kind=ev(r,"kind"); target=ev(r,"target")
        print "| " table_cell(ev(r,"name")) " | " table_list(ef(r,"manifest_requirements")) " | " table_list(ef(r,"resolved_versions")) " | " table_list(ef(r,"used_by")) " | " (kind=="" ? "normal" : table_cell(kind)) " | " (target=="" ? "all" : table_cell(target)) " |"
    }
    print "<!-- dependency-baseline:end -->"
}
