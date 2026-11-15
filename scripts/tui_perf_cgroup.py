"""Optional Linux cgroup-v2 observations; counters are group-wide, not per frame."""
from pathlib import Path


def cgroup_snapshot(pid, proc_root=Path('/proc'), cgroup_root=Path('/sys/fs/cgroup')):
    """Read the child's leaf group; unavailable evidence is never reported as zero."""
    try:
        membership = (proc_root / str(pid) / 'cgroup').read_text().splitlines()
        relative = next(line[3:] for line in membership if line.startswith('0::'))
        root = cgroup_root.resolve()
        group = (root / relative.lstrip('/')).resolve()
        if not group.is_relative_to(root):
            return {'unavailable': 'cgroup path outside mounted hierarchy'}
    except (OSError, StopIteration, RuntimeError) as error:
        return {'unavailable': type(error).__name__}
    result = {'path': relative}
    for name in ('cpu.stat', 'cpu.max', 'cpu.pressure', 'memory.events',
                 'memory.current', 'memory.pressure', 'io.pressure'):
        try:
            result[name] = (group / name).read_text().strip()
        except OSError as error:
            result[name] = {'unavailable': type(error).__name__}
    return result
