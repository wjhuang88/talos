# Bounded JSON reader for the dependency audit. Run with LC_ALL=C (byte offsets).
# AST: jt[id] type, jv[id] decoded scalar, jr[id] raw scalar, jf/jn child links,
# jo[parent SUBSEP key] object lookup. Containers preserve input order.
function jfail(message) {
    jfailed=1
    print "dependency audit: invalid JSON at byte " jp ": " message > "/dev/stderr"
    exit 2
}
# Some awk implementations recompute a large string's length during substr.
# Bounded chunks avoid scanning the entire metadata document for every character.
function jspan(pos,n,    result,part,offset,take) {
    result=""
    while(n>0 && pos<=jsize) {
        part=int((pos-1)/4096); offset=(pos-1)%4096+1
        take=4097-offset; if(take>n) take=n
        result=result substr(jchunks[part],offset,take)
        pos+=take; n-=take
    }
    return result
}
function jspace() {
    while (jspan(jp,1) ~ /^[ \t\r\n]$/) jp++
}
function jhex(    i,c,n) {
    n=0
    for(i=0;i<4;i++) {
        c=index("0123456789abcdef",tolower(jspan(jp+i,1)))-1
        if(c<0 || jp+i>jsize) jfail("invalid Unicode escape")
        n=n*16+c
    }
    jp+=4
    return n
}
function jutf(n) {
    if(n==0) jfail("NUL escape unsupported by portable awk")
    if(n<128) return sprintf("%c",n)
    if(n<2048) return sprintf("%c%c",192+int(n/64),128+n%64)
    if(n<65536) return sprintf("%c%c%c",224+int(n/4096),128+int(n/64)%64,128+n%64)
    return sprintf("%c%c%c%c",240+int(n/262144),128+int(n/4096)%64,128+int(n/64)%64,128+n%64)
}
function jstring(    out,c,e,n,lo) {
    if(jspan(jp,1)!="\"") jfail("expected string")
    jp++; out=""
    while(jp<=jsize) {
        c=jspan(jp++,1)
        if(c=="\"") return out
        if(c ~ /[[:cntrl:]]/) jfail("unescaped control byte")
        if(c!="\\") { out=out c; continue }
        e=jspan(jp++,1)
        if(e=="\"" || e=="\\" || e=="/") out=out e
        else if(e=="b") out=out sprintf("%c",8)
        else if(e=="f") out=out sprintf("%c",12)
        else if(e=="n") out=out "\n"
        else if(e=="r") out=out "\r"
        else if(e=="t") out=out "\t"
        else if(e=="u") {
            n=jhex()
            if(n>=55296 && n<=56319) {
                if(jspan(jp,2)!="\\u") jfail("unpaired high surrogate")
                jp+=2; lo=jhex()
                if(lo<56320 || lo>57343) jfail("invalid low surrogate")
                n=65536+(n-55296)*1024+lo-56320
            } else if(n>=56320 && n<=57343) jfail("unpaired low surrogate")
            out=out jutf(n)
        } else jfail("unknown escape")
    }
    jfail("unterminated string")
}
function jparse(depth,    id,start,c,closec,key,child,last,token,keyraw,keystart) {
    if(depth>128) jfail("nesting limit exceeded")
    jspace(); start=jp; c=jspan(jp,1)
    id=++jcount
    if(jcount>1000000) jfail("node limit exceeded")
    if(c=="{" || c=="[") {
        jt[id]=(c=="{" ? "object" : "array"); closec=(c=="{" ? "}" : "]")
        jp++; jspace()
        if(jspan(jp,1)==closec) { jp++; return id }
        while(1) {
            if(jt[id]=="object") {
                keystart=jp; key=jstring(); keyraw=jspan(keystart,jp-keystart); jspace()
                if(jspan(jp++,1)!=":") jfail("expected colon")
                if((id SUBSEP key) in jo) jfail("duplicate object key")
            }
            child=jparse(depth+1)
            if(jt[id]=="object") { jo[id SUBSEP key]=child; jkeyraw[child]=keyraw }
            if(last) jn[last]=child; else jf[id]=child
            last=child; jl[id]++
            jspace(); c=jspan(jp++,1)
            if(c==closec) break
            if(c!=",") jfail("expected comma or container end")
            jspace()
        }
        return id
    }
    if(c=="\"") { jt[id]="string"; jv[id]=jstring() }
    else if(jspan(jp,4)=="null") { jt[id]="null"; jp+=4 }
    else if(jspan(jp,4)=="true") { jt[id]="boolean"; jv[id]="true"; jp+=4 }
    else if(jspan(jp,5)=="false") { jt[id]="boolean"; jv[id]="false"; jp+=5 }
    else {
        token=jspan(jp,256)
        if(!match(token,/^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?/)) jfail("expected value")
        jt[id]="number"; jv[id]=substr(token,1,RLENGTH); jp+=RLENGTH
    }
    jr[id]=jspan(start,jp-start)
    return id
}
{
    js=js $0 "\n"
    if(length(js)>33554432) jfail("input exceeds 32 MiB")
}
END {
    if(!jfailed) {
        jsize=length(js)
        for(jchunk=0;jchunk*4096<jsize;jchunk++) jchunks[jchunk]=substr(js,jchunk*4096+1,4096)
        js=""
        jp=1; jroot=jparse(0); jspace()
        if(jp<=jsize) jfail("trailing input")
        if(json_mode=="validate") print "JSON accepted"
    }
}
