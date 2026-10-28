# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded async durable control without scientific runtime construction."""

from __future__ import annotations

import math
import time
from typing import TYPE_CHECKING, Self, TypeVar
from urllib.parse import quote, urlsplit

import anyio
import httpx2
import msgspec
import msgspec.inspect

from pse.codec import _check_version_headers
from pse.contracts.documents import (
    ControlBrowseRequest,
    ControlBrowseResponse,
    ControlCancelRequest,
    ControlCommandReceipt,
    ControlCommandRefusal,
    ControlReconcileRequest,
    ControlReconcileResponse,
    StudyWorkbenchPageOutput,
)

if TYPE_CHECKING:
    from collections.abc import Callable

_T = TypeVar("_T")
_Wire = TypeVar("_Wire")


class ControlError(Exception):
    """A local contract or delivered control-protocol failure."""


class ControlRefusedError(ControlError):
    """The endpoint definitively refused access before a command effect."""


class ControlUnresolvedError(ControlError):
    """A cancellation may have committed; reconcile this original identity."""

    def __init__(self, operation_id: str) -> None:
        """Retain the immutable command identity for reconciliation."""
        self.operation_id = operation_id
        super().__init__(f"Cancellation outcome unresolved; reconcile {operation_id!r}")


def _extent(value: object, remaining: int) -> int:
    """Conservatively admit declared request JSON before allocating its encoding."""
    if remaining < 0:
        message = "Control request exceeds its byte allowance"
        raise ControlError(message)
    if value is None or isinstance(value, bool):
        size = 5
    elif isinstance(value, str):
        # Six bytes per character covers JSON escaping without first encoding it.
        size = 6 * len(value) + 2
    elif isinstance(value, int):
        if not 0 <= value <= (1 << 64) - 1:
            message = "Control integer is outside u64"
            raise ControlError(message)
        size = 20
    elif isinstance(value, dict):
        remaining -= 2
        for name, field in value.items():
            remaining = _extent(name, remaining) - 2
            remaining = _extent(field, remaining)
        return _extent(None, remaining)
    elif isinstance(value, msgspec.Struct):
        remaining -= 2
        for name in value.__struct_fields__:
            remaining = _extent(name, remaining) - 2
            remaining = _extent(getattr(value, name), remaining)
        return _extent(None, remaining)
    else:
        message = "Unsupported control request value"
        raise ControlError(message)
    if size > remaining:
        message = "Control request exceeds its byte allowance"
        raise ControlError(message)
    return remaining - size


def _uint(value: str) -> int:
    if (
        not value
        or not value.isascii()
        or not value.isdecimal()
        or len(value) > 20
        or (len(value) > 1 and value.startswith("0"))
    ):
        message = "Noncanonical control integer encoding"
        raise ControlError(message)
    number = int(value)
    if number > (1 << 64) - 1:
        message = "Control integer is outside u64"
        raise ControlError(message)
    return number


def _decode_response(content: bytearray, result_type: type[_Wire]) -> _Wire:
    # Generated metadata keeps required presence distinct from constructor defaults.
    _check_version_headers(
        msgspec.Raw(content),
        msgspec.inspect.type_info(result_type),
        "$",
        native_header=False,
    )
    return msgspec.json.decode(content, type=result_type)


class _CloseRound:
    def __init__(self) -> None:
        self.completed = anyio.Event()
        self.error: BaseException | None = None


class ControlClient:
    """One owned async HTTP lifetime for a selected local canonical database.

    Limits and timeout are explicit policy. The supplied token is a record actor's
    bearer token, never the deployment's administrative bootstrap credential.
    """

    def __init__(
        self,
        endpoint: str,
        namespace: str,
        database: str,
        token: str | None,
        *,
        issuing_store: str,
        timeout: float,
        maximum_request_bytes: int,
        maximum_response_bytes: int,
    ) -> None:
        """Select the original issuer, bounded transport, and local database."""
        parts = urlsplit(endpoint)
        if (
            parts.scheme != "http"
            or parts.hostname != "127.0.0.1"
            or parts.port is None
            or parts.username is not None
            or parts.password is not None
            or parts.path not in {"", "/"}
            or parts.query
            or parts.fragment
        ):
            message = "Control requires the selected HTTP loopback endpoint"
            raise ValueError(message)
        if (
            isinstance(timeout, bool)
            or not isinstance(timeout, (int, float))
            or not math.isfinite(timeout)
            or timeout <= 0
        ):
            message = "Control timeout must be positive and finite"
            raise ValueError(message)
        for extent in (maximum_request_bytes, maximum_response_bytes):
            if type(extent) is not int or extent <= 0:
                message = "Control byte allowances must be positive integers"
                raise ValueError(message)
        if (
            not namespace
            or not database
            or (token is not None and (not token or any(c.isspace() for c in token)))
        ):
            message = "Control database selection and bearer token are required"
            raise ValueError(message)
        if not issuing_store or len(issuing_store) > 4096:
            message = "Original issuing-store identity is required"
            raise ValueError(message)
        self._issuing_store = issuing_store
        self._endpoint = endpoint.rstrip("/")
        self._namespace = namespace
        self._database = database
        self._base = (
            endpoint.rstrip("/")
            + "/api/"
            + quote(namespace, safe="")
            + "/"
            + quote(database, safe="")
            + "/pse/control/v1/"
        )
        self._token = token
        self._timeout = float(timeout)
        self._request_bytes = maximum_request_bytes
        self._response_bytes = maximum_response_bytes
        self._http: httpx2.AsyncClient | None = None
        self._active = 0
        self._closing = False
        self._close_round: _CloseRound | None = None
        self._idle: anyio.Event | None = None
        self._pending_responses: dict[httpx2.Response, BaseException | None] = {}
        self._pool_close_error: BaseException | None = None

    async def __aenter__(self) -> Self:
        """Open this client owned transport and admission lifetime."""
        if self._http is not None:
            message = "Control client is already open"
            raise ControlError(message)
        self._closing = False
        self._idle = anyio.Event()
        self._idle.set()
        self._close_round = None
        self._http = httpx2.AsyncClient(
            timeout=self._timeout, trust_env=False, follow_redirects=False
        )
        return self

    async def __aexit__(self, *_exception: object) -> None:
        """Seal admission and await owned cleanup."""
        await self.close()

    async def close(self) -> None:
        """Settle this close round, retaining owners after unproved cleanup."""
        with anyio.CancelScope(shield=True):
            previous = self._close_round
            if previous is not None and not previous.completed.is_set():
                await previous.completed.wait()
                if previous.error is not None:
                    raise previous.error
                return
            client = self._http
            if client is None:
                return
            self._closing = True
            current = _CloseRound()
            self._close_round = current
            errors: list[BaseException] = []
            try:
                if self._idle is not None:
                    await self._idle.wait()
                # Both response and provider stream wrappers may mark closed
                # before awaited cleanup. A retry's no-op is no settlement proof.
                # Retain each failed owner and its original failure explicitly.
                errors.extend(
                    error
                    if error is not None
                    else ControlError("Response cleanup remains unsettled")
                    for error in self._pending_responses.values()
                )
                # HTTPX marks CLOSED before awaiting its transport. A later
                # no-op cannot certify settlement after that await failed.
                if self._pool_close_error is not None and client.is_closed:
                    errors.append(self._pool_close_error)
                else:
                    try:
                        await client.aclose()
                    except BaseException as error:
                        self._pool_close_error = error
                        errors.append(error)
                    else:
                        self._pool_close_error = None
                self._require_settled(errors)
                self._http = None
            except BaseException as error:
                current.error = error
                raise
            finally:
                current.completed.set()

    @staticmethod
    def _require_settled(errors: list[BaseException]) -> None:
        if errors:
            message = "Control lifetime cleanup failed"
            raise BaseExceptionGroup(message, errors)

    async def _request(
        self,
        operation: str,
        request: ControlBrowseRequest
        | ControlCancelRequest
        | ControlReconcileRequest
        | dict[str, str],
        result_type: type[_Wire],
        convert: Callable[[_Wire], _T],
        mutation_id: str | None = None,
    ) -> _T:
        deadline = anyio.current_time() + self._timeout
        expiry = time.time_ns() // 1000 + int(self._timeout * 1_000_000)
        if self._http is None or self._closing:
            message = "Control client is not open for admission"
            raise ControlError(message)
        if (
            isinstance(request, (ControlCancelRequest, ControlReconcileRequest))
            and request.issuing_store != self._issuing_store
        ):
            message = "Command issuing store differs from this client"
            raise ControlError(message)
        if operation != "signin" and self._token is None:
            message = "Control requires a record-actor bearer token"
            raise ControlRefusedError(message)
        if self._active == 0:
            self._idle = anyio.Event()
        self._active += 1
        try:
            return await self._perform(
                operation, request, result_type, convert, deadline, expiry, mutation_id
            )
        finally:
            self._active -= 1
            if self._active == 0 and self._idle is not None:
                self._idle.set()

    async def _perform(
        self,
        operation: str,
        request: ControlBrowseRequest
        | ControlCancelRequest
        | ControlReconcileRequest
        | dict[str, str],
        result_type: type[_Wire],
        convert: Callable[[_Wire], _T],
        deadline: float,
        expiry: int,
        mutation_id: str | None = None,
    ) -> _T:
        client = self._http
        if client is None:
            message = "Control client must be used as an async context manager"
            raise ControlError(message)
        payload = (
            request
            if operation == "signin"
            else {
                "issuing_store": self._issuing_store,
                "expires_at": expiry,
                "request": request,
            }
        )
        _extent(payload, self._request_bytes)
        # Struct constructors do not validate; check fields before dispatch.
        if isinstance(request, msgspec.Struct):
            msgspec.convert(
                msgspec.structs.asdict(request), type=type(request), strict=True
            )
        body = msgspec.json.encode(payload)
        if len(body) > self._request_bytes:
            message = "Control request exceeds its byte allowance"
            raise ControlError(message)
        issued = False
        response: httpx2.Response | None = None
        primary: BaseException | None = None
        try:
            with anyio.fail_after(max(0, deadline - anyio.current_time())):
                await anyio.lowlevel.checkpoint()
                issued = True
                headers = {
                    "Accept-Encoding": "identity",
                    "Content-Type": "application/json",
                }
                if operation != "signin":
                    headers["Authorization"] = "Bearer " + str(self._token)
                outgoing = client.build_request(
                    "POST",
                    self._endpoint + "/signin"
                    if operation == "signin"
                    else self._base + operation,
                    content=body,
                    headers=headers,
                )
                response = await client.send(outgoing, stream=True)
                self._pending_responses[response] = None
                return await self._read_response(
                    response, result_type, convert, deadline, mutation_id
                )
        except anyio.get_cancelled_exc_class() as error:
            primary = error
            if issued and mutation_id is not None:
                error.add_note(
                    f"Cancellation delivery unresolved; reconcile {mutation_id!r}"
                )
            raise
        except ControlRefusedError as error:
            primary = error
            raise
        except (
            httpx2.HTTPError,
            TimeoutError,
            msgspec.DecodeError,
            ControlError,
        ) as error:
            if issued and mutation_id is not None:
                primary = ControlUnresolvedError(mutation_id)
            else:
                primary = ControlError("Control delivery failed")
            raise primary from error
        except BaseException as error:
            primary = error
            raise
        finally:
            if response is not None:
                with anyio.CancelScope(shield=True):
                    try:
                        await response.aclose()
                    except BaseException as cleanup:
                        self._pending_responses[response] = cleanup
                        identity = (
                            f"; original operation {mutation_id!r}"
                            if mutation_id is not None
                            else ""
                        )
                        if primary is not None:
                            primary.add_note("Response cleanup also failed" + identity)
                            if primary.__cause__ is not None:
                                cleanup = BaseExceptionGroup(
                                    "Control delivery and cleanup causes",
                                    [primary.__cause__, cleanup],
                                )
                            raise primary from cleanup
                        if issued and mutation_id is not None:
                            raise ControlUnresolvedError(mutation_id) from cleanup
                        message = "Control response cleanup failed"
                        raise ControlError(message) from cleanup
                    else:
                        del self._pending_responses[response]

    async def _read_response(
        self,
        response: httpx2.Response,
        result_type: type[_Wire],
        convert: Callable[[_Wire], _T],
        deadline: float,
        mutation_id: str | None,
    ) -> _T:
        if not isinstance(response.stream, httpx2.AsyncByteStream):
            message = "Control response is not an async raw stream"
            raise ControlError(message)
        if response.status_code in (401, 403):
            message = "Control actor is not authorized"
            raise ControlRefusedError(message)
        if response.status_code not in (200, 409):
            message = f"Control endpoint returned HTTP {response.status_code}"
            raise ControlError(message)
        if response.headers.get("content-encoding", "identity").lower() != "identity":
            message = "Encoded control responses are not admitted"
            raise ControlError(message)
        length = response.headers.get("content-length")
        if length is not None and (
            not length.isascii()
            or not length.isdecimal()
            or len(length) > 20
            or int(length) > self._response_bytes
        ):
            message = "Control response exceeds its byte allowance"
            raise ControlError(message)
        content = bytearray()
        async for chunk in response.stream:
            if len(chunk) > self._response_bytes - len(content):
                message = "Control response exceeds its byte allowance"
                raise ControlError(message)
            content.extend(chunk)
        if response.status_code == 409:
            refusal = _decode_response(content, ControlCommandRefusal)
            if (
                mutation_id is None
                or refusal.issuing_store != self._issuing_store
                or refusal.operation_id != mutation_id
                or refusal.reason != "control_command_identity_reused"
            ):
                message = "Unrecognized control refusal"
                raise ControlError(message)
            raise ControlRefusedError(refusal.reason)
        value = convert(_decode_response(content, result_type))
        if anyio.current_time() >= deadline:
            message = "Control original deadline elapsed"
            raise TimeoutError(message)
        return value

    async def sign_in(self, actor: str, password: str) -> None:
        """Authenticate a provisioned record actor through the bounded owner."""
        if self._active:
            message = "Authentication cannot replace active sibling credentials"
            raise ControlError(message)
        if not actor or not password:
            message = "Control record-actor credentials are required"
            raise ValueError(message)

        def token(raw: msgspec.Raw) -> str:
            fields = msgspec.json.decode(raw, type=dict[str, msgspec.Raw])
            encoded = fields.get("token")
            if encoded is None:
                message = "Authentication response omitted its bearer token"
                raise ControlError(message)
            value = msgspec.json.decode(encoded, type=str)
            if not value or any(c.isspace() for c in value):
                message = "Authentication returned an invalid bearer token"
                raise ControlError(message)
            return value

        value = await self._request(
            "signin",
            {
                "NS": self._namespace,
                "DB": self._database,
                "AC": "pse_control",
                "actor": actor,
                "password": password,
            },
            msgspec.Raw,
            token,
        )
        self._token = value

    async def study_workbench_page(
        self, request: ControlBrowseRequest
    ) -> StudyWorkbenchPageOutput:
        """Read compact live headers; this acquires no scientific payload reader."""

        def convert(page: ControlBrowseResponse) -> StudyWorkbenchPageOutput:
            if len(page.headers) > request.maximum:
                message = "Control page exceeds its requested row count"
                raise ControlError(message)
            value = msgspec.structs.asdict(page)
            headers = []
            for row in page.headers:
                header = msgspec.structs.asdict(row)
                header["point_count"] = _uint(row.point_count)
                header["generation"] = _uint(row.generation)
                headers.append(header)
            value["headers"] = headers
            if page.next is not None:
                value["next"] = msgspec.to_builtins(page.next)
            return msgspec.convert(value, type=StudyWorkbenchPageOutput, strict=True)

        return await self._request("browse", request, ControlBrowseResponse, convert)

    async def cancel_study(
        self, request: ControlCancelRequest
    ) -> ControlCommandReceipt:
        """Issue one cancellation, retaining its identity if unresolved."""

        def convert(receipt: ControlCommandReceipt) -> ControlCommandReceipt:
            if (
                receipt.issuing_store != request.issuing_store
                or receipt.operation_id != request.operation_id
                or receipt.study != request.study
            ):
                raise ControlUnresolvedError(request.operation_id)
            if receipt.generation is not None:
                _uint(receipt.generation)
            return receipt

        return await self._request(
            "cancel", request, ControlCommandReceipt, convert, request.operation_id
        )

    async def reconcile(
        self, request: ControlReconcileRequest
    ) -> ControlReconcileResponse:
        """Read the original receipt; absence never permits mutation replay."""

        def convert(result: ControlReconcileResponse) -> ControlReconcileResponse:
            if result.receipt is not None:
                if (
                    result.receipt.issuing_store != request.issuing_store
                    or result.receipt.operation_id != request.operation_id
                ):
                    message = "Reconciliation returned another operation"
                    raise ControlError(message)
                if result.receipt.generation is not None:
                    _uint(result.receipt.generation)
            return result

        return await self._request(
            "reconcile", request, ControlReconcileResponse, convert
        )
