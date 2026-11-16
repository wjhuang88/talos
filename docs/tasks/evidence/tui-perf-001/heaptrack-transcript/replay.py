import hashlib, json, os, pathlib, subprocess, sys
root=pathlib.Path('/home/wjhuang/WorkSpace/personal/talos/target/tui-perf-artifacts/heaptrack-transcript-10000')
root.mkdir(exist_ok=False)
profiler=pathlib.Path('/tmp/talos-perf-profiler/usr/bin/heaptrack')
printer=profiler.with_name('heaptrack_print')
repo=pathlib.Path('/tmp/talos-tui-perf-claim')
binaries={'baseline':pathlib.Path('/home/wjhuang/WorkSpace/personal/talos/target/tui-perf-artifacts/transcript-baseline-15ef42c1/release'),'candidate':pathlib.Path('/home/wjhuang/WorkSpace/personal/talos/target/tui-perf-artifacts/transcript-candidate-e42b9c9c/release')}
manifest={'status':'running','profiler':'heaptrack1.5.0 Arch1.5.0-11','scenario':'transcript-commit','history':10000,'seconds':35,'warmup':5,'rounds':3,'binaries':{k:{'path':str(v),'sha256':hashlib.sha256(v.read_bytes()).hexdigest()} for k,v in binaries.items()},'completed':[],'limitations':['heaptrack totals include startup and shutdown, not just steady window','sampler observes profiler launcher; process tree includes interpreter/compressor overhead','allocation counts are allocator events, not CPU attribution or leaks proof']}
def save(): (root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
save()
env=os.environ.copy()
for key in ['TALOS_TUI_METRICS','TALOS_TUI_METRICS_WARMUP_MS','LD_PRELOAD']: env.pop(key,None)
try:
 for r in range(1,4):
  for label,binary in binaries.items():
   name=f'{label}-r{r}'
   wrapper=root/(name+'-wrapper')
   wrapper.write_text(f'#!/bin/sh\nexec {profiler} --record-only -o {root/name} {binary} "$@"\n')
   wrapper.chmod(0o700)
   subprocess.run([sys.executable,str(repo/'scripts/measure_tui_perf.py'),str(wrapper),'--scenario','transcript-commit','--history','10000','--seconds','35','--warmup','5','--process-tree','--output',str(root/(name+'-sample.json'))],env=env,check=True)
   with (root/(name+'-analysis.txt')).open('w') as output:
    subprocess.run([str(printer),str(root/(name+'.zst')),'-n','5','-H',str(root/(name+'-histogram.txt')),'-M',str(root/(name+'-massif.txt')),'--massif-detailed-freq','0'],stdout=output,check=True)
   manifest['completed'].append(name);save();print(name,flush=True)
 manifest['status']='complete'
except BaseException:
 manifest['status']='failed';raise
finally:save()
