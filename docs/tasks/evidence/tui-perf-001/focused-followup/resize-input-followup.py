import os,sys,subprocess,json
from pathlib import Path
root=Path(__file__).resolve().parent
repo=Path('/home/wjhuang/WorkSpace/personal/talos/target/tui-perf-acceptance')
sys.path.insert(0,str(repo/'scripts'))
from tui_perf_validation import validate_sample,validate_metrics
from summarize_tui_perf import resize_budget,percentile_bounds
rows=[]
for i in range(1,4):
 name=f'resize-input-r{i}'
 env=dict(os.environ,TALOS_TUI_METRICS=str(root/(name+'-metrics.json')),TALOS_TUI_METRICS_WARMUP_MS='5000')
 subprocess.run([sys.executable,str(repo/'scripts/measure_tui_perf.py'),str(root/'debug'),'--scenario','resize','--history','10000','--seconds','35','--warmup','5','--input-probe','--cgroup','--output',str(root/(name+'-sample.json'))],env=env,check=True)
 s=json.loads((root/(name+'-sample.json')).read_text());m=validate_metrics(root/(name+'-metrics.json'),'resize',s['action_count'],5000,35)
 rows.append({'round':i,'coverage':validate_sample(root/(name+'-sample.json'),35,5),'budget':resize_budget(m['steady_stages']),'p99':percentile_bounds(m['steady_stages']['input_to_submit'],99),'actions':s['action_count'],'input_probe_count':s['input_probe_count'],'input_probe_skipped_count':s['input_probe_skipped_count']})
 (root/'resize-input-validation.json').write_text(json.dumps(rows,indent=2));print(rows[-1],flush=True)
