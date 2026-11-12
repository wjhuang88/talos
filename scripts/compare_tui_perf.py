#!/usr/bin/env python3
"""Evaluate matching replay cases against the task's frozen CPU/PSS budgets."""
import argparse
import json
from pathlib import Path

from summarize_tui_perf import summarize


def compare(directory):
    manifest = json.loads((directory / 'manifest.json').read_text())
    labels = {binary['label'] for binary in manifest['binaries']}
    if labels != {'baseline', 'candidate'} or len(manifest['binaries']) != 2:
        raise ValueError('comparison requires exactly baseline and candidate binaries')
    summary = summarize(directory)
    args = manifest['arguments']
    cases = {}
    for scenario in args['scenarios']:
        for history in args['histories']:
            key = f'{scenario}-{history}'
            baseline = summary['cases'][f'baseline-{key}']
            candidate = summary['cases'][f'candidate-{key}']
            base_cpu = baseline['aggregate']['cpu_percent']['median']
            next_cpu = candidate['aggregate']['cpu_percent']['median']
            memory_limit = baseline['aggregate']['peak_pss_kib']['max'] * 1.2 + 8192
            checks = {'peak_pss': candidate['aggregate']['peak_pss_kib']['max'] <= memory_limit}
            if scenario in ('animation', 'stream', 'scroll', 'resize', 'workspace-switch', 'transcript-commit'):
                checks['active_cpu'] = next_cpu <= base_cpu * 1.2
            if scenario == 'idle' and not args.get('input_probe'):
                if args['metrics'] == 'off':
                    checks['idle_cpu'] = next_cpu <= (0.2 if args['profile'] == 'debug' else 0.1)
                else:
                    checks['idle_frames'] = all(
                        row['stage_counts'][stage] == 0
                        for row in candidate['rounds'] for stage in ('draw', 'diff', 'flush'))
            if args['metrics'] == 'on' and scenario == 'approval' and not args.get('input_probe'):
                checks['approval_wait_frames'] = all(
                    row['stage_counts'][stage] == 0
                    for row in candidate['rounds'] for stage in ('draw', 'diff', 'flush'))
            if args['metrics'] == 'on' and scenario == 'animation':
                checks['stable_layout_history'] = all(
                    row['stage_counts'][stage] == 0
                    for row in candidate['rounds'] for stage in ('layout', 'history_rebuild'))
                if not args['workspace']:
                    checks['no_git_without_workspace'] = all(
                        row['stage_counts']['git_scan'] == 0 for row in candidate['rounds'])
            latency_budgets_ms = None
            if args['metrics'] == 'on' and args.get('input_probe'):
                if scenario == 'resize':
                    latency_budgets_ms = (150, 200) if args['profile'] == 'debug' else (100, 150)
                else:
                    latency_budgets_ms = (20, 35) if args['profile'] == 'debug' else (10, 20)
                for percentile, budget in zip(('p95', 'p99'), latency_budgets_ms, strict=True):
                    checks[f'input_{percentile}'] = all(
                        row['resize_debug_budget']['p95_within_150ms']
                        if scenario == 'resize' and args['profile'] == 'debug'
                        and percentile == 'p95' and row.get('resize_debug_budget') is not None else
                        row['latency_bounds']['input_to_submit'][percentile] is not None
                        and row['latency_bounds']['input_to_submit'][percentile]['upper_ns'] <= budget * 1_000_000
                        for row in candidate['rounds'])
            if args['metrics'] == 'on' and scenario == 'animation':
                checks['animation_deadline_p99'] = all(
                    row['animation_deadline_budget']['p99_within_50ms']
                    if row.get('animation_deadline_budget') is not None else
                    row['latency_bounds']['deadline_lateness']['p99'] is not None
                    and row['latency_bounds']['deadline_lateness']['p99']['upper_ns'] <= 50_000_000
                    for row in candidate['rounds'])
            cases[key] = {
                'baseline_cpu_median': base_cpu, 'candidate_cpu_median': next_cpu,
                'cpu_ratio': next_cpu / base_cpu if base_cpu else None,
                'paired_cpu_percentage_point_deltas': [
                    c['cpu_percent'] - b['cpu_percent']
                    for b, c in zip(baseline['rounds'], candidate['rounds'], strict=True)],
                'candidate_peak_pss_kib': candidate['aggregate']['peak_pss_kib']['max'],
                'peak_pss_budget_kib': memory_limit, 'input_p95_p99_budget_ms': latency_budgets_ms,
                'checks': checks,
                'resize_p95_evidence': [
                    '150ms threshold count' if args['profile'] == 'debug'
                    and row.get('resize_debug_budget') is not None
                    else 'histogram upper bound'
                    for row in candidate['rounds']] if scenario == 'resize' else None,
                'animation_deadline_evidence': [
                    'visible-animation threshold count' if row.get('animation_deadline_budget') is not None
                    else 'legacy mixed-deadline histogram upper bound'
                    for row in candidate['rounds']] if scenario == 'animation' else None,
            }
    return {'schema': 1, 'source_commits_from_external_receipts_required': True,
            'matrix': str(directory.resolve()), 'cases': cases,
            'evaluated_checks_pass': all(all(case['checks'].values()) for case in cases.values()),
            'limitations': [
                'Only listed checks are evaluated; this is not whole-task acceptance.',
                'CPU ratios at zero baseline are unavailable, not zero.',
                'Native terminal, allocations and long-run heap stability require separate evidence; input latency is checked only with probes and animation lateness only in animation cases.',
                'Pair deltas follow manifest round order; fixed baseline-first ordering can retain host drift.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('matrix', type=Path)
    args = parser.parse_args()
    print(json.dumps(compare(args.matrix), indent=2))


if __name__ == '__main__':
    main()
