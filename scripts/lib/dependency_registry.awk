# Read crates.io version evidence, not crate.newest_version (may be prerelease).
END {
    if(jfailed) exit 2
    versions=jo[jroot SUBSEP "versions"]
    if(jt[versions]!="array" || !jf[versions]) jfail("registry response has no version evidence")
    latest=""; latestRaw="null"
    for(item=jf[versions];item;item=jn[item]) {
        number=jo[item SUBSEP "num"]; yanked=jo[item SUBSEP "yanked"]
        if(jt[number]!="string" || jt[yanked]!="boolean") jfail("invalid registry version evidence")
        if(!version_parse(jv[number],candidate)) jfail("invalid registry SemVer")
        if(jv[yanked]=="true" || candidate["pre"]!="") continue
        if(latest=="" || version_compare(candidate,best)>0) {
            latest=jv[number]; latestRaw=jr[number]
            version_parse(latest,best)
        }
    }
    printf "{\"latest_stable\":%s,\"yanked_versions\":[",latestRaw
    first=1
    for(item=jf[versions];item;item=jn[item]) {
        if(jv[jo[item SUBSEP "yanked"]]!="true") continue
        if(!first) printf ","; first=0
        printf "%s",jr[jo[item SUBSEP "num"]]
    }
    print "]}"
}
