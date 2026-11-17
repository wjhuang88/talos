from pathlib import Path
import json,subprocess,hashlib
root=Path('/home/wjhuang/WorkSpace/personal/talos'); wt=root/'target/tui-perf-acceptance'; build=root/'target/tui-idle-git-validation/debug'; out=Path('/tmp/talos-current-watch-probe');out.mkdir(exist_ok=True)
source=wt/'crates/talos-tui/src/git_watch.rs'; (out/'git_watch.rs').write_bytes(source.read_bytes())
s=(wt/'docs/tasks/evidence/tui-perf-001/git-idle-watch/git_watch_probe.rs.txt').read_text().replace('../src/git_watch.rs','git_watch.rs').replace('from_secs(30)','from_secs(3)').replace('observed_seconds=30','observed_seconds=3')
(out/'main.rs').write_text(s)
f=json.loads((build/'.fingerprint/talos-tui-d19d818cd83574e8/example-git_watch_probe.json').read_text());cmd=['rustc','+1.99.0','--edition=2024',str(out/'main.rs'),'-L','dependency='+str(build/'deps'),'-o',str(out/'probe')]
for _,name,_,value in f['deps']:
 if name not in ['notify','tokio','gix','ignore']:continue
 candidates=[p for p in (build/'.fingerprint').glob(name+'-*/lib-'+name) if int.from_bytes(bytes.fromhex(p.read_text().strip()),'little')==value]
 assert len(candidates)==1,(name,candidates)
 lib=build/'deps'/('lib'+candidates[0].parent.name+'.rlib');assert lib.exists();cmd+=['--extern',name+'='+str(lib)]
subprocess.run(cmd,check=True)
(out/'receipt.json').write_text(json.dumps({'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=wt,text=True).strip(),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'binary_sha256':hashlib.sha256((out/'probe').read_bytes()).hexdigest(),'command':cmd,'scope':'Current watcher source linked against preserved dependency artifacts; diagnostic only, not full Cargo build'},indent=2))
print('compiled source-bound diagnostic')
