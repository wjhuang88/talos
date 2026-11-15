import os,sys,subprocess,json,hashlib
from pathlib import Path
root=Path(__file__).resolve().parent
repo=Path('/home/wjhuang/WorkSpace/personal/talos/target/tui-perf-acceptance')
sys.path.insert(0,str(repo/'scripts'))
from tui_perf_validation import validate_sample,validate_metrics
profiler=Path('/tmp/talos-perf-profiler/usr/bin/heaptrack')
manifest={'source':'1eabf2964b5a81005f626644bd99053000518fdb','binary_sha256':hashlib.sha256((root/'debug').read_bytes()).hexdigest(),'seconds':305,'warmup':5,'completed':[],'status':'running'}
def save(): (root/'heap-manifest.json').write_text(json.dumps(manifest,indent=2))
save()
try:
 for scenario in ('resize','transcript-commit'):
  name='heap-'+scenario
  wrapper=root/(name+'-wrapper')
  wrapper.write_text(f'#!/bin/sh\nexec {profiler} --record-only -o {root/name} {root/"debug"} "$@"\n');wrapper.chmod(0o700)
  env=dict(os.environ,TALOS_TUI_METRICS=str(root/(name+'-metrics.json')),TALOS_TUI_METRICS_WARMUP_MS='5000')
  subprocess.run([sys.executable,str(repo/'scripts/measure_tui_perf.py'),str(wrapper),'--scenario',scenario,'--history','10000','--seconds','305','--warmup','5','--process-tree','--cgroup','--output',str(root/(name+'-sample.json'))],env=env,check=True)
  sample=json.loads((root/(name+'-sample.json')).read_text())
  validate_sample(root/(name+'-sample.json'),305,5)
  validate_metrics(root/(name+'-metrics.json'),scenario,sample['action_count'],5000,305)
  with (root/(name+'-analysis.txt')).open('w') as out:
   subprocess.run([str(profiler.with_name('heaptrack_print')),str(root/(name+'.zst')),'-n','5','-M',str(root/(name+'-massif.txt')),'--massif-detailed-freq','0'],stdout=out,check=True)
  manifest['completed'].append(scenario);save();print('completed '+scenario,flush=True)
 manifest['status']='complete'
except BaseException:
 manifest['status']='failed';raise
finally:save()
