# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent IDAES properties for the selected ordinary BT process cases.

Only input artifacts are shared with the candidate. Historical IDAES example
parameters, native evaluation functions and candidate results are never imported.
"""

from __future__ import annotations

import csv
import json
import math
from collections.abc import Collection, Mapping
from dataclasses import dataclass
from importlib.metadata import version
from pathlib import Path
from typing import Protocol, cast

import pyarrow.parquet as pq
from idaes.core import Component, LiquidPhase, VaporPhase
from idaes.core.util.constants import Constants
from idaes.models.properties.modular_properties.eos.ceos import Cubic, CubicType
from idaes.models.properties.modular_properties.eos.ideal import Ideal
from idaes.models.properties.modular_properties.phase_equil import SmoothVLE
from idaes.models.properties.modular_properties.phase_equil.bubble_dew import (
    IdealBubbleDew,
    LogBubbleDew,
)
from idaes.models.properties.modular_properties.phase_equil.forms import (
    fugacity,
    log_fugacity,
)
from idaes.models.properties.modular_properties.pure import Perrys, RPP5
from idaes.models.properties.modular_properties.state_definitions import FTPx
from pyomo.core.expr.numeric_expr import NumericValue
from pyomo.environ import Param, exp, units, value

ROOT = Path(__file__).resolve().parents[3]
SPECIES = {"benzene": "71-43-2", "toluene": "108-88-3"}
type Five = tuple[float, float, float, float, float]


@dataclass(frozen=True)
class SpeciesInput:
    """Original publication coordinates, before IDAES unit conversion."""

    gas_cp_kmol: Five
    liquid_cp_mol: Five
    vaporization_298: float
    critical_temperature: float
    critical_pressure: float
    omega: float
    wagner: tuple[float, float, float, float, float, float]


def _mapping(raw: object) -> dict[str, object]:
    if not isinstance(raw, dict) or not all(isinstance(key, str) for key in raw):
        raise ValueError("scientific input requires a string-keyed object")
    return cast("dict[str, object]", raw)


def _number(raw: object) -> float:
    if isinstance(raw, bool) or not isinstance(raw, (int, float, str)):
        raise ValueError("scientific coefficient is not a number")
    result = float(raw)
    if not math.isfinite(result):
        raise ValueError("scientific coefficient must be finite")
    return result


def _five(raw: object) -> Five:
    if not isinstance(raw, (tuple, list)) or len(raw) != 5:
        raise ValueError("five caloric coefficients are required")
    return cast("Five", tuple(_number(item) for item in raw))


def _csv_rows(path: Path) -> dict[str, dict[str, str]]:
    selected: dict[str, dict[str, str]] = {}
    with path.open(newline="", encoding="utf-8") as stream:
        for row in csv.DictReader(stream, delimiter="\t"):
            cas = row["CAS"]
            if cas in SPECIES.values():
                if cas in selected:
                    raise ValueError(f"duplicate selected CAS in {path}")
                selected[cas] = row
    if set(selected) != set(SPECIES.values()):
        raise ValueError(f"incomplete selected CAS population in {path}")
    return selected


def load_inputs(root: Path = ROOT) -> dict[str, SpeciesInput]:
    """Read the actual frozen source artifacts, without parsing native formulas."""
    source = root / "packages/reference/data/ordinary-bt/source-artifacts"
    critical = _csv_rows(source / "yaws-critical.tsv")
    latent = _csv_rows(source / "crc-vaporization.tsv")
    pressure = _csv_rows(source / "poling-wagner.tsv")
    gas: dict[str, Five] = {}
    raw: object = json.loads((source / "feos-poling2000.json").read_bytes())
    if not isinstance(raw, list):
        raise ValueError("FeOS input must contain a record population")
    for item in raw:
        row = _mapping(item)
        cas = _mapping(row["identifier"]).get("cas")
        if isinstance(cas, str) and cas in SPECIES.values():
            if cas in gas:
                raise ValueError("duplicate selected FeOS record")
            gas[cas] = _five(row["DIPPR100"])
    liquid: dict[str, Five] = {}
    records: object = pq.read_table(
        root / "packages/reference/data/perry7/data/perry_liquid.parquet"
    ).to_pylist()
    if not isinstance(records, list):
        raise ValueError("Perry source must contain records")
    for item in records:
        row = _mapping(item)
        cas = row["subject"]
        if isinstance(cas, str) and cas in SPECIES.values():
            if cas in liquid or row["phase_type"] != "liquidPhase":
                raise ValueError("ambiguous selected liquid caloric record")
            liquid[cas] = _five([row[f"c{i}"] for i in range(1, 6)])
    if set(gas) != set(SPECIES.values()) or set(liquid) != set(gas):
        raise ValueError("incomplete ordinary caloric source closure")
    return {
        name: SpeciesInput(
            gas[cas],
            liquid[cas],
            _number(latent[cas]["Hvap298"]),
            _number(critical[cas]["Tc"]),
            _number(critical[cas]["Pc"]),
            _number(critical[cas]["omega"]),
            cast(
                "tuple[float, float, float, float, float, float]",
                tuple(
                    _number(pressure[cas][key])
                    for key in ("A", "B", "C", "D", "Tc", "Pc")
                ),
            ),
        )
        for name, cas in SPECIES.items()
    }


class _MethodConfig(Protocol):
    parameter_data: dict[str, object]


class _WagnerComponent(Protocol):
    config: _MethodConfig
    ordinary_wagner_a: NumericValue
    ordinary_wagner_b: NumericValue
    ordinary_wagner_c: NumericValue
    ordinary_wagner_d: NumericValue
    ordinary_wagner_tc: NumericValue
    ordinary_wagner_pc: NumericValue


class Wagner25:
    """Poling's 2.5,5 pressure law with its separate fitted critical coordinates."""

    @staticmethod
    def build_parameters(component: _WagnerComponent) -> None:
        """Install fixed coefficients using the public IDAES property-method hook."""
        raw = component.config.parameter_data["ordinary_wagner"]
        if not isinstance(raw, tuple) or len(raw) != 6:
            raise ValueError("Wagner2.5,5 requires six explicit source coefficients")
        for name, coefficient in zip(
            ("a", "b", "c", "d", "tc", "pc"), raw, strict=True
        ):
            unit = (
                units.K
                if name == "tc"
                else units.Pa
                if name == "pc"
                else units.dimensionless
            )
            setattr(
                component,
                f"ordinary_wagner_{name}",
                Param(initialize=_number(coefficient), units=unit),
            )

    @staticmethod
    def return_expression(
        _state: object,
        component: _WagnerComponent,
        temperature: NumericValue,
        dT: bool = False,
    ) -> NumericValue:
        """Evaluate pressure or its analytical temperature derivative for initialization."""
        theta = temperature / component.ordinary_wagner_tc
        tau = 1 - theta
        terms = (
            (component.ordinary_wagner_a, 1.0),
            (component.ordinary_wagner_b, 1.5),
            (component.ordinary_wagner_c, 2.5),
            (component.ordinary_wagner_d, 5.0),
        )
        polynomial = sum(coefficient * tau**power for coefficient, power in terms)
        pressure = component.ordinary_wagner_pc * exp(polynomial / theta)
        if not dT:
            return pressure
        slope = sum(
            power * coefficient * tau ** (power - 1) for coefficient, power in terms
        )
        return (
            pressure
            * (-slope * theta - polynomial)
            / (component.ordinary_wagner_tc * theta**2)
        )


class _CaloricPhase(Protocol):
    def is_liquid_phase(self) -> bool: ...
    def is_vapor_phase(self) -> bool: ...


class _CaloricParameters(Protocol):
    def get_phase(self, name: str) -> _CaloricPhase: ...
    def get_component(self, name: str) -> object: ...


class _CaloricState(Protocol):
    params: _CaloricParameters
    temperature: NumericValue
    mole_frac_phase_comp: Mapping[tuple[str, str], NumericValue]

    def components_in_phase(self, phase: str) -> Collection[str]: ...


class CaloricIdeal(Ideal):
    """Retain ideal equilibrium with the selected pressure-independent caloric law.

    The ordinary source explicitly selects h(T) for liquid and vapor. IDAES's
    default incompressible-liquid pressure-volume enthalpy increment is a
    different realization and would require density data this case does not use.
    """

    @staticmethod
    def enth_mol_phase_comp(
        state: _CaloricState, phase: str, name: str
    ) -> NumericValue:
        """Delegate pure caloric evaluation to the selected IDAES library methods."""
        component = state.params.get_component(name)
        selected = state.params.get_phase(phase)
        if selected.is_liquid_phase():
            return cast(
                "NumericValue",
                Perrys.enth_mol_liq_comp.return_expression(
                    state, component, state.temperature
                ),
            )
        if selected.is_vapor_phase():
            return cast(
                "NumericValue",
                RPP5.enth_mol_ig_comp.return_expression(
                    state, component, state.temperature
                ),
            )
        raise ValueError(
            "the ordinary caloric selection has only liquid and vapor phases"
        )

    @staticmethod
    def enth_mol_phase(state: _CaloricState, phase: str) -> NumericValue:
        """Use the explicit ordinary ideal-mixture enthalpy convention."""
        return cast(
            "NumericValue",
            sum(
                state.mole_frac_phase_comp[phase, name]
                * CaloricIdeal.enth_mol_phase_comp(state, phase, name)
                for name in state.components_in_phase(phase)
            ),
        )


def configuration(kind: str, root: Path = ROOT) -> dict[str, object]:
    """Build a fresh selected ideal, liquid-only or PR parameter configuration."""
    if version("idaes-pse") != "2.13.0" or version("pyomo") != "6.10.1":
        raise RuntimeError("ordinary reference requires IDAES2.13.0 and Pyomo6.10.1")
    if kind not in {"ideal", "liquid", "pr"}:
        raise ValueError("unknown ordinary thermodynamic realization")
    gas_constant = float(value(Constants.gas_constant))
    components: dict[str, object] = {}
    for name, source in load_inputs(root).items():
        data: dict[str, object] = {
            "temperature_crit": (source.critical_temperature, units.K),
            "pressure_crit": (source.critical_pressure, units.Pa),
            "omega": source.omega,
            "cp_mol_ig_comp_coeff": {
                f"a{i}": (coefficient / (1000 * gas_constant), units.K**-i)
                for i, coefficient in enumerate(source.gas_cp_kmol)
            },
            "cp_mol_liq_comp_coeff": {
                str(i): (coefficient, units.J / units.mol / units.K**i)
                for i, coefficient in enumerate(source.liquid_cp_mol, start=1)
            },
            "enth_mol_form_vap_comp_ref": (0.0, units.J / units.mol),
            "entr_mol_form_vap_comp_ref": (0.0, units.J / units.mol / units.K),
            "enth_mol_form_liq_comp_ref": (
                -source.vaporization_298,
                units.J / units.mol,
            ),
            "ordinary_wagner": source.wagner,
        }
        components[name] = {
            "type": Component,
            "enth_mol_ig_comp": RPP5,
            "entr_mol_ig_comp": RPP5,
            "cp_mol_ig_comp": RPP5,
            "enth_mol_liq_comp": Perrys,
            "cp_mol_liq_comp": Perrys,
            "pressure_sat_comp": Wagner25,
            "phase_equilibrium_form": {
                ("Vap", "Liq"): log_fugacity if kind == "pr" else fugacity
            },
            "parameter_data": data,
        }
    phase = (
        {
            "equation_of_state": Cubic,
            "equation_of_state_options": {"type": CubicType.PR},
        }
        if kind == "pr"
        else {"equation_of_state": CaloricIdeal}
    )
    settings: dict[str, object] = {
        "components": components,
        "phases": {"Liq": {"type": LiquidPhase, **phase}},
        "base_units": {
            "time": units.s,
            "length": units.m,
            "mass": units.kg,
            "amount": units.mol,
            "temperature": units.K,
        },
        "state_definition": FTPx,
        "state_bounds": {
            "flow_mol": (0, 1, 1000, units.mol / units.s),
            "temperature": (300, 368, 450, units.K),
            "pressure": (50000, 101325, 200000, units.Pa),
        },
        "temperature_ref": (298.15, units.K),
        "pressure_ref": (100000, units.Pa),
        "include_enthalpy_of_formation": True,
    }
    if kind != "liquid":
        cast("dict[str, object]", settings["phases"])["Vap"] = {
            "type": VaporPhase,
            **phase,
        }
        settings.update(
            phases_in_equilibrium=[("Vap", "Liq")],
            phase_equilibrium_state={("Vap", "Liq"): SmoothVLE},
            bubble_dew_method=LogBubbleDew if kind == "pr" else IdealBubbleDew,
        )
    if kind == "pr":
        settings["parameter_data"] = {
            "PR_kappa": {(i, j): 0.0 for i in SPECIES for j in SPECIES}
        }
    return settings
