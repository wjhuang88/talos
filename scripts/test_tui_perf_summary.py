"""Evidence summary regressions for quantile precision and incomplete matrices."""
import json
from pathlib import Path
import tempfile
import unittest

from summarize_tui_perf import percentile_bounds, summarize


class SummaryValidation(unittest.TestCase):
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
