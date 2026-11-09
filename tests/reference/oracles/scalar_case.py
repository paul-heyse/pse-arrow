# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent scalar wire reader for the original primitive physical fixtures."""

from __future__ import annotations

import hashlib
import math
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from google.protobuf import message_factory

from proto_exchange import encode_reference, reference_type

from .ordinary_exchange import (
    _bounded_read,
    _identity,
    _message,
    _read_artifact,
    _validate_physical,
    bits_float,
    float_bits,
)

if TYPE_CHECKING:
    from collections.abc import Iterable, Sequence

    from google.protobuf.message import Message


_ARTIFACTS = (
    "tests/fixtures/packages/minimal_explicit/package.toml",
    "tests/fixtures/packages/physical-primitives/materials/physical.yaml",
    "tests/fixtures/packages/physical-primitives/materials/time.yaml",
    "tests/fixtures/packages/physical-primitives/package.toml",
)


def require(condition: bool, reason: str) -> None:
    """Refuse a violated prerequisite even in an optimized interpreter."""
    if not condition:
        raise ValueError(reason)


def _artifacts(contract: Message) -> None:
    require(
        tuple(item.path for item in contract.artifacts) == _ARTIFACTS,
        "scalar source artifact population differs",
    )
    for item in contract.artifacts:
        content = _read_artifact(item.path, 1 << 20)
        require(
            bytes(item.sha256) == hashlib.sha256(content).digest(),
            "scalar source artifact differs from the selected original",
        )


def _physical(item: Message) -> Message:
    physical = _message(item, "physical")
    _validate_physical(physical, "Scalar", original_unit_symbol="dimensionless")
    return physical


@dataclass(frozen=True)
class ScalarCase:
    """Read one original descriptor world and exact ordered physical population."""

    envelope: type[Message]
    contract: Message

    @classmethod
    def read(cls, descriptors: Path, source: Path) -> ScalarCase:
        """Verify descriptor closure, version, original identities and artifacts."""
        envelope = reference_type(_bounded_read(descriptors))
        pool = envelope.DESCRIPTOR.file.pool
        contract_type = message_factory.GetMessageClass(
            pool.FindMessageTypeByName("pse.reference.v1.ReferenceCaseContract")
        )
        contract = contract_type.FromString(_bounded_read(source))
        require(
            _message(contract, "version").value == 1, "unsupported scalar case version"
        )
        meaning = _message(contract, "meaning")
        for field, size in (
            ("case", 16),
            ("source", 32),
            ("formulation", 32),
            ("physical", 32),
            ("configuration", 32),
        ):
            original = bytes(getattr(meaning, field))
            require(
                len(original) == size and original != bytes(size),
                "scalar case meaning is incomplete",
            )
        _artifacts(contract)
        return cls(envelope, contract)

    def inputs(self, keys: Sequence[str]) -> dict[str, float]:
        """Check the original physical inputs and return exact IEEE values."""
        fixed = self.contract.specification
        require(
            tuple(item.key for item in fixed) == tuple(keys),
            "scalar input population differs",
        )
        for item in fixed:
            _physical(item)
        return {item.key: bits_float(int(item.value)) for item in fixed}

    def outputs(self, keys: Sequence[str]) -> None:
        """Validate exact observable identities and physical meanings before solver work."""
        observables = self.contract.observables
        require(
            tuple(item.key for item in observables) == tuple(keys),
            "scalar output population differs",
        )
        output_ids = [bytes(item.output) for item in observables]
        require(
            len(set(output_ids)) == len(output_ids)
            and all(
                len(identity) == 16 and identity != bytes(16) for identity in output_ids
            ),
            "scalar output identities are invalid or repeated",
        )
        for item in observables:
            _physical(item)

    def publish(self, destination: Path, values: Iterable[float]) -> None:
        """Publish independent outputs under their validated original correspondence."""
        envelope = self.envelope(version=1)
        envelope.meaning.CopyFrom(_message(self.contract, "meaning"))
        observables = self.contract.observables
        envelope.required.extend(item.output for item in observables)
        for coordinate, value in zip(observables, values, strict=True):
            require(
                math.isfinite(value), "scalar reference returned nonfinite evidence"
            )
            physical = _message(coordinate, "physical")
            envelope.outcome.completed.outputs.add(
                id=coordinate.output,
                quantity=_identity(_message(physical, "quantity")),
                unit=_identity(_message(physical, "representation_unit")),
                value=float_bits(value),
            )
        encoded = encode_reference(envelope)
        _artifacts(self.contract)
        with destination.open("xb") as stream:
            stream.write(encoded)

    def publish_result(self, destination: Path, result: Message, limit: int) -> None:
        """Publish a specialized result in this same original descriptor world."""
        require(
            result.DESCRIPTOR.file.pool is self.envelope.DESCRIPTOR.file.pool,
            "specialized result belongs to another descriptor world",
        )
        require(
            result.ByteSize() <= limit, "specialized result exceeds its byte allowance"
        )
        encoded = result.SerializeToString(deterministic=True)
        _artifacts(self.contract)
        with destination.open("xb") as stream:
            stream.write(encoded)
