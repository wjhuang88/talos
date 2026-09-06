function field(id,key) { return jo[id SUBSEP key] }
function val(id,key) { return jv[field(id,key)] }
function scalar(id,key,    n) {
    n=field(id,key); if(!n) jfail("missing metadata field " key)
    return jr[n]
}
function quote(s,    i,c,r) {
    r="\""
    for(i=1;i<=length(s);i++) {
        c=substr(s,i,1)
        if(c=="\\" || c=="\"") r=r "\\" c
        else if(c=="\n") r=r "\\n"
        else if(c=="\r") r=r "\\r"
        else if(c=="\t") r=r "\\t"
        else if(c ~ /[[:cntrl:]]/) jfail("unsupported control in output")
        else r=r c
    }
    return r "\""
}
function sorted_set(values,    a,n,k,i,j,tmp,out) {
    n=split(values,a,"\034")
    for(i=2;i<=n;i++) {
        tmp=a[i];j=i-1
        while(j>0 && a[j]>tmp) { a[j+1]=a[j];j-- }
        a[j+1]=tmp
    }
    out="["; k=0
    for(i=1;i<=n;i++) if(a[i]!="" && (i==1 || a[i]!=a[i-1])) {
        if(k++) out=out ","
        out=out quote(a[i])
    }
    return out "]"
}
END {
    if(jfailed) exit 2
    if(val(jroot,"version")!="1" || jt[field(jroot,"resolve")]!="object") jfail("unsupported or unresolved metadata")
    packages=field(jroot,"packages"); members=field(jroot,"workspace_members")
    nodes=field(field(jroot,"resolve"),"nodes")
    if(jt[packages]!="array" || jt[members]!="array" || jt[nodes]!="array") jfail("missing metadata collections")
    for(p=jf[packages];p;p=jn[p]) {
        id=val(p,"id"); if(id=="" || id in pkg) jfail("missing or duplicate package identity")
        pkg[id]=p
    }
    for(n=jf[nodes];n;n=jn[n]) {
        id=val(n,"id"); if(id=="" || id in node) jfail("missing or duplicate resolve identity")
        node[id]=n
    }
    for(m=jf[members];m;m=jn[m]) {
        p=pkg[jv[m]]; n=node[jv[m]]
        if(!p || !n) jfail("member package or resolve node absent")
        for(d=jf[field(p,"dependencies")];d;d=jn[d]) {
            if(jt[field(d,"source")]=="null") continue
            if(jt[field(d,"source")]!="string") jfail("invalid dependency source")
            source=val(d,"source")
            name=val(d,"name"); req=val(d,"req")
            kind=scalar(d,"kind"); target=scalar(d,"target")
            key=source "|" name "|" req "|" kind "|" target
            if(!(key in row)) { row[key]=++count; order[count]=key }
            r=row[key]; rname[r]=name; rsource[r]=source; rreq[r]=req; rkind[r]=kind; rtarget[r]=target
            users[r]=users[r] "\034" val(p,"name")
            alias=val(d,"rename"); if(alias=="") alias=name
            gsub(/-/,"_",alias)
            for(e=jf[field(n,"deps")];e;e=jn[e]) {
                if(val(e,"name")!=alias) continue
                resolved=pkg[val(e,"pkg")]; if(!resolved) jfail("resolved package absent")
                if(val(resolved,"name")!=name || val(resolved,"source")!=source) continue
                for(k=jf[field(e,"dep_kinds")];k;k=jn[k]) {
                    if(scalar(k,"kind")==kind && scalar(k,"target")==target) {
                        versions[r]=versions[r] "\034" val(resolved,"version"); break
                    }
                }
            }
        }
    }
    for(i=2;i<=count;i++) {
        tmp=order[i]; j=i-1
        while(j>0 && order[j]>tmp) { order[j+1]=order[j];j-- }
        order[j+1]=tmp
    }
    printf "{\"schema\":\"talos.dependency-audit/v1\",\"status\":\"local-only\",\"registry\":\"not-queried\",\"dependencies\":["
    for(i=1;i<=count;i++) {
        r=row[order[i]]; if(i>1) printf ","
        printf "{\"name\":%s,\"source\":%s,\"manifest_requirements\":[%s],\"kind\":%s,\"target\":%s,\"used_by\":%s,\"resolved_versions\":%s,\"accepted_version\":null,\"latest_stable\":null,\"classification\":[\"registry-unavailable\"],\"signals\":{\"deprecated\":\"unknown\",\"security\":\"unknown\"}}", quote(rname[r]),quote(rsource[r]),quote(rreq[r]),rkind[r],rtarget[r],sorted_set(users[r]),sorted_set(versions[r])
    }
    print "]}"
}
