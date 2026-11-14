import json,re,statistics
from pathlib import Path
root=Path(__file__).resolve().parent
rows=[]
for path in sorted(root.glob('heap-*-massif.txt')):
 pairs=[(float(t),int(b)) for t,b in re.findall(r'time=([\d.]+)\nmem_heap_B=(\d+)',path.read_text())]
 windows=[]
 for start in range(30,300,30):
  values=[b for t,b in pairs if start<=t<start+30]
  if values: windows.append({'start_seconds':start,'end_seconds':start+30,'samples':len(values),'min_bytes':min(values),'median_bytes':statistics.median(values),'max_bytes':max(values)})
 rows.append({'case':path.stem,'profile_first_seconds':pairs[0][0],'profile_last_seconds':pairs[-1][0],'windows':windows,'last_minus_first_median_bytes':windows[-1]['median_bytes']-windows[0]['median_bytes']})
(root/'heap-windows.json').write_text(json.dumps(rows,indent=2))
print(json.dumps(rows,indent=2))
