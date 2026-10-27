# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed foreground native read session; EOF closes its actual retained owners."""

from __future__ import annotations

import hashlib
import os
import sys
import tempfile
from pathlib import Path

from scripts import python_extension
from scripts.capabilities import execution_basis, native_contracts, native_worker
from scripts.capabilities.native_extension import (
    ExtensionSelectionError,
    extension_currency,
)
from scripts.capabilities.native_read_context import (
    NativeReadContext,
    ReadContextChangedError,
)
from scripts.capabilities.native_read_image import (
    LoadedReadImage,
    ReadImageChangedError,
)
from scripts.capabilities.native_read_protocol import (
    MAXIMUM_ADMISSION_BYTES,
    MAXIMUM_REQUEST_BYTES,
    MAXIMUM_REQUEST_FRAME_BYTES,
    MAXIMUM_RESPONSE_FRAME_BYTES,
    SESSION_ENVIRONMENT,
    ReadDeadlineExpiredError,
    ReadRequest,
    ReadResponse,
    SessionAdmission,
    SessionProtocolError,
    decode,
    read,
    write,
)
from scripts.capabilities.native_read_sessions import context_identity
from scripts.capabilities.workers import WORKERS


class ReadCurrencyError(ReadImageChangedError):
    """Keep the producing owner's redacted refusal rather than inventing a cause."""

    def __init__(self, current: python_extension.ExtensionCurrency) -> None:
        super().__init__("native read artifact is not currently admitted")
        self.current = current


def _startup() -> SessionAdmission:
    supplied = os.environ.get(SESSION_ENVIRONMENT)
    if supplied is None:
        raise SessionProtocolError(
            "native read worker requires explicit foreground session admission"
        )
    source = Path(supplied)
    if (
        source.is_symlink()
        or not source.is_file()
        or source.stat().st_size > MAXIMUM_ADMISSION_BYTES
    ):
        raise SessionProtocolError("native read session admission file refused")
    return decode(source.read_bytes(), SessionAdmission, MAXIMUM_ADMISSION_BYTES)


def _verify(
    startup: SessionAdmission,
    request: ReadRequest,
    root: Path,
    *,
    entering: bool = True,
) -> native_contracts.Binding:
    if (
        startup.incarnation != request.session
        or Path(startup.checkout) != root
        or startup.route_revision != execution_basis.route_revision(startup.worker)
    ):
        raise SessionProtocolError(
            "native read session incarnation or fixed route changed"
        )
    if request.operation not in WORKERS[startup.worker].operations:
        raise SessionProtocolError(
            "operation is outside the declared retained read route"
        )
    binding = native_contracts.binding_for(request.operation)
    if binding.read_eligibility != "exact_context":
        raise SessionProtocolError(
            "operation does not declare exact-context read eligibility"
        )
    raw = request.original_document()
    if len(raw) > MAXIMUM_REQUEST_BYTES or request.context != context_identity(
        binding, request.arguments
    ):
        raise SessionProtocolError(
            "native read request extent or immutable context changed"
        )
    execution_basis.verify_handoff(
        root, startup.worker, request.operation, raw, request.admission
    )
    if entering:
        request.require_live_deadline()
    return binding


def _currency(binding: native_contracts.Binding) -> python_extension.ExtensionCurrency:
    requirement = binding.extension_requirement
    if requirement is None:
        raise ReadImageChangedError(
            "retained native read lacks its declared extension prerequisite"
        )
    current = extension_currency(
        python_extension.bind_interpreter(os.environ),
        requirement.mode,
        validated=requirement.validated,
    )
    if current.state != "current":
        raise ReadCurrencyError(current)
    return current


def _admit_image(current: python_extension.ExtensionCurrency) -> LoadedReadImage:
    image = LoadedReadImage.admit(current)
    image.attach()
    return image


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    startup = _startup()
    if Path(startup.checkout) != root:
        raise SessionProtocolError("native read session belongs to another checkout")
    context: NativeReadContext | None = None
    image: LoadedReadImage | None = None
    with tempfile.TemporaryDirectory(prefix="pse-native-read-context-") as temporary:
        scratch = Path(temporary)
        try:
            while (
                request := read(
                    sys.stdin.buffer, ReadRequest, MAXIMUM_REQUEST_FRAME_BYTES
                )
            ) is not None:
                retired = False
                try:
                    binding = _verify(startup, request, root)
                    if image is None:
                        image = _admit_image(_currency(binding))
                    requirement = binding.extension_requirement
                    if requirement is None:
                        raise ReadImageChangedError("native read lacks its owned prerequisite")
                    image.require(
                        native=requirement.mode == "native_correctness",
                        validated=requirement.validated,
                    )
                    bound_context = image.context_identity(request.context)
                    if context is None:
                        context = NativeReadContext(bound_context)
                    context.require(bound_context)
                    # Native request/header decoding still runs on every request. Only
                    # successfully admitted immutable receiver objects are retained.
                    value = native_worker.execute(
                        request.operation,
                        request.arguments,
                        scratch=scratch,
                        context=context,
                    )
                    _verify(startup, request, root, entering=False)
                    if value.get("outcome") in {
                        "stale_interface",
                        "prerequisite_unavailable",
                    }:
                        retired = True
                except ExtensionSelectionError as error:
                    retired = True
                    value = error.detail()
                except ReadCurrencyError as error:
                    retired = True
                    value = {
                        "outcome": "prerequisite_unavailable",
                        "code": "pse.native.artifact_not_current",
                        "currency": error.current.model_dump(mode="json"),
                    }
                except (ImportError, ModuleNotFoundError) as error:
                    retired = True
                    value = {
                        "outcome": "prerequisite_unavailable",
                        "code": "pse.native.extension_unavailable",
                        "cause": type(error).__name__,
                    }
                except ReadDeadlineExpiredError:
                    value = {
                        "outcome": "deadline_expired",
                        "code": "pse.native.read_deadline_expired",
                        "native_receiver": "not_entered",
                    }
                except (
                    ReadImageChangedError,
                    ReadContextChangedError,
                    execution_basis.AdmissionRefusedError,
                    SessionProtocolError,
                ):
                    retired = True
                    value = {
                        "outcome": "stale_interface",
                        "code": "pse.native.retained_read_context_changed",
                        "remedy": "The original session is retired; reselect the current context explicitly. No receiver replay or native drain is inferred.",
                    }
                except Exception as error:
                    value = {
                        "outcome": "native_refusal"
                        if native_worker.native_report(error) is not None
                        else "unexpected_error",
                        "error": native_worker.scientific_error(
                            error, operation=request.operation
                        ),
                    }
                projected = native_worker.project_result(request.operation, value)
                response = ReadResponse(
                    session=startup.incarnation,
                    request=request.request,
                    context=request.context,
                    request_sha256=hashlib.sha256(
                        request.original_document()
                    ).hexdigest(),
                    deadline_monotonic_ns=request.deadline_monotonic_ns,
                    result=projected,
                    retired=retired,
                )
                write(sys.stdout.buffer, response, MAXIMUM_RESPONSE_FRAME_BYTES)
                if retired:
                    break
        finally:
            if context is not None:
                context.close()
    return 0


if __name__ == "__main__":
    from scripts.telemetry_process import run_helper

    raise SystemExit(run_helper("pse-native-read-worker", main))
