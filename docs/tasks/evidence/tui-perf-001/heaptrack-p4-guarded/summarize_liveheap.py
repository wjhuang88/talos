"""Summarize existing heaptrack massif files; does not run the application."""
import json
from pathlib import Path
import re
import statistics
import sys
rows = []
for path in sorted(Path(sys.argv[1]).glob("candidate-*-massif.txt")):
    pairs = [(float(t), int(b)) for t, b in re.findall(
        r"time=([\d.]+)\nmem_heap_B=(\d+)", path.read_text())]
    first = [b for t, b in pairs if 5 <= t < 10]
    last = [b for t, b in pairs if 25 <= t < 30]
    rows.append({"case": path.stem, "samples": len(pairs),
                 "first_window_count": len(first), "last_window_count": len(last),
                 "first_window_median_bytes": statistics.median(first),
                 "last_window_median_bytes": statistics.median(last),
                 "delta_bytes": statistics.median(last) - statistics.median(first)})
print(json.dumps(rows, indent=2))
