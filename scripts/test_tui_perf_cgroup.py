"""Resource diagnostics must distinguish unavailable data from zero pressure."""
from pathlib import Path
import tempfile
import unittest
from tui_perf_cgroup import cgroup_snapshot


class CgroupSampling(unittest.TestCase):
    def test_leaf_counters_and_missing_files(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            proc = root / 'proc' / '42'
            group = root / 'cgroup' / 'task'
            proc.mkdir(parents=True)
            group.mkdir(parents=True)
            (proc / 'cgroup').write_text('0::/task\n')
            (group / 'cpu.stat').write_text('nr_throttled 7\nthrottled_usec 900\n')
            sample = cgroup_snapshot(42, root / 'proc', root / 'cgroup')
            self.assertEqual(sample['path'], '/task')
            self.assertIn('throttled_usec 900', sample['cpu.stat'])
            self.assertIn('unavailable', sample['memory.events'])

    def test_missing_process_is_not_zero(self):
        with tempfile.TemporaryDirectory() as folder:
            self.assertIn('unavailable', cgroup_snapshot(42, Path(folder)))

    def test_path_cannot_escape_mount(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            proc = root / '42'
            proc.mkdir()
            (proc / 'cgroup').write_text('0::/../../outside\n')
            self.assertIn('unavailable', cgroup_snapshot(42, root, root / 'cg'))


if __name__ == '__main__':
    unittest.main()
