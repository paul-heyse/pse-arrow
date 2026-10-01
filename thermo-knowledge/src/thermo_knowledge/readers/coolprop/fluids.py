# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Pure-fluid files `dev/fluids/*.json`: identifiers, equation-of-state entries with their
residual and ideal-gas terms, states, ancillaries, melting lines, surface tension and the
superancillary expansions. Transport is in `transport.py`.

Locators are the JSON pointer of the object within the file (`<file>#/EOS/0/alphar/3`); a
coefficient row adds `[row]`.
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.coolprop import transport
from thermo_knowledge.readers.coolprop.common import (
    NOT_APPLICABLE,
    Col,
    Fields,
    ObjectSpec,
    Sink,
    flag,
    index,
    integer,
    load_json,
    locator,
    merge_schemas,
    num,
    pointer,
    text,
    texts,
)
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import FLOAT64, table_schema

FLUID = text("fluid", source_name="INFO/NAME", nullable=False)
EOS_INDEX = index("eos_index", "position in the EOS list")
TERM_INDEX = index("term_index", "position in the alphar or alpha0 list")
TYPE = text("type", source_name="type")

# -- fluid identity ------------------------------------------------------------------------

INFO_COLUMNS: tuple[tuple[str, Col], ...] = (
    ("NAME", text("NAME", source_name="INFO/NAME", nullable=False)),
    ("CAS", text("CAS", source_name="INFO/CAS")),
    ("REFPROP_NAME", text("REFPROP_NAME", source_name="INFO/REFPROP_NAME")),
    ("ALIASES", texts("ALIASES", source_name="INFO/ALIASES")),
    ("FORMULA", text("FORMULA", source_name="INFO/FORMULA")),
    ("INCHI_STRING", text("INCHI_STRING", source_name="INFO/INCHI_STRING")),
    ("INCHI_KEY", text("INCHI_KEY", source_name="INFO/INCHI_KEY")),
    ("SMILES", text("SMILES", source_name="INFO/SMILES")),
    ("CHEMSPIDER_ID", integer("CHEMSPIDER_ID", source_name="INFO/CHEMSPIDER_ID")),
    ("2DPNG_URL", text("2DPNG_URL", source_name="INFO/2DPNG_URL")),
)
ENVIRONMENTAL_COLUMNS: tuple[tuple[str, Col], ...] = (
    ("ASHRAE34", text("ENVIRONMENTAL_ASHRAE34", source_name="INFO/ENVIRONMENTAL/ASHRAE34")),
    ("FH", integer("ENVIRONMENTAL_FH", source_name="INFO/ENVIRONMENTAL/FH")),
    ("GWP100", num("ENVIRONMENTAL_GWP100", source_name="INFO/ENVIRONMENTAL/GWP100")),
    ("GWP20", num("ENVIRONMENTAL_GWP20", source_name="INFO/ENVIRONMENTAL/GWP20")),
    ("GWP500", num("ENVIRONMENTAL_GWP500", source_name="INFO/ENVIRONMENTAL/GWP500")),
    ("HH", integer("ENVIRONMENTAL_HH", source_name="INFO/ENVIRONMENTAL/HH")),
    ("Name", text("ENVIRONMENTAL_Name", source_name="INFO/ENVIRONMENTAL/Name")),
    ("ODP", num("ENVIRONMENTAL_ODP", source_name="INFO/ENVIRONMENTAL/ODP")),
    ("PH", num("ENVIRONMENTAL_PH", source_name="INFO/ENVIRONMENTAL/PH")),
)

# -- equation-of-state entries ---------------------------------------------------------------

EOS_ENTRIES = ObjectSpec(
    "eos_entries",
    keys=(FLUID, EOS_INDEX),
    scalars=(
        text("BibTeX_CP0"),
        text("BibTeX_EOS"),
        num("T_max", "K", unit_from="T_max_units"),
        text("T_max_units"),
        num("Ttriple", "K", unit_from="Ttriple_units"),
        text("Ttriple_units"),
        num("acentric", "-", unit_from="acentric_units"),
        text("acentric_note"),
        text("acentric_units"),
        num("gas_constant", "J/mol/K", unit_from="gas_constant_units"),
        text("gas_constant_units"),
        num("molar_mass", "kg/mol", unit_from="molar_mass_units"),
        text("molar_mass_units"),
        num("p_max", "Pa", unit_from="p_max_units"),
        text("p_max_units"),
        flag("pseudo_pure"),
    ),
    lengths=False,
)

STATES = ObjectSpec(
    "states",
    keys=(
        FLUID,
        EOS_INDEX,
        text("scope", source_name="STATES (fluid) or EOS/STATES (eos)", nullable=False),
        text("state_name", source_name="key within STATES", nullable=False),
    ),
    scalars=(
        num("T", "K", unit_from="T_units"),
        text("T_units"),
        num("hmolar", "J/mol", unit_from="hmolar_units"),
        text("hmolar_units"),
        num("p", "Pa", unit_from="p_units"),
        text("p_units"),
        num("rhomolar", "mol/m^3", unit_from="rhomolar_units"),
        text("rhomolar_units"),
        num("smolar", "J/mol/K", unit_from="smolar_units"),
        text("smolar_units"),
    ),
    lengths=False,
)

_ALPHAR_ARRAYS = ("A", "B", "C", "D", "a", "b", "beta", "d", "epsilon", "eta", "g", "gamma")
_ALPHAR_ARRAYS += ("gd", "gt", "l", "ld", "lt", "m", "n", "t")

ALPHAR = ObjectSpec(
    "alphar_terms",
    keys=(FLUID, EOS_INDEX, TERM_INDEX, TYPE),
    scalars=(num("a"), num("epsilonbar"), num("kappabar"), num("m"), num("vbarn")),
    rows_table="alphar_term_rows",
    arrays=tuple(num(name) for name in _ALPHAR_ARRAYS),
    lift=("type",),
)


def alpha0_spec(scalars_table: str, rows_table: str, keys: tuple[Col, ...]) -> ObjectSpec:
    """The ideal-gas term objects, which the cubic fluid records share."""
    return ObjectSpec(
        scalars_table,
        keys=keys,
        scalars=(
            num("R"),
            num("T0"),
            num("Tc"),
            num("Tcrit", "K", unit_from="Tcrit_units"),
            text("Tcrit_units"),
            text("_note"),
            num("a"),
            num("a1"),
            num("a2"),
            num("cp_over_R"),
            text("reference"),
        ),
        rows_table=rows_table,
        arrays=tuple(num(name) for name in ("c", "d", "n", "t", "v")),
        lift=("type",),
    )


ALPHA0 = alpha0_spec("alpha0_terms", "alpha0_term_rows", (FLUID, EOS_INDEX, TERM_INDEX, TYPE))

CRITICAL_REGION_SPLINES = ObjectSpec(
    "critical_region_splines",
    keys=(FLUID, EOS_INDEX),
    scalars=(
        num("T_max"),
        num("T_min"),
        text("_note"),
        num("rhomolar_max"),
        num("rhomolar_min"),
    ),
    rows_table="critical_region_spline_rows",
    arrays=(num("cL"), num("cV")),
)

ANCILLARY = Col("ancillary", pa.string(), "key under ANCILLARIES", nullable=False)
ANCILLARY_EQUATIONS = ObjectSpec(
    "ancillary_equations",
    keys=(FLUID, ANCILLARY),
    scalars=(
        num("T_r"),
        num("Tmax"),
        num("Tmin"),
        text("description"),
        num("max_abserror_percentage"),
        num("reducing_value"),
        text("type"),
        flag("using_tau_r"),
        text("_note"),
        text("__note"),
        num("max_abs_error", "as stated in the max_abs_error_units column"),
        text("max_abs_error_units"),
    ),
    rows_table="ancillary_equation_rows",
    arrays=tuple(num(name) for name in ("A", "B", "n", "t")),
)

MELTING_LINES = ObjectSpec(
    "melting_lines",
    keys=(FLUID,),
    scalars=(text("BibTeX"), num("T_m"), text("_note"), text("type")),
    lengths=False,
)
MELTING_LINE_PARTS = ObjectSpec(
    "melting_line_parts",
    keys=(FLUID, index("part_index", "position in melting_line/parts")),
    scalars=(num("T_0"), num("T_max"), num("T_min"), num("a"), num("c"), num("p_0")),
    rows_table="melting_line_part_rows",
    arrays=(num("a"), num("t")),
)
SURFACE_TENSION = ObjectSpec(
    "surface_tension",
    keys=(FLUID,),
    scalars=(text("BibTeX"), num("Tc"), text("description")),
    rows_table="surface_tension_rows",
    arrays=(num("a"), num("n")),
)

_ANCILLARY_EQUATIONS = ("pS", "pL", "pV", "rhoL", "rhoV", "hL", "hLV", "sL", "sLV")

# -- superancillary --------------------------------------------------------------------------

_META = (
    ("BrhoL / mol/m^3", "meta_BrhoL", "mol/m^3"),
    ("BrhoV / mol/m^3", "meta_BrhoV", "mol/m^3"),
    ("Tcrit / K", "meta_Tcrit", "K"),
    ("Tcrittrue / K", "meta_Tcrittrue", "K"),
    ("Treducing / K", "meta_Treducing", "K"),
    ("Ttriple / K", "meta_Ttriple", "K"),
    ("gas_constant / J/mol/K", "meta_gas_constant", "J/mol/K"),
    ("rhocrittrue / mol/m^3", "meta_rhocrittrue", "mol/m^3"),
)
_CRIT_ANC = (
    ("Tc / K", "crit_anc_Tc", "K"),
    ("Theta_min", "crit_anc_Theta_min", None),
    ("rhoc / mol/m^3", "crit_anc_rhoc", "mol/m^3"),
)
_CHECK_POINTS = (
    ("T / K", "T", "K"),
    ("p(SA)/p(mp)", "p_SA_over_p_mp", None),
    ("p(mp) / Pa", "p_mp", "Pa"),
    ("rho''(SA)/rho''(mp)", "rho_dprime_SA_over_rho_dprime_mp", None),
    ("rho''(mp) / mol/m^3", "rho_dprime_mp", "mol/m^3"),
    ("rho'(SA)/rho'(mp)", "rho_prime_SA_over_rho_prime_mp", None),
    ("rho'(mp) / mol/m^3", "rho_prime_mp", "mol/m^3"),
)
_EXPANSIONS = ("jexpansions_p", "jexpansions_rhoL", "jexpansions_rhoV")


def _superancillary_columns() -> list[Col]:
    columns = [text("source_eos_hash", source_name="SUPERANCILLARY/source_eos_hash")]
    for source, name, unit in _META:
        columns.append(num(name, unit or "not stated", source_name=f"SUPERANCILLARY/meta/{source}"))
    for source, name, unit in _CRIT_ANC:
        columns.append(
            num(name, unit or "not stated", source_name=f"SUPERANCILLARY/crit_anc/{source}")
        )
    columns += [
        text("crit_anc_note", source_name="SUPERANCILLARY/crit_anc/_note"),
        Col("crit_anc_cL", pa.list_(FLOAT64), "SUPERANCILLARY/crit_anc/cL"),
        Col("crit_anc_cV", pa.list_(FLOAT64), "SUPERANCILLARY/crit_anc/cV"),
    ]
    return columns


SUPERANCILLARY_COLUMNS = _superancillary_columns()

SCHEMAS: dict[str, pa.Schema] = merge_schemas(
    {
        "fluids": table_schema(
            *(c.field() for _, c in INFO_COLUMNS),
            *(c.field() for _, c in ENVIRONMENTAL_COLUMNS),
        ),
        "superancillaries": table_schema(
            FLUID.field(), EOS_INDEX.field(), *(c.field() for c in SUPERANCILLARY_COLUMNS)
        ),
        "superancillary_check_points": table_schema(
            FLUID.field(),
            EOS_INDEX.field(),
            index("check_point_index", "position in SUPERANCILLARY/check_points").field(),
            *(
                num(
                    name, unit or "not stated", source_name=f"SUPERANCILLARY/check_points/{source}"
                ).field()
                for source, name, unit in _CHECK_POINTS
            ),
        ),
        "superancillary_expansions": table_schema(
            FLUID.field(),
            EOS_INDEX.field(),
            text("expansion", source_name="key under SUPERANCILLARY", nullable=False).field(),
            index("interval_index", "position in the expansion list").field(),
            num("x_min", source_name="xmin").field(),
            num("x_max", source_name="xmax").field(),
            Col("coef", pa.list_(FLOAT64), "coef").field(),
        ),
    },
    EOS_ENTRIES.schemas(),
    STATES.schemas(),
    ALPHAR.schemas(),
    ALPHA0.schemas(),
    CRITICAL_REGION_SPLINES.schemas(),
    ANCILLARY_EQUATIONS.schemas(),
    MELTING_LINES.schemas(),
    MELTING_LINE_PARTS.schemas(),
    SURFACE_TENSION.schemas(),
    transport.SCHEMAS,
)


def read_fluid(tree: Path, artifact: str, sink: Sink) -> None:
    """One fluid file. Every key of the document is consumed by a table or refused."""
    top = Fields(load_json(tree, artifact), artifact)
    info = top.take_object("INFO")
    eos_list = top.take_list("EOS")
    states = top.take_object("STATES")
    ancillaries = top.take_object("ANCILLARIES")
    transport_block = top.take_object("TRANSPORT")
    top.done()
    if info is None or eos_list is None or states is None or ancillaries is None:
        raise StagingError(f"{artifact}: INFO, EOS, STATES and ANCILLARIES are required")

    name = _info(sink, artifact, info)
    key: dict[str, object] = {"fluid": name}
    for state_name, state in states.items():
        sink.add_object(
            STATES,
            artifact,
            pointer("STATES", state_name),
            {**key, "eos_index": None, "scope": "fluid", "state_name": state_name},
            state,
        )
    for eos_index, entry in enumerate(eos_list):
        _eos(sink, artifact, key, eos_index, entry)
    _ancillaries(sink, artifact, key, ancillaries)
    if transport_block is not None:
        transport.read_transport(sink, artifact, name, transport_block)


def _info(sink: Sink, artifact: str, info: dict[str, object]) -> str:
    body = Fields(info, f"{artifact}#/INFO")
    row: dict[str, object] = {"_artifact": artifact, "_locator": locator(artifact, "/INFO")}
    for source, spec in INFO_COLUMNS:
        row[spec.name] = body.take(source)
    environmental = body.take_object("ENVIRONMENTAL")
    body.done()
    if environmental is not None:
        env = Fields(environmental, f"{artifact}#/INFO/ENVIRONMENTAL")
        for source, spec in ENVIRONMENTAL_COLUMNS:
            row[spec.name] = env.take(source)
        env.done()
    name = row["NAME"]
    if not isinstance(name, str):
        raise StagingError(f"{artifact}#/INFO: NAME is required")
    sink.add("fluids", row)
    return name


def _eos(sink: Sink, artifact: str, key: dict[str, object], eos_index: int, entry: object) -> None:
    base = pointer("EOS", eos_index)
    body = Fields(entry, locator(artifact, base))
    alphar = body.take_list("alphar")
    alpha0 = body.take_list("alpha0")
    states = body.take_object("STATES")
    splines = body.take_object("critical_region_splines")
    superancillary = body.take_object("SUPERANCILLARY")
    scalars = dict(body.remaining())
    sink.add_object(EOS_ENTRIES, artifact, base, {**key, "eos_index": eos_index}, scalars)
    eos_key = {**key, "eos_index": eos_index}
    for term_index, term in enumerate(alphar or []):
        sink.add_object(
            ALPHAR,
            artifact,
            pointer("EOS", eos_index, "alphar", term_index),
            {**eos_key, "term_index": term_index},
            term,
        )
    for term_index, term in enumerate(alpha0 or []):
        sink.add_object(
            ALPHA0,
            artifact,
            pointer("EOS", eos_index, "alpha0", term_index),
            {**eos_key, "term_index": term_index},
            term,
        )
    for state_name, state in (states or {}).items():
        sink.add_object(
            STATES,
            artifact,
            pointer("EOS", eos_index, "STATES", state_name),
            {**eos_key, "scope": "eos", "state_name": state_name},
            state,
        )
    if splines is not None:
        sink.add_object(
            CRITICAL_REGION_SPLINES,
            artifact,
            pointer("EOS", eos_index, "critical_region_splines"),
            eos_key,
            splines,
        )
    if superancillary is not None:
        _superancillary(sink, artifact, eos_key, eos_index, superancillary)


def _ancillaries(
    sink: Sink, artifact: str, key: dict[str, object], ancillaries: dict[str, object]
) -> None:
    body = Fields(ancillaries, locator(artifact, "/ANCILLARIES"))
    for name in _ANCILLARY_EQUATIONS:
        equation = body.take(name)
        if equation is not None:
            sink.add_object(
                ANCILLARY_EQUATIONS,
                artifact,
                pointer("ANCILLARIES", name),
                {**key, "ancillary": name},
                equation,
            )
    melting = body.take_object("melting_line")
    if melting is not None:
        parts = melting.get("parts")
        if not isinstance(parts, list):
            raise StagingError(f"{artifact}#/ANCILLARIES/melting_line: parts must be a list")
        head = {k: v for k, v in melting.items() if k != "parts"}
        sink.add_object(MELTING_LINES, artifact, pointer("ANCILLARIES", "melting_line"), key, head)
        for part_index, part in enumerate(parts):
            sink.add_object(
                MELTING_LINE_PARTS,
                artifact,
                pointer("ANCILLARIES", "melting_line", "parts", part_index),
                {**key, "part_index": part_index},
                part,
            )
    surface = body.take_object("surface_tension")
    if surface is not None:
        sink.add_object(
            SURFACE_TENSION, artifact, pointer("ANCILLARIES", "surface_tension"), key, surface
        )
    body.done()


def _superancillary(
    sink: Sink, artifact: str, eos_key: dict[str, object], eos_index: int, block: dict[str, object]
) -> None:
    base = pointer("EOS", eos_index, "SUPERANCILLARY")
    body = Fields(block, locator(artifact, base))
    row: dict[str, object] = {
        "_artifact": artifact,
        "_locator": locator(artifact, base),
        **eos_key,
        "source_eos_hash": body.take("source_eos_hash"),
    }
    meta = Fields(body.take_object("meta"), locator(artifact, f"{base}/meta"))
    for source, name, _ in _META:
        row[name] = meta.take(source)
    meta.done()
    crit = Fields(body.take_object("crit_anc"), locator(artifact, f"{base}/crit_anc"))
    for source, name, _ in _CRIT_ANC:
        row[name] = crit.take(source)
    row["crit_anc_note"] = crit.take("_note")
    row["crit_anc_cL"] = crit.take("cL")
    row["crit_anc_cV"] = crit.take("cV")
    crit.done()
    sink.add("superancillaries", row)
    for point_index, point in enumerate(body.take_list("check_points") or []):
        where = locator(artifact, f"{base}/check_points/{point_index}")
        fields = Fields(point, where)
        entry: dict[str, object] = {
            "_artifact": artifact,
            "_locator": where,
            **eos_key,
            "check_point_index": point_index,
        }
        for source, name, _ in _CHECK_POINTS:
            entry[name] = fields.take(source)
        fields.done()
        sink.add("superancillary_check_points", entry)
    for expansion in _EXPANSIONS:
        for interval_index, interval in enumerate(body.take_list(expansion) or []):
            where = locator(artifact, f"{base}/{expansion}/{interval_index}")
            fields = Fields(interval, where)
            sink.add(
                "superancillary_expansions",
                {
                    "_artifact": artifact,
                    "_locator": where,
                    **eos_key,
                    "expansion": expansion,
                    "interval_index": interval_index,
                    "x_min": fields.take("xmin"),
                    "x_max": fields.take("xmax"),
                    "coef": fields.take("coef"),
                },
            )
            fields.done()
    body.done()
