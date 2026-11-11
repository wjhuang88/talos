#!/usr/bin/env python3
"""Summarize completed replay matrices without inventing histogram precision."""
import argparse
import json
import math
from pathlib import Path
import statistics

from tui_perf_validation import validate_metrics, validate_sample


def percentile_bounds(metric, percentile):
    """Return enclosing nanosecond bounds; the last bucket is overflow."""
    if not metric['count']:
        return None
    rank = math.ceil(metric['count'] * percentile / 100)
    cumulative = 0
    for index, count in enumerate(metric['log2_ns_buckets']):
        cumulative += count
        if cumulative >= rank:
            lower = 0 if index == 0 else 2 ** (index - 1)
            upper = metric['max_ns'] if index == 31 else 2 ** index - 1
            return {'lower_ns': lower, 'upper_ns': min(upper, metric['max_ns'])}
    raise ValueError('histogram does not contain percentile')


def spread(values):
    return {'median': statistics.median(values), 'min': min(values), 'max': max(values)}


def summarize(directory):
    manifest = json.loads((directory / 'manifest.json').read_text())
    if manifest['status'] != 'complete':
        raise ValueError('matrix is incomplete')
    args = manifest['arguments']
    expected = {f"{binary['label']}-{scenario}-{history}-r{round_index}": (binary, scenario, history)
                for binary in manifest['binaries'] for scenario in args['scenarios']
                for history in args['histories'] for round_index in range(1, args['rounds'] + 1)}
    if set(manifest['completed']) != set(expected) or len(manifest['completed']) != len(expected):
        raise ValueError('matrix case count is inconsistent')
    groups = {}
    for name in manifest['completed']:
        path = directory / (name + '.json')
        validate_sample(path, args['seconds'], args['warmup'])
        sample = json.loads(path.read_text())
        binary, scenario, history = expected[name]
        if (sample['binary_sha256'], sample['scenario'], sample['history']) != (binary['sha256'], scenario, history):
            raise ValueError('sample identity differs from manifest')
        key = name.rsplit('-r', 1)[0]
        row = {'cpu_percent': sample['cpu_percent'], 'peak_pss_kib': sample['peak_pss_kib'],
               'steady_output_bytes': sample['steady_output_bytes'],
               'steady_covered_seconds': sample['steady_covered_seconds']}
        if args['metrics'] == 'on':
            metrics = validate_metrics(directory / (name + '-metrics.json'), sample['scenario'],
                                       sample['action_count'], args['warmup'] * 1000, args['seconds'])
            row['stage_counts'] = {k: v['count'] for k, v in metrics['steady_stages'].items()}
            row['stage_total_ns'] = {k: v['total_ns'] for k, v in metrics['steady_stages'].items()}
            row['latency_bounds'] = {
                stage: {f'p{p}': percentile_bounds(metrics['steady_stages'][stage], p) for p in (95, 99)}
                for stage in ('input_to_submit', 'deadline_lateness')}
        groups.setdefault(key, []).append(row)
    result = {}
    for key, rows in groups.items():
        if len(rows) != args['rounds']:
            raise ValueError('case lacks required rounds')
        result[key] = {'rounds': rows, 'aggregate': {
            field: spread([row[field] for row in rows])
            for field in ('cpu_percent', 'peak_pss_kib', 'steady_output_bytes', 'steady_covered_seconds')}}
    return {'schema': 1, 'matrix': str(directory.resolve()), 'cases': result,
            'limitations': ['Latency percentiles are histogram bounds, not exact samples or pixel latency.',
                            'Stage timers overlap; do not sum nested timings as CPU cost.',
                            'Process PSS is not heap allocation volume; synthetic PTY excludes terminal emulator.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('matrix', type=Path)
    args = parser.parse_args()
    print(json.dumps(summarize(args.matrix), indent=2))


if __name__ == '__main__':
    main()
