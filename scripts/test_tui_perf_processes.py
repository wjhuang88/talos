"""Process-tree accounting checks without launching measurement competitors."""
from pathlib import Path
import tempfile
import unittest

from tui_perf_processes import process_tree_snapshot


def fixture(root, pid, children=(), extra_thread_children=()):
    directory = root / str(pid)
    directory.mkdir()
    fields = ['0'] * 20
    fields[0] = 'S'
    fields[11], fields[12], fields[19] = '10', '5', str(pid * 100)
    (directory / 'stat').write_text(f'{pid} (name with ) parens) ' + ' '.join(fields))
    (directory / 'smaps_rollup').write_text('Pss: 12 kB\n')
    for tid, descendants in ((pid, children), (pid + 1000, extra_thread_children)):
        thread = directory / 'task' / str(tid)
        thread.mkdir(parents=True)
        (thread / 'status').write_text('voluntary_ctxt_switches: 3\nnonvoluntary_ctxt_switches: 2\n')
        (thread / 'children').write_text(' '.join(map(str, descendants)))


class ProcessAccounting(unittest.TestCase):
    def test_children_from_all_threads_and_shared_descendant(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            fixture(root, 1, [2], [3, 2])
            fixture(root, 2)
            fixture(root, 3)
            result = process_tree_snapshot(1, root)
            self.assertTrue(result['complete'])
            self.assertEqual(len(result['processes']), 3)
            self.assertEqual(result['observed_tree_pss_kib'], 36)
            self.assertEqual(result['processes'][0]['live_thread_voluntary_context_switches'], 6)

    def test_missing_child_and_traversal_limit_are_explicit(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            fixture(root, 1, [2])
            result = process_tree_snapshot(1, root)
            self.assertFalse(result['complete'])
            self.assertEqual(result['missing_pids'], [2])
            limited = process_tree_snapshot(1, root, max_processes=1)
            self.assertTrue(limited['truncated'])
            self.assertFalse(limited['complete'])


if __name__ == '__main__':
    unittest.main()
