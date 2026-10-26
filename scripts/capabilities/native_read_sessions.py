# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Endpoint-owned foreground native read workers, requests and cursor associations."""

from __future__ import annotations

import asyncio
import hashlib
import json
import tempfile
import uuid
from collections.abc import Coroutine
from contextlib import ExitStack, contextmanager
from pathlib import Path
from typing import TYPE_CHECKING, Literal, cast

from scripts.capabilities import execution_basis
from scripts.capabilities.cleanup import settle
from scripts.capabilities.native_extension import ExtensionSelectionError
from scripts.capabilities.native_read_protocol import (
    MAXIMUM_REQUEST_BYTES,
    MAXIMUM_REQUEST_FRAME_BYTES,
    MAXIMUM_RESPONSE_FRAME_BYTES,
    SESSION_ENVIRONMENT,
    ReadDeadlineExpiredError,
    ReadRequest,
    ReadResponse,
    SessionAdmission,
    SessionProtocolError,
    encoded,
    operation_document,
    receive,
)
from scripts.capabilities.source_basis import Sources
from scripts.capabilities.workers import WORKERS, selected_worker_environment
from scripts.telemetry_process import helper_environment

if TYPE_CHECKING:
    from collections.abc import Callable, Generator
    from typing import BinaryIO

    from pydantic import JsonValue

    from scripts.capabilities.native_contracts import Binding


def _digest(value: object) -> str:
    return hashlib.sha256(
        json.dumps(
            value, sort_keys=True, allow_nan=False, separators=(",", ":")
        ).encode()
    ).hexdigest()


def _at(value: JsonValue, path: tuple[str, ...]) -> JsonValue:
    for name in path:
        if not isinstance(value, dict) or name not in value:
            return None
        value = value[name]
    return value


def page_basis(binding: Binding, arguments: dict[str, JsonValue]) -> str:
    """Original request facts, removing only owner-declared continuation inputs."""
    copied: dict[str, JsonValue] = json.loads(json.dumps(arguments, allow_nan=False))
    for continuation in binding.continuations:
        path = tuple(continuation.input)
        parent: JsonValue = copied
        for name in path[:-1]:
            parent = parent.get(name) if isinstance(parent, dict) else None
        if path and isinstance(parent, dict):
            parent.pop(path[-1], None)
    return _digest(copied)


def context_identity(binding: Binding, arguments: dict[str, JsonValue]) -> str:
    """Project immutable context fields declared by the operation's Rust owner."""
    return _digest(
        {
            "immutable_fields": {
                name: arguments.get(name) for name in binding.context_fields
            },
            "route": "native-read-prepared"
            if binding.prepare_import
            else "native-read",
            "prerequisites": [
                item.model_dump(mode="json") for item in binding.requirements
            ],
        }
    )


def _continuation_key(
    operation: str,
    path: tuple[str, ...],
    kind: str,
    token: JsonValue,
    identity: str,
    basis: str,
) -> tuple[object, ...]:
    if kind == "offset":
        # An ordinal is an independently admitted selector in each original query.
        return (operation, path, kind, identity, basis, _digest(token))
    if kind == "opaque":
        # A semantic token known to this endpoint cannot be made "unknown" merely
        # by supplying another physical context or changing the original query.
        return (operation, path, kind, _digest(token))
    raise SessionProtocolError("unsupported owner-declared continuation kind")


@contextmanager
def _session_log() -> Generator[BinaryIO]:
    with tempfile.TemporaryFile() as stream:
        yield stream


class ReadSession:
    """One supervised process; active reads finish before EOF retires its Runtime."""

    def __init__(self, root: Path, identity: str) -> None:
        self.root = root
        self.identity = identity
        self.incarnation = uuid.uuid4().hex
        self.busy = False
        self.retired = False
        self._lock = asyncio.Lock()
        self._process: asyncio.subprocess.Process | None = None
        self._temporary: tempfile.TemporaryDirectory[str] | None = None
        self._files = ExitStack()
        self._closed = False
        self._close_failure: BaseException | None = None

    async def _start(self, handoff: execution_basis.Handoff) -> None:
        declaration = WORKERS[handoff.worker]
        environment = helper_environment()
        selected_environment = selected_worker_environment(
            handoff.worker, handoff.operation, environment
        )
        self._temporary = tempfile.TemporaryDirectory(prefix="pse-native-read-session-")
        directory = Path(self._temporary.name)
        # Capture the actual fixed route and its local import closure once per worker.
        sources = Sources(
            self.root,
            (
                declaration.module.replace(".", "/") + ".py",
                *declaration.sources,
                *handoff.definition.sources,
            ),
        )
        definition = handoff.definition.model_copy(
            update={
                "sources": {
                    name: source.digest for name, source in sources.sources.items()
                }
            }
        )
        admission = SessionAdmission(
            incarnation=self.incarnation,
            checkout=str(self.root),
            worker=cast(
                "Literal['native-read', 'native-read-prepared']", handoff.worker
            ),
            route_revision=execution_basis.route_revision(handoff.worker),
            definition=definition,
        )
        source = directory / "session.json"
        with source.open("xb") as stream:
            stream.write(admission.model_dump_json().encode())
        source.chmod(0o600)
        command = [str(self.root / ".venv/bin/python"), "-m", declaration.module]
        if declaration.launch == "environment":
            command = [
                str(self.root / "scripts/pse-env"),
                *selected_environment,
                "--",
                *command,
            ]
        environment = {**environment, SESSION_ENVIRONMENT: str(source)}
        # A retained session is not a loop under an old one-request environment file.
        environment.pop(execution_basis.ENVIRONMENT, None)
        errors = self._files.enter_context(_session_log())
        self._process = await asyncio.create_subprocess_exec(
            *command,
            cwd=self.root,
            env=environment,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=errors,
            limit=MAXIMUM_RESPONSE_FRAME_BYTES,
        )

    async def invoke(
        self,
        binding: Binding,
        arguments: dict[str, JsonValue],
        deadline_monotonic_ns: int | None,
    ) -> ReadResponse:
        async with self._lock:
            if self._closed or self.retired:
                raise SessionProtocolError("native read session is retired")
            raw = operation_document(binding.operation, arguments)
            route = "native-read-prepared" if binding.prepare_import else "native-read"
            admitted = execution_basis.handoff(self.root, route, binding.operation, raw)
            request = ReadRequest(
                session=self.incarnation,
                request=uuid.uuid4().hex,
                context=self.identity,
                operation=binding.operation,
                arguments=arguments,
                admission=admitted,
                deadline_monotonic_ns=deadline_monotonic_ns,
            )
            request.require_live_deadline()
            if self._process is None:
                await self._start(admitted)
            request.require_live_deadline()
            process = self._process
            if process is None or process.stdin is None or process.stdout is None:
                raise SessionProtocolError("native read worker pipe is unavailable")
            process.stdin.write(encoded(request, MAXIMUM_REQUEST_FRAME_BYTES))
            await process.stdin.drain()
            response = await receive(
                process.stdout, ReadResponse, MAXIMUM_RESPONSE_FRAME_BYTES
            )
            response.require_request(request)
            self.retired = response.retired
            return response

    async def close(self) -> None:
        # The manager first settles its observed/native request tasks. The lock also
        # protects explicitly associated cursor reads awaiting this same state owner.
        async with self._lock:
            if self._closed:
                if self._close_failure is not None:
                    raise self._close_failure
                return
            self._closed = True
            self.retired = True
            process = self._process
            failures: list[BaseException] = []
            cancellation: asyncio.CancelledError | None = None
            try:
                if process is not None:
                    if process.stdin is not None:
                        process.stdin.close()

                    # Invalid framing must not leave a worker blocked writing to a
                    # full output pipe while its native owners are being drained.
                    async def discard() -> None:
                        if process.stdout is not None:
                            while await process.stdout.read(64 << 10):
                                pass

                    await settle(discard(), self._wait(process))
            except asyncio.CancelledError as error:
                cancellation = error
                if error.__cause__ is not None:
                    failures.append(error.__cause__)
            except BaseException as error:
                failures.append(error)
            # These resources are independent: a process or log failure cannot
            # suppress cleanup of the other authenticated, session-owned paths.
            for cleanup in (
                self._files.close,
                self._temporary.cleanup if self._temporary is not None else None,
            ):
                if cleanup is not None:
                    try:
                        cleanup()
                    except BaseException as error:
                        failures.append(error)
            if failures:
                self._close_failure = BaseExceptionGroup(
                    "Native read session settlement failed", failures
                )
            if cancellation is not None:
                raise cancellation from self._close_failure
            if self._close_failure is not None:
                raise self._close_failure

    @staticmethod
    async def _wait(process: asyncio.subprocess.Process) -> None:
        code = await process.wait()
        if code:
            raise RuntimeError(
                "native read worker failed during owned settlement; source diagnostics withheld"
            )


class ReadSessions:
    """Retain idle contexts; busy independent reads get their own admitted worker.

    There is no new host width/resource limit. Each new worker uses the existing
    supervised/native admission, and the endpoint owns idle state, pending results,
    request tasks and exact physical worker teardown.
    """

    def __init__(
        self, root: Path, *, factory: Callable[[Path, str], ReadSession] = ReadSession
    ) -> None:
        self.root = root.resolve()
        self._factory = factory
        self._sessions: dict[str, list[ReadSession]] = {}
        self._cursors: dict[tuple[object, ...], list[tuple[str, str, ReadSession]]] = {}
        self._pending: set[asyncio.Task[dict[str, JsonValue]]] = set()
        self._failures: list[BaseException] = []
        self._reservations: dict[ReadSession, int] = {}
        self._lock = asyncio.Lock()
        self._closed = False
        self.foreground_selected = False
        self._close_failure: BaseException | None = None

    def successor(self) -> ReadSessions:
        if not self._closed:
            raise SessionProtocolError(
                "an active foreground lifetime cannot be replaced"
            )
        if self._close_failure is not None:
            raise self._close_failure
        return ReadSessions(self.root)

    @property
    def closed(self) -> bool:
        return self._closed

    def observe(
        self, work: Coroutine[object, object, dict[str, JsonValue]]
    ) -> asyncio.Task[dict[str, JsonValue]]:
        """Cancellation detaches the observer, not native work or source protections."""
        if self._closed:
            if isinstance(work, Coroutine):
                work.close()
            raise SessionProtocolError("foreground native read endpoint is closed")
        task = asyncio.create_task(work)
        self._pending.add(task)

        def completed(done: asyncio.Task[dict[str, JsonValue]]) -> None:
            self._pending.discard(done)
            if not done.cancelled() and (error := done.exception()) is not None:
                self._failures.append(error)

        task.add_done_callback(completed)
        return task

    async def run(
        self,
        binding: Binding,
        arguments: dict[str, JsonValue],
        *,
        deadline_monotonic_ns: int | None = None,
    ) -> dict[str, JsonValue]:
        if binding.read_eligibility != "exact_context":
            raise SessionProtocolError(
                "operation declaration does not permit retained native reads"
            )
        try:
            raw = operation_document(binding.operation, arguments)
        except (TypeError, ValueError):
            return {
                "outcome": "malformed_invocation",
                "code": "pse.native.non_json_input",
            }
        if len(raw) > MAXIMUM_REQUEST_BYTES:
            return {
                "outcome": "malformed_invocation",
                "code": "pse.native.request_bound",
            }
        task = self.observe(self._run(binding, arguments, deadline_monotonic_ns))
        return await asyncio.shield(task)

    async def _run(
        self,
        binding: Binding,
        arguments: dict[str, JsonValue],
        deadline_monotonic_ns: int | None,
    ) -> dict[str, JsonValue]:
        identity, basis = (
            context_identity(binding, arguments),
            page_basis(binding, arguments),
        )
        session: ReadSession | None = None
        async with self._lock:
            if self._closed:
                return {
                    "outcome": "transport_failure",
                    "code": "pse.native.endpoint_closed_before_admission",
                    "native_receiver": "not_entered",
                }
            for continuation in binding.continuations:
                token = _at(arguments, tuple(continuation.input))
                if token is not None:
                    known = self._cursors.get(
                        _continuation_key(
                            binding.operation,
                            tuple(continuation.input),
                            continuation.kind,
                            token,
                            identity,
                            basis,
                        )
                    )
                    if known:
                        if len(known) != 1:
                            return {
                                "outcome": "stale_interface",
                                "code": "pse.native.ambiguous_session_cursor",
                            }
                        original_context, original_basis, issued = known[0]
                        if issued.retired:
                            return {
                                "outcome": "stale_interface",
                                "code": "pse.native.retired_session_cursor",
                            }
                        if (original_context, original_basis) != (identity, basis):
                            return {
                                "outcome": "stale_interface",
                                "code": "pse.native.changed_session_cursor_basis",
                            }
                        if session is not None and session is not issued:
                            return {
                                "outcome": "malformed_invocation",
                                "code": "pse.native.mixed_session_cursors",
                            }
                        session = issued
            peers = self._sessions.setdefault(identity, [])
            if session is None:
                session = next(
                    (peer for peer in peers if not peer.busy and not peer.retired), None
                )
            if session is None:
                session = self._factory(self.root, identity)
                peers.append(session)
            self._reservations[session] = self._reservations.get(session, 0) + 1
            session.busy = True
        try:
            # Previously retired workers settle before a replacement for that context
            # enters a receiver. Independent live peers need not wait for one another.
            await settle(
                *(
                    peer.close()
                    for peer in peers
                    if peer.retired and peer is not session
                )
            )
            response = await session.invoke(binding, arguments, deadline_monotonic_ns)
            async with self._lock:
                for continuation in binding.continuations:
                    token = _at(response.result, tuple(continuation.output))
                    if token is not None:
                        key = _continuation_key(
                            binding.operation,
                            tuple(continuation.input),
                            continuation.kind,
                            token,
                            identity,
                            basis,
                        )
                        owners = self._cursors.setdefault(key, [])
                        association = (identity, basis, session)
                        if association not in owners:
                            owners.append(association)
            if response.retired:
                await session.close()
        except ExtensionSelectionError as error:
            session.retired = True
            await session.close()
            return error.detail()
        except ReadDeadlineExpiredError:
            return {
                "outcome": "deadline_expired",
                "code": "pse.native.read_deadline_expired",
                "native_receiver": "not_entered",
            }
        except (OSError, EOFError, asyncio.IncompleteReadError, SessionProtocolError):
            session.retired = True
            try:
                await session.close()
            except BaseException as error:
                self._failures.append(error)
            return {
                "outcome": "transport_failure",
                "code": "pse.native.read_session_refused",
                "message": "The original read session could not return its correlated result; no native settlement or automatic replay is inferred.",
            }
        else:
            return response.result
        finally:
            async with self._lock:
                self._reservations[session] -= 1
                session.busy = self._reservations[session] > 0

    async def close(self) -> None:
        if self._closed:
            if self._close_failure is not None:
                raise self._close_failure
            return
        self._closed = True
        # Snapshot after refusing new requests, then settle all work and all resources
        # even when individual owners fail. Observation cancellation is repropagated
        # only after these independent obligations have all reached their own outcome.
        pending = tuple(self._pending)
        failures: list[BaseException] = []
        cancellation: asyncio.CancelledError | None = None
        try:

            async def finish(task: asyncio.Task[dict[str, JsonValue]]) -> None:
                await asyncio.shield(task)

            await settle(*(finish(task) for task in pending))
        except asyncio.CancelledError as error:
            cancellation = error
            if error.__cause__ is not None:
                failures.append(error.__cause__)
        except BaseException as error:
            failures.append(error)
        try:
            await settle(
                *(
                    session.close()
                    for peers in self._sessions.values()
                    for session in peers
                )
            )
        except asyncio.CancelledError as error:
            cancellation = error
            if error.__cause__ is not None:
                failures.append(error.__cause__)
        except BaseException as error:
            failures.append(error)
        # Completed task failures were retrieved even if their observers detached.
        failures.extend(error for error in self._failures if error not in failures)
        failure = (
            BaseExceptionGroup(
                "Foreground native read lifetime settlement failed", failures
            )
            if failures
            else None
        )
        self._close_failure = failure
        if cancellation is not None:
            raise cancellation from failure
        if failure is not None:
            raise failure
