function canonical_set(id,    a,n,c,i,j,tmp,out) {
    if(jt[id]!="array") jfail("baseline set must be an array")
    for(c=jf[id];c;c=jn[c]) {
        if(jt[c]!="string") jfail("baseline set must contain strings")
        a[++n]=jv[c]
    }
    for(i=2;i<=n;i++){tmp=a[i];j=i-1;while(j>0 && a[j]>tmp){a[j+1]=a[j];j--}a[j+1]=tmp}
    out=""
    for(i=1;i<=n;i++) if(i==1 || a[i]!=a[i-1]) out=out length(a[i]) ":" a[i]
    return out
}
function identity(row,    i,id,key,s) {
    s=""
    for(i=1;i<=4;i++) {
        key=(i==1 ? "name" : (i==2 ? "source" : (i==3 ? "kind" : "target")))
        id=ef(row,key)
        if(!id) jfail("missing baseline identity field")
        s=s jt[id] ":" length(jv[id]) ":" jv[id]
    }
    return s canonical_set(ef(row,"manifest_requirements"))
}
END {
    if(jfailed) exit 2
    audit=ef(jroot,"audit"); baseline=ef(jroot,"baseline")
    sha=ev(baseline,"source_commit"); timestamp=ev(baseline,"accepted_at")
    if(ev(baseline,"schema")!="talos.dependency-baseline/v1" || length(sha)!=40 || sha !~ /^[0-9a-f]+$/ ||
       timestamp !~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z$/ ||
       ev(baseline,"validation_evidence")=="" || jt[ef(baseline,"dependencies")]!="array") jfail("invalid baseline provenance or schema")
    for(row=jf[ef(baseline,"dependencies")];row;row=jn[row]) {
        key=identity(row)
        if(key in accepted) jfail("duplicate baseline identity")
        accepted[key]=row
        canonical_set(ef(row,"resolved_versions")); canonical_set(ef(row,"used_by"))
    }
    rows="["; first=1; drift=0
    for(row=jf[ef(audit,"dependencies")];row;row=jn[row]) {
        key=identity(row); prior=accepted[key]; seen[key]=1
        changed=(!prior || canonical_set(ef(row,"resolved_versions"))!=canonical_set(ef(prior,"resolved_versions")) ||
            canonical_set(ef(row,"used_by"))!=canonical_set(ef(prior,"used_by")))
        if(changed) {
            drift=1; id=ef(row,"classification"); text=emit(id)
            jr[id]=substr(text,1,length(text)-1) (jl[id]>0 ? "," : "") "\"baseline-drift\"]"; jt[id]="scalar"
        }
        text=emit(row)
        text=substr(text,1,length(text)-1) ",\"accepted_versions\":" (prior ? emit(ef(prior,"resolved_versions")) : "[]") ",\"baseline_drift\":" (changed ? "true" : "false") "}"
        if(!first) rows=rows ",";first=0;rows=rows text
    }
    rows=rows "]"; removed="["; first=1
    for(row=jf[ef(baseline,"dependencies")];row;row=jn[row]) if(!(identity(row) in seen)) {
        if(!first) removed=removed ",";first=0;removed=removed emit(row);drift=1
    }
    removed=removed "]"
    id=ef(audit,"dependencies");jr[id]=rows;jt[id]="scalar"
    text=emit(audit)
    printf "%s,\"baseline\":{\"source_commit\":%s,\"accepted_at\":%s,\"validation_evidence\":%s,\"removed_dependencies\":%s,\"drift\":%s}}\n",substr(text,1,length(text)-1),emit(ef(baseline,"source_commit")),emit(ef(baseline,"accepted_at")),emit(ef(baseline,"validation_evidence")),removed,(drift ? "true" : "false")
}
