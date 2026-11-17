import subprocess,tempfile,json,time
from pathlib import Path
results=[]
for mode in ['read','ignored-write','tracked-ignored-write','visible-write']:
 root=Path(tempfile.mkdtemp(prefix='talos-watch-control-'));subprocess.run(['git','init','-q',str(root)],check=True)
 (root/'.gitignore').write_text('noise.log\n'); path=root/('visible.txt' if mode=='visible-write' else 'noise.log');path.write_text('initial')
 if mode=='tracked-ignored-write':subprocess.run(['git','-C',str(root),'add','-f','noise.log'],check=True)
 p=subprocess.Popen(['/tmp/talos-current-watch-probe/probe',str(root)],stdout=subprocess.PIPE,text=True)
 setup=p.stdout.readline().strip();assert 'healthy=true' in setup
 for i in range(5):
  if mode=='read':path.read_text()
  else:path.write_text(str(i))
  time.sleep(.1)
 output=p.communicate(timeout=10)[0];assert p.returncode==0
 results.append(dict(mode=mode,setup=setup,result=output))
Path('/tmp/talos-current-watch-probe/controls.json').write_text(json.dumps(results,indent=2));print(json.dumps(results,indent=2))
