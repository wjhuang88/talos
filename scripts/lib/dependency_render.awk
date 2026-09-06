function get(id,key) { return jo[id SUBSEP key] }
function cell(s) {
    gsub(/\r/,"\\r",s); gsub(/\n/,"\\n",s); gsub(/\t/,"\\t",s)
    if(output_format=="markdown") gsub(/\|/,"\\|",s)
    return s
}
function list(id,    n,s) {
    s=""
    for(n=jf[id];n;n=jn[n]) { if(s!="") s=s ","; s=s cell(jv[n]) }
    return s
}
END {
    if(jfailed) exit 2
    sep=(output_format=="markdown" ? " | " : "\t")
    print "status: " cell(jv[get(jroot,"status")]) "; registry: " cell(jv[get(jroot,"registry")])
    if(get(jroot,"baseline")) {
        b=get(jroot,"baseline")
        print "baseline: " cell(jv[get(b,"source_commit")]) "; accepted-at: " cell(jv[get(b,"accepted_at")])
    }
    print ""
    print "name" sep "source" sep "kind" sep "target" sep "manifest" sep "used-by" sep "resolved" sep "accepted" sep "latest-stable" sep "classification" sep "deprecated" sep "security"
    if(output_format=="markdown") print "--- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---"
    for(d=jf[get(jroot,"dependencies")];d;d=jn[d]) {
        print cell(jv[get(d,"name")]) sep cell(jv[get(d,"source")]) sep cell(jv[get(d,"kind")]) sep cell(jv[get(d,"target")]) sep list(get(d,"manifest_requirements")) sep list(get(d,"used_by")) sep list(get(d,"resolved_versions")) sep list(get(d,"accepted_versions")) sep cell(jv[get(d,"latest_stable")]) sep list(get(d,"classification")) sep cell(jv[get(get(d,"signals"),"deprecated")]) sep cell(jv[get(get(d,"signals"),"security")])
    }
}
