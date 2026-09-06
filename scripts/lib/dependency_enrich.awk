function classify(row,registry,    entry,versionNode,num,yank,candidate,best,latest,latestRaw,versions,i,distance,classes,n,j,tmp,result,found) {
    enrichLatest="null"
    if(ev(row,"source")!="registry+https://github.com/rust-lang/crates.io-index") { partial=1; return "[\"unknown\"]" }
    entry=ef(registry,ev(row,"name"))
    if(!entry || jt[entry]=="null") { partial=1; return "[\"registry-unavailable\"]" }
    versions=ef(entry,"versions")
    if(jt[versions]!="array" || !jf[versions]) { partial=1; return "[\"unknown\"]" }
    latest=""; latestRaw="null"
    for(i=jf[versions];i;i=jn[i]) {
        num=ef(i,"num"); yank=ef(i,"yanked")
        if(jt[num]!="string" || jt[yank]!="boolean" || !version_parse(jv[num],candidate)) { partial=1; return "[\"unknown\"]" }
        if(jv[yank]=="true" || candidate["pre"]!="") continue
        if(latest=="" || version_compare(candidate,best)>0) {
            latest=jv[num]; latestRaw=jr[num]; version_parse(latest,best)
        }
    }
    enrichLatest=latestRaw
    if(latest=="" || !jf[ef(row,"resolved_versions")]) { partial=1; return "[\"unknown\"]" }
    for(i=jf[ef(row,"resolved_versions")];i;i=jn[i]) {
        distance=version_distance(jv[i],latest)
        if(distance=="unknown") partial=1
        found=0; for(j=1;j<=n;j++) if(classes[j]==distance) found=1
        if(!found) classes[++n]=distance
    }
    for(i=2;i<=n;i++) { tmp=classes[i];j=i-1;while(j>0 && classes[j]>tmp){classes[j+1]=classes[j];j--}classes[j+1]=tmp }
    result="["
    for(i=1;i<=n;i++) result=result (i>1 ? "," : "") "\"" classes[i] "\""
    found=0
    for(i=jf[versions];i;i=jn[i]) if(ev(i,"yanked")=="true") {
        for(j=jf[ef(row,"resolved_versions")];j;j=jn[j]) if(jv[j]==ev(i,"num")) found=1
    }
    if(found) result=result ",\"yanked\""
    return result "]"
}
END {
    if(jfailed) exit 2
    audit=ef(jroot,"audit"); registry=ef(jroot,"registry")
    if(jt[registry]!="object" || jt[audit]!="object") jfail("expected audit and registry objects")
    for(row=jf[ef(audit,"dependencies")];row;row=jn[row]) {
        classification=classify(row,registry)
        jr[ef(row,"classification")]=classification; jt[ef(row,"classification")]="scalar"
        jr[ef(row,"latest_stable")]=enrichLatest; jt[ef(row,"latest_stable")]="scalar"
    }
    jr[ef(audit,"registry")]=(registry_mode=="queried" ? "\"queried\"" : "\"fixture\"")
    jr[ef(audit,"status")]=(partial ? "\"partial\"" : "\"audited\"")
    print emit(audit)
}
