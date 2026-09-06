# SemVer operations keep numeric components as decimal strings to avoid awk's
# floating-point precision limit. Functions return empty/unknown on invalid input.
function version_parse(text,a,    key,s,n,p,parts,i) {
    for(key in a) delete a[key]
    s=text
    n=index(s,"+")
    if(n) {
        a["build"]=substr(s,n+1); s=substr(s,1,n-1)
        if(a["build"] !~ /^[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*$/) return 0
    }
    n=index(s,"-")
    if(n) {
        a["pre"]=substr(s,n+1); s=substr(s,1,n-1)
        if(a["pre"] !~ /^[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*$/) return 0
        p=split(a["pre"],parts,".")
        for(i=1;i<=p;i++) if(parts[i] ~ /^0[0-9]+$/) return 0
    }
    if(s !~ /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/) return 0
    split(s,parts,".")
    for(i=1;i<=3;i++) a[i]=parts[i]
    return 1
}
function decimal_compare(a,b) {
    if(length(a)!=length(b)) return length(a)>length(b) ? 1 : -1
    if(("x" a)==("x" b)) return 0
    return ("x" a)>("x" b) ? 1 : -1
}
function version_compare(a,b,    i,c,aa,bb,na,nb,an,bn) {
    for(i=1;i<=3;i++) { c=decimal_compare(a[i],b[i]); if(c) return c }
    if(a["pre"]=="" && b["pre"]=="") return 0
    if(a["pre"]=="") return 1
    if(b["pre"]=="") return -1
    na=split(a["pre"],aa,"."); nb=split(b["pre"],bb,".")
    for(i=1;i<=na && i<=nb;i++) {
        an=(aa[i] ~ /^[0-9]+$/); bn=(bb[i] ~ /^[0-9]+$/)
        if(an && bn) c=decimal_compare(aa[i],bb[i])
        else if(an) c=-1
        else if(bn) c=1
        else if(aa[i]==bb[i]) c=0
        else c=(aa[i]>bb[i] ? 1 : -1)
        if(c) return c
    }
    return na==nb ? 0 : (na>nb ? 1 : -1)
}
function version_distance(current,latest,    a,b,c) {
    if(!version_parse(current,a) || !version_parse(latest,b)) return "unknown"
    if(a["pre"]!="" || b["pre"]!="") return "prerelease"
    c=version_compare(a,b)
    if(!c) return "current"
    if(c>0) return "ahead-of-stable"
    if(decimal_compare(a[1],b[1])) return "major"
    if(decimal_compare(a[2],b[2])) return a[1]=="0" ? "pre-1.0-breaking-minor" : "minor"
    if(a[1]=="0" && a[2]=="0") return "pre-1.0-breaking-patch"
    return "patch"
}
