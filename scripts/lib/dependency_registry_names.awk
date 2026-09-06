END {
    if(jfailed) exit 2
    deps=jo[jroot SUBSEP "dependencies"]
    for(row=jf[deps];row;row=jn[row]) {
        source=jv[jo[row SUBSEP "source"]]; name=jv[jo[row SUBSEP "name"]]
        if(source!="registry+https://github.com/rust-lang/crates.io-index") continue
        if(name !~ /^[A-Za-z0-9_-]+$/) jfail("unsafe registry package name")
        if(!(name in seen)) { print name; seen[name]=1 }
    }
}
