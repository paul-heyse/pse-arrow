# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The fitted-correlation JSON files under `thermo/Misc/`.

Every file is `{key: {property: {model_key: {label: leaf}}}}`: the key is a CAS (for
`mixture_correlations.json` a sorted `'CAS1 CAS2'` pair), the property a class name such as
`VaporPressure`, the model key a parameter-set name ending in `_parameters` (or `tabular_data`),
the label a fit name such as `Fit 2023`, and the leaf an object of equation coefficients or, for
the tabular series, arrays.

| Table | Content |
|---|---|
| `correlation_leaves` | one row per leaf, identified by `key`, `property`, `model_key`, `label`; one column per coefficient name the leaves use, spelled as the file spells it (`A` to `G` and `a` to `d` are different columns), null where a leaf has no such coefficient |
| `correlation_series` | the nested arrays: a leaf that is itself a list of arrays (the tabular series of `Ho_1972...json`; `field` is null) and a `coeffs` value that is a list of lists (the Redlich-Kister series; `field` is `coeffs`), one row per inner array |

`coeffs` and `int_T_coeffs` are list columns when the leaf holds a flat array. The files state no
units; the equation form is named by the model key and the coefficients are positional, so the
mapping step must take both from the model key. One file's leaf can carry the same
`(key, property, model_key, label)` as another file's; the tables keep both, `_artifact` telling
them apart (the library merges files in a fixed order, which is not recorded here).
"""

from __future__ import annotations

from pathlib import Path

import pyarrow as pa

from thermo_knowledge.readers.chemicals import tabular
from thermo_knowledge.readers.chemicals.tabular import index, integer, number, numbers, text
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.writer import Writer

DIRECTORY = "thermo/Misc"
FILES = (
    "coolprop_correlations.json",
    "elements.json",
    "Ho_1972_thermal_conductivity_solid.json",
    "inorganic_correlations.json",
    "janaf_correlations.json",
    "mixture_correlations.json",
    "organic_correlations.json",
    "pycalphad_unary50.json",
    "refprop_correlations.json",
)
_SCALARS = (
    "Tmin", "Tmax", "Tc", "Pc", "A", "B", "C", "D", "E", "F", "G", "a", "b", "c", "d", "n0", "n1",
    "n2", "sigma0", "sigma1", "sigma2", "int_T_log_coeff",
)  # fmt: skip
_INTEGERS = ("N_T", "N_terms")
_ARRAYS = ("coeffs", "int_T_coeffs")
_KEYS = (*_SCALARS, *_INTEGERS, *_ARRAYS)

_IDENTITY = (
    text("key", "top-level key", note="a CAS, or a sorted 'CAS1 CAS2' pair in mixture_correlations.json"),
    text("property", "second-level key"),
    text("model_key", "third-level key"),
    text("label", "fourth-level key"),
)
_LEAVES = (
    *_IDENTITY,
    *(number(name) for name in _SCALARS),
    *(integer(name) for name in _INTEGERS),
    *(numbers(name, note="a flat array; a list of lists goes to correlation_series") for name in _ARRAYS),
)
_SERIES = (
    *_IDENTITY,
    text("field", "leaf key of the list of lists", note="null when the leaf itself is the list of lists"),
    index("series_index", "position of the inner array"),
    numbers("values", "inner array"),
)

SCHEMAS: dict[str, pa.Schema] = {
    "correlation_leaves": tabular.schema(*_LEAVES),
    "correlation_series": tabular.schema(*_SERIES),
}


def _is_nested(value: object) -> bool:
    return isinstance(value, list) and bool(value) and all(isinstance(item, list) for item in value)


def read_correlations(tree: Path, artifact: str, writer: Writer) -> None:
    leaves: list[dict[str, object]] = []
    series: list[dict[str, object]] = []
    document = tabular.as_object(tabular.load_json(tree, artifact), artifact)
    for key, properties in document.items():
        for prop, models in tabular.as_object(properties, f"{artifact}#{tabular.pointer(key)}").items():
            for model_key, labels in tabular.as_object(
                models, f"{artifact}#{tabular.pointer(key, prop)}"
            ).items():
                for label, leaf in tabular.as_object(
                    labels, f"{artifact}#{tabular.pointer(key, prop, model_key)}"
                ).items():
                    place = f"{artifact}#{tabular.pointer(key, prop, model_key, label)}"
                    identity: dict[str, object] = {
                        "key": key,
                        "property": prop,
                        "model_key": model_key,
                        "label": label,
                    }
                    row: dict[str, object] = {"_artifact": artifact, "_locator": place, **identity}
                    leaves.append(row)
                    if isinstance(leaf, list):
                        _series(artifact, place, identity, None, leaf, series)
                        continue
                    body = tabular.as_object(leaf, place)
                    tabular.only_keys(body, _KEYS, place)
                    for name in _SCALARS:
                        row[name] = tabular.json_float(body.get(name), f"{place}/{name}")
                    for name in _INTEGERS:
                        row[name] = tabular.json_int(body.get(name), f"{place}/{name}")
                    for name in _ARRAYS:
                        value = body.get(name)
                        if _is_nested(value):
                            _series(artifact, f"{place}/{name}", identity, name, value, series)
                        elif value is not None:
                            row[name] = tabular.json_floats(value, f"{place}/{name}")
    writer.rows("correlation_leaves", leaves)
    writer.rows("correlation_series", series)
    writer.opened(artifact)


def _series(
    artifact: str,
    place: str,
    identity: dict[str, object],
    field: str | None,
    value: object,
    out: list[dict[str, object]],
) -> None:
    if not _is_nested(value):
        raise StagingError(f"{place}: expected a list of arrays")
    assert isinstance(value, list)
    for position, inner in enumerate(value):
        out.append(
            {
                "_artifact": artifact,
                "_locator": f"{place}/{position}",
                **identity,
                "field": field,
                "series_index": position,
                "values": tabular.json_floats(inner, f"{place}/{position}"),
            }
        )


HANDLERS: dict[str, tabular.Handler] = {
    f"{DIRECTORY}/{name}": read_correlations for name in FILES
}
