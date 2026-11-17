import json,time,os,hashlib
from pathlib import Path
pids=[614051,12986]
def sample(pid):
 p=Path('/proc')/str(pid)
 s=(p/'stat').read_text().rsplit(')',1)[1].split()
 mem={k:int(v.split()[0]) for k,v in (line.split(':',1) for line in (p/'smaps_rollup').read_text().splitlines() if ':' in line) if k in ['Pss','Swap']}
 return dict(pid=pid,start_ticks=s[19],cpu_seconds=(int(s[11])+int(s[12]))/os.sysconf('SC_CLK_TCK'),**mem)
before=[sample(p) for p in pids];t=time.monotonic();time.sleep(30);elapsed=time.monotonic()-t;after=[sample(p) for p in pids]
for a,b in zip(before,after):
 assert a['start_ticks']==b['start_ticks']
 b['cpu_percent_one_core']=100*(b['cpu_seconds']-a['cpu_seconds'])/elapsed
exe=Path('/proc/614051/exe')
with exe.open('rb') as f:digest=hashlib.file_digest(f,'sha256').hexdigest()
result=dict(elapsed_seconds=elapsed,before=before,after=after,binary_sha256=digest,executable=os.readlink(exe),limitations=['User confirmed idle before sample','Ghostty resources may include other tabs','Binary hash alone does not attest build source'])
Path('/tmp/talos-native-quiet-round2.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
