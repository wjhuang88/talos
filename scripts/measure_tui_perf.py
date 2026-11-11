#!/usr/bin/env python3
"""Linux PTY CPU/PSS/output sampler; synthetic terminal, not device acceptance."""
import argparse, fcntl, json, os, pty, select, struct, subprocess, termios, time, tempfile, hashlib, platform, signal
from pathlib import Path
from tui_perf_validation import validate_sample

p = argparse.ArgumentParser()
p.add_argument('binary')
p.add_argument('--scenario', choices=['idle', 'animation', 'stream', 'approval', 'scroll', 'resize'], default='idle')
p.add_argument('--history', type=int, choices=[100, 1000, 10000], default=100)
p.add_argument('--seconds', type=int, default=35)
p.add_argument('--warmup', type=int, default=5)
p.add_argument('--workspace', default='')
p.add_argument('--output', required=True)
p.add_argument('--input-probe', action='store_true', help='type then erase one character every 500ms')
a = p.parse_args()
if not 0 <= a.warmup < a.seconds <= 3600:
    p.error('require 0 <= warmup < seconds <= 3600')
with open(a.binary, 'rb') as binary_file:
    binary_digest = hashlib.file_digest(binary_file, 'sha256').hexdigest()
ready_dir = tempfile.TemporaryDirectory(prefix="talos-perf-")
ready_path = Path(ready_dir.name) / "ready"
ready_at = None
master, slave = pty.openpty()
fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 120, 0, 0))
fixture_scenario = 'idle' if a.scenario in ('scroll', 'resize') else a.scenario
proc = subprocess.Popen([a.binary, fixture_scenario, str(a.history), str(a.seconds), a.workspace],
                        stdin=slave, stdout=slave, stderr=slave,
                        env={**os.environ, 'TERM': 'xterm-256color', 'TALOS_REPLAY_READY': str(ready_path)}, start_new_session=True)
os.close(slave)
start = time.monotonic()
last_sample = start
samples = []
byte_count = 0
pending = b''
exit_code = None
last_probe = 0.0
probe_count = 0
last_action = 0.0
action_count = 0
try:
    while time.monotonic() - start < a.seconds + 30:
        now = time.monotonic()
        if ready_at is None and ready_path.exists():
            ready_at = now - start
        if a.scenario in ('scroll', 'resize') and ready_at is not None and now-start >= ready_at+a.warmup and now-last_action >= .5:
            if a.scenario == 'scroll':
                os.write(master, b'\x1b[5~' if action_count % 2 == 0 else b'\x1b[6~')
            else:
                rows, columns = (30, 80) if action_count % 2 == 0 else (40, 120)
                fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack('HHHH', rows, columns, 0, 0))
                os.kill(proc.pid, signal.SIGWINCH)
            action_count += 1
            last_action = now
        if a.input_probe and ready_at is not None and now-start >= ready_at+a.warmup and now-last_probe >= .5:
            os.write(master, b'x\x7f')
            last_probe = now
            probe_count += 1
        if now - last_sample >= .25:
            try:
                stat = Path(f'/proc/{proc.pid}/stat').read_text().rsplit(')', 1)[1].split()
                cpu = (int(stat[11]) + int(stat[12])) / os.sysconf('SC_CLK_TCK')
                pss = next(int(line.split()[1]) for line in Path(f'/proc/{proc.pid}/smaps_rollup').read_text().splitlines() if line.startswith('Pss:'))
                status = dict(line.split(':', 1) for line in Path(f'/proc/{proc.pid}/status').read_text().splitlines() if ':' in line)
                children = Path(f'/proc/{proc.pid}/task/{proc.pid}/children').read_text().split()
                samples.append({'seconds': now-start, 'cpu_seconds': cpu, 'pss_kib': pss, 'output_bytes': byte_count,
                                'voluntary_context_switches': int(status['voluntary_ctxt_switches']),
                                'involuntary_context_switches': int(status['nonvoluntary_ctxt_switches']),
                                'threads': int(status['Threads']), 'observed_direct_children': len(children)})
            except (FileNotFoundError, ProcessLookupError):
                pass
            last_sample = now
        ready, _, _ = select.select([master], [], [], .05)
        if ready:
            try:
                data = os.read(master, 65536)
            except OSError:
                break
            if not data:
                break
            byte_count += len(data)
            # Respond to terminal position queries; retain only a bounded parser tail.
            pending += data
            while b'\x1b[6n' in pending:
                _, pending = pending.split(b'\x1b[6n', 1)
                os.write(master, b'\x1b[1;1R')
            pending = pending[-8:]
        if proc.poll() is not None and not ready:
            break
    if proc.poll() is None:
        proc.terminate()
    try:
        exit_code = proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        exit_code = proc.wait()
finally:
    os.close(master)
    if proc.poll() is None:
        proc.kill()
        proc.wait()
steady = [s for s in samples if ready_at is not None and ready_at + a.warmup <= s['seconds'] <= ready_at + a.seconds]
ready_dir.cleanup()
result = {'binary_sha256': binary_digest, 'platform': platform.platform(),
          'action_count': action_count, 'input_probe_count': probe_count, 'scenario': a.scenario, 'history': a.history, 'geometry': [120,40],
          'terminal': 'synthetic Linux PTY', 'exit_code': exit_code, 'samples': samples,
          'ready_seconds': ready_at, 'warmup_seconds': a.warmup, 'wall_seconds': time.monotonic()-start,
          'limitations': ['parent process only; child and terminal emulator accounting pending',
                         'software input latency requires TALOS_TUI_METRICS; no pixel latency or allocation metrics']}
if len(steady) >= 2:
    first, last = steady[0], steady[-1]
    result['cpu_percent'] = 100*(last['cpu_seconds']-first['cpu_seconds'])/(last['seconds']-first['seconds'])
    result['peak_pss_kib'] = max(s['pss_kib'] for s in steady)
    result['steady_output_bytes'] = last['output_bytes']-first['output_bytes']
    result['context_switches'] = {key: last[key]-first[key] for key in ('voluntary_context_switches', 'involuntary_context_switches')}
    result['max_threads'] = max(s['threads'] for s in steady)
    result['max_observed_direct_children'] = max(s['observed_direct_children'] for s in steady)
Path(a.output).write_text(json.dumps(result, indent=2)+'\n')
if exit_code != 0:
    raise SystemExit(f'replay exited {exit_code}; inspect report')

if ready_at is None or len(steady) < 2:
    raise SystemExit('insufficient ready-state samples; report is not baseline evidence')

# Standalone invocations enforce the same evidence gate as matrix runs.
coverage = validate_sample(a.output, a.seconds, a.warmup)
result.update(coverage)
Path(a.output).write_text(json.dumps(result, indent=2)+"\n")
