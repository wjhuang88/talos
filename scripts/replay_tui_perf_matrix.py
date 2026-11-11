#!/usr/bin/env python3
"""Sequential TUI measurements; baseline/candidate binaries alternate per case."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
from tui_perf_validation import validate_sample, validate_metrics


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def command_output(args):
    try:
        return subprocess.check_output(args, text=True, stderr=subprocess.DEVNULL).strip()
    except (OSError, subprocess.CalledProcessError):
        return 'unavailable'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', action='append', required=True, metavar='LABEL=PATH')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--profile', choices=['debug', 'release-with-symbols'], required=True)
    parser.add_argument('--scenarios', nargs='+', default=['idle', 'animation', 'stream', 'approval', 'scroll', 'resize'],
                        choices=['idle', 'animation', 'stream', 'approval', 'scroll', 'resize'])
    parser.add_argument('--histories', nargs='+', type=int, default=[100, 1000, 10000], choices=[100, 1000, 10000])
    parser.add_argument('--rounds', type=int, default=3)
    parser.add_argument('--seconds', type=int, default=35)
    parser.add_argument('--warmup', type=int, default=5)
    parser.add_argument('--workspace', default='')
    parser.add_argument('--input-probe', action='store_true')
    parser.add_argument('--metrics', choices=['on', 'off'], default='on')
    args = parser.parse_args()
    if not 1 <= args.rounds <= 100 or not 0 <= args.warmup < args.seconds <= 3600:
        parser.error('invalid round or time bounds')
    binaries = []
    for spec in args.binary:
        label, sep, value = spec.partition('=')
        if not sep or not label or any(c not in 'abcdefghijklmnopqrstuvwxyz0123456789-_' for c in label):
            parser.error('binary labels must use lowercase ASCII letters, digits, dash or underscore')
        if label in [b['label'] for b in binaries]:
            parser.error('duplicate binary label')
        path = Path(value).resolve(strict=True)
        binaries.append({'label': label, 'path': str(path), 'sha256': digest(path)})
    # Never overwrite or silently resume an old experiment with new conditions.
    args.output.mkdir(parents=True, exist_ok=False)
    source_root = Path(__file__).resolve().parents[1]
    sources = {str(p.relative_to(source_root)): digest(p)
               for folder in ['crates/talos-tui/src', 'crates/talos-tui/examples']
               for p in sorted((source_root / folder).rglob('*.rs'))}
    power = {}
    for location in ['/sys/firmware/acpi/platform_profile', '/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor']:
        try:
            power[location] = Path(location).read_text().strip()
        except OSError:
            power[location] = 'unavailable'
    manifest = {'schema': 1, 'status': 'running', 'binaries': binaries, 'source_sha256': sources,
                'source_revision': command_output(['git', '-C', str(source_root), 'rev-parse', 'HEAD']),
                'rustc': command_output(['rustc', '--version']), 'platform': platform.platform(),
                'power': power, 'profile_asserted_by_caller': args.profile,
                'arguments': {k: str(v) if isinstance(v, Path) else v for k,v in vars(args).items()},
                'completed': [], 'limitations': ['Source hashes describe current checkout; caller must bind each prebuilt binary to its own source.',
                                               'Synthetic PTY is not native device or terminal emulator evidence.']}
    manifest_path = args.output / 'manifest.json'
    def save():
        manifest_path.write_text(json.dumps(manifest, indent=2)+'\n')
    save()
    try:
        for round_index in range(1, args.rounds+1):
            for history in args.histories:
                for scenario in args.scenarios:
                    for binary in binaries:
                        if digest(Path(binary['path'])) != binary['sha256']:
                            raise RuntimeError('binary changed during experiment')
                        name = f"{binary['label']}-{scenario}-{history}-r{round_index}"
                        env = os.environ.copy()
                        env.pop('TALOS_TUI_METRICS', None)
                        env.pop('TALOS_TUI_METRICS_WARMUP_MS', None)
                        if args.metrics == 'on':
                            env['TALOS_TUI_METRICS_WARMUP_MS'] = str(args.warmup*1000)
                            env['TALOS_TUI_METRICS'] = str((args.output / (name+'-metrics.json')).resolve())
                        command = [sys.executable, str(source_root/'scripts/measure_tui_perf.py'), binary['path'],
                                   '--scenario', scenario, '--history', str(history), '--seconds', str(args.seconds),
                                   '--warmup', str(args.warmup), '--workspace', args.workspace,
                                   '--output', str(args.output/(name+'.json'))]
                        if args.input_probe:
                            command.append('--input-probe')
                        subprocess.run(command, env=env, check=True)
                        coverage = validate_sample(args.output/(name+'.json'), args.seconds, args.warmup)
                        if args.metrics == 'on':
                            sample = json.loads((args.output/(name+'.json')).read_text())
                            validate_metrics(env['TALOS_TUI_METRICS'], scenario, sample['action_count'], args.warmup*1000, args.seconds)
                        (args.output/(name+'-coverage.json')).write_text(json.dumps(coverage, indent=2)+'\n')
                        manifest['completed'].append(name)
                        save()
                        print(name, flush=True)
        manifest['status'] = 'complete'
    except BaseException:
        manifest['status'] = 'interrupted_or_failed'
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
