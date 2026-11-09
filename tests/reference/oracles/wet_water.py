# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent IDAES 2.13.0 IAPWS95 solve for the authored wet Water turbine."""

from __future__ import annotations

import hashlib
import json
import math
import sys
from pathlib import Path
from typing import TYPE_CHECKING

from proto_exchange import encode_reference

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
    from google.protobuf.message import Message

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
    if tuple(item.path for item in contract.artifacts) != ARTIFACTS:
        raise ValueError("wet Water source artifact population differs")
    for item in contract.artifacts:
        if hashlib.sha256(_read_artifact(item.path, 1 << 20)).digest() != item.sha256:
            raise ValueError(f"wet Water original source differs: {item.path}")


def _admit_case(
    contract: Message, meaning: Message
) -> tuple[dict[str, float], list[Message]]:
    if contract.case_name != CASE or bytes(meaning.case) != CASE_ID:
        raise ValueError("wet Water original authored case identity differs")
    _verify_artifacts(contract)
    inputs = list(contract.specification)
    if tuple(item.key for item in inputs) != tuple(key for key, _, _ in INPUTS):
        raise ValueError("wet Water original fixed input population differs")
    fixed: dict[str, float] = {}
    for item, (key, quantity, expected) in zip(inputs, INPUTS, strict=True):
        _validate_physical(_message(item, "physical"), quantity)
        if item.value != float_bits(expected):
            raise ValueError(f"wet Water original fixed value differs: {key}")
        fixed[key] = bits_float(item.value)
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
    from pyomo.environ import ConcreteModel, assert_optimal_termination, units, value

    model = ConcreteModel()
    model.process = FlowsheetBlock(dynamic=False)
    model.process.water = iapws95.Iapws95ParameterBlock()
    turbine = model.process.expander = Turbine(property_package=model.process.water)
    first = model.process.time.first()
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
        near(flow * (float(value(outlet.enth_mol)) - float(value(inlet.enth_mol))), work, 0.01),
        work < 0.0 and 0.0 < reversible_quality < quality < 1.0,
    )
    if not all(checks):
        raise RuntimeError("independent turbine did not retain the original wet balances")
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
    envelope.meaning.CopyFrom(meaning)
    envelope.required.extend(item.output for item in outputs)
    complete_envelope(envelope, outputs, values)
    encoded = encode_reference(envelope)
    with destination.open("xb") as stream:
        stream.write(encoded)
    evidence = {**environment.evidence, **property_images}
    sys.stderr.write(
        "wet_water_reference_environment " + json.dumps(evidence, sort_keys=True) + "\n"
    )
