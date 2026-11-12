"""Validation shared by matrix execution and post-run evidence audits."""
import json
from pathlib import Path

STAGES = ('draw', 'layout', 'history_rebuild', 'history_hit', 'git_scan',
          'diff', 'flush', 'input', 'output', 'timer', 'input_to_submit', 'resize', 'stream')


def validate_sample(path, seconds, warmup):
    data = json.loads(Path(path).read_text())
    if data.get('exit_code') != 0:
        raise ValueError('replay did not exit successfully')
    ready = data.get('ready_seconds')
    if not isinstance(ready, (int, float)) or ready < 0:
        raise ValueError('missing valid ready boundary')
    samples = data.get('samples', [])
    steady = [s for s in samples if ready+warmup <= s['seconds'] <= ready+seconds]
    if len(steady) < 2:
        raise ValueError('insufficient steady samples')
    times = [s['seconds'] for s in samples]
    if any(b <= a for a,b in zip(times, times[1:])):
        raise ValueError('non-monotonic sample times')
    if any(b['seconds']-a['seconds'] > .75 for a,b in zip(steady, steady[1:])):
        raise ValueError('sampling gap exceeds 750ms')
    span = steady[-1]['seconds']-steady[0]['seconds']
    # Two 250ms sample boundaries plus 250ms scheduling tolerance. Report the
    # actual covered span; nominal duration alone is never completion evidence.
    if span < seconds-warmup-.75 or data['wall_seconds'] < ready+seconds-.25:
        raise ValueError('replay exited before the required measurement window')
    return {'steady_first_seconds': steady[0]['seconds'],
            'steady_last_seconds': steady[-1]['seconds'],
            'steady_covered_seconds': span, 'nominal_steady_seconds': seconds-warmup}


def validate_metrics(path, scenario=None, actions=0, warmup_ms=None, measurement_seconds=None):
    data = json.loads(Path(path).read_text())
    if data.get('schema') not in (1, 2) or not isinstance(data.get('stages'), dict):
        raise ValueError('invalid metrics schema')
    required = STAGES + (('history_scroll', 'screen_resize', 'deadline_lateness', 'frame_preparation') if data['schema'] == 2 else ())
    validate_stage_table(data['stages'], required)
    if data['stages']['draw']['count'] == 0:
        raise ValueError('no rendered frames')
    if warmup_ms is not None:
        if data.get('warmup_ms') != warmup_ms or not isinstance(data.get('steady_stages'), dict) or data.get('steady_elapsed_ns', 0) <= 0:
            raise ValueError('missing or inconsistent steady metrics window')
        validate_stage_table(data['steady_stages'], required)
        if measurement_seconds is not None and data['steady_elapsed_ns']/1e9 < measurement_seconds-warmup_ms/1000-.75:
            raise ValueError('metrics steady window too short')
        if scenario in ('animation', 'stream', 'scroll', 'resize', 'transcript-commit') and data['steady_stages']['draw']['count'] == 0:
            raise ValueError('active scenario has no steady frames')
    if scenario == 'transcript-commit':
        table = data['steady_stages'] if warmup_ms is not None else data['stages']
        minimum = max(1, int(measurement_seconds - warmup_ms / 1000) - 2) if measurement_seconds is not None and warmup_ms is not None else 1
        if table['history_rebuild']['count'] < minimum or table['output']['count'] < minimum * 3:
            raise ValueError('insufficient actual transcript updates')
    if scenario == 'workspace-switch':
        table = data['steady_stages'] if warmup_ms is not None else data['stages']
        validate_stage_table(table, ('workspace_change',))
        minimum = max(1, int(measurement_seconds - warmup_ms / 1000) - 2) if measurement_seconds is not None and warmup_ms is not None else 1
        if table['workspace_change']['count'] < minimum:
            raise ValueError('insufficient actual workspace changes')
    if scenario in ('scroll', 'resize'):
        stage = 'history_scroll' if scenario == 'scroll' else 'screen_resize'
        table = data['steady_stages'] if warmup_ms is not None else data['stages']
        if actions <= 0 or data['schema'] < 2 or table[stage]['count'] < max(1, actions-1):
            raise ValueError('no evidence of actual scenario state changes')
    return data


def validate_stage_table(table, required):
    for stage in required:
        metric = table.get(stage, {})
        for field in ('count', 'total_ns', 'max_ns'):
            if type(metric.get(field)) is not int or not 0 <= metric[field] <= 2**64-1:
                raise ValueError(f'invalid {stage}.{field}')
        buckets = metric.get('log2_ns_buckets')
        if not isinstance(buckets, list) or len(buckets) != 32 or any(type(v) is not int or not 0 <= v <= 2**64-1 for v in buckets):
            raise ValueError(f'invalid {stage} histogram')
        if min(sum(buckets), 2**64-1) != metric['count']:
            raise ValueError(f'inconsistent {stage} count/histogram')
