# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent ordinary-case producer over the published binary contract.

The constructor receives descriptors and shared inputs only. There is no
candidate argument, candidate reader, copied protobuf schema or JSON protocol.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import stat
import struct
import sys
import tomllib
from collections.abc import Mapping
from fractions import Fraction
from importlib.metadata import version
from pathlib import Path, PurePosixPath
from typing import cast

from google.protobuf import message_factory
from google.protobuf.message import Message
from pyomo.core.expr.numeric_expr import NumericValue
from pyomo.environ import units, value
from pyomo.opt import TerminationCondition

from proto_exchange import MAXIMUM_BYTES, encode_reference, reference_type

from . import ordinary_exchanger, ordinary_recycle
from .ordinary_thermo import (
    PROPERTY_ARTIFACTS,
    ROOT,
    SPECIES,
    SpeciesInput,
    parse_inputs,
)
from .reference_environment import ReferenceEnvironment

CASE_NAMES = (
    "ordinary_delivery.exchanger_design",
    "ordinary_delivery.exchanger_rating",
    "ordinary_delivery.exchanger_arithmetic",
    "ordinary_delivery.ideal_recycle",
    "ordinary_delivery.overheated_recycle_failure",
)
RECYCLE_KEYS = (
    "fresh.flow",
    "vapor.flow",
    "purge.flow",
    "recycle.flow",
    "fresh.energy",
    "heater.duty",
    "vapor.energy",
    "purge.energy",
    "fresh.benzene",
    "vapor.benzene",
    "purge.benzene",
    "fresh.toluene",
    "vapor.toluene",
    "purge.toluene",
)
EXCHANGER_KEYS = (
    *(
        f"exchanger.{side}.{position}.{coordinate}"
        for side in ("hot", "cold")
        for position in ("inlet", "outlet")
        for coordinate in ("flow", "T", "h", "energy_flow")
    ),
    "exchanger.hot.duty",
    "exchanger.cold.duty",
    "exchanger.coefficient",
    "exchanger.area",
    "cost.area_count_ft2",
    "cost.base",
    "cost.material_factor",
    "cost.pressure_factor",
    "cost.capital",
)
_MANIFEST = "packages/reference/conformance.toml"
_SOURCE_LIMIT = 64 << 20
_SOURCE_FILE_LIMIT = 1024


def float_bits(number: float) -> int:
    """Carry original IEEE bits, including signed zero, without decimal exchange."""
    return struct.unpack("<Q", struct.pack("<d", number))[0]


def bits_float(bits: int) -> float:
    """Decode a generated fixed64 scalar before scientific finite-value admission."""
    return struct.unpack("<d", struct.pack("<Q", bits))[0]


def _message(parent: Message, field: str) -> Message:
    if not parent.HasField(field):
        raise ValueError(f"required original physical field absent: {field}")
    return cast("Message", getattr(parent, field))


_IDENTITY_OWNERS = {
    "pse.reference.v1.QuantityType": "pse.reference.v1.QuantityTypeId",
    "pse.reference.v1.PhysicalQuantityKind": "pse.reference.v1.QuantityKindId",
    "pse.reference.v1.PhysicalUnit": "pse.reference.v1.UnitId",
    "pse.reference.v1.PhysicalBasis": "pse.reference.v1.BasisId",
    "pse.reference.v1.PhysicalEntityKind": "pse.reference.v1.EntityKindId",
    "pse.reference.v1.ReferenceState": "pse.reference.v1.ReferenceStateId",
}
_IDENTITY_WRAPPERS = frozenset(_IDENTITY_OWNERS.values())
# Original selected subject owner: physical/models/chemistry.pse, entity kind species.
_SPECIES_KIND_ID = bytes.fromhex("1c998211e5d74955863f377662f56526")


def _identity(owner: Message) -> bytes:
    """Project only the generated physical owners and their declared ID wrappers."""
    name = owner.DESCRIPTOR.full_name
    if name in _IDENTITY_OWNERS:
        identity_owner = _message(owner, "id")
        if identity_owner.DESCRIPTOR.full_name != _IDENTITY_OWNERS[name]:
            raise ValueError("original physical owner has a different identity type")
    elif name in _IDENTITY_WRAPPERS:
        identity_owner = owner
    else:
        raise ValueError(f"unsupported original physical identity owner: {name}")
    if not identity_owner.HasField("id"):
        raise ValueError("original physical identity is absent")
    identity = bytes(identity_owner.id)
    if len(identity) != 16 or identity == bytes(16):
        raise ValueError("original physical identity is malformed or nil")
    return identity


def _optional_identity(parent: Message, field: str) -> bytes | None:
    return _identity(_message(parent, field)) if parent.HasField(field) else None


def _word(parent: Message, field: str) -> str:
    return str(_message(parent, field).value)


def _number(parent: Message, field: str) -> float:
    return bits_float(int(getattr(parent, field)))


def _dimension(owner: Message) -> tuple[Fraction, ...]:
    exponents = _message(owner, "exponents").values
    if len(exponents) != 8:
        raise ValueError("physical dimension requires all eight declared coordinates")
    result: list[Fraction] = []
    for exponent in exponents:
        numerator, denominator = int(exponent.num), int(exponent.den)
        if not -32768 <= numerator <= 32767 or not 1 <= denominator <= 32767:
            raise ValueError("physical rational dimension is outside its exact width")
        reduced = Fraction(numerator, denominator)
        if (reduced.numerator, reduced.denominator) != (numerator, denominator):
            raise ValueError("physical rational dimension is not canonical")
        result.append(reduced)
    return tuple(result)


def _safe_path(relative: str) -> Path:
    path = PurePosixPath(relative)
    if (
        path.is_absolute()
        or not path.parts
        or any(part in {".", ".."} for part in path.parts)
        or str(path) != relative
        or "\\" in relative
    ):
        raise ValueError("artifact path must be an exact checkout-relative path")
    candidate = ROOT
    for part in path.parts:
        candidate = candidate / part
        if candidate.is_symlink():
            raise ValueError("artifact paths cannot traverse symlinks")
    if not candidate.resolve().is_relative_to(ROOT.resolve()):
        raise ValueError("artifact path escaped the checkout")
    return candidate


def artifact_paths(manifest_bytes: bytes | None = None) -> tuple[str, ...]:
    """Select the original fixture closure and scientific inputs independently."""
    if manifest_bytes is None:
        manifest_bytes = _read_artifact(_MANIFEST, _SOURCE_LIMIT)
    manifest = tomllib.loads(manifest_bytes.decode("utf-8"))
    seeds = [run for run in manifest["runs"] if run["name"] == "seed"]
    if len(seeds) != 1:
        raise ValueError("ordinary source closure needs exactly one seed run")
    selected = {
        seeds[0]["package"],
        *seeds[0]["dependencies"],
        "campaign",
        "ordinary-delivery",
        "data/ordinary-bt",
    }
    result = {_MANIFEST, *PROPERTY_ARTIFACTS}

    def visit(directory: Path) -> None:
        for child in directory.iterdir():
            if child.is_symlink():
                raise ValueError("ordinary source closure contains a symlink")
            if child.is_dir():
                visit(child)
            elif child.suffix in {".toml", ".yaml", ".yml", ".pse", ".parquet"}:
                if not child.is_file():
                    raise ValueError("ordinary source artifact is not a regular file")
                result.add(child.relative_to(ROOT).as_posix())
                if len(result) > _SOURCE_FILE_LIMIT:
                    raise ValueError("ordinary source closure exceeds its file limit")

    for package in selected:
        visit(_safe_path(f"packages/reference/{package}"))
    return tuple(sorted(result))


def _read_artifact(relative: str, remaining: int) -> bytes:
    path = _safe_path(relative)
    # Avoid opening a substituted symlink or waiting on a pipe in place of a file.
    with os.fdopen(
        os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), "rb"
    ) as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode):
            raise ValueError("scientific input artifact is not a regular file")
        if before.st_size > remaining:
            raise ValueError("ordinary source closure exceeds its byte limit")
        content = stream.read(before.st_size + 1)
        after = os.fstat(stream.fileno())
        if len(content) != before.st_size or (
            before.st_size,
            before.st_mtime_ns,
            before.st_ctime_ns,
        ) != (after.st_size, after.st_mtime_ns, after.st_ctime_ns):
            raise ValueError(
                f"scientific input artifact changed during read: {relative}"
            )
    return content


def _validate_artifacts(
    contract: Message, *, retain_properties: bool = False
) -> dict[str, bytes]:
    entries = contract.artifacts
    if len(entries) > _SOURCE_FILE_LIMIT:
        raise ValueError("ordinary source closure exceeds its file limit")
    paths = {str(entry.path) for entry in entries}
    manifest = _read_artifact(_MANIFEST, _SOURCE_LIMIT)
    if len(paths) != len(entries) or paths != set(artifact_paths(manifest)):
        raise ValueError(
            "artifact population differs from the complete selected source closure"
        )
    retained = {}
    remaining = _SOURCE_LIMIT - len(manifest)
    for entry in entries:
        expected = bytes(entry.sha256)
        if len(expected) != 32:
            raise ValueError("artifact SHA256 does not contain 32 original bytes")
        path = str(entry.path)
        content = manifest if path == _MANIFEST else _read_artifact(path, remaining)
        if hashlib.sha256(content).digest() != expected:
            raise ValueError(f"scientific input artifact SHA256 differs: {path}")
        if path != _MANIFEST:
            remaining -= len(content)
        if retain_properties and path in PROPERTY_ARTIFACTS:
            retained[path] = content
    return retained


def _type_for_key(key: str) -> str:
    if key.endswith(".delta_pressure"):
        return "DeltaPressure"
    if key.endswith(".pressure"):
        return "Pressure"
    if key.endswith(".T"):
        return "Temperature"
    if ".z[" in key:
        return "MoleFraction"
    if key.endswith(".flow"):
        return "Flow"
    if key.endswith(".h"):
        return "MolarEnthalpy"
    if key.endswith(".duty"):
        return "EnergyTransferRate"
    if key.endswith((".energy", ".energy_flow")):
        return "Power"
    if key == "exchanger.coefficient":
        return "HeatTransferCoefficient"
    if key == "exchanger.area":
        return "Area"
    if key in {"cost.base", "cost.capital"}:
        return "CapitalCost"
    if key.startswith("cost."):
        return "Scalar"
    if key.rsplit(".", 1)[-1] in SPECIES:
        return "ComponentFlow"
    raise ValueError(f"unknown scientific scalar role: {key}")


def _reference_unit(type_name: str) -> NumericValue:
    # An independent scientific unit vocabulary, not another wire/physical DTO.
    return {
        "Flow": units.mol / units.s,
        "ComponentFlow": units.mol / units.s,
        "Temperature": units.K,
        "Pressure": units.Pa,
        "DeltaPressure": units.Pa,
        "MoleFraction": units.dimensionless,
        "MolarEnthalpy": units.J / units.mol,
        "Power": units.W,
        "EnergyTransferRate": units.W,
        "HeatTransferCoefficient": units.W / units.m**2 / units.K,
        "Area": units.m**2,
        "Scalar": units.dimensionless,
        "CapitalCost": units.USD_CE500,
    }[type_name]


def _reference_symbol(type_name: str) -> str:
    return {
        "Flow": "mol/s",
        "ComponentFlow": "mol/s",
        "Temperature": "K",
        "Pressure": "Pa",
        "DeltaPressure": "Pa",
        "MoleFraction": "1",
        "MolarEnthalpy": "J/mol",
        "Power": "W",
        "EnergyTransferRate": "W",
        "HeatTransferCoefficient": "W/(K*m^2)",
        "Area": "m^2",
        "Scalar": "1",
        "CapitalCost": "USD_CE500",
    }[type_name]


def _validate_physical(physical: Message, type_name: str) -> None:
    quantity = _message(physical, "quantity")
    key = _message(quantity, "key")
    kind = _message(physical, "quantity_kind")
    canonical = _message(physical, "canonical_unit")
    representation = _message(physical, "representation_unit")
    if not quantity.HasField("name") or quantity.name != type_name:
        raise ValueError(
            f"original quantity type {quantity.name!r} does not match independent role {type_name!r}"
        )
    _identity(quantity)
    if _identity(_message(key, "kind")) != _identity(kind):
        raise ValueError("quantity kind identity differs from its actual owner")
    if _identity(_message(quantity, "canonical_unit")) != _identity(canonical):
        raise ValueError("quantity canonical unit differs from its actual owner")
    _identity(representation)
    if any(
        unit.symbol != _reference_symbol(type_name) or unit.is_affine
        for unit in (canonical, representation)
    ):
        raise ValueError(
            "declared original unit spelling/affinity differs from the reference"
        )
    dimension = _dimension(_message(kind, "dimension"))
    if any(
        _dimension(_message(unit, "dimension")) != dimension
        for unit in (canonical, representation)
    ):
        raise ValueError("actual unit and quantity dimensions differ")
    # The native owner declares this exact order, including Currency as coordinate8.
    base = (
        units.m,
        units.kg,
        units.s,
        units.K,
        units.mol,
        units.A,
        units.cd,
        units.USD_CE500,
    )
    declared = units.dimensionless
    for unit, exponent in zip(base, dimension, strict=True):
        declared = declared * unit ** float(exponent)
    try:
        factor = units.convert_value(
            1.0, from_units=_reference_unit(type_name), to_units=declared
        )
    except Exception as error:
        raise ValueError("reference and actual physical dimensions differ") from error
    if factor != 1.0 or any(
        _number(unit, "scale_to_canonical") != 1.0
        or _number(unit, "offset_to_canonical") != 0.0
        for unit in (canonical, representation)
    ):
        raise ValueError(
            "declared unit scaling differs from the unmodified reference magnitude"
        )
    if (
        _number(physical, "scale_to_canonical") != 1.0
        or _number(physical, "offset_to_canonical") != 0.0
    ):
        raise ValueError(
            "ordinary comparison cannot silently convert original magnitudes"
        )
    if _word(key, "scale_kind") != (
        "difference" if type_name == "DeltaPressure" else "point"
    ):
        raise ValueError("physical point/difference arithmetic differs")
    if _word(kind, "addition_kind") != (
        "origin_sensitive"
        if type_name in {"Temperature", "Pressure", "DeltaPressure", "MolarEnthalpy"}
        else "additive"
    ):
        raise ValueError("physical additive/origin-sensitive arithmetic differs")
    if bool(kind.extensive) != (
        type_name
        in {
            "Flow",
            "ComponentFlow",
            "Power",
            "EnergyTransferRate",
            "Area",
            "CapitalCost",
        }
    ) or kind.HasField("category"):
        raise ValueError("physical extensive/discrete convention differs")
    if len(key.shape) or len(physical.shape):
        raise ValueError("ordinary scalar reports cannot acquire an array shape")
    molar = type_name in {"Flow", "ComponentFlow", "MoleFraction", "MolarEnthalpy"}
    if key.HasField("basis") != molar or physical.HasField("basis") != molar:
        raise ValueError("original amount/composition basis presence differs")
    if molar:
        basis = _message(physical, "basis")
        if (
            _optional_identity(key, "basis") != _identity(basis)
            or _word(basis, "kind") != "molar"
            or any(
                basis.HasField(field)
                for field in ("composition_basis", "rate_basis", "reference_conditions")
            )
        ):
            raise ValueError("ordinary reference requires its exact molar basis")
    if physical.HasField("basis_reference_state"):
        raise ValueError("ordinary reference has no standard-volume conditions")
    species = type_name in {"MoleFraction", "ComponentFlow"}
    if (
        key.HasField("subject_kind") != species
        or physical.HasField("subject") != species
    ):
        raise ValueError("original scalar species subject presence differs")
    if species:
        subject = _message(physical, "subject")
        if (
            _optional_identity(key, "subject_kind") != _identity(subject)
            or _identity(subject) != _SPECIES_KIND_ID
        ):
            raise ValueError("original species subject meaning differs")
    datum = type_name in {"Power", "MolarEnthalpy"}
    if (
        key.HasField("reference_state") != datum
        or physical.HasField("reference_state") != datum
    ):
        raise ValueError("original datum presence differs")
    if datum:
        reference = _message(physical, "reference_state")
        if (
            _optional_identity(key, "reference_state") != _identity(reference)
            or reference.name != "stock"
            or _word(reference, "kind") != "ideal_gas_at_conditions"
            or reference.include_enthalpy_of_formation
            or reference.HasField("subject")
        ):
            raise ValueError("native stock reference meaning differs")
        if (
            _number(_message(reference, "temperature"), "value") != 298.15
            or _number(_message(reference, "pressure"), "value") != 100000.0
        ):
            raise ValueError(
                "stock datum is298.15K/100000Pa, not the operating pressure"
            )
    if any(unit.HasField("reference_state") for unit in (canonical, representation)):
        raise ValueError("ordinary representation cannot impose another unit datum")


def _fixed_inputs(
    case_name: str, inputs: Mapping[str, SpeciesInput] | None = None
) -> dict[str, NumericValue]:
    if case_name in CASE_NAMES[:3]:
        rating = case_name == CASE_NAMES[1]
        return ordinary_exchanger.fixed_inputs(
            ordinary_exchanger.build_exchanger(
                arithmetic=case_name == CASE_NAMES[2], rating=rating, inputs=inputs
            ),
            rating=rating,
        )
    fresh = ordinary_recycle.build_recycle(inputs=inputs).fs.fresh[0]
    return {
        "feed.state.flow": fresh.flow_mol,
        "feed.state.T": fresh.temperature,
        "feed.state.pressure": fresh.pressure,
        **{
            f"feed.state.z[{species}]": fresh.mole_frac_comp[species]
            for species in SPECIES
        },
    }


def _validate_context(physicals: list[Message]) -> None:
    """Check repeated original owners and typed datum-coordinate cross-links."""
    seen: dict[tuple[str, bytes], bytes] = {}
    types: dict[str, Message] = {}
    for physical in physicals:
        quantity = _message(physical, "quantity")
        types[str(quantity.name)] = physical
        for field in (
            "quantity",
            "quantity_kind",
            "canonical_unit",
            "representation_unit",
            "basis",
            "reference_state",
            "subject",
        ):
            if not physical.HasField(field):
                continue
            owner = _message(physical, field)
            key = (owner.DESCRIPTOR.full_name, _identity(owner))
            encoded = owner.SerializeToString(deterministic=True)
            if key in seen and seen[key] != encoded:
                raise ValueError(
                    "one original physical identity has conflicting descriptions"
                )
            seen[key] = encoded
    for physical in physicals:
        if not physical.HasField("reference_state"):
            continue
        datum = _message(physical, "reference_state")
        for field, type_name in (
            ("temperature", "Temperature"),
            ("pressure", "Pressure"),
        ):
            selected = types[type_name]
            condition = _message(datum, field)
            if _identity(_message(condition, "quantity_type")) != _identity(
                _message(selected, "quantity")
            ) or _identity(_message(condition, "unit")) != _identity(
                _message(selected, "canonical_unit")
            ):
                raise ValueError(
                    "typed stock reference coordinate has a different quantity or unit"
                )


def _reference_outputs(
    case_name: str,
    *,
    inputs: Mapping[str, SpeciesInput] | None = None,
    environment: ReferenceEnvironment | None = None,
) -> dict[str, float] | None:
    if case_name in CASE_NAMES[:3]:
        raw = ordinary_exchanger.observations(
            ordinary_exchanger.solve_exchanger(
                arithmetic=case_name == CASE_NAMES[2],
                rating=case_name == CASE_NAMES[1],
                inputs=inputs,
                environment=environment,
            )
        )
        renamed = {key: raw[key] for key in EXCHANGER_KEYS[:16]}
        labels = (
            "ordinary hot duty",
            "ordinary cold duty",
            "ordinary heat transfer coefficient",
            "solved heat-transfer area",
            "area in square feet",
            "base purchase cost",
            "ordinary material factor",
            "ordinary pressure factor",
            "capital cost at CEPCI 500",
        )
        renamed.update(
            zip(EXCHANGER_KEYS[16:], (raw[label] for label in labels), strict=True)
        )
        return renamed
    if case_name == CASE_NAMES[3]:
        raw = ordinary_recycle.observations(
            ordinary_recycle.solve_recycle(inputs=inputs, environment=environment)
        )
        labels = (
            "ordinary feed flow",
            "vapor product",
            "ordinary purge flow",
            "liquid recycle",
            "ordinary feed energy",
            "ordinary heater duty",
            "ordinary vapor energy",
            "ordinary purge energy",
            *(
                f"ordinary {'feed' if stream == 'fresh' else stream} {species}"
                for key in RECYCLE_KEYS[8:]
                for stream, species in [key.split(".")]
            ),
        )
        return dict(zip(RECYCLE_KEYS, (raw[label] for label in labels), strict=True))
    failure = ordinary_recycle.overheated_failure(
        inputs=inputs, environment=environment
    )
    witness = ordinary_recycle.overheated_observations(failure)
    if (
        failure.result.solver.termination_condition != TerminationCondition.infeasible
        or failure.original_before != failure.original_after
        or not all(math.isfinite(number) for number in witness.values())
        or witness["source temperature"] != 500.0
        or witness["source caloric temperature"] != 368.0
        or witness["receiver upper bound"] != 450.0
        or witness["receiver temperature"] > 450.0 + 1e-8
        or witness["temperature connection residual"] > -49.999
        or abs(witness["energy transport residual"]) <= 1.0
        or any(witness[f"source {species}"] != 0.005 for species in SPECIES)
    ):
        raise RuntimeError(
            "actual overheated failure did not retain the original physical contradiction"
        )
    # Separate retained failure evidence; this diagnostic never supplies the
    # binary reference response or substitutes for the proof checks above.
    print(
        "ordinary_reference_failure stage=overheated_tear "
        f"termination={failure.result.solver.termination_condition} "
        f"message={str(failure.result.solver.message)!r} original_changed=False "
        f"witness_hex={[(key, number.hex()) for key, number in witness.items()]!r}",
        file=sys.stderr,
    )
    return None


def produce_reference(
    descriptors: bytes,
    contract_payload: bytes,
    *,
    evidence: dict[str, str] | None = None,
) -> bytes:
    """Validate shared correspondence before independently constructing answers."""
    if version("protobuf") != "7.36.2":
        raise RuntimeError("ordinary binary reference requires protobuf7.36.2")
    if not contract_payload or len(contract_payload) > MAXIMUM_BYTES:
        raise ValueError("ordinary contract exceeds the selected binary byte bound")
    envelope_type = reference_type(descriptors)
    pool = envelope_type.DESCRIPTOR.file.pool
    contract_type = message_factory.GetMessageClass(
        pool.FindMessageTypeByName("pse.reference.v1.ReferenceCaseContract")
    )
    contract = contract_type.FromString(contract_payload)
    if _message(contract, "version").value != 1 or contract.case_name not in CASE_NAMES:
        raise ValueError("unsupported ordinary case contract interpretation")
    meaning = _message(contract, "meaning")
    for field, extent in (
        ("case", 16),
        ("source", 32),
        ("formulation", 32),
        ("physical", 32),
        ("configuration", 32),
    ):
        original = bytes(getattr(meaning, field))
        if len(original) != extent or original == bytes(extent):
            raise ValueError("original case meaning is incomplete")
    keys = EXCHANGER_KEYS if contract.case_name in CASE_NAMES[:3] else RECYCLE_KEYS
    observables = list(contract.observables)
    if tuple(item.key for item in observables) != keys:
        raise ValueError("complete ordered scientific output roles differ")
    identities = [bytes(item.output) for item in observables]
    if len(set(identities)) != len(keys) or any(
        len(identity) != 16 or identity == bytes(16) for identity in identities
    ):
        raise ValueError(
            "required original output identities are malformed or duplicated"
        )
    inputs = parse_inputs(_validate_artifacts(contract, retain_properties=True))
    expected = _fixed_inputs(contract.case_name, inputs)
    original_inputs = list(contract.specification)
    if len(original_inputs) != len(expected) or {
        item.key for item in original_inputs
    } != set(expected):
        raise ValueError(
            "original fixed input population differs; numerical starts are excluded"
        )
    for item in original_inputs:
        coordinate = expected[item.key]
        if (
            units.convert_value(
                1.0,
                from_units=units.get_units(coordinate),
                to_units=_reference_unit(_type_for_key(item.key)),
            )
            != 1.0
        ):
            raise ValueError("independent original fixed input uses a different unit")
        if int(item.value) != float_bits(float(value(coordinate))):
            raise ValueError(f"original fixed input value differs: {item.key}")
        _validate_physical(_message(item, "physical"), _type_for_key(item.key))
    for item in observables:
        _validate_physical(_message(item, "physical"), _type_for_key(item.key))
    _validate_context(
        [_message(item, "physical") for item in (*original_inputs, *observables)]
    )
    environment = ReferenceEnvironment.from_process()
    values = _reference_outputs(
        contract.case_name, inputs=inputs, environment=environment
    )
    _validate_artifacts(contract)
    if evidence is not None:
        evidence.update(environment.evidence)
    envelope = envelope_type(version=1)
    envelope.meaning.CopyFrom(meaning)
    envelope.required.extend(identities)
    if values is None:
        envelope.outcome.failed.code = "trial_rejected"
        envelope.outcome.failed.stage = "overheated_tear"
    else:
        if tuple(values) != keys or not all(
            math.isfinite(number) for number in values.values()
        ):
            raise RuntimeError(
                "independent reference output population is incomplete or nonfinite"
            )
        for contract_output in observables:
            physical = contract_output.physical
            key = physical.quantity.key
            output = envelope.outcome.completed.outputs.add(
                id=contract_output.output,
                quantity=_identity(physical.quantity),
                unit=_identity(physical.representation_unit),
                value=float_bits(values[contract_output.key]),
            )
            for field, source in (("basis", "basis"), ("datum", "reference_state")):
                identity = _optional_identity(key, source)
                if identity is not None:
                    setattr(output, field, identity)
    return encode_reference(envelope)


def _bounded_read(path: Path) -> bytes:
    with path.open("rb") as stream:
        content = stream.read(MAXIMUM_BYTES + 1)
    if len(content) > MAXIMUM_BYTES:
        raise ValueError("reference input exceeds the selected binary byte bound")
    return content


def main() -> None:
    """Write one new binary result; candidate files are not part of this interface."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--descriptors", type=Path, required=True)
    parser.add_argument("--contract", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    evidence: dict[str, str] = {}
    payload = produce_reference(
        _bounded_read(arguments.descriptors),
        _bounded_read(arguments.contract),
        evidence=evidence,
    )
    with arguments.output.open("xb") as stream:
        stream.write(payload)
    print(
        "ordinary_reference_environment " + json.dumps(evidence, sort_keys=True),
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
