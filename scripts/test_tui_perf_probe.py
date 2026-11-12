"""Exercise the real PTY sampler against a bounded, provider-free child."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class ProbeSampling(unittest.TestCase):
    def test_fast_input_is_delivered_without_sleep_interval_drift(self):
        sampler = Path(__file__).with_name('measure_tui_perf.py')
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            child = root / 'fixture'
            child.write_text(f'#!{sys.executable}\n' + '''
import os, pathlib, select, time, tty
tty.setraw(0)
pathlib.Path(os.environ['TALOS_REPLAY_READY']).write_text('ready')
end = time.monotonic() + 2.2
received = 0
while time.monotonic() < end:
    readable, _, _ = select.select([0], [], [], .01)
    if readable:
        received += os.read(0, 4096).count(b'x')
pathlib.Path(__file__).with_name('received').write_text(str(received))
''')
            child.chmod(0o700)
            output = root / 'sample.json'
            result = subprocess.run([sys.executable, str(sampler), str(child),
                '--seconds', '2', '--warmup', '0', '--input-probe', '--cgroup',
                '--input-probe-interval-ms', '20', '--output', str(output)],
                capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            sample = json.loads(output.read_text())
            received = int((root / 'received').read_text())
            self.assertEqual(sample['input_probe_interval_ms'], 20)
            self.assertTrue(sample['cgroup_enabled'])
            self.assertTrue(all('cgroup' in item for item in sample['samples']))
            self.assertGreaterEqual(received, 80, '50 ms poll/drift must not throttle 20 ms probes')
            self.assertLessEqual(received, sample['input_probe_count'])
            self.assertLessEqual(sample['input_probe_count'] - received, 1)
            self.assertGreaterEqual(sample['input_probe_skipped_count'], 0)

    def test_invalid_probe_interval_fails_before_launching_binary(self):
        sampler = Path(__file__).with_name('measure_tui_perf.py')
        result = subprocess.run([sys.executable, str(sampler), '/does-not-exist',
            '--input-probe-interval-ms', '0', '--output', '/does-not-exist'],
            capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 2)
        self.assertIn('input probe interval', result.stderr)


if __name__ == '__main__':
    unittest.main()
