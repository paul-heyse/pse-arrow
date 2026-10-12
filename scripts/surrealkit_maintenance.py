# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exclusive beta rollout execution; operation owners separately settle and activate.

This is a maintenance foundation, not persistent rollout qualification. CLI exit,
TTL renewal and catalog parity never prove a submitted SQL step settled. The
supervisor guards surrealkit_rollout_pending until settle receives actual domain
operation evidence and drains the exact maintenance listener.
"""
from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import math
import os
import re
import signal
import subprocess
import sys
import threading
import time
import tomllib
import uuid
from collections.abc import Callable, Mapping
from pathlib import Path
from types import MappingProxyType

from scripts import native_operation, surreal_server, surrealkit

PENDING = 'surrealkit_rollout_pending'
VERSION = 'surrealkit-maintenance-v1'


class MaintenanceError(RuntimeError):
    """Exact maintenance authority or operation settlement is unavailable."""


def command(mode: str, rollout_id: str | None) -> list[str]:
    if mode == 'up' and rollout_id is None:
        return ['rollout', 'up', '--complete']
    if mode == 'status' and rollout_id is None:
        return ['rollout', 'status']
    if mode in {'start', 'complete', 'repair'} and rollout_id and re.fullmatch(r'[A-Za-z0-9_-]+', rollout_id):
        return ['rollout', mode, rollout_id]
    raise MaintenanceError('Unsupported forward rollout command or identity')


def inputs(folder: Path) -> tuple[dict[str, bytes], str]:
    """Bind all selected frozen inputs; reject ambient target/config injection."""
    if not folder.is_absolute() or folder != folder.resolve(strict=True):
        raise MaintenanceError('Input requires an exact nonsymlinked directory')
    files: dict[str, bytes] = {}
    for path in sorted(folder.rglob('*')):
        if path.is_symlink() or (not path.is_dir() and not path.is_file()):
            raise MaintenanceError('Symlink or special rollout input refused')
        if path.name in {'.env', '.env.local', 'surrealkit.toml'}:
            raise MaintenanceError('Ambient target configuration refused')
        if path.is_file():
            files[path.relative_to(folder).as_posix()] = path.read_bytes()
    manifests = {name: data for name, data in files.items() if Path(name).parent == Path('rollouts') and name.endswith('.toml')}
    if not manifests:
        raise MaintenanceError('Frozen rollout chain is absent')
    for name, data in manifests.items():
        manifest = tomllib.loads(data.decode())
        identity = manifest.get('id')
        if identity != Path(name).stem or not isinstance(identity, str):
            raise MaintenanceError('Rollout manifest identity differs from its file')
        steps = manifest.get('steps', [])
        if not isinstance(steps, list):
            raise MaintenanceError('Malformed rollout steps')
        for step in steps:
            if not isinstance(step, dict):
                raise MaintenanceError('Malformed rollout step')
            references = step.get('files', [])
            if not isinstance(references, list):
                raise MaintenanceError('Malformed frozen file references')
            for frozen in references:
                if not isinstance(frozen, dict) or not isinstance(frozen.get('path'), str):
                    raise MaintenanceError('Frozen beta file identity is absent')
                relative = Path(frozen['path'])
                if relative.is_absolute() or '..' in relative.parts:
                    raise MaintenanceError('External rollout SQL refused')
                frozen_name = (Path('rollouts') / identity / relative).as_posix()
                payload = files.get(frozen_name)
                if payload is None or hashlib.sha256(payload).hexdigest() != frozen.get('hash'):
                    raise MaintenanceError('Frozen SQL missing or falsified')
    digest = hashlib.sha256()
    for name, data in files.items():
        encoded = name.encode()
        digest.update(len(encoded).to_bytes(8, 'big'))
        digest.update(encoded)
        digest.update(len(data).to_bytes(8, 'big'))
        digest.update(data)
    return files, digest.hexdigest()


def environment(config: Mapping[str, object], credentials: Mapping[str, object], folder: Path, endpoint: str) -> dict[str, str]:
    result = {key: value for key, value in os.environ.items() if not key.startswith(('SURREAL', 'DATABASE_'))}
    result.update(SURREALDB_HOST=endpoint, SURREALDB_NAMESPACE=str(config['namespace']),
                  SURREALDB_NAME=str(config['database']), SURREALDB_USER=str(credentials['username']),
                  SURREALDB_PASSWORD=str(credentials['password']), SURREALDB_AUTH_LEVEL='root',
                  SURREALDB_FOLDER=str(folder), SURREALDB_CONNECT_TIMEOUT_SECS='15',
                  SURREALDB_QUERY_TIMEOUT_SECS='30')
    return result


def selected(state: Path) -> dict[str, object]:
    config = surreal_server.config_for(state)
    if (surreal_server.service_directory(state) != state or surreal_server.managed_contexts(state) != [state]
            or config.get('accepting_writes') is not False or config.get('admission') not in {'quiesced', 'validation_required'}):
        raise MaintenanceError('Rollout requires its exclusive closed service owner')
    surreal_server.all_contexts_drained(state)
    if not (state / '.maintenance-listener.json').exists() and surreal_server.active(state):
        raise MaintenanceError('Stop and drain the selected service explicitly before rollout')
    return config


def pending(state: Path, nonce: str) -> dict[str, object]:
    config = selected(state)
    value = config.get(PENDING)
    if not isinstance(value, dict) or value.get('kind') != VERSION or value.get('nonce') != nonce:
        raise MaintenanceError('Exact pending rollout identity required')
    if any(value.get(key) != config.get(key) for key in ('instance_id', 'namespace', 'database')):
        raise MaintenanceError('Pending rollout service selection changed')
    return value


def update(state: Path, value: dict[str, object]) -> None:
    with surreal_server.state_lock(state, cleanup=True):
        config = surreal_server.config_for(state)
        current = config.get(PENDING)
        if current is not None and (not isinstance(current, dict) or current.get('nonce') != value['nonce']):
            raise MaintenanceError('Competing pending rollout retained')
        config[PENDING] = value
        config['accepting_writes'] = False
        surreal_server.write_json(state / 'config.json', config)


def exact_owner(state: Path, owner: Mapping[str, object]) -> None:
    if surreal_server.read_json(state / 'lifecycle-owner.json') != dict(owner) or not surreal_server.reservation_live(dict(owner)):
        raise MaintenanceError('Maintenance owner changed; admission remains closed')


def require_child_drains(state: Path, value: dict[str, object]) -> None:
    """No maintenance restart may reconnect a predecessor's undrained writer."""
    for attempt in value['attempts']:
        if not attempt.get('child_scope_requested'):
            continue
        if attempt.get('child', {}).get('owned_drain') == 'complete':
            continue
        directory = Path(attempt['directory'])
        expected = state / '.surrealkit' / str(value['nonce'])
        if directory != directory.resolve(strict=True) or directory.parent != expected:
            raise MaintenanceError('Unresolved child scope requires exact drain recovery')
        result = surreal_server.read_json(directory / 'child.json')
        native_record = result.get('native_record')
        if not isinstance(native_record, dict):
            raise MaintenanceError('Unresolved child scope requires exact drain recovery')
        drain = native_operation.drained(native_record)
        if not drain.complete:
            raise MaintenanceError(drain.diagnostic() + '; maintenance listener remains closed')
        attempt.setdefault('child', {})['owned_drain'] = 'complete'
        attempt['child']['scope'] = dict(drain.owner or {})
        update(state, value)


def execute(binary: Path, state: Path, folder: Path, mode: str, *, rollout_id: str | None = None,
            pending_nonce: str | None = None, timeout: float = 120) -> dict[str, object]:
    """Execute one fixed command, preserving SQL uncertainty until explicit settlement."""
    if not math.isfinite(timeout) or timeout <= 0:
        raise MaintenanceError('Tool execution needs a finite positive clock')
    args = command(mode, rollout_id)
    provenance = surrealkit.verify(binary)
    files, input_digest = inputs(folder)
    with surreal_server.lifecycle_reservation(state):
        config = selected(state)
        owner = surreal_server.read_json(state / 'lifecycle-owner.json')
        if mode in {'status', 'repair'}:
            if pending_nonce is None:
                raise MaintenanceError('Recovery requires the exact pending nonce')
            value = pending(state, pending_nonce)
            if value['tool'] != provenance:
                raise MaintenanceError('Recovery tool provenance differs from submitted rollout')
            require_child_drains(state, value)
            if value['input_sha256'] != input_digest:
                raise MaintenanceError('Recovery frozen input differs from submitted rollout')
            if mode == 'repair' and rollout_id not in value['rollouts']:
                raise MaintenanceError('Repair does not name a submitted rollout')
        else:
            if config.get(PENDING) is not None or pending_nonce is not None:
                raise MaintenanceError('Pending SQL requires exact settlement; blind replay refused')
            nonce = uuid.uuid4().hex
            value = {'kind': VERSION, 'nonce': nonce, 'instance_id': config['instance_id'],
                     'namespace': config['namespace'], 'database': config['database'],
                     'input_sha256': input_digest, 'command': args, 'phase': 'prepared',
                     'sql_outcome': 'not_submitted', 'rollouts': sorted(Path(name).stem for name in files
                         if Path(name).parent == Path('rollouts') and name.endswith('.toml')),
                     'tool': provenance, 'owner': owner, 'attempts': []}
            if rollout_id is not None and rollout_id not in value['rollouts']:
                raise MaintenanceError('Selected rollout is absent from frozen chain')
            if rollout_id is not None:
                value['rollouts'] = [rollout_id]
            update(state, value)
        owner = surreal_server.read_json(state / 'lifecycle-owner.json')
        nonce = str(value['nonce'])
        directory = surreal_server.checked_directory(state / '.surrealkit' / nonce)
        attempt = directory / uuid.uuid4().hex
        attempt.mkdir(mode=0o700)
        snapshot = attempt / 'input'
        snapshot.mkdir(mode=0o700)
        for name, data in files.items():
            path = snapshot / name
            path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            with path.open('xb') as output:
                output.write(data)
            path.chmod(0o400)
        credentials = surreal_server.read_json(state / 'credentials.json')
        env = environment(config, credentials, snapshot, 'http://127.0.0.1:1')
        # Beta owns chain, fork/order, and frozen-manifest validation. Lint does
        # not connect, and runs before any maintenance listener or SQL submission.
        lint = subprocess.run([str(binary), 'rollout', 'lint'], env=env, cwd=snapshot,
                              capture_output=True, text=True, timeout=15, check=False)
        (attempt / 'lint.stdout').write_text(lint.stdout)
        (attempt / 'lint.stderr').write_text(lint.stderr)
        if lint.returncode:
            raise MaintenanceError('Frozen chain lint failed; pending selection retained')
        record: dict[str, object] = {'command': args, 'directory': str(attempt), 'phase': 'prepared', 'tool': provenance}
        value['attempts'].append(record)
        update(state, value)
        observation: dict[str, object] = {}
        try:
            if mode in {'status', 'repair'}:
                surreal_server.drain_maintenance_listener(state)
            endpoint = surreal_server.maintenance_endpoint(state)
            surreal_server.require_maintenance_owner(state, surreal_server.config_for(state))
            record['phase'] = 'submitted'
            # Recovery inspection/repair cannot erase the original uncertain SQL.
            if mode not in {'status', 'repair'}:
                value.update(phase='submitted', sql_outcome='unknown')
            update(state, value)
            env = environment(config, credentials, snapshot, endpoint)
            surreal_server.write_json(attempt / 'authority.json', {'owner': owner,
                'listener': surreal_server.read_json(state / '.maintenance-listener.json'), 'endpoint': endpoint})
            record['child_scope_requested'] = True
            update(state, value)
            status = native_operation.run_child_scope(
                [sys.executable, '-m', 'scripts.surrealkit_maintenance', '_child',
                 '--state', str(state), '--binary', str(binary), '--folder', str(snapshot),
                 '--attempt', str(attempt), '--mode', mode, '--timeout', str(timeout),
                 *(['--rollout-id', rollout_id] if rollout_id else [])],
                env, cwd=surrealkit.ROOT, observation=observation)
            record.update(exit_code=status, child=observation)
            child_result = surreal_server.read_json(attempt / 'child.json')
            surreal_server.require_maintenance_owner(state, surreal_server.config_for(state))
            record['result'] = child_result
            if status == 0 and child_result.get('reason') == 'exited' and observation.get('owned_drain') == 'complete':
                record['phase'] = 'cli_succeeded'
                if mode not in {'status', 'repair'}:
                    value['phase'] = 'cli_succeeded'
            else:
                record['phase'] = 'unknown'
                if mode not in {'status', 'repair'}:
                    value['phase'] = 'unknown'
        except BaseException as error:
            record.update(phase='unknown', error=f'{type(error).__name__}: {error}', child=observation)
            if mode not in {'status', 'repair'}:
                value['phase'] = 'unknown'
            raise
        finally:
            update(state, value)
            try:
                surreal_server.drain_maintenance_listener(state)
                record['listener_drain'] = 'complete'
                exact_owner(state, owner)
            except BaseException as error:
                record.update(listener_drain='unresolved', drain_error=f'{type(error).__name__}: {error}')
                update(state, value)
                raise
            update(state, value)
        return value


def immutable(value: object) -> object:
    if isinstance(value, dict):
        return MappingProxyType({key: immutable(item) for key, item in value.items()})
    if isinstance(value, list):
        return tuple(immutable(item) for item in value)
    return value


def settle(state: Path, nonce: str,
           operation_settler: Callable[[Path, str, Mapping[str, object]], Mapping[str, object]]) -> dict[str, object]:
    """Join actual operation settlement under authority, then drain; never activate.

    The integration supplies its domain owner, which must inspect actual SQL
    effects and semantic obligations, including uncertain data steps. Its proof
    identifies the selected nonce/input, operation owner/identity and phase/steps.
    A command exit, beta repair result or arbitrary boolean is not that evidence.
    """
    with surreal_server.lifecycle_reservation(state):
        value = pending(state, nonce)
        require_child_drains(state, value)
        owner = surreal_server.read_json(state / 'lifecycle-owner.json')
        surreal_server.drain_maintenance_listener(state)
        endpoint = surreal_server.maintenance_endpoint(state)
        try:
            surreal_server.require_maintenance_owner(state, surreal_server.config_for(state))
            evidence = operation_settler(state, endpoint, immutable(value))
            if (not isinstance(evidence, Mapping) or evidence.get('outcome') != 'settled'
                    or evidence.get('nonce') != nonce or evidence.get('input_sha256') != value['input_sha256']
                    or not isinstance(evidence.get('operation'), Mapping)
                    or not all(isinstance(evidence['operation'].get(key), str) and evidence['operation'][key]
                               for key in ('owner', 'identity'))
                    or not isinstance(evidence.get('phase'), str) or not evidence['phase']
                    or not isinstance(evidence.get('steps'), list) or not evidence['steps']
                    or any(not isinstance(step, Mapping) or not isinstance(step.get('id'), str) or not step['id']
                           or step.get('outcome') not in {'acknowledged', 'reconciled', 'not_submitted'}
                           for step in evidence['steps'])):
                raise MaintenanceError('Actual structured operation settlement evidence required')
            surreal_server.require_maintenance_owner(state, surreal_server.config_for(state))
            value['settlement'] = dict(evidence)
            update(state, value)
        finally:
            surreal_server.drain_maintenance_listener(state)
        exact_owner(state, owner)
        with surreal_server.state_lock(state, cleanup=True):
            config = surreal_server.config_for(state)
            current = config.get(PENDING)
            if not isinstance(current, dict) or current.get('nonce') != nonce:
                raise MaintenanceError('Pending selection changed during settlement')
            surreal_server.write_json(state / '.surrealkit' / nonce / 'settled.json', value)
            config.pop(PENDING)
            config['accepting_writes'] = False
            surreal_server.write_json(state / 'config.json', config)
        return value


def owned_child(binary: Path, state: Path, folder: Path, attempt: Path, mode: str,
                rollout_id: str | None, timeout: float) -> int:
    """Internal admitted child: watch exact owner, stop/reap only this new session."""
    if native_operation.scope_owner() is None:
        raise MaintenanceError('Internal tool child requires its admitted native scope')
    if not math.isfinite(timeout) or timeout <= 0:
        raise MaintenanceError('Invalid internal child clock')
    owner_path = native_operation.current()
    if owner_path is None:
        raise MaintenanceError('Internal child lacks its exact native record')
    native_record = surreal_server.read_json(owner_path)
    surreal_server.write_json(attempt / 'child.json', {'reason': 'prepared', 'native_record': native_record})
    authority = surreal_server.read_json(attempt / 'authority.json')
    def require_bound_owner() -> None:
        config = surreal_server.config_for(state)
        exact_owner(state, authority['owner'])
        surreal_server.require_maintenance_owner(state, config)
        if (surreal_server.read_json(state / '.maintenance-listener.json') != authority['listener']
                or authority['endpoint'] != 'http://127.0.0.1:' + str(config['port'])
                or os.environ.get('SURREALDB_HOST') != authority['endpoint']):
            raise MaintenanceError('Original maintenance listener binding changed')

    owner_lost = threading.Event()
    def watch_bound_owner(child: subprocess.Popen[bytes]) -> None:
        while child.poll() is None:
            try:
                require_bound_owner()
            except (MaintenanceError, surreal_server.SupervisorError, OSError, KeyError, ValueError):
                owner_lost.set()
                return
            time.sleep(0.1)

    interrupted = threading.Event()
    old_handlers = {sig: signal.signal(sig, lambda _sig, _frame: interrupted.set())
                    for sig in (signal.SIGTERM, signal.SIGINT)}
    child: subprocess.Popen[bytes] | None = None
    child_pidfd: int | None = None
    reason = 'not_started'
    status = 125
    try:
        require_bound_owner()
        with (attempt / 'stdout').open('xb') as stdout, (attempt / 'stderr').open('xb') as stderr:
            child = subprocess.Popen([str(binary), *command(mode, rollout_id)], env=dict(os.environ),
                                     cwd=folder, stdout=stdout, stderr=stderr, start_new_session=True)
            # The unreaped Popen child pins its PID until this fd is captured.
            child_pidfd = os.pidfd_open(child.pid)
            threading.Thread(target=watch_bound_owner, args=(child,), daemon=True).start()
            deadline = time.monotonic() + timeout
            while True:
                try:
                    require_bound_owner()
                except (MaintenanceError, surreal_server.SupervisorError, OSError, KeyError, ValueError):
                    reason = 'owner_lost'
                    break
                if owner_lost.is_set():
                    reason = 'owner_lost'
                    break
                if interrupted.is_set():
                    reason = 'interrupted'
                    break
                if time.monotonic() >= deadline:
                    reason = 'timeout'
                    break
                try:
                    status = child.wait(timeout=min(0.1, deadline - time.monotonic()))
                    try:
                        require_bound_owner()
                    except (MaintenanceError, surreal_server.SupervisorError, OSError, KeyError, ValueError):
                        reason = 'owner_lost'
                    else:
                        reason = 'exited'
                    break
                except subprocess.TimeoutExpired:
                    continue
    finally:
        try:
            if child is not None:
                if reason != 'exited':
                    with contextlib.suppress(ProcessLookupError):
                        if child_pidfd is not None:
                            signal.pidfd_send_signal(child_pidfd, signal.SIGTERM)
                    try:
                        child.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        with contextlib.suppress(ProcessLookupError):
                            if child_pidfd is not None:
                                signal.pidfd_send_signal(child_pidfd, signal.SIGKILL)
                        child.wait(timeout=8)
                # Process reap is distinct from the outer native scope drain.
                status = child.returncode if child.returncode is not None else 125
            surreal_server.write_json(attempt / 'child.json', {'reason': reason, 'exit_code': status, 'native_record': native_record})
        finally:
            if child_pidfd is not None:
                os.close(child_pidfd)
            for sig, handler in old_handlers.items():
                signal.signal(sig, handler)
    return status if reason == 'exited' and status >= 0 else 125


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['up', 'start', 'complete', 'status', 'repair', '_child'])
    parser.add_argument('--state', type=Path, required=True)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--folder', type=Path, required=True)
    parser.add_argument('--rollout-id')
    parser.add_argument('--pending-nonce')
    parser.add_argument('--timeout', type=float, default=120)
    parser.add_argument('--attempt', type=Path)
    parser.add_argument('--mode', dest='child_mode')
    args = parser.parse_args()
    try:
        if args.mode == '_child':
            if args.attempt is None or args.child_mode is None:
                raise MaintenanceError('Internal child identity missing')
            return owned_child(args.binary, args.state, args.folder, args.attempt, args.child_mode, args.rollout_id, args.timeout)
        result = execute(args.binary, args.state, args.folder, args.mode, rollout_id=args.rollout_id,
                         pending_nonce=args.pending_nonce, timeout=args.timeout)
        print(json.dumps(result, indent=2))
        return 0 if result['attempts'][-1]['phase'] == 'cli_succeeded' else 1
    except (MaintenanceError, surrealkit.ToolError, surreal_server.SupervisorError, OSError, ValueError, subprocess.SubprocessError) as error:
        print(f'surrealkit-maintenance: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
