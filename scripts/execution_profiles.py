# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Production allocation transport; Rust owns materialization and relational rules."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path
from typing import TYPE_CHECKING, cast

import msgspec

from scripts import host_metadata

if TYPE_CHECKING:
    from collections.abc import Mapping
    from scripts.generated import deployment

ROOT = Path(__file__).resolve().parents[1]
# Ephemeral exact validator receipts, never a second admission or resource ledger.
# Identity changes invalidate the receipt before it can authorize another use.
FileIdentity = tuple[int, int, int, int, int]
_REQUESTS: dict[tuple[FileIdentity, bytes, FileIdentity | None], bytes] = {}
_VALIDATED: set[tuple[FileIdentity, bytes]] = set()


def _identity(path: Path) -> FileIdentity:
    metadata = path.stat()
    return (metadata.st_dev, metadata.st_ino, metadata.st_size, metadata.st_mtime_ns, metadata.st_ctime_ns)


def _encoded(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()



class DeploymentError(ValueError):
    """The bounded product deployment operator refused its original request."""


def operator() -> Path:
    """Prefer this immutable generation's operator before a development artifact."""
    frozen = ROOT / "bin/pse-deployment"
    selected = os.environ.get("PSE_DEPLOYMENT_OPERATOR")
    if frozen.is_file():
        if selected is not None and Path(selected).absolute() != frozen:
            raise DeploymentError("Frozen deployment operator differs from the caller selection")
        return frozen
    return Path(selected) if selected is not None else ROOT / "target/debug/pse-deployment"


def request(value: Mapping[str, object], *, deadline: float | None = None) -> dict[str, object]:
    # Bootstrap records are emitted from the same Rust document family. Import
    # only when a deployment operation is requested; ordinary command placement
    # remains usable while that product artifact is being bootstrapped.
    from scripts.generated import deployment  # noqa: PLC0415 -- generated bootstrap family

    wire = {"document_version": 1, **value}
    payload = _encoded(wire)
    if len(payload) > 1024 * 1024:
        raise DeploymentError("Deployment request exceeds its bounded envelope")
    try:
        msgspec.json.decode(payload, type=deployment.DeploymentRequest)
        binary = operator()
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise DeploymentError("Deployment operator unavailable; build pse-runtime --bin pse-deployment")
        identity = _identity(binary)
        declaration = value.get("declaration")
        declaration_identity = _identity(Path(declaration)) if isinstance(declaration, str) else None
        key = (identity, payload, declaration_identity)
        cached = _REQUESTS.get(key)
        if cached is not None:
            host_metadata.remaining(deadline, 1)
            return cast("dict[str, object]", json.loads(cached))
        with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
            result = subprocess.run([str(binary)], input=payload, stdout=output, stderr=errors,
                                    timeout=host_metadata.remaining(deadline, 10), check=False)
            output.seek(0)
            response_bytes = output.read(1024 * 1024 + 1)
            errors.seek(0)
            error_bytes = errors.read(4096)
        if _identity(binary) != identity:
            raise DeploymentError("Deployment operator changed during validation")
        if declaration_identity is not None and _identity(Path(str(declaration))) != declaration_identity:
            raise DeploymentError("Production declaration changed during materialization")
    except (OSError, subprocess.SubprocessError, msgspec.DecodeError) as error:
        raise DeploymentError(f"Deployment validation unavailable: {type(error).__name__}") from error
    host_metadata.remaining(deadline, 1)
    if result.returncode:
        detail = error_bytes.decode(errors="replace").strip()
        raise DeploymentError(f"Deployment allocation refused before effects: {detail}")
    if len(response_bytes) > 1024 * 1024:
        raise DeploymentError("Deployment response exceeds its bounded envelope")
    try:
        msgspec.json.decode(response_bytes, type=deployment.DeploymentResponse)
        response = json.loads(response_bytes)
        allocation = response["allocation"]
        if not isinstance(allocation, dict):
            raise DeploymentError("Deployment response has no allocation")
        encoded = _encoded(allocation)
        _REQUESTS[key] = encoded
        _VALIDATED.add((identity, encoded))
        return cast("dict[str, object]", allocation)
    except (ValueError, KeyError, TypeError) as error:
        raise DeploymentError("Deployment operator response is malformed") from error


def profile(name: str, *, deadline: float | None = None) -> dict[str, object]:
    return request({"operation": "profile", "declaration": str(ROOT / ".config/agent-capacity.toml"),
                    "profile": name}, deadline=deadline)


def execution(name: str) -> dict[str, int]:
    allocation = profile(name)
    value = allocation.get("execution")
    if not isinstance(value, dict):
        raise DeploymentError("Production profile omitted its execution allocation")
    return cast("dict[str, int]", value)


def materialize(total: int, server: int, workers: int, worker: int,
                execution: object = None, *, deadline: float | None = None) -> dict[str, object]:
    return request({"operation": "materialize", "total_memory_bytes": total,
                    "server_memory_bytes": server, "native_workers": workers,
                    "native_worker_memory_bytes": worker, "execution": execution}, deadline=deadline)


def validate(allocation: Mapping[str, object], *, deadline: float | None = None) -> None:
    host_metadata.remaining(deadline, 1)
    if (_identity(operator()), _encoded(allocation)) not in _VALIDATED:
        request({"operation": "validate", "allocation": dict(allocation)}, deadline=deadline)


def declarations() -> deployment.ProductionProfiles:
    """Decode only the separately declared production section from its Rust schema."""
    from scripts.generated import deployment  # noqa: PLC0415 -- generated bootstrap family
    import tomllib  # noqa: PLC0415 -- source-only declaration decoding

    raw = tomllib.loads((ROOT / ".config/agent-capacity.toml").read_text())["production"]
    try:
        return msgspec.convert(raw, type=deployment.ProductionProfiles, strict=True)
    except msgspec.ValidationError as error:
        raise DeploymentError("Production profile declaration is malformed") from error


def default_allocation(*, deadline: float | None = None) -> dict[str, object]:
    return profile(declarations().default_profile.value, deadline=deadline)


def _projection(config: Mapping[str, object], model: type[msgspec.Struct]) -> dict[str, object]:
    # Rust declares these field names once. Other configuration owners retain
    # their fields without this consumer inventing a whole-config replica.
    return {field.name: config[field.name] for field in msgspec.structs.fields(model) if field.name in config}


def validate_configuration(config: Mapping[str, object], *, deadline: float | None = None) -> None:
    from scripts.generated import deployment  # noqa: PLC0415 -- generated bootstrap family

    try:
        # Preserve the required persisted header; generated construction defaults
        # are appropriate for producers, never a substitute for an observed header.
        if "profile_version" not in config:
            raise DeploymentError("Service configuration has no persisted profile version")
        for model in (deployment.ServiceIdentity, deployment.CanonicalConnectionSettings,
                      deployment.ManagedExecutionPolicy):
            msgspec.convert(_projection(config, model), type=model, strict=True)
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            raise DeploymentError("Configuration has no materialized allocation")
        validate(allocation, deadline=deadline)
    except (msgspec.ValidationError, KeyError) as error:
        raise DeploymentError("Service configuration projection is malformed") from error
