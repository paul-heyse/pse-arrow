# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Author shared physical declarations; never compute a product or solver oracle."""

from __future__ import annotations

import json
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "tests/fixtures/plan14"
NS = uuid.UUID("84982a44-54a4-4f53-a59b-1d961756350f")


def sid(name: str) -> str:
    return uuid.uuid5(NS, name).hex


def write(name: str, value: object) -> None:
    path = DEST / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def main() -> None:
    primitive = (
        ROOT / "tests/fixtures/packages/physical-primitives/materials/physical.yaml"
    )
    d = json.loads(
        "\n".join(
            line
            for line in primitive.read_text().splitlines()
            if not line.startswith("#")
        )
    )
    reference, molar, composition = [
        sid(s) for s in ("datum", "molar", "mole-fraction")
    ]
    d["reference_states"] = [
        {
            "reference_state_id": reference,
            "kind": "custom",
            "temperature": 298.15,
            "pressure": None,
            "include_enthalpy_of_formation": False,
            "phase_id": None,
            "doc": "DIPPR100 integral origin.",
        }
    ]
    entropy_reference = sid("entropy-datum")
    d["reference_states"].append(
        dict(
            d["reference_states"][0],
            reference_state_id=entropy_reference,
            pressure=100000.0,
            doc="DIPPR ideal-gas entropy origin at 298.15 K and 1 bar.",
        )
    )
    d["bases"] = [
        {
            "basis_id": i,
            "kind": "molar",
            "composition_basis": c,
            "rate_basis": None,
            "reference_conditions_id": None,
        }
        for i, c in [(molar, None), (composition, "mole_fraction")]
    ]
    quantities, units, kinds = {}, {}, {}

    def quantity(
        name: str,
        dim: list[int],
        *,
        basis: str | None = None,
        ref: str | None = None,
        origin: bool = False,
    ) -> None:
        unit, kind = sid("unit." + name), sid("kind." + name)
        units[name], kinds[name] = unit, kind
        dimensions = [{"num": e, "den": 1} for e in dim]
        d["units"].append(
            {
                "unit_id": unit,
                "symbol": "acceptance_" + name,
                "name": name,
                "dimension": dimensions,
                "scale_to_canonical": 1.0,
                "offset_to_canonical": 0.0,
                "is_affine": False,
                "reference_state_id": None,
                "system": "SI",
                "doc": "Explicit SI fixture.",
            }
        )
        d["quantity_kinds"].append(
            {
                "quantity_kind_id": kind,
                "name": name,
                "dimension": dimensions,
                "extensive": name in ["amount", "energy", "power", "flow", "volume"],
                "addition_kind": "origin_sensitive" if origin else "additive",
                "doc": "Typed physical acceptance contract.",
            }
        )
        for scale in ["point", "difference"] if origin else ["point"]:
            qname = name if scale == "point" else name + "_difference"
            qid = sid("quantity." + qname)
            quantities[qname] = qid
            d["quantity_types"].append(
                {
                    "quantity_type_id": qid,
                    "quantity_kind_id": kind,
                    "basis_id": basis,
                    "reference_state_id": ref,
                    "scale_kind": scale,
                    "shape": [],
                    "subject_kind": None,
                    "canonical_unit_id": unit,
                    "nominal_magnitude": None,
                    "doc": "Explicit complete physical key.",
                }
            )

    quantity("temperature", [0, 0, 0, 1, 0, 0, 0, 0], origin=True)
    quantity("density", [-3, 0, 0, 0, 1, 0, 0, 0])
    quantity("fraction", [0] * 8, basis=composition)
    quantity("pressure", [-1, 1, -2, 0, 0, 0, 0, 0])
    quantity(
        "enthalpy", [2, 1, -2, 0, -1, 0, 0, 0], basis=molar, ref=reference, origin=True
    )
    quantity(
        "entropy",
        [2, 1, -2, -1, -1, 0, 0, 0],
        basis=molar,
        ref=entropy_reference,
        origin=True,
    )
    quantity("time", [0, 0, 1, 0, 0, 0, 0, 0])
    quantity("amount", [0, 0, 0, 0, 1, 0, 0, 0])
    quantity("volume", [3, 0, 0, 0, 0, 0, 0, 0])
    quantity("flow", [0, 0, -1, 0, 1, 0, 0, 0])
    quantity("energy", [2, 1, -2, 0, 0, 0, 0, 0], ref=reference, origin=True)
    quantity("power", [2, 1, -3, 0, 0, 0, 0, 0], ref=reference)
    quantity(
        "sqrt_pressure", [-1, 1, -2, 0, 0, 0, 0, 0]
    )  # rational dimensions replaced below
    for row in d["units"] + d["quantity_kinds"]:
        if (
            row.get("unit_id") == units["sqrt_pressure"]
            or row.get("quantity_kind_id") == kinds["sqrt_pressure"]
        ):
            row["dimension"] = [
                {"num": e, "den": 2 if i < 2 and e else 1}
                for i, e in enumerate([-1, 1, -1, 0, 0, 0, 0, 0])
            ]
    quantity("valve", [0, 0, 0, 0, 0, 0, 0, 0])
    for row in d["units"] + d["quantity_kinds"]:
        if (
            row.get("unit_id") == units["valve"]
            or row.get("quantity_kind_id") == kinds["valve"]
        ):
            row["dimension"] = [
                {"num": e, "den": 2 if i < 2 and e else 1}
                for i, e in enumerate([1, -1, 0, 0, 1, 0, 0, 0])
            ]
    quantities["neutral"] = "1f" * 16
    units["neutral"] = "0a" * 16
    kinds["neutral"] = "15" * 16
    d["quantity_operations"], d["quantity_preconditions"] = [], []

    def op(opcode: str, args: Sequence[str], result: str) -> None:
        tag = opcode + "." + ".".join(args)
        pre = []
        for i, arg in enumerate(args):
            p = sid("pre." + tag + "." + str(i))
            pre.append(p)
            d["quantity_preconditions"].append(
                {
                    "invariant_id": p,
                    "kind": "operand_quantity_contract",
                    "operand_positions": [i],
                    "required_basis_id": None,
                    "required_quantity_type_id": quantities[arg],
                    "match_shape": True,
                }
            )
        target = next(
            r
            for r in d["quantity_types"]
            if r["quantity_type_id"] == quantities[result]
        )
        d["quantity_operations"].append(
            {
                "operation_id": sid("op." + tag),
                "opcode": opcode,
                "input_kind_ids": [kinds[a.removesuffix("_difference")] for a in args],
                "result_kind_id": target["quantity_kind_id"],
                "basis_rule": "declared_result",
                "reference_rule": "declared_result",
                "scale_rule": target["scale_kind"],
                "shape_rule": "scalar",
                "basis_source": None,
                "reference_source": None,
                "scale_source": None,
                "shape_source": None,
                "subject_rule": "preserve",
                "subject_source": 0,
                "result_subject_kind": None,
                "result_basis_id": target["basis_id"],
                "result_reference_state_id": target["reference_state_id"],
                "input_conversions": [],
                "precondition_invariant_ids": pre,
            }
        )

    for args, out in [
        (("density", "volume"), "amount"),
        (("flow", "enthalpy"), "power"),
        (("amount", "enthalpy"), "energy"),
        (("valve", "sqrt_pressure"), "flow"),
    ]:
        op("Mul", args, out)
    op("Div", ("pressure", "density"), "enthalpy_difference")
    op("Div", ("pressure", "pressure"), "neutral")
    op("Sqrt", ("pressure",), "sqrt_pressure")
    op("Derivative", ("amount",), "flow")
    op("Derivative", ("energy",), "power")
    op("Log", ("fraction",), "neutral")
    # Equivalent affine temperature representation exercises conversion before provider evaluation.
    celsius = sid("unit.celsius")
    c = dict(next(u for u in d["units"] if u["unit_id"] == units["temperature"]))
    c.update(
        unit_id=celsius,
        symbol="acceptance_degC",
        name="Celsius",
        offset_to_canonical=273.15,
        is_affine=True,
    )
    d["units"].append(c)
    shaped = dict(
        next(
            t
            for t in d["quantity_types"]
            if t["quantity_type_id"] == quantities["flow"]
        )
    )
    shaped.update(quantity_type_id=sid("quantity.flow.inlets"), shape=["port_set"])
    d["quantity_types"].append(shaped)
    write("package/materials/physical.yaml", d)
    (DEST / "package/package.toml").write_text(
        (ROOT / "tests/fixtures/packages/physical-primitives/package.toml")
        .read_text()
        .replace("physical-primitives", "plan14-processes")
    )
    ids = {
        "quantities": quantities,
        "units": units,
        "celsius": celsius,
        "datum": reference,
        "molar": molar,
    }

    def port(name: str, qty: str, unit: str | None = None) -> dict:
        return {
            "symbol_id": sid("port." + name),
            "quantity_id": quantities[qty],
            "unit_id": unit or units[qty.removesuffix("_difference")],
        }

    provider = {
        "model_id": sid("model"),
        "name": "pressure",
        "kind": "feos-pcsaft-dippr",
        "material_system_id": None,
        "envelope": {
            "temperature": [250.0, 500.0],
            "density": [0.0, 25000.0],
            "pressure": [0.0, 1e8],
            "composition": [[0.0, 1.0]] * 3,
            "provenance": "declared acceptance operating window; empirical validity unestablished",
        },
        "output": 0,
        "enthalpy_reference": reference,
        "entropy_reference": entropy_reference,
        "components": [
            {"species_id": sid(name), "pcsaft_cas": cas, "ideal_gas_cas": cas}
            for name, cas in [
                ("methane", "74-82-8"),
                ("ethane", "74-84-0"),
                ("propane", "74-98-6"),
            ]
        ],
        "dependent_species": sid("propane"),
        "quantity_kinds": {
            role: kinds["neutral" if role == "ln_fugacity" else role]
            for role in [
                "temperature",
                "density",
                "fraction",
                "pressure",
                "enthalpy",
                "entropy",
                "ln_fugacity",
            ]
        },
        "data": {
            "pcsaft": (
                ROOT / "crates/pse-kernels/data/pcsaft-light-hydrocarbons.json"
            ).read_text(),
            "ideal_gas": (
                ROOT / "crates/pse-kernels/data/ideal-gas-light-hydrocarbons.json"
            ).read_text(),
            "binary": "[]",
            "provenance": "Bundled FeOS light-hydrocarbon example records; explicit zero binary interactions",
            "missing_interactions": "zero",
        },
        "formulation": "homogeneous_density",
        "stability": "unchecked",
        "inputs": [
            port("provider." + q, q)
            for q in ["temperature", "density", "fraction", "fraction"]
        ],
        "outputs": [
            port("provider.out." + str(i), q)
            for i, q in enumerate(
                ["pressure", "enthalpy", "entropy", "neutral", "neutral", "neutral"]
            )
        ],
    }
    provider["inputs"][3]["symbol_id"] = sid("provider.ethane")
    aliases: list[dict] = [
        dict(provider, name=n, output=i)
        for n, i in [
            ("pressure", 0),
            ("enthalpy", 1),
            ("phi_m", 3),
            ("phi_e", 4),
            ("phi_p", 5),
        ]
    ]
    refs = json.loads((DEST / "thermo-reference.json").read_text())["cases"]
    target = refs[1]
    temperature, density, xm, xe = target["input"]
    model = {
        "model_id": sid("model"),
        "name": "Native physical acceptance",
        "domains": [],
        "groups": [],
        "definitions": [],
        "cases": [],
    }

    def case(
        name: str,
        variables: list[tuple[str, str]],
        parameters: list[tuple[str, str]],
        expressions: list[tuple[str, str, str, float | None, float | None]],
        bounds: dict,
        starts: dict,
        objective: str | None = None,
    ) -> tuple[dict, dict]:
        ports = {
            n: port(name + "." + n, q, celsius if n == "temperature" else None)
            for n, q in variables + parameters
        }
        formals = [
            {"path": n, "quantity_id": p["quantity_id"]} for n, p in ports.items()
        ]
        slots = [
            {
                "source_id": p["symbol_id"],
                "formal_quantity_id": p["quantity_id"],
                "formal_unit_id": next(
                    t["canonical_unit_id"]
                    for t in d["quantity_types"]
                    if t["quantity_type_id"] == p["quantity_id"]
                ),
            }
            for p in ports.values()
        ]
        rows = []
        instances: list[dict] = []
        for n, expr, q, lower, upper in expressions:
            row = sid(name + ".row." + n)
            definition = sid(name + ".definition." + n)
            model["definitions"].append(
                {
                    "definition_id": definition,
                    "sources": [expr],
                    "formals": [dict(f) for f in formals],
                    "domains": [],
                    "groups": [],
                    "providers": [
                        p["name"] for p in aliases if p["name"] + "(" in expr
                    ],
                    "units": [{"spelling": "1", "unit_id": units["neutral"]}],
                    "literals": [],
                }
            )
            rows.append(
                {
                    "row_id": row,
                    "quantity_id": quantities[q],
                    "lower": lower,
                    "upper": upper,
                }
            )
            instances.append(
                {
                    "instance_id": sid(name + ".instance." + n),
                    "definition_id": definition,
                    "slots": [dict(s) for s in slots],
                    "contributions": [{"output": 0, "row_id": row, "scale": 1.0}],
                }
            )
        declaration: dict = {
            "case_id": sid("case." + name),
            "name": name,
            "variables": [
                {
                    "port": ports[n],
                    "fixed": False,
                    "domain": "continuous",
                    "lower": bounds[n][0],
                    "upper": bounds[n][1],
                }
                for n, _ in variables
            ],
            "parameters": [ports[n] for n, _ in parameters],
            "instances": instances,
            "rows": rows,
            "objective": None,
            "values": [
                {"symbol_id": ports[n]["symbol_id"], "value": v}
                for n, v in starts.items()
            ],
        }
        if objective:
            objective_id = sid(name + ".row." + objective)
            objective_row = next(r for r in rows if r["row_id"] == objective_id)
            declaration["objective"] = {
                "quantity_id": objective_row["quantity_id"],
                "sense": "minimize",
            }
            declaration["rows"] = [r for r in rows if r["row_id"] != objective_id]
            for instance in instances:
                for contribution in instance["contributions"]:
                    if contribution["row_id"] == objective_id:
                        contribution["row_id"] = None
        model["cases"].append(declaration)
        return declaration, ports

    free = [("temperature", "temperature"), ("density", "density"), ("recycle", "flow")]
    parameters = [
        ("methane", "fraction"),
        ("ethane", "fraction"),
        ("feed", "flow"),
        ("fraction", "neutral"),
    ]
    expr = [
        (
            "pressure",
            "kernel.pressure(temperature,density,methane,ethane)",
            "pressure",
            target["pressure"],
            target["pressure"],
        ),
        (
            "enthalpy",
            "kernel.enthalpy(temperature,density,methane,ethane)",
            "enthalpy",
            target["enthalpy"],
            target["enthalpy"],
        ),
        ("recycle", "recycle - fraction * recycle - feed", "flow", 0, 0),
    ]
    root, rootports = case(
        "heater-recycle",
        free,
        parameters,
        expr,
        {"temperature": (1.85, 226.85), "density": (1, 2000), "recycle": (0, 100)},
        {
            "temperature": 40.0,
            "density": 80.0,
            "recycle": 1.0,
            "methane": xm,
            "ethane": xe,
            "feed": 1.0,
            "fraction": 0.8,
        },
    )
    # Indexed inlet flow reduction shares the scalar body across concrete port bindings.
    inlet_members = [sid("inlet.1"), sid("inlet.2")]
    model["domains"].append(
        {
            "name": "inlets",
            "domain_id": sid("inlets"),
            "members": inlet_members,
            "kind": "port_set",
        }
    )
    root["parameters"].append(port("heater-recycle.feed_aux", "flow"))
    root["values"].append(
        {"symbol_id": sid("port.heater-recycle.feed_aux"), "value": 0.6}
    )
    next(v for v in root["values"] if v["symbol_id"] == rootports["feed"]["symbol_id"])[
        "value"
    ] = 0.4
    for definition in model["definitions"]:
        definition["formals"].append(
            {"path": "feed_aux", "quantity_id": quantities["flow"]}
        )
    for instance in root["instances"]:
        instance["slots"].append(
            {
                "source_id": sid("port.heater-recycle.feed_aux"),
                "formal_quantity_id": quantities["flow"],
                "formal_unit_id": units["flow"],
            }
        )
    model["groups"].append(
        {
            "name": "feeds",
            "quantity_id": shaped["quantity_type_id"],
            "axes": ["inlets"],
            "slots": [
                {"members": [member], "slot": slot}
                for member, slot in zip(inlet_members, [5, 7], strict=True)
            ],
        }
    )
    definition = next(
        d
        for d in model["definitions"]
        if d["definition_id"] == sid("heater-recycle.definition.recycle")
    )
    definition.update(
        sources=["recycle - fraction * recycle - sum(i in inlets | feeds[i])"],
        domains=["inlets"],
        groups=["feeds"],
    )
    opt_expr = [
        expr[0],
        ("temperature", "temperature", "temperature", temperature, None),
        (
            "duty",
            "kernel.enthalpy(temperature,density,methane,ethane)",
            "enthalpy",
            None,
            None,
        ),
    ]
    optimize, _ = case(
        "heater-optimization",
        free[:2],
        parameters[:2],
        opt_expr,
        {"temperature": (1.85, 226.85), "density": (1, 2000)},
        {"temperature": 90.0, "density": 40.0, "methane": xm, "ethane": xe},
        objective="duty",
    )
    # Prescribed split separator: species/total balances are independently analytic, no VLE claim.
    for i, z in enumerate([xm, xe, 1 - xm - xe]):
        case(
            "separator-" + str(i),
            [("product", "flow")],
            [("feed", "flow"), ("split", "neutral")],
            [("balance", "product - split * feed", "flow", 0, 0)],
            {"product": (0, 10)},
            {"product": 0.1, "feed": 5 * z, "split": 0.4},
        )
    roles = {
        "n": "amount",
        "u": "energy",
        "temperature": "temperature",
        "density": "density",
        "pressure": "pressure",
        "time": "time",
        "volume": "volume",
        "methane": "fraction",
        "ethane": "fraction",
        "inflow": "flow",
        "outflow": "flow",
        "inlet_enthalpy": "enthalpy",
        "heat": "power",
        "n0": "amount",
        "u0": "energy",
        "temperature0": "temperature",
        "density0": "density",
        "pressure0": "pressure",
        "valve_k": "valve",
        "downstream": "pressure",
        "valve_width": "pressure",
    }
    vessel_ports = {r: port("vessel." + r, q) for r, q in roles.items()}
    n0 = density * 0.1
    u0 = n0 * (target["enthalpy"] - target["pressure"] / density)
    initial = {
        "n": n0,
        "u": u0,
        "temperature": temperature,
        "density": density,
        "pressure": target["pressure"],
        "time": 0.0,
        "volume": 0.1,
        "methane": xm,
        "ethane": xe,
        "inflow": 0.0,
        "outflow": 0.0,
        "inlet_enthalpy": target["enthalpy"],
        "heat": 10.0,
        "n0": n0,
        "u0": u0,
        "temperature0": temperature,
        "density0": density,
        "pressure0": target["pressure"],
        "valve_k": 1e-5,
        "downstream": 50000.0,
        "valve_width": 10000.0,
    }
    flash = json.loads((DEST / "thermo-reference.json").read_text())["flash"]
    variables = [
        ("rl", "density"),
        ("rv", "density"),
        ("xm", "fraction"),
        ("xe", "fraction"),
        ("ym", "fraction"),
        ("ye", "fraction"),
        ("beta", "neutral"),
    ]
    expressions = [
        (
            "liquid_pressure",
            "kernel.pressure(temperature,rl,xm,xe)",
            "pressure",
            flash["pressure"],
            flash["pressure"],
        ),
        (
            "vapor_pressure",
            "kernel.pressure(temperature,rv,ym,ye)",
            "pressure",
            flash["pressure"],
            flash["pressure"],
        ),
    ]
    for species, left, right in [
        ("m", "xm", "ym"),
        ("e", "xe", "ye"),
        ("p", "one-xm-xe", "one-ym-ye"),
    ]:
        expressions.append(
            (
                "equilibrium_" + species,
                f"log({left}) + kernel.phi_{species}(temperature,rl,xm,xe) - log({right}) - kernel.phi_{species}(temperature,rv,ym,ye)",
                "neutral",
                0,
                0,
            )
        )
    expressions += [
        ("balance_m", "(1-beta)*xm+beta*ym-feed_m", "fraction", 0, 0),
        ("balance_e", "(1-beta)*xe+beta*ye-feed_e", "fraction", 0, 0),
    ]
    case(
        "flash",
        variables,
        [
            ("temperature", "temperature"),
            ("one", "fraction"),
            ("feed_m", "fraction"),
            ("feed_e", "fraction"),
        ],
        expressions,
        {
            "rl": (6000, 25000),
            "rv": (1, 5999),
            "xm": (0.001, 0.999),
            "xe": (0.001, 0.999),
            "ym": (0.001, 0.999),
            "ye": (0.001, 0.999),
            "beta": (0.001, 0.999),
        },
        {
            "rl": flash["liquid_density"] * 1.02,
            "rv": flash["vapor_density"] * 0.9,
            "xm": flash["liquid"][0] * 1.03,
            "xe": flash["liquid"][1] * 0.98,
            "ym": flash["vapor"][0] * 0.98,
            "ye": flash["vapor"][1] * 1.01,
            "beta": 0.4,
            "temperature": flash["temperature"] - 273.15,
            "one": 1.0,
            "feed_m": 0.2,
            "feed_e": 0.3,
        },
    )
    balances = []

    def balance(
        case_name: str,
        row_name: str,
        terms: list[tuple[str, str]],
        quantity: str | None = None,
    ) -> None:
        selected = next(c for c in model["cases"] if c["name"] == case_name)
        row_id = sid(case_name + ".row." + row_name)
        row = next(r for r in selected["rows"] if r["row_id"] == row_id)
        instance = next(
            i
            for i in selected["instances"]
            if any(c["row_id"] == row_id for c in i["contributions"])
        )
        definition = next(
            d
            for d in model["definitions"]
            if d["definition_id"] == instance["definition_id"]
        )
        definition["sources"] = [expression for _, expression in terms]
        selected["rows"].remove(row)
        instance["contributions"] = []
        balances.append(
            {
                "balance_id": row_id,
                "model_id": model["model_id"],
                "case_id": selected["case_id"],
                "quantity_id": quantities[quantity] if quantity else row["quantity_id"],
                "accumulation": None,
                "tolerance": 1e-5 if quantity == "power" else 1e-7,
                "integral_tolerance": None,
                "provenance": "declared physical fixture contributions; empirical validity unestablished",
                "terms": [
                    {
                        "source_id": sid(case_name + ".physical." + row_name + str(i)),
                        "role": role,
                        "multiplier": 1.0,
                        "transfer_id": None,
                        "mode": None,
                        "instance_id": instance["instance_id"],
                        "output": i,
                    }
                    for i, (role, _) in enumerate(terms)
                ],
                "impulses": [],
            }
        )

    balance(
        "heater-recycle",
        "recycle",
        [
            ("inlet", "fraction * recycle"),
            ("inlet", "sum(i in inlets | feeds[i])"),
            ("outlet", "recycle"),
        ],
    )
    # Add explicit heater energy ports in the same canonical caloric reference.
    for role, qty, value in [
        ("inlet_h", "enthalpy", 0.0),
        ("duty", "power", 5.0 * target["enthalpy"]),
    ]:
        physical_port = port("heater-recycle." + role, qty)
        root["parameters"].append(physical_port)
        root["values"].append({"symbol_id": physical_port["symbol_id"], "value": value})
        for instance in root["instances"]:
            definition = next(
                d
                for d in model["definitions"]
                if d["definition_id"] == instance["definition_id"]
            )
            definition["formals"].append({"path": role, "quantity_id": quantities[qty]})
            instance["slots"].append(
                {
                    "source_id": physical_port["symbol_id"],
                    "formal_quantity_id": quantities[qty],
                    "formal_unit_id": units[qty],
                }
            )
    balance(
        "heater-recycle",
        "enthalpy",
        [
            ("inlet", "recycle * inlet_h"),
            ("work_in", "duty"),
            ("outlet", "recycle * kernel.enthalpy(temperature,density,methane,ethane)"),
        ],
        "power",
    )
    for i in range(3):
        balance(
            "separator-" + str(i),
            "balance",
            [("inlet", "split * feed"), ("outlet", "product")],
        )
    for species in ["m", "e"]:
        balance(
            "flash",
            "balance_" + species,
            [
                ("outlet", "(1-beta)*x" + species),
                ("outlet", "beta*y" + species),
                ("inlet", "feed_" + species),
            ],
        )
    write("balances.json", balances)
    write("model.json", model)
    write("providers.json", aliases)
    # Public Python dynamics/fitting uses the same authored physical inventory.
    # The independent reference is n(t)=2+3*t and dn/dq=t.
    dynamic, dynamic_ports = case(
        "accumulator",
        [("n", "amount")],
        [("time", "time"), ("q", "flow"), ("n0", "amount")],
        [
            ("rate", "q", "flow", None, None),
            ("initial", "n0", "amount", None, None),
            ("output", "n", "amount", None, None),
        ],
        {"n": (None, None)},
        {"n": 2.0, "time": 0.0, "q": 3.0, "n0": 2.0},
    )
    definitions = {instance["definition_id"] for instance in dynamic["instances"]}
    write(
        "dynamic-model.json",
        {
            **model,
            "domains": [],
            "groups": [],
            "definitions": [
                definition
                for definition in model["definitions"]
                if definition["definition_id"] in definitions
            ],
            "cases": [dynamic],
        },
    )
    dynamic_id, fit_id, experiment_id, dataset_id, observation_id = [
        sid("accumulator." + name)
        for name in ("dynamic", "fit", "experiment", "dataset", "observation")
    ]
    write(
        "dynamic-source.json",
        {
            "dynamic_id": dynamic_id,
            "model_id": model["model_id"],
            "case_id": dynamic["case_id"],
            "time_id": dynamic_ports["time"]["symbol_id"],
            "time_origin": None,
            "states": [
                {
                    "symbol_id": dynamic_ports["n"]["symbol_id"],
                    "differential": True,
                    "initial_row": sid("accumulator.row.initial"),
                    "offset": 0.0,
                    "scale": 1.0,
                    "residual_scale": 1.0,
                }
            ],
            "parameters": [dynamic_ports["q"]["symbol_id"]],
            "outputs": [sid("accumulator.row.output")],
            "modes": [{"rhs_rows": [sid("accumulator.row.rate")], "events": []}],
        },
    )
    write(
        "fit-source.json",
        {
            "fit_id": fit_id,
            "model_id": model["model_id"],
            "parameters": [
                {
                    "symbol_id": dynamic_ports["q"]["symbol_id"],
                    "fixed": False,
                    "value": 1.0,
                    "lower": 0.0,
                    "upper": 10.0,
                    "scale": 1.0,
                }
            ],
            "experiments": [
                {
                    "experiment_id": experiment_id,
                    "case_id": dynamic["case_id"],
                    "dynamic_id": dynamic_id,
                }
            ],
            "observations": [
                {
                    "observation_id": observation_id,
                    "experiment_id": experiment_id,
                    "output_id": sid("accumulator.row.output"),
                    "time": 1.0,
                    "time_basis": None,
                    "time_unit_id": None,
                    "included": True,
                    "importance": 1.0,
                }
            ],
        },
    )
    write(
        "fit-observation.json",
        {
            "observation_id": observation_id,
            "dataset_id": dataset_id,
            "target": "accumulated amount",
            "value": 5.0,
            "unit_id": units["amount"],
            "std_dev": 1.0,
            "timestamp": None,
            "tag": None,
            "source_span": {"document_id": dataset_id, "start": 0, "end": 0},
        },
    )
    write(
        "fit-dataset.json",
        {
            "dataset_id": dataset_id,
            "name": "analytic accumulation",
            "source": "n(t)=2+3*t",
            "content_hash": "blake3:" + "03" * 32,
        },
    )
    write(
        "bindings.json",
        {
            "ids": ids,
            "root_ports": rootports,
            "root_case": root["case_id"],
            "optimization": optimize["case_id"],
            "vessel_id": sid("vessel"),
            "vessel_ports": vessel_ports,
            "vessel_values": initial,
            "provider": provider,
            "functions": {
                k: quantities[q]
                for k, q in {
                    "rate_n": "flow",
                    "rate_u": "power",
                    "closure_n": "amount",
                    "closure_u": "energy_difference",
                    "closure_p": "pressure",
                }.items()
            },
            "state_scales": [n0, abs(u0), temperature, density, target["pressure"]],
            "residual_scales": [n0, abs(u0), target["pressure"]],
            "expected": {
                "temperature": temperature,
                "density": density,
                "recycle": 5.0,
                "n0": n0,
                "u0": u0,
                "heat": 10.0,
            },
        },
    )


if __name__ == "__main__":
    main()
