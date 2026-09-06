BEGIN { FS="\t" }
{
    if(NF!=3) { print "bad version fixture" > "/dev/stderr"; failed=1; exit 1 }
    got=version_distance($1,$2)
    if(got!=$3) {
        print $1 " -> " $2 ": " got " expected " $3 > "/dev/stderr"
        failed=1; exit 1
    }
    cases++
}
END {
    if(failed) exit 1
    if(!cases) exit 1
    version_parse("1.0.0-rc.9",a); version_parse("1.0.0-rc.10",b)
    if(version_compare(a,b)>=0) exit 1
    print "awk dependency versions: PASS (" cases " shared cases and prerelease order)"
}
