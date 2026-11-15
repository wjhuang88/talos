"""Linux procfs process-tree snapshots for developer performance measurements.

Polling cannot observe children that start and exit between samples. Keep that
limitation in reports; these snapshots are not complete lifetime accounting.
"""
import os
from pathlib import Path


def process_snapshot(pid, proc_root=Path('/proc')):
    """Read one process and its live threads, reporting races as incomplete."""
    root = proc_root / str(pid)
    stat = (root / 'stat').read_text().rsplit(')', 1)[1].split()
    start_ticks = int(stat[19])
    cpu_seconds = (int(stat[11]) + int(stat[12])) / os.sysconf('SC_CLK_TCK')
    pss_kib = next(int(line.split()[1]) for line in (root / 'smaps_rollup').read_text().splitlines()
                   if line.startswith('Pss:'))
    children = set()
    voluntary = involuntary = threads = 0
    complete = True
    for thread in (root / 'task').iterdir():
        try:
            status = dict(line.split(':', 1) for line in (thread / 'status').read_text().splitlines()
                          if ':' in line)
            voluntary += int(status['voluntary_ctxt_switches'])
            involuntary += int(status['nonvoluntary_ctxt_switches'])
            children.update(int(value) for value in (thread / 'children').read_text().split())
            threads += 1
        except (OSError, ValueError, KeyError):
            complete = False
    # A PID may be reused between reads. Never combine two process lifetimes.
    final_stat = (root / 'stat').read_text().rsplit(')', 1)[1].split()
    if int(final_stat[19]) != start_ticks:
        raise ProcessLookupError('PID reused during snapshot')
    return {'pid': pid, 'start_ticks': start_ticks, 'cpu_seconds': cpu_seconds,
            'pss_kib': pss_kib, 'live_threads': threads,
            'live_thread_voluntary_context_switches': voluntary,
            'live_thread_involuntary_context_switches': involuntary,
            'children': sorted(children), 'complete': complete}


def process_tree_snapshot(pid, proc_root=Path('/proc'), max_processes=256):
    """Bound traversal and retain failures instead of silently reporting zeros."""
    pending = [pid]
    visited = set()
    records = []
    missing = []
    while pending and len(visited) < max_processes:
        current = pending.pop()
        if current in visited:
            continue
        visited.add(current)
        try:
            record = process_snapshot(current, proc_root)
        except (OSError, ValueError, KeyError, StopIteration):
            missing.append(current)
            continue
        records.append(record)
        pending.extend(child for child in record['children'] if child not in visited)
    truncated = any(current not in visited for current in pending)
    return {'processes': records, 'missing_pids': missing, 'truncated': truncated,
            'complete': not missing and not truncated and all(r['complete'] for r in records),
            'observed_tree_pss_kib': sum(r['pss_kib'] for r in records)}
