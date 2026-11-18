import time,json,os
from pathlib import Path
root=Path('/proc/518450'); hz=os.sysconf('SC_CLK_TCK')
def snap():
 out={}
 for p in (root/'task').iterdir():
  try:
   s=(p/'stat').read_text().rsplit(')',1)[1].split(); sched=(p/'schedstat').read_text().split()
   out[p.name]={'name':(p/'comm').read_text().strip(),'ticks':int(s[11])+int(s[12]),'start':s[19],'runtime_ns':int(sched[0]),'wait_ns':int(sched[1]),'slices':int(sched[2])}
  except FileNotFoundError:pass
 return out
start=(root/'stat').read_text().rsplit(')',1)[1].split()[19]
a=snap(); t=time.monotonic();time.sleep(30);b=snap();elapsed=time.monotonic()-t
assert (root/'stat').read_text().rsplit(')',1)[1].split()[19]==start
rows=[]
for tid,v in b.items():
 if tid in a and a[tid]['start']==v['start']:
  old=a[tid];rows.append(dict(tid=tid,name=v['name'],cpu_percent=100*(v['ticks']-old['ticks'])/hz/elapsed,runtime_ms=(v['runtime_ns']-old['runtime_ns'])/1e6,runqueue_wait_ms=(v['wait_ns']-old['wait_ns'])/1e6,slices=v['slices']-old['slices']))
rows.sort(key=lambda v:v['runtime_ms'],reverse=True)
d=dict(pid=518450,elapsed_seconds=elapsed,threads=rows,created=list(b.keys()-a.keys()),exited=list(a.keys()-b.keys()),limitations=['Two endpoint samples miss threads created and exited within interval','Thread names do not prove call stack attribution'])
Path('/tmp/talos-thread-idle-20261117.json').write_text(json.dumps(d,indent=2)+'\n');print(json.dumps(d,indent=2))
