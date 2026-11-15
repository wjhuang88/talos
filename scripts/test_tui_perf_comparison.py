import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from compare_tui_perf import compare


class ComparisonBudgets(unittest.TestCase):
    def run_case(self, scenario, baseline_cpu, candidate_cpu, candidate_peak=100,
                 metrics='on', counts=None, input_probe=False, latency_ns=1_000_000,
                 animation_budget=None):
        def case(cpu, peak):
            return {'aggregate': {'cpu_percent': {'median': cpu},
                                  'peak_pss_kib': {'max': peak}},
                    'rounds': [{'cpu_percent': cpu, 'animation_deadline_budget': animation_budget,
                                'stage_counts': counts or {
                        name: 0 for name in ('draw', 'diff', 'flush', 'layout', 'history_rebuild', 'git_scan')
                    }, 'latency_bounds': {stage: {p: None if latency_ns is None else {'upper_ns': latency_ns}
                        for p in ('p95', 'p99')} for stage in ('input_to_submit', 'deadline_lateness')}}]}
        manifest = {'binaries': [{'label': 'baseline'}, {'label': 'candidate'}],
                    'arguments': {'scenarios': [scenario], 'histories': [100],
                                  'metrics': metrics, 'profile': 'debug', 'workspace': '', 'input_probe': input_probe}}
        summary = {'cases': {f'baseline-{scenario}-100': case(baseline_cpu, 100),
                             f'candidate-{scenario}-100': case(candidate_cpu, candidate_peak)}}
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            (directory / 'manifest.json').write_text(json.dumps(manifest))
            with patch('compare_tui_perf.summarize', return_value=summary):
                return compare(directory)

    def test_active_regression_and_memory_are_not_hidden_by_other_checks(self):
        result = self.run_case('stream', 1, 1.21, candidate_peak=8313)
        self.assertFalse(result['evaluated_checks_pass'])
        self.assertEqual(result['cases']['stream-100']['checks'],
                         {'peak_pss': False, 'active_cpu': False})
        self.assertTrue(self.run_case('stream', 1, 1.2, candidate_peak=8312)['evaluated_checks_pass'])

    def test_idle_cpu_requires_metrics_off_and_zero_baseline_is_unavailable_ratio(self):
        result = self.run_case('idle', 0, .201, metrics='off')
        self.assertFalse(result['evaluated_checks_pass'])
        self.assertIsNone(result['cases']['idle-100']['cpu_ratio'])
        self.assertTrue(self.run_case('idle', 0, .2, metrics='off')['evaluated_checks_pass'])

    def test_missing_latency_is_not_zero_and_resize_uses_separate_budget(self):
        self.assertFalse(self.run_case('stream', 1, 1, input_probe=True,
                                       latency_ns=None)['evaluated_checks_pass'])
        self.assertFalse(self.run_case('stream', 1, 1, input_probe=True,
                                       latency_ns=21_000_000)['evaluated_checks_pass'])
        self.assertTrue(self.run_case('resize', 1, 1, input_probe=True,
                                      latency_ns=150_000_000)['evaluated_checks_pass'])
        self.assertFalse(self.run_case('animation', 1, 1,
                                       latency_ns=51_000_000)['evaluated_checks_pass'])

    def test_approval_wait_requires_zero_frames_only_without_input(self):
        counts = {'draw': 1, 'diff': 1, 'flush': 0}
        self.assertFalse(self.run_case('approval', 1, .1, counts=counts)['evaluated_checks_pass'])
        self.assertTrue(self.run_case('approval', 1, .1, counts=counts,
                                      input_probe=True)['evaluated_checks_pass'])

    def test_animation_stable_layout_rebuild_is_a_failure(self):
        result = self.run_case('animation', 1, .5, counts={
            'layout': 1, 'history_rebuild': 0, 'git_scan': 0})
        self.assertFalse(result['evaluated_checks_pass'])

    def test_specific_animation_evidence_is_not_overridden_by_mixed_deadlines(self):
        self.assertTrue(self.run_case('animation', 1, 1, latency_ns=63_000_000,
            animation_budget={'p99_within_50ms': True})['evaluated_checks_pass'])
        self.assertFalse(self.run_case('animation', 1, 1, latency_ns=1_000_000,
            animation_budget={'p99_within_50ms': False})['evaluated_checks_pass'])


if __name__ == '__main__':
    unittest.main()
