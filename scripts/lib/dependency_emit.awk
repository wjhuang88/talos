function ef(id,key) { return jo[id SUBSEP key] }
function ev(id,key) { return jv[ef(id,key)] }
function emit(id,    child,key,first,s) {
    if(jt[id]!="object" && jt[id]!="array") return jr[id]
    s=(jt[id]=="object" ? "{" : "["); first=1
    if(jt[id]=="object") {
        for(child=jf[id];child;child=jn[child]) {
            if(!first) s=s ","; first=0
            s=s jkeyraw[child] ":" emit(child)
        }
    } else for(child=jf[id];child;child=jn[child]) {
        if(!first) s=s ","; first=0; s=s emit(child)
    }
    return s (jt[id]=="object" ? "}" : "]")
}
