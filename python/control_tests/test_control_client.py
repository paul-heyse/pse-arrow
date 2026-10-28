# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-only control imports and actual bounded HTTP lifetime policies."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

import anyio
import httpcore2
import httpx2
import msgspec
import pytest
from httpcore2._async.connection_pool import AsyncPoolRequest, PoolByteStream
from httpx2._transports.default import AsyncResponseStream

import pse.control as control
from pse.contracts.documents import (
    ControlBrowseRequest,
    ControlCancelRequest,
    ControlReconcileRequest,
    StudyWorkbenchFilter,
)
from pse.control import (
    ControlClient,
    ControlError,
    ControlRefusedError,
    ControlUnresolvedError,
)

if TYPE_CHECKING:
    from collections.abc import AsyncIterator, Awaitable, Callable


@pytest.mark.unit
def test_control_import_never_loads_scientific_extension() -> None:
    root = Path(__file__).resolve().parents[2]
    code = """
import importlib.abc
import sys
class RefuseScience(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname in {'pse._native', 'pse._build', 'pyarrow'}:
            raise AssertionError('unrelated scientific load: ' + fullname)
sys.meta_path.insert(0, RefuseScience())
import pse
from pse.control import ControlClient
from pse.contracts.documents import ControlCancelRequest
assert 'pse._native' not in sys.modules
assert 'pse._build' not in sys.modules
assert 'pyarrow' not in sys.modules
request = ControlCancelRequest(
    document_version=1, operation_id='original', issuing_store='issuer', study='study'
)
assert request.study == 'study'
try:
    pse.Runtime
except AssertionError:
    pass
else:
    raise AssertionError('scientific access did not consume its native gate')
"""
    subprocess.run([sys.executable, "-c", code], cwd=root, check=True)


def _client() -> ControlClient:

    return ControlClient(
        "http://127.0.0.1:8000",
        "pse",
        "canonical",
        "record-token",
        issuing_store="issuer",
        timeout=0.1,
        maximum_request_bytes=4096,
        maximum_response_bytes=4096,
    )


def _transport(
    monkeypatch: pytest.MonkeyPatch,
    handler: Callable[[httpx2.Request], Awaitable[httpx2.Response]],
) -> None:
    constructor = httpx2.AsyncClient

    def create(
        *, timeout: float, trust_env: bool, follow_redirects: bool
    ) -> httpx2.AsyncClient:
        assert trust_env is False
        assert follow_redirects is False
        return constructor(
            transport=httpx2.MockTransport(handler),
            timeout=timeout,
            trust_env=trust_env,
            follow_redirects=follow_redirects,
        )

    monkeypatch.setattr(httpx2, "AsyncClient", create)


@pytest.mark.unit
def test_control_browse_exact_integer_conversion(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    async def handler(request: httpx2.Request) -> httpx2.Response:
        assert request.url.path.endswith("/browse")
        assert request.headers["accept-encoding"] == "identity"
        body = msgspec.json.decode(request.content, type=dict[str, msgspec.Raw])
        assert msgspec.json.decode(body["expires_at"], type=int) > 0
        row = {
            "study": "study",
            "run": "run",
            "problem": "problem",
            "source": "source",
            "physical_revision": None,
            "physical_identity": None,
            "context": None,
            "point_count": "18446744073709551615",
            "generation": "9007199254740993",
            "active": True,
            "cancelled": False,
            "terminal": False,
            "result_attempt": None,
            "outcome": None,
        }
        return httpx2.Response(
            200,
            stream=httpx2.ByteStream(
                msgspec.json.encode(
                    {"document_version": 1, "headers": [row], "next": None}
                )
            ),
        )

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            page = await client.study_workbench_page(
                ControlBrowseRequest(
                    document_version=1,
                    filter=StudyWorkbenchFilter(),
                    maximum=1,
                    cursor=None,
                )
            )
            assert page.headers[0].generation == 9007199254740993
            assert page.headers[0].point_count == (1 << 64) - 1

    anyio.run(run)


@pytest.mark.unit
@pytest.mark.parametrize("mode", ["oversize", "compression", "trickle", "malformed"])
def test_control_uncertain_cancel_retains_identity_and_closes(
    monkeypatch: pytest.MonkeyPatch, mode: str
) -> None:

    closed: list[bool] = []

    class Body(httpx2.AsyncByteStream):
        async def __aiter__(self) -> AsyncIterator[bytes]:
            if mode == "oversize":
                yield b"x" * 4097
            elif mode == "trickle":
                while True:
                    await anyio.sleep(0.03)
                    yield b" "
            else:
                yield b"broken"

        async def aclose(self) -> None:
            closed.append(True)

    async def handler(_request: httpx2.Request) -> httpx2.Response:
        headers = {"Content-Encoding": "gzip"} if mode == "compression" else {}
        return httpx2.Response(200, headers=headers, stream=Body())

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            with pytest.raises(ControlUnresolvedError) as outcome:
                await client.cancel_study(
                    ControlCancelRequest(
                        document_version=1,
                        issuing_store="issuer",
                        operation_id="original-command",
                        study="study",
                    )
                )
            assert outcome.value.operation_id == "original-command"
        assert closed

    anyio.run(run)


@pytest.mark.unit
def test_control_reconciliation_absence_issues_no_cancel(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    calls: list[str] = []

    async def handler(request: httpx2.Request) -> httpx2.Response:
        calls.append(request.url.path)
        return httpx2.Response(
            200, stream=httpx2.ByteStream(b'{"document_version":1,"receipt":null}')
        )

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            result = await client.reconcile(
                ControlReconcileRequest(
                    document_version=1, issuing_store="issuer", operation_id="original"
                )
            )
            assert result.receipt is None
        assert len(calls) == 1
        assert calls[0].endswith("/reconcile")

    anyio.run(run)


@pytest.mark.unit
def test_control_original_issuer_refuses_before_dispatch(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    async def handler(_request: httpx2.Request) -> httpx2.Response:
        message = "Foreign issuing store reached dispatch"
        raise AssertionError(message)

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            with pytest.raises(ControlError, match="issuing store"):
                await client.cancel_study(
                    ControlCancelRequest(
                        document_version=1,
                        issuing_store="restored-issuer",
                        operation_id="original",
                        study="study",
                    )
                )

    anyio.run(run)


@pytest.mark.unit
def test_control_external_cancellation_closes_and_preserves_identity(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    closed: list[bool] = []
    errors: list[BaseException] = []

    class Body(httpx2.AsyncByteStream):
        async def __aiter__(self) -> AsyncIterator[bytes]:
            yield b" "
            await anyio.sleep_forever()

        async def aclose(self) -> None:
            closed.append(True)

    async def handler(_request: httpx2.Request) -> httpx2.Response:
        return httpx2.Response(200, stream=Body())

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            with anyio.move_on_after(0.02) as stop:
                try:
                    await client.cancel_study(
                        ControlCancelRequest(
                            document_version=1,
                            issuing_store="issuer",
                            operation_id="retained",
                            study="study",
                        )
                    )
                except anyio.get_cancelled_exc_class() as error:
                    errors.append(error)
                    raise
            assert stop.cancelled_caught
        assert closed
        assert "retained" in str(getattr(errors[0], "__notes__", []))

    anyio.run(run)


@pytest.mark.unit
def test_control_close_waits_for_actual_response_cleanup(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    closed: list[bool] = []

    async def run() -> None:
        entered, release, close_done = anyio.Event(), anyio.Event(), anyio.Event()

        class Body(httpx2.AsyncByteStream):
            async def __aiter__(self) -> AsyncIterator[bytes]:
                entered.set()
                await release.wait()
                yield b'{"document_version":1,"receipt":null}'

            async def aclose(self) -> None:
                closed.append(True)

        async def handler(_request: httpx2.Request) -> httpx2.Response:
            return httpx2.Response(200, stream=Body())

        _transport(monkeypatch, handler)
        client = await _client().__aenter__()
        request = ControlReconcileRequest(
            document_version=1, issuing_store="issuer", operation_id="original"
        )

        async def request_task() -> None:
            await client.reconcile(request)

        async def close_task() -> None:
            await client.close()
            assert closed
            close_done.set()

        async with anyio.create_task_group() as group:
            group.start_soon(request_task)
            await entered.wait()
            group.start_soon(close_task)
            await anyio.sleep(0)
            assert not close_done.is_set()
            with pytest.raises(ControlError, match="admission"):
                await client.reconcile(request)
            release.set()
        assert close_done.is_set()

    anyio.run(run)


@pytest.mark.unit
def test_control_signin_uses_bounded_owned_transport(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    async def handler(request: httpx2.Request) -> httpx2.Response:
        if request.url.path == "/signin":
            assert "authorization" not in request.headers
            body = msgspec.json.decode(request.content, type=dict[str, str])
            assert body == {
                "NS": "pse",
                "DB": "canonical",
                "AC": "pse_control",
                "actor": "reader",
                "password": "private-password",
            }
            return httpx2.Response(
                200, stream=httpx2.ByteStream(b'{"token":"new-record-token"}')
            )
        assert request.headers["authorization"] == "Bearer new-record-token"
        return httpx2.Response(
            200, stream=httpx2.ByteStream(b'{"document_version":1,"receipt":null}')
        )

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            await client.sign_in("reader", "private-password")
            await client.reconcile(
                ControlReconcileRequest(
                    document_version=1, issuing_store="issuer", operation_id="original"
                )
            )

    anyio.run(run)


@pytest.mark.unit
@pytest.mark.parametrize("foreign", [False, True])
def test_control_explicit_identity_refusal_is_bound_to_original_command(
    monkeypatch: pytest.MonkeyPatch, foreign: bool
) -> None:

    async def handler(_request: httpx2.Request) -> httpx2.Response:
        body = msgspec.json.encode(
            {
                "document_version": 1,
                "issuing_store": "foreign" if foreign else "issuer",
                "operation_id": "original",
                "reason": "control_command_identity_reused",
            }
        )
        return httpx2.Response(409, stream=httpx2.ByteStream(body))

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            expected = ControlUnresolvedError if foreign else ControlRefusedError
            with pytest.raises(expected):
                await client.cancel_study(
                    ControlCancelRequest(
                        document_version=1,
                        issuing_store="issuer",
                        operation_id="original",
                        study="changed",
                    )
                )

    anyio.run(run)


@pytest.mark.unit
@pytest.mark.parametrize(
    "target", ["browse", "cancel", "reconcile", "nested_receipt", "refusal"]
)
@pytest.mark.parametrize("version", [None, 2, True])
def test_control_requires_explicit_generated_version_headers(
    monkeypatch: pytest.MonkeyPatch, target: str, version: int | bool | None
) -> None:

    receipt = {
        "document_version": 1,
        "issuing_store": "issuer",
        "operation_id": "original",
        "study": "study",
        "status": "accepted",
        "already_concluded": False,
        "cancelled": True,
        "generation": "1",
        "reason": None,
    }
    if target == "browse":
        document = {"document_version": 1, "headers": [], "next": None}
    elif target in {"reconcile", "nested_receipt"}:
        document = {"document_version": 1, "receipt": receipt}
    elif target == "refusal":
        document = {
            "document_version": 1,
            "issuing_store": "issuer",
            "operation_id": "original",
            "reason": "control_command_identity_reused",
        }
    else:
        document = receipt
    changed = receipt if target == "nested_receipt" else document
    if version is None:
        changed.pop("document_version")
    else:
        changed["document_version"] = version

    async def handler(_request: httpx2.Request) -> httpx2.Response:
        return httpx2.Response(
            409 if target == "refusal" else 200,
            stream=httpx2.ByteStream(msgspec.json.encode(document)),
        )

    _transport(monkeypatch, handler)

    async def run() -> None:
        async with _client() as client:
            expected = (
                ControlUnresolvedError
                if target in {"cancel", "refusal"}
                else ControlError
            )

            async def invoke() -> None:
                if target == "browse":
                    await client.study_workbench_page(
                        ControlBrowseRequest(filter=StudyWorkbenchFilter(), maximum=1)
                    )
                elif target in {"reconcile", "nested_receipt"}:
                    await client.reconcile(
                        ControlReconcileRequest(
                            issuing_store="issuer", operation_id="original"
                        )
                    )
                else:
                    await client.cancel_study(
                        ControlCancelRequest(
                            issuing_store="issuer",
                            operation_id="original",
                            study="study",
                        )
                    )

            with pytest.raises(expected):
                await invoke()

    anyio.run(run)


@pytest.mark.unit
def test_control_close_waiter_retains_its_failed_round_during_retry(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    async def run() -> None:
        first_entered, release_first = anyio.Event(), anyio.Event()
        waiter_joined, release_waiter = anyio.Event(), anyio.Event()
        retry_entered, release_retry = anyio.Event(), anyio.Event()
        original_round = control._CloseRound
        rounds = 0

        class DelayedCompletion:
            def __init__(self) -> None:
                self.event = anyio.Event()

            def is_set(self) -> bool:
                return self.event.is_set()

            def set(self) -> None:
                self.event.set()

            async def wait(self) -> None:
                waiter_joined.set()
                await self.event.wait()
                await release_waiter.wait()

        class Round(original_round):
            def __init__(self) -> None:
                nonlocal rounds
                super().__init__()
                rounds += 1
                if rounds == 1:
                    self.completed = DelayedCompletion()

        monkeypatch.setattr(control, "_CloseRound", Round)
        client = await _client().__aenter__()
        assert client._http is not None
        actual_close = client._http.aclose
        calls = 0
        failure = RuntimeError("first pool close failed")

        async def pool_close() -> None:
            nonlocal calls
            calls += 1
            if calls == 1:
                first_entered.set()
                await release_first.wait()
                raise failure
            retry_entered.set()
            await release_retry.wait()
            await actual_close()

        monkeypatch.setattr(client._http, "aclose", pool_close)
        outcomes: dict[str, BaseException | None] = {}
        first_done, waiter_done = anyio.Event(), anyio.Event()

        async def close_named(name: str) -> None:
            try:
                await client.close()
            except BaseException as error:
                outcomes[name] = error
            else:
                outcomes[name] = None
            if name == "first":
                first_done.set()
            elif name == "waiter":
                waiter_done.set()

        async with anyio.create_task_group() as group:
            group.start_soon(close_named, "first")
            await first_entered.wait()
            group.start_soon(close_named, "waiter")
            await waiter_joined.wait()
            release_first.set()
            await first_done.wait()
            group.start_soon(close_named, "retry")
            await retry_entered.wait()
            release_waiter.set()
            await waiter_done.wait()
            assert outcomes["waiter"] is outcomes["first"]
            assert outcomes["waiter"] is not None
            assert "retry" not in outcomes
            assert client._http is not None
            release_retry.set()
        assert outcomes["retry"] is None
        assert client._http is None

    anyio.run(run)


@pytest.mark.unit
@pytest.mark.parametrize(
    "mode",
    ["malformed", "external_cancel", "delivered", "delivered_during_error_handler"],
)
def test_control_response_close_failure_preserves_outcome_and_owner(
    monkeypatch: pytest.MonkeyPatch, mode: str
) -> None:

    async def run() -> None:
        cleanup = RuntimeError("response stream close failed")
        close_calls = 0

        class Body(httpx2.AsyncByteStream):
            async def __aiter__(self) -> AsyncIterator[bytes]:
                if mode == "external_cancel":
                    yield b" "
                    await anyio.sleep_forever()
                elif mode == "malformed":
                    yield b"broken"
                else:
                    yield msgspec.json.encode(
                        {
                            "document_version": 1,
                            "issuing_store": "issuer",
                            "operation_id": "retained",
                            "study": "study",
                            "status": "accepted",
                            "already_concluded": False,
                            "cancelled": True,
                            "generation": "1",
                            "reason": None,
                        }
                    )

            async def aclose(self) -> None:
                nonlocal close_calls
                close_calls += 1
                if close_calls == 1:
                    raise cleanup

        async def handler(_request: httpx2.Request) -> httpx2.Response:
            return httpx2.Response(200, stream=Body())

        _transport(monkeypatch, handler)
        client = await _client().__aenter__()
        request = ControlCancelRequest(
            issuing_store="issuer", operation_id="retained", study="study"
        )
        errors: list[BaseException] = []
        if mode == "external_cancel":
            with anyio.move_on_after(0.02) as stop:
                try:
                    await client.cancel_study(request)
                except anyio.get_cancelled_exc_class() as error:
                    errors.append(error)
                    raise
            assert stop.cancelled_caught
            assert errors[0].__cause__ is cleanup
            assert "retained" in str(getattr(errors[0], "__notes__", []))
        else:

            def unrelated_error() -> None:
                message = "unrelated caller error"
                raise ValueError(message)

            async def invoke() -> None:
                if mode == "delivered_during_error_handler":
                    try:
                        unrelated_error()
                    except ValueError:
                        await client.cancel_study(request)
                else:
                    await client.cancel_study(request)

            with pytest.raises(ControlUnresolvedError) as outcome:
                await invoke()
            assert outcome.value.operation_id == "retained"
            errors.append(outcome.value)
            assert outcome.value.__cause__ is not None
        assert errors
        assert client._active == 0
        assert client._http is not None
        assert len(client._pending_responses) == 1
        with pytest.raises(BaseExceptionGroup) as closing:
            await client.close()
        assert closing.value.exceptions == (cleanup,)
        assert close_calls == 1
        assert len(client._pending_responses) == 1
        assert client._http is not None

    anyio.run(run)


@pytest.mark.unit
def test_control_failed_pool_close_cannot_be_settled_by_library_noop(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    async def handler(_request: httpx2.Request) -> httpx2.Response:
        message = "No request is needed"
        raise AssertionError(message)

    async def run() -> None:
        _transport(monkeypatch, handler)
        client = await _client().__aenter__()
        assert client._http is not None
        # Exercise HTTPX's real aclose state transition and a failing transport.
        calls = 0
        failure = RuntimeError("pool transport close failed")

        async def fail_close() -> None:
            nonlocal calls
            calls += 1
            raise failure

        monkeypatch.setattr(httpx2.MockTransport, "aclose", lambda _self: fail_close())
        with pytest.raises(BaseExceptionGroup) as first:
            await client.close()
        assert first.value.exceptions == (failure,)
        assert client._http is not None
        assert client._http.is_closed
        with pytest.raises(BaseExceptionGroup) as retry:
            await client.close()
        assert retry.value.exceptions == (failure,)
        assert calls == 1
        assert client._http is not None

    anyio.run(run)


@pytest.mark.unit
def test_control_actual_provider_body_close_noop_keeps_unsettled_owner(
    monkeypatch: pytest.MonkeyPatch,
) -> None:

    async def run() -> None:
        calls = 0
        failure = RuntimeError("provider body close failed")

        class Body:
            async def __aiter__(self) -> AsyncIterator[bytes]:
                yield b"broken"

            async def aclose(self) -> None:
                nonlocal calls
                calls += 1
                raise failure

        pool = httpcore2.AsyncConnectionPool()
        pool_request = AsyncPoolRequest(
            httpcore2.Request("GET", "http://example.invalid")
        )
        pool._requests.append(pool_request)
        provider = PoolByteStream(Body(), pool_request, pool)

        async def handler(_request: httpx2.Request) -> httpx2.Response:
            return httpx2.Response(200, stream=AsyncResponseStream(provider))

        _transport(monkeypatch, handler)
        client = await _client().__aenter__()
        with pytest.raises(ControlUnresolvedError) as outcome:
            await client.cancel_study(
                ControlCancelRequest(
                    issuing_store="issuer", operation_id="retained", study="study"
                )
            )
        assert outcome.value.operation_id == "retained"
        assert calls == 1
        assert len(client._pending_responses) == 1
        response = next(iter(client._pending_responses))
        # This is the installed default-wrapper chain, including BoundAsyncStream
        # supplied by AsyncClient.send. Its retry returns while the pool request
        # remains retained and the underlying close has not succeeded.
        await response.stream.aclose()
        assert calls == 1
        assert pool._requests == [pool_request]
        for _ in range(2):
            with pytest.raises(BaseExceptionGroup) as closing:
                await client.close()
            assert closing.value.exceptions == (failure,)
            assert client._http is not None
            assert len(client._pending_responses) == 1
        assert calls == 1
        assert pool._requests == [pool_request]
        await pool.aclose()

    anyio.run(run)
