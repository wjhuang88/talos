"""Evidence summary regressions for quantile precision and incomplete matrices."""
import json
from pathlib import Path
import tempfile
import unittest

from summarize_tui_perf import animation_budget, observed_tree_usage, percentile_bounds, summarize


class SummaryValidation(unittest.TestCase):
    def test_animation_budget_checks_exact_threshold_without_quantile_guessing(self):
        self.assertIsNone(animation_budget({}))
        metric = {'count': 213, 'total_ns': 213 * 49_000_000,
                  'max_ns': 50_000_001, 'log2_ns_buckets': [0] * 32,
                  'over_50ms_count': 2}
        metric['log2_ns_buckets'][26] = 213
        table = {'animation_deadline_lateness': metric}
        self.assertTrue(animation_budget(table)['p99_within_50ms'])
        metric['over_50ms_count'] = 3
        self.assertFalse(animation_budget(table)['p99_within_50ms'])
        metric['over_50ms_count'] = 214
        with self.assertRaisesRegex(ValueError, 'threshold count'):
            animation_budget(table)
        metric.update(count=0, total_ns=0, max_ns=0, over_50ms_count=0,
                      log2_ns_buckets=[0] * 32)
        self.assertFalse(animation_budget(table)['p99_within_50ms'])
        metric.update(count=1, total_ns=50_000_000, max_ns=50_000_000,
                      over_50ms_count=1)
        metric['log2_ns_buckets'][26] = 1
        with self.assertRaisesRegex(ValueError, 'contradicts maximum'):
            animation_budget(table)

    def test_tree_cpu_does_not_subtract_changing_membership(self):
        def sample(processes):
            return {'process_tree': {'complete': True, 'observed_tree_pss_kib': 10,
                                    'processes': [{'pid': p, 'start_ticks': start, 'cpu_seconds': cpu}
                                                  for p, start, cpu in processes]}}
        result = observed_tree_usage([
            sample([(1, 100, 10), (2, 200, 50)]),
            sample([(1, 100, 11)]),
            sample([(1, 100, 12), (2, 300, 1)]),
            sample([(1, 100, 13), (2, 300, 2)]),
        ])
        self.assertEqual(result['observed_interval_cpu_seconds'], 4)
        self.assertEqual(result['observed_process_lifetimes'], 3)
        with self.assertRaisesRegex(ValueError, 'missing'):
            observed_tree_usage([{}])

    def test_histogram_bounds_and_overflow(self):
        metric = {'count': 100, 'max_ns': 2**40, 'log2_ns_buckets': [0]*32}
        metric['log2_ns_buckets'][2] = 95
        metric['log2_ns_buckets'][31] = 5
        self.assertEqual(percentile_bounds(metric, 95), {'lower_ns': 2, 'upper_ns': 3})
        self.assertEqual(percentile_bounds(metric, 99), {'lower_ns': 2**30, 'upper_ns': 2**40})
        self.assertIsNone(percentile_bounds({'count': 0}, 99))

    def test_wrong_cases_cannot_replace_required_rounds(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            manifest = {'status': 'complete', 'binaries': [{'label': 'base'}],
                        'arguments': {'scenarios': ['idle'], 'histories': [100], 'rounds': 2},
                        'completed': ['base-idle-100-r1', 'base-idle-100-r3']}
            (root/'manifest.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, 'case count'):
                summarize(root)


if __name__ == '__main__':
    unittest.main()
