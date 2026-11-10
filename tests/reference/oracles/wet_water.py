# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent IDAES 2.13.0 IAPWS95 solve for the authored wet Water turbine."""

from __future__ import annotations

import hashlib
import json
import math
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Protocol, cast

from google.protobuf.message import Message

from proto_exchange import encode_reference, message_field, repeated_field, scalar_field

from .ordinary_exchange import (
    _bounded_read,
    _message,
    _read_artifact,
    _validate_context,
    _validate_physical,
    bits_float,
    complete_envelope,
    float_bits,
    ordered_observables,
    read_case_contract,
)
from .reference_environment import ReferenceEnvironment

if TYPE_CHECKING:
    from collections.abc import Callable

    from pyomo.core.base.var import IndexedVar, VarData
    from pyomo.core.expr.numeric_expr import NumericValue
    from pyomo.dae import ContinuousSet


class SteamState(Protocol):
    """Selected variables and IAPWS95 expressions of one turbine state."""

    flow_mol: VarData
    pressure: VarData
    enth_mol: VarData
    temperature: NumericValue
    entr_mol: NumericValue
    vapor_frac: NumericValue


class SteamStates(Protocol):
    """Actual IAPWS95 state blocks indexed by time."""

    def __getitem__(self, time: float) -> SteamState:
        """Read an original turbine state at one flowsheet time."""
        ...


class SteamVolume(Protocol):
    """The inlet and outlet IAPWS95 state owners."""

    properties_in: SteamStates
    properties_out: SteamStates


class SteamTurbine(Protocol):
    """The selected dynamically constructed IDAES turbine component surface."""

    control_volume: SteamVolume
    properties_isentropic: SteamStates
    deltaP: IndexedVar
    efficiency_isentropic: IndexedVar
    work_mechanical: IndexedVar

    def initialize(self, *, solver: str, optarg: dict[str, float]) -> None:
        """Initialize the actual turbine under the witnessed solver alias."""
        ...


class SteamFlowsheet(Protocol):
    """Actual flowsheet owner for the water property package and turbine."""

    time: ContinuousSet
    water: object
    expander: SteamTurbine


CASE = "steam_fixtures.wet_water_turbine"
CASE_ID = bytes.fromhex("64d0300000000000000000000000021d")
# IDAES 2.13.0, TestIAPWS.test_verify's wet case. Model topology is supplied
# by the installed IDAES Turbine and IAPWS95 owners. Only case facts are frozen here.
INPUTS = (
    ("inlet_temperature", "Temperature", 400.0),
    ("inlet_pressure", "Pressure", 200000.0),
    ("inlet_flow", "Flow", 1000.0),
    ("water_fraction", "MoleFraction", 1.0),
    ("pressure_change", "DeltaPressure", -60000.0),
    ("efficiency", "Scalar", 0.9),
)
OUTPUTS = (
    ("outlet_temperature", "Temperature"),
    ("isentropic_temperature", "Temperature"),
    ("outlet_pressure", "Pressure"),
    ("shaft_work", "EnergyTransferRate"),
    ("outlet_quality", "Scalar"),
    ("isentropic_quality", "Scalar"),
)
ARTIFACTS = (
    "packages/reference/process/models/pressure-changer.pse",
    "packages/reference/process/models/steam-equipment.pse",
    "packages/reference/seed-data/models/steam-fixtures.pse",
    "packages/reference/thermodynamics/models/steam.pse",
)
_INITIALIZE = "steam.wet_turbine.initialize"
_SOLVE = "steam.wet_turbine.solve"


def _verify_artifacts(contract: Message) -> None:
    if (
        tuple(
            scalar_field(item, "path", str)
            for item in repeated_field(contract, "artifacts", Message)
        )
        != ARTIFACTS
    ):
        raise ValueError("wet Water source artifact population differs")
    for item in repeated_field(contract, "artifacts", Message):
        if hashlib.sha256(
            _read_artifact(scalar_field(item, "path", str), 1 << 20)
        ).digest() != scalar_field(item, "sha256", bytes):
            raise ValueError(
                f"wet Water original source differs: {scalar_field(item, 'path', str)}"
            )


def _admit_case(
    contract: Message, meaning: Message
) -> tuple[dict[str, float], list[Message]]:
    if (
        scalar_field(contract, "case_name", str) != CASE
        or scalar_field(meaning, "case", bytes) != CASE_ID
    ):
        raise ValueError("wet Water original authored case identity differs")
    _verify_artifacts(contract)
    inputs = list(repeated_field(contract, "specification", Message))
    if tuple(scalar_field(item, "key", str) for item in inputs) != tuple(
        key for key, _, _ in INPUTS
    ):
        raise ValueError("wet Water original fixed input population differs")
    fixed: dict[str, float] = {}
    for item, (key, quantity, expected) in zip(inputs, INPUTS, strict=True):
        _validate_physical(_message(item, "physical"), quantity)
        if scalar_field(item, "value", int) != float_bits(expected):
            raise ValueError(f"wet Water original fixed value differs: {key}")
        fixed[key] = bits_float(scalar_field(item, "value", int))
    outputs = ordered_observables(contract, tuple(key for key, _ in OUTPUTS))
    for item, (_, quantity) in zip(outputs, OUTPUTS, strict=True):
        _validate_physical(_message(item, "physical"), quantity)
    _validate_context([_message(item, "physical") for item in (*inputs, *outputs)])
    return fixed, outputs


def _property_images() -> dict[str, str]:
    """Bind the imported IAPWS95 library and H2O EOS to their prepared files."""
    import idaes
    from idaes.models.properties import iapws95
    from idaes.models.properties.general_helmholtz import helmholtz_functions
    from idaes.models.properties.general_helmholtz.components import parameters

    if not idaes.bin_directory or not iapws95.iapws95_available():
        raise RuntimeError("prepared IDAES IAPWS95 extension is unavailable")
    library = (Path(idaes.bin_directory) / "general_helmholtz_external.so").resolve(
        strict=True
    )
    # The pinned Python property owner exposes availability but not the path it
    # actually supplied to ctypes and Pyomo. Observe its selected binding.
    loaded = helmholtz_functions._flib  # noqa: SLF001 - pinned IDAES selected library
    if loaded is None or Path(loaded).resolve(strict=True) != library:
        raise RuntimeError(
            "performing IAPWS95 extension differs from the prepared library"
        )
    data = Path(parameters.get_parameter_path()).resolve(strict=True)
    owner = parameters.__file__
    if owner is None or data != Path(owner).resolve(strict=True).parent:
        raise RuntimeError("performing H2O data differs from the pinned IDAES package")
    images = {
        "library": library,
        "eos": data / "h2o_expressions_eos.nl",
        "parameters": data / "h2o_parameters.json",
    }
    evidence = {}
    for role, path in images.items():
        prefix = f"steam.wet_turbine.iapws95.{role}"
        evidence[f"{prefix}.path"] = str(path)
        evidence[f"{prefix}.sha256"] = hashlib.sha256(_bounded_read(path)).hexdigest()
    return evidence


def solve(
    fixed: dict[str, float], environment: ReferenceEnvironment
) -> dict[str, float]:
    """Initialize and solve the pinned IAPWS95 turbine under the checked Ipopt image."""
    from idaes.core import FlowsheetBlock
    from idaes.core.util.model_statistics import degrees_of_freedom
    from idaes.models.properties import iapws95
    from idaes.models.unit_models import Turbine
    from pyomo.environ import ConcreteModel, assert_optimal_termination

    from .pyomo_support import units, value

    model = ConcreteModel()
    flowsheet = cast("SteamFlowsheet", FlowsheetBlock(dynamic=False))
    model.process = flowsheet
    flowsheet.water = cast(
        "Callable[[], object]", vars(iapws95)["Iapws95ParameterBlock"]
    )()
    turbine = flowsheet.expander = cast(
        "SteamTurbine", Turbine(property_package=flowsheet.water)
    )
    first = flowsheet.time.first()
    inlet = turbine.control_volume.properties_in[first]
    pressure = fixed["inlet_pressure"]
    flow = fixed["inlet_flow"]
    inlet_h = iapws95.htpx(
        T=fixed["inlet_temperature"] * units.K, P=pressure * units.Pa
    )
    inlet.flow_mol.fix(flow)
    inlet.enth_mol.fix(inlet_h)
    inlet.pressure.fix(pressure)
    turbine.deltaP[first].fix(fixed["pressure_change"])
    turbine.efficiency_isentropic[first].fix(fixed["efficiency"])
    if degrees_of_freedom(model) != 0:
        raise RuntimeError("independent wet Water turbine is not square")

    alias = "pse_reference_wet_water_initializer"
    with environment.registration("ipopt", _INITIALIZE, alias=alias):
        turbine.initialize(solver=alias, optarg={"tol": 1e-6})
    environment.require_solves(_INITIALIZE, 2)
    solver = environment.solver("ipopt", _SOLVE)
    solver.options["tol"] = 1e-8
    assert_optimal_termination(solver.solve(model))
    environment.require_solves(_SOLVE, 1)

    outlet = turbine.control_volume.properties_out[first]
    reversible = turbine.properties_isentropic[first]
    work = float(value(turbine.work_mechanical[first]))
    outlet_pressure = float(value(outlet.pressure))
    quality = float(value(outlet.vapor_frac))
    reversible_quality = float(value(reversible.vapor_frac))

    # These checks use only results within the independent reference datum. Shaft
    # work is compared across models without passing either one's absolute h or s.
    def near(left: float, right: float, tolerance: float) -> bool:
        return math.isclose(left, right, rel_tol=0.0, abs_tol=tolerance)

    checks = (
        near(float(value(inlet.temperature)), fixed["inlet_temperature"], 1e-4),
        near(float(value(outlet.flow_mol)), flow, 1e-6),
        near(outlet_pressure, pressure + fixed["pressure_change"], 1e-3),
        near(float(value(reversible.pressure)), outlet_pressure, 1e-3),
        near(float(value(reversible.entr_mol)), float(value(inlet.entr_mol)), 1e-6),
        near(
            flow * (float(value(outlet.enth_mol)) - float(value(inlet.enth_mol))),
            work,
            0.01,
        ),
        work < 0.0 and 0.0 < reversible_quality < quality < 1.0,
    )
    if not all(checks):
        raise RuntimeError(
            "independent turbine did not retain the original wet balances"
        )
    return {
        "outlet_temperature": float(value(outlet.temperature)),
        "isentropic_temperature": float(value(reversible.temperature)),
        "outlet_pressure": outlet_pressure,
        "shaft_work": work,
        "outlet_quality": quality,
        "isentropic_quality": reversible_quality,
    }


def main() -> None:
    """Admit one original binary case and publish only a fresh binary result."""
    if len(sys.argv) != 4 or sys.flags.isolated != 1:
        raise ValueError(
            "select the isolated interpreter and descriptor, contract and result paths"
        )
    descriptors, source, destination = map(Path, sys.argv[1:])
    envelope_type, contract, meaning = read_case_contract(
        _bounded_read(descriptors), _bounded_read(source)
    )
    fixed, outputs = _admit_case(contract, meaning)
    environment = ReferenceEnvironment.from_process()
    property_images = _property_images()
    values = solve(fixed, environment)
    if _property_images() != property_images:
        raise RuntimeError("prepared IAPWS95 property images changed during the solve")
    _verify_artifacts(contract)
    envelope = envelope_type(version=1)
    message_field(envelope, "meaning").CopyFrom(meaning)
    repeated_field(envelope, "required", bytes).extend(
        scalar_field(item, "output", bytes) for item in outputs
    )
    complete_envelope(envelope, outputs, values)
    encoded = encode_reference(envelope)
    with destination.open("xb") as stream:
        stream.write(encoded)
    evidence = {**environment.evidence, **property_images}
    sys.stderr.write(
        "wet_water_reference_environment " + json.dumps(evidence, sort_keys=True) + "\n"
    )
