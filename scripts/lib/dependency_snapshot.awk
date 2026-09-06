function snapshot_quote(s,    i,c,r) {
    r="\""
    for(i=1;i<=length(s);i++) {
        c=substr(s,i,1)
        if(c=="\\" || c=="\"") r=r "\\" c
        else if(c=="\n") r=r "\\n"
        else if(c=="\r") r=r "\\r"
        else if(c=="\t") r=r "\\t"
        else if(c ~ /[[:cntrl:]]/) jfail("unsupported control in snapshot provenance")
        else r=r c
    }
    return r "\""
}
END {
    if(jfailed) exit 2
    source=ENVIRON["TALOS_SNAPSHOT_SOURCE"]; time=ENVIRON["TALOS_SNAPSHOT_TIME"]; evidence=ENVIRON["TALOS_SNAPSHOT_EVIDENCE"]
    if(length(source)!=40 || source !~ /^[0-9a-f]+$/ ||
       time !~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z$/ || evidence !~ /[^[:space:]]/) jfail("snapshot requires source SHA, UTC timestamp and validation evidence")
    # Generation attests nothing: callers must verify provenance before accepting it.
    printf "{\"schema\":\"talos.dependency-baseline/v1\",\"source_commit\":%s,\"accepted_at\":%s,\"validation_evidence\":%s,\"dependencies\":%s}\n",snapshot_quote(source),snapshot_quote(time),snapshot_quote(evidence),emit(ef(jroot,"dependencies"))
}
