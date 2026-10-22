# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit actual-workstation Bash bridge controls using owned disposable checkouts."""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from scripts import pse_env


def main() -> int:
    bridge = Path.home() / '.config/codex/bash-env.bash'
    if not bridge.is_file():
        raise ValueError('workstation Codex Bash bridge is unavailable')
    with tempfile.TemporaryDirectory(prefix='pse-plan50-shell-') as directory:
        root = Path(directory)
        main = root / 'main'
        worktree = root / 'worktree'
        subprocess.run(['git', 'init', '--quiet', str(main)], check=True)
        subprocess.run(['git', '-C', str(main), '-c', 'user.name=fixture', '-c', 'user.email=fixture@invalid',
                        'commit', '--quiet', '--allow-empty', '-m', 'owned shell fixture'], check=True)
        subprocess.run(['git', '-C', str(main), 'worktree', 'add', '--quiet', '--detach', str(worktree), 'HEAD'], check=True)
        env = {**os.environ, 'BASH_ENV': str(bridge)}
        # A fresh shell must select each checkout even when its parent came from another.
        for checkout in [main, worktree]:
            (checkout / '.envrc').write_text(f'export PSE_BRIDGE_CHECKOUT={checkout.name}\nexport CARGO_BUILD_JOBS=16\n')
            subprocess.run(['/usr/bin/direnv', 'allow', str(checkout)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        probe = "import json,os; print(json.dumps({k:os.environ.get(k) for k in ['PSE_BRIDGE_CHECKOUT','BASH_ENV','CARGO_BUILD_JOBS']}))"
        for checkout in [main, worktree]:
            result = subprocess.run(['bash', '-c', 'exec "$@"', 'bridge-probe', sys.executable, '-c', probe],
                                    cwd=checkout, env=env, text=True, capture_output=True, check=True)
            actual = json.loads(result.stdout)
            if actual != {'PSE_BRIDGE_CHECKOUT': checkout.name, 'BASH_ENV': None, 'CARGO_BUILD_JOBS': '16'}:
                raise ValueError('fresh checkout activation differs from selected bridge contract')
        nested = (
            "import json,os,subprocess,sys; e=dict(os.environ); "
            "e.update(CARGO_BUILD_JOBS='16', RUSTC_WRAPPER='', OMP_NUM_THREADS='3', SCCACHE_DIR='/selected/cache literal', SCCACHE_CONF='/selected/cache.toml', PSE_BRIDGE_LITERAL='$HOME `literal`'); "
            "s=subprocess.check_output(['bash','-c','exec \\\"$@\\\"','nested',sys.executable,'-c',"
            "\"import json,os; print(json.dumps({k:os.environ.get(k) for k in ['BASH_ENV','CARGO_BUILD_JOBS','RUSTC_WRAPPER','OMP_NUM_THREADS','SCCACHE_DIR','SCCACHE_CONF','PSE_BRIDGE_LITERAL']}))\"],env=e,text=True); "
            "print(s,end='')"
        )
        # Pass shell source literally; no interpolated environment values are executed.
        nested = nested.replace('\\"$@\\"', '"$@"')
        actual = json.loads(subprocess.check_output(['bash', '-c', 'exec "$@"', 'bridge-probe', sys.executable, '-c', nested],
                                                  cwd=main, env=env, text=True, stderr=subprocess.PIPE))
        expected = {'BASH_ENV': None, 'CARGO_BUILD_JOBS': '16', 'RUSTC_WRAPPER': '',
                    'OMP_NUM_THREADS': '3', 'SCCACHE_DIR': '/selected/cache literal',
                    'SCCACHE_CONF': '/selected/cache.toml', 'PSE_BRIDGE_LITERAL': '$HOME `literal`'}
        if actual != expected:
            raise ValueError('nested compiler/native selections changed')
        # Exercise this checkout's actual composition owner against real local defaults.
        (main / '.envrc.local').write_text(
            'export CARGO_BUILD_JOBS=8\nexport SCCACHE_DIR="/local cache"\n'
            'export OMP_NUM_THREADS=2\n'
        )
        baseline = {'HOME': str(root), 'PATH': '/foreign/venv/bin:/usr/bin:/bin'}
        local = pse_env.base_environment(main, baseline)
        if local['CARGO_BUILD_JOBS'] != '8' or local['SCCACHE_DIR'] != '/local cache':
            raise ValueError('real local default selection differs from the owner contract')
        caller = pse_env.base_environment(main, {
            **baseline, 'CARGO_BUILD_JOBS': '16', 'SCCACHE_DIR': '/caller cache',
            'OMP_NUM_THREADS': '3',
        })
        if (caller['CARGO_BUILD_JOBS'], caller['SCCACHE_DIR'], caller['OMP_NUM_THREADS']) != ('16', '/caller cache', '3'):
            raise ValueError('explicit caller selections did not win over local defaults')
        if str(main / '.venv/bin') not in caller['PATH'].split(os.pathsep)[:2]:
            raise ValueError('selected checkout environment did not precede foreign PATH')
        if (main / '.venv').exists():
            raise ValueError('fixture unexpectedly created or borrowed a checkout environment')
        for name, declaration, approved in [('unapproved', 'export PSE_BRIDGE_CHECKOUT=wrong\n', False),
                                             ('failed', 'echo controlled-activation-failure >&2; exit 7\n', True)]:
            selected = root / name
            selected.mkdir()
            (selected / '.envrc').write_text(declaration)
            if approved:
                subprocess.run(['/usr/bin/direnv', 'allow', str(selected)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
            result = subprocess.run(['bash', '-c', 'touch command-must-not-run'], cwd=selected,
                                    env=env, capture_output=True, check=False)
            if result.returncode != 125 or (selected / 'command-must-not-run').exists():
                raise ValueError('failed/unapproved activation did not refuse before command effects')
        print(json.dumps({'passed': 7, 'failed': 0, 'scope': 'actual bridge; fresh main/worktree, nested selected compiler/native/cache values, real local/caller defaults, foreign PATH precedence without environment creation, unapproved and failed activation; no real compiler failure reproduced'}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
