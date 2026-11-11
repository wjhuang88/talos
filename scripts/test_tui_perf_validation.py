"""Regression tests for accepting incomplete or malformed performance evidence."""
import json
from pathlib import Path
import tempfile
import unittest
from tui_perf_validation import STAGES, validate_metrics, validate_sample


class EvidenceValidation(unittest.TestCase):
    def test_early_success_is_not_a_complete_measurement(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder)/'sample.json'
            sample = {'exit_code': 0, 'ready_seconds': 2, 'wall_seconds': 9,
                      'samples': [{'seconds': 7.1}, {'seconds': 7.4}]}
            path.write_text(json.dumps(sample))
            with self.assertRaisesRegex(ValueError, 'required measurement'):
                validate_sample(path, 35, 5)
            sample['samples'] = [{'seconds': 7.1+i*.25} for i in range(120)]
            sample['wall_seconds'] = 37
            path.write_text(json.dumps(sample))
            self.assertGreater(validate_sample(path, 35, 5)['steady_covered_seconds'], 29)

    def test_file_presence_does_not_prove_metrics(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder)/'metrics.json'
            path.write_text('')
            with self.assertRaises(ValueError):
                validate_metrics(path)
            stages = {name: {'count': 1, 'total_ns': 0, 'max_ns': 0,
                             'log2_ns_buckets': [1]+[0]*31} for name in STAGES}
            data = {'schema': 1, 'stages': stages}
            path.write_text(json.dumps(data))
            validate_metrics(path)
            with self.assertRaisesRegex(ValueError, 'actual scenario'):
                validate_metrics(path, 'scroll', 1)
            data['schema'] = 2
            for name in ('history_scroll', 'screen_resize', 'deadline_lateness', 'frame_preparation'):
                data['stages'][name] = {'count': 1, 'total_ns': 0, 'max_ns': 0, 'log2_ns_buckets': [1]+[0]*31}
            path.write_text(json.dumps(data))
            validate_metrics(path, 'scroll', 1)
            validate_metrics(path, 'resize', 1)
            data['warmup_ms'] = 1000
            data['steady_elapsed_ns'] = 2_000_000_000
            data['steady_stages'] = {name: dict(metric) for name,metric in data['stages'].items()}
            data['steady_stages']['history_scroll'] = {'count': 0, 'total_ns': 0, 'max_ns': 0, 'log2_ns_buckets': [0]*32}
            path.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, 'actual scenario'):
                validate_metrics(path, 'scroll', 1, 1000, 3)
            with self.assertRaisesRegex(ValueError, 'too short'):
                validate_metrics(path, 'idle', 0, 1000, 35)
            data['steady_stages']['workspace_change'] = {'count': 0, 'total_ns': 0, 'max_ns': 0,
                                                        'log2_ns_buckets': [0]*32}
            path.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, 'actual workspace'):
                validate_metrics(path, 'workspace-switch', 0, 1000, 3)
            data['steady_stages']['workspace_change'] = {'count': 2, 'total_ns': 0, 'max_ns': 0,
                                                        'log2_ns_buckets': [2]+[0]*31}
            path.write_text(json.dumps(data))
            validate_metrics(path, 'workspace-switch', 0, 1000, 3)
            data['stages']['draw']['count'] = 2
            path.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, 'inconsistent'):
                validate_metrics(path)


if __name__ == '__main__':
    unittest.main()
