# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers for the qualification tests: the fixture declaration (the committed model and forms
plus `fixtures/qualify/forms`), a canonical source written through the writer and built into a
disposable database, and case files. Contains no tests."""

from __future__ import annotations

import shutil
import uuid
from dataclasses import dataclass
from pathlib import Path

from build_support import fingerprint, inputs_of, write_source
from mapping_support import carrier, origin

from thermo_knowledge import config, identity
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.canonical.writer import CanonicalWriter, FamilyRow, NestedSet
from thermo_knowledge.declaration import Declaration, load_declaration

FIXTURES = Path(__file__).parent / "fixtures" / "qualify"
SATURATION = "vapor_pressure_exp_series_tau.pure"
PIN = "0123456789ab"
"""The resolved pin of the fixture carrier `src` (see `mapping_support.carrier`)."""

# What the fake harness (fixtures/qualify/oracles/fake.py) holds for each fluid.
FLUIDS = {
    "sp-a": {
        "T_r": 400.0,
        "p_r": 5.0e6,
        "n": [-7.5, 1.6, -3.2],
        "t": [1.0, 1.5, 3.0],
        "range": (250.0, 399.0),
    },
    "sp-b": {
        "T_r": 520.0,
        "p_r": 3.2e6,
        "n": [-8.1, 2.4, -2.7],
        "t": [1.0, 1.8, 3.5],
        "range": (300.0, 519.0),
    },
    "sp-c": {
        "T_r": 310.0,
        "p_r": 4.1e6,
        "n": [-6.9, 1.2, -4.4],
        "t": [1.0, 1.4, 2.6],
        "range": (200.0, 309.0),
    },
}
QFIX_A, QFIX_B = "sp-a", "sp-b"


def fixture_declaration(destination: Path) -> Declaration:
    """The committed declaration plus the forms of `fixtures/qualify/forms`."""
    shutil.copytree(config.TREE_DIR / "model", destination / "model")
    shutil.copytree(config.TREE_DIR / "forms", destination / "forms")
    for form in (FIXTURES / "forms").glob("*.toml"):
        shutil.copy(form, destination / "forms" / form.name)
    return load_declaration(destination / "model", destination / "forms").require()


@dataclass
class World:
    """What `build_world` wrote: identifiers of the things the tests refer to."""

    decl: Declaration
    canonical: Path
    species: dict[str, uuid.UUID]
    sets: dict[str, uuid.UUID]
    """The saturation set of each fluid."""
    repeated: dict[str, uuid.UUID]
    """The sets of the parameterization `sat_twice`, by `<fluid>#<occurrence>`."""


def species_key(name: str) -> str:
    return f"K-{name}"


def write_world(
    w: CanonicalWriter, decl: Declaration, world: dict[str, dict[str, uuid.UUID]]
) -> None:
    carrier_info = carrier("src", "a.json", "b.json")
    carrier_id = identity.identifier("source", [carrier_info.key])
    species: dict[str, uuid.UUID] = {}
    for index, name in enumerate(FLUIDS):
        locator = f"a.json#/{name}"
        species[name] = w.kind(
            "species",
            {"canonical_key": species_key(name), "label": name},
            origins=[origin(locator)],
        )
        w.kind(
            "source_entity",
            {
                "carrier": carrier_id,
                "scope": "items",
                "local_key": name,
                "status": "unique",
                "rule": "structural",
                "target": species[name],
            },
            origins=[origin(locator)],
        )
    world["species"] = species
    published = origin("b.json#/0", "published")
    sat = w.kind(
        "parameterization",
        {"key": "sat", "revision": PIN, "title": "Saturation", "coherence": "independent_records"},
        origins=[published],
    )
    temperature = decl.observable_entity("temperature")
    assert temperature is not None
    sets: dict[str, uuid.UUID] = {}
    for name, fluid in FLUIDS.items():
        sets[name] = w.parameter_set(
            parameterization=sat,
            slot_group=SATURATION,
            subjects=[species[name]],
            slots={"T_r": Quantity(fluid["T_r"], "K"), "p_r": Quantity(fluid["p_r"], "Pa")},  # type: ignore[arg-type]
            families={
                "term": [
                    FamilyRow({"k": k}, {"n": n, "t": t})
                    for k, (n, t) in enumerate(zip(fluid["n"], fluid["t"]), 1)  # type: ignore[arg-type]
                ]
            },
            origins=[published],
        )
        low, high = fluid["range"]  # type: ignore[misc]
        w.kind(
            "envelope",
            {
                "parameter_set": sets[name],
                "axis": temperature.id,
                "kind": "fitted_range",
                "lower": Quantity(low, "K"),
                "upper": Quantity(high, "K"),
            },
        )
    world["sets"] = sets
    # A parameterization that asserts the saturation set of sp-a twice: occurrence 1 as `sat` has
    # it, occurrence 2 with another reducing pressure. sp-b is asserted once.
    twice = w.kind(
        "parameterization",
        {
            "key": "sat_twice",
            "revision": PIN,
            "title": "Saturation, repeated",
            "coherence": "independent_records",
        },
        origins=[published],
    )
    repeated: dict[str, uuid.UUID] = {}
    for name, occurrence, scale in (("sp-a", 1, 1.0), ("sp-a", 2, 1.01), ("sp-b", 1, 1.0)):
        fluid = FLUIDS[name]
        identifier = w.parameter_set(
            parameterization=twice,
            slot_group=SATURATION,
            subjects=[species[name]],
            slots={"T_r": Quantity(fluid["T_r"], "K"), "p_r": Quantity(fluid["p_r"] * scale, "Pa")},  # type: ignore[operator]
            families={
                "term": [
                    FamilyRow({"k": k}, {"n": n, "t": t})
                    for k, (n, t) in enumerate(zip(fluid["n"], fluid["t"]), 1)  # type: ignore[arg-type]
                ]
            },
            origins=[published],
            occurrence=occurrence,
        )
        repeated[f"{name}#{occurrence}"] = identifier
        low, high = fluid["range"]  # type: ignore[misc]
        w.kind(
            "envelope",
            {
                "parameter_set": identifier,
                "axis": temperature.id,
                "kind": "fitted_range",
                "lower": Quantity(low, "K"),
                "upper": Quantity(high, "K"),
            },
        )
    world["repeated"] = repeated
    # A pair form: pure weights and terms, a symmetric pair with a nested function, a shift.
    qfix = w.kind(
        "parameterization",
        {"key": "qfix", "revision": PIN, "title": "Fixture", "coherence": "independent_records"},
        origins=[published],
    )
    shift = w.kind(
        "parameterization",
        {"key": "shift", "revision": "1", "title": "Shift", "coherence": "independent_records"},
        origins=[published],
    )
    a, b = species[QFIX_A], species[QFIX_B]
    for position, species_id in enumerate((a, b), 1):
        w.parameter_set(
            parameterization=qfix,
            slot_group="qfix_pair_form.pure",
            subjects=[species_id],
            slots={"w": 0.5 * position},
            families={
                "term": [
                    FamilyRow({"n": n}, {"c": 0.1 * n * position, "e": 0.5 * n}) for n in (1, 2)
                ]
            },
            origins=[published],
        )
    w.parameter_set(
        parameterization=qfix,
        slot_group="qfix_pair_form.pair",
        subjects=[a, b],
        slots={"k": 1.75, "f": NestedSet("qfix_linear.core", {"a": 0.25, "b": 0.002})},
        origins=[published],
    )
    w.parameter_set(
        parameterization=shift,
        slot_group="qfix_linear.core",
        subjects=[],
        slots={"a": 10.0, "b": -0.01},
        origins=[published],
    )
    # A stateful group: a constant that is known, withheld, or taken from another set.
    base = w.kind(
        "parameterization",
        {"key": "base", "revision": "1", "title": "Base", "coherence": "independent_records"},
        origins=[published],
    )
    derived = {}
    for key, title in (("redirect", "Redirecting"), ("withheld", "Withholding")):
        derived[key] = w.kind(
            "parameterization",
            {"key": key, "revision": "1", "title": title, "coherence": "independent_records"},
            origins=[published],
        )
    from thermo_knowledge.canonical.values import Redirect, Withheld

    known = w.parameter_set(
        parameterization=base,
        slot_group="qfix_stateful.core",
        subjects=[],
        slots={"a": 2.0, "b": 3.0},
        origins=[published],
    )
    w.parameter_set(
        parameterization=derived["redirect"],
        slot_group="qfix_stateful.core",
        subjects=[],
        slots={"a": Redirect(known), "b": 5.0},
        origins=[published],
    )
    w.parameter_set(
        parameterization=derived["withheld"],
        slot_group="qfix_stateful.core",
        subjects=[],
        slots={"a": Withheld(), "b": 7.0},
        origins=[published],
    )


def build_world(canonical: Path, decl: Declaration, database_url: str) -> World:
    """Write the fixture source `src` under `canonical` and build it into the database."""
    made: dict[str, dict[str, uuid.UUID]] = {}
    write_source(
        canonical,
        "src",
        lambda w: write_world(w, decl, made),
        decl=decl,
        declaration=fingerprint(decl),
    )
    build_database(database_url, decl, inputs_of(canonical))
    return World(decl, canonical, made["species"], made["sets"], made["repeated"])


def case_text(
    *,
    tolerance: str = "1e-10",
    extra_comparison: str = "",
    call: str = "ok",
    subjects: str = 'select = "all"',
    argument: str = 'points = 5\nwithin = "envelope"\nobservable = "temperature"\ninset = 0.0',
    revision: str = "{pin}",
    library: str = "fake",
    key: str = "sat",
    occurrence: int | None = None,
) -> str:
    """A case of the saturation form against the fake harness."""
    carrier_line = 'carrier = "src"\n' if "{pin}" in revision else ""
    carrier_line += f"occurrence = {occurrence}\n" if occurrence is not None else ""
    return f"""
doc = "A fixture case."
form = "vapor_pressure_exp_series_tau"
output = "p_sat"
basis = "source_library"

[parameterization]
key = "{key}"
revision = "{revision}"
{carrier_line}
[subjects]
group = "{SATURATION}"
{subjects}

[subjects.library_key]
carrier = "src"
scope = "items"

[arguments.T]
{argument}

[comparison]
relative_tolerance = {tolerance}
{extra_comparison}

[harness]
library = "{library}"
environment = "core"
call = "{call}"
"""
