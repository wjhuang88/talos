#!/usr/bin/env python3
"""Create isolated, deterministic developer-only Git measurement repositories."""
import argparse
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', type=Path)
    args = parser.parse_args()
    root = args.destination.resolve()
    root.mkdir(parents=True, exist_ok=False)
    (root/'non-repository').mkdir()
    empty_hooks = root/'empty-hooks'
    empty_hooks.mkdir()
    for name in ('small-clean', 'many-untracked'):
        repo = root/name
        repo.mkdir()
        # Host Git only prepares offline test fixtures. The TUI itself uses gix.
        subprocess.run(['git', '-c', 'init.templateDir=', '-c', 'init.defaultBranch=main',
                        'init', '--quiet', str(repo)], check=True)
        (repo/'tracked.txt').write_text('deterministic TUI-PERF-001 fixture\n')
        subprocess.run(['git', '-C', str(repo), 'add', 'tracked.txt'], check=True)
        subprocess.run(['git', '-C', str(repo), '-c', 'user.name=Fixture',
                        '-c', 'user.email=fixture@example.invalid',
                        '-c', f'core.hooksPath={empty_hooks}', '-c', 'commit.gpgsign=false',
                        'commit', '--quiet',
                        '-m', 'offline measurement fixture'], check=True,
                       env={**os.environ,
                            'GIT_AUTHOR_DATE': '2026-01-01T00:00:00Z',
                            'GIT_COMMITTER_DATE': '2026-01-01T00:00:00Z'})
    for directory in range(100):
        folder = root/'many-untracked'/f'group-{directory:03}'
        folder.mkdir()
        for index in range(100):
            (folder/f'file-{index:03}.txt').write_text('fixture\n')
    (root/'manifest.json').write_text(json.dumps({
        'schema': 1, 'seed': 'fixed numeric filenames',
        'cases': {'non-repository': {'files': 0},
                  'small-clean': {'tracked_files': 1, 'untracked_files': 0},
                  'many-untracked': {'tracked_files': 1, 'untracked_files': 10000}},
        'scope': 'offline fixture preparation only; no production repository touched',
    }, indent=2)+'\n')


if __name__ == '__main__':
    main()
