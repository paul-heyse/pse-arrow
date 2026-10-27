# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit offline HTML/scientific-plot view of an original native comparison.

Scientific fields and interpretation belong to the original generated Rust schema.
No original source reference is followed, no service is contacted and no result
is promoted to an execution, statistical or causal qualification.
"""

from __future__ import annotations

import argparse
import hashlib
import html
import json
import math
import os
import stat
from pathlib import Path
from typing import TYPE_CHECKING
from urllib.parse import quote

from scripts.capabilities.native_schema import (
    JSON_OBJECT,
    SchemaSet,
    object_value,
    text_value,
)

if TYPE_CHECKING:
    from collections.abc import Sequence

    from pydantic import JsonValue


ROOT = Path(__file__).resolve().parents[1]


def _rows(value: JsonValue) -> list[dict[str, JsonValue]]:
    if not isinstance(value, list):
        raise TypeError("native owner rows must be an array")
    return [object_value(item) for item in value]


def _optional(value: JsonValue) -> dict[str, JsonValue] | None:
    return None if value is None else object_value(value)


def _strings(value: JsonValue) -> list[str]:
    if not isinstance(value, list):
        raise TypeError("native owner fields must be an array")
    return [text_value(item) for item in value]


MAX_COMPARISON_BYTES = 16 << 20
MAX_PLOT_SAMPLES = 200
MAX_PLOT_GROUPS = 12


def _cell(value: object) -> str:
    return html.escape("unknown" if value is None else str(value))


def _table(headers: list[str], rows: Sequence[Sequence[object]]) -> str:
    head = "".join("<th>" + _cell(value) + "</th>" for value in headers)
    body = "".join(
        "<tr>" + "".join("<td>" + _cell(value) + "</td>" for value in row) + "</tr>"
        for row in rows
    )
    return (
        "<table><thead><tr>" + head + "</tr></thead><tbody>" + body + "</tbody></table>"
    )


def _summary(comparison: dict[str, JsonValue]) -> list[list[object]]:
    names = (
        "paired",
        "resolved_failures",
        "new_failures",
        "persistent_failures",
        "unchanged_successes",
        "unexecuted",
        "inconclusive",
        "unmatched_baseline",
        "unmatched_candidate",
        "ambiguous",
        "mixed_outcome_cohorts",
        "changed_selection_denominator",
        "selection_membership_changed_or_unresolved",
    )
    return [[name, object_value(comparison["summary"])[name]] for name in names]


def _plots(comparison: dict[str, JsonValue], destination: Path) -> list[str]:
    # The optional investigation dependency stays outside the scientific package.
    from matplotlib.backends.backend_agg import FigureCanvasAgg  # noqa: PLC0415
    from matplotlib.figure import Figure  # noqa: PLC0415

    figure = Figure(figsize=(10, 6), layout="constrained")
    FigureCanvasAgg(figure)
    axes = figure.subplots()
    selected = _summary(comparison)[:11]
    labels = [str(row[0]).replace("_", " ") for row in selected]
    values = []
    for row in selected:
        value = row[1]
        if not isinstance(value, int):
            raise TypeError("native summary count is not an integer")
        values.append(value)
    axes.barh(labels, values, color="#326b91")
    axes.invert_yaxis()
    axes.set_xlabel("Owner cohort counts; axes can overlap")
    axes.set_title("Selected native comparison: full semantic counts")
    for index, value in enumerate(values):
        axes.annotate(
            str(value),
            (value, index),
            xytext=(4, 0),
            textcoords="offset points",
            va="center",
        )
    axes.margins(x=0.15)
    figure.suptitle("Cut " + text_value(comparison["cut"])[:24] + "…", fontsize=10)
    outputs = []
    for extension in ("svg", "png"):
        name = "outcomes." + extension
        figure.savefig(destination / name, dpi=150)
        outputs.append(name)
    figure.clear()
    if comparison["samples"] is not None:
        groups: dict[tuple[str, str, str], list[tuple[str, float]]] = {}
        for item in _rows(object_value(comparison["samples"])["rows"]):
            fact = object_value(item["fact"])
            sample_value = fact.get("value_ns")
            if not isinstance(sample_value, (int, float)) or not math.isfinite(
                sample_value
            ):
                continue
            key = (
                text_value(item["side"]),
                text_value(item["generation"]),
                text_value(fact.get("population_id") or "unknown population"),
            )
            groups.setdefault(key, []).append(
                (
                    text_value(fact.get("sample_id") or fact["member"]),
                    float(sample_value),
                )
            )
        if groups:
            selected_groups = list(groups.items())[:MAX_PLOT_GROUPS]
            figure = Figure(
                figsize=(10, max(3.5, len(selected_groups) * 0.7)), layout="constrained"
            )
            FigureCanvasAgg(figure)
            axes = figure.subplots()
            labels = []
            for index, ((side, generation, population), rows) in enumerate(
                selected_groups
            ):
                selected_rows = rows[:MAX_PLOT_SAMPLES]
                axes.scatter(
                    [value for _identity, value in selected_rows],
                    [index] * len(selected_rows),
                    alpha=0.6,
                    label=side if index < 2 else None,
                )
                labels.append(
                    f"{side}: {population}\n{generation[:14]}… ({len(selected_rows)}/{len(rows)} selected-page samples)"
                )
            axes.set_yticks(list(range(len(labels))), labels)
            axes.set_xlabel(
                "Original sample value (ns); no pairing or temporal order inferred"
            )
            axes.set_title(
                "Descriptive samples only; measurement verdict: "
                + text_value(object_value(comparison["measurement_summary"])["verdict"])
            )
            figure.suptitle(
                f"{len(selected_groups)}/{len(groups)} selected-page source/population groups; original conditions remain in the report",
                fontsize=10,
            )
            for extension in ("svg", "png"):
                name = "samples." + extension
                figure.savefig(destination / name, dpi=150)
                outputs.append(name)
            figure.clear()
    return outputs


def render(
    comparison: dict[str, JsonValue],
    *,
    original_link: str,
    original_sha256: str,
    plots: list[str],
) -> str:
    """Render generated owner fields; neither reinterpret nor hydrate missing pages."""
    sections = (
        "subjects",
        "members",
        "samples",
        "measurements",
        "contexts",
        "associations",
    )
    pages: list[list[object]] = []
    for name in sections:
        page = _optional(comparison[name])
        if page is None:
            pages.append([name, "not requested", "unknown", "unknown", "unknown"])
        else:
            if page["cut"] != comparison["cut"]:
                raise ValueError(
                    "selected native section belongs to another comparison cut"
                )
            pages.append(
                [
                    name,
                    f"{len(_rows(page['rows']))}/{page['total_rows']} rows",
                    page["offset"],
                    page["complete"],
                    page["next_offset"]
                    if page["next_offset"] is not None
                    else "none (end of section)",
                ]
            )
    source_rows = [
        [
            side,
            item["generation"],
            item["authority"],
            item["codec"],
            item["source_id"],
            item["snapshot_digest"],
            item["coverage"],
            item["expected_members"],
            item["freshness"],
        ]
        for side, values in (
            ("baseline", _rows(comparison["baseline_sources"])),
            ("candidate", _rows(comparison["candidate_sources"])),
        )
        for item in values
    ]
    denominator_names = (
        "selected_tests",
        "selected_members",
        "failures",
        "successes",
        "unexecuted",
        "other_outcomes",
        "samples",
        "declared_crate_tests",
        "declared_context_tests",
    )
    denominators = [
        [
            name,
            object_value(comparison["baseline_denominator"])[name],
            object_value(comparison["candidate_denominator"])[name],
        ]
        for name in denominator_names
    ]
    fragments = [
        "<h1>Evidence comparison</h1>",
        "<p class='notice'>Descriptive owner data. Missing execution, changed selection and unavailable association remain explicit. This view establishes no statistical, causal or completion verdict.</p>",
        "<p>Exact comparison cut: <code>" + _cell(comparison["cut"]) + "</code></p>",
        "<p><a href='"
        + html.escape(original_link, quote=True)
        + "'>Open original native comparison JSON</a>. SHA256: <code>"
        + _cell(original_sha256)
        + "</code>. Follow source identities through their original protected owner; references are not permission to open raw detail.</p>",
        "<p>" + _cell(comparison["interpretation"]) + "</p>",
        "<h2>Full selected denominators</h2>",
        _table(["Meaning", "Baseline", "Candidate"], denominators),
        "<h2>Subject comparison</h2><p>Counts are independent axes and can overlap; detail page limits do not reduce these denominators.</p>",
        _table(["Owner category", "Count / flag"], _summary(comparison)),
        "<h2>Original source cut</h2>",
        _table(
            [
                "Side",
                "Generation",
                "Authority",
                "Codec",
                "Source",
                "Snapshot digest",
                "Coverage",
                "Expected members",
                "Freshness",
            ],
            source_rows,
        ),
        "<h2>Detail completeness</h2>",
        _table(
            [
                "Section",
                "Selected / full rows",
                "Offset",
                "Whole section",
                "Next offset",
            ],
            pages,
        ),
    ]
    if comparison["subjects"] is not None:
        rows = []
        for item in _rows(object_value(comparison["subjects"])["rows"]):
            rows.append(
                [
                    item["cohort"],
                    item["classification"],
                    item["correspondence"],
                    object_value(item["baseline"])["binary_id"]
                    if item["baseline"]
                    else None,
                    object_value(item["baseline"])["test_id"]
                    if item["baseline"]
                    else None,
                    object_value(item["candidate"])["binary_id"]
                    if item["candidate"]
                    else None,
                    object_value(item["candidate"])["test_id"]
                    if item["candidate"]
                    else None,
                    f"failure={object_value(item['baseline_outcomes'])['failures']}; success={object_value(item['baseline_outcomes'])['successes']}; unexecuted={object_value(item['baseline_outcomes'])['unexecuted']}",
                    f"failure={object_value(item['candidate_outcomes'])['failures']}; success={object_value(item['candidate_outcomes'])['successes']}; unexecuted={object_value(item['candidate_outcomes'])['unexecuted']}",
                    item["mixed_outcomes"],
                ]
            )
        fragments.extend(
            (
                "<h2>Selected original subjects</h2>",
                _table(
                    [
                        "Cohort",
                        "Classification",
                        "Correspondence",
                        "Baseline binary",
                        "Baseline test",
                        "Candidate binary",
                        "Candidate test",
                        "Baseline evidence",
                        "Candidate evidence",
                        "Mixed",
                    ],
                    rows,
                ),
            )
        )
    if comparison["members"] is not None:
        fragments.extend(
            (
                "<h2>Selected original members</h2>",
                _table(
                    [
                        "Side",
                        "Generation",
                        "Member",
                        "Binary",
                        "Test",
                        "Status",
                        "Context",
                    ],
                    [
                        [
                            item["side"],
                            object_value(item["original"])["generation"],
                            object_value(item["original"])["member"],
                            object_value(item["identity"])["binary_id"],
                            object_value(item["identity"])["test_id"],
                            item["status"],
                            item["context_id"],
                        ]
                        for item in _rows(object_value(comparison["members"])["rows"])
                    ],
                ),
            )
        )
    measurement = object_value(comparison["measurement_summary"])
    fragments.extend(
        (
            "<h2>Measurement interpretation</h2>",
            "<p>Native verdict: <strong>"
            + _cell(measurement["verdict"])
            + "</strong>. Comparable conditions do not establish sample pairing, invocation association, regression or causality.</p>",
            _table(
                [
                    "Populations",
                    "Comparable",
                    "Incompatible",
                    "Unavailable",
                    "Unavailable association",
                ],
                [
                    [
                        measurement["populations"],
                        measurement["comparable"],
                        measurement["incompatible"],
                        measurement["unavailable"],
                        measurement["unavailable_association_populations"],
                    ]
                ],
            ),
        )
    )
    if comparison["measurements"] is not None:
        fragments.append(
            _table(
                [
                    "Correspondence",
                    "Conditions",
                    "Changed fields",
                    "Unknown fields",
                    "Association",
                    "Verdict",
                    "Revision/build interpretation",
                ],
                [
                    [
                        item["correspondence"],
                        item["conditions"],
                        ", ".join(_strings(item["changed_fields"])),
                        ", ".join(_strings(item["unknown_fields"])),
                        item["association"],
                        item["verdict"],
                        item["revision_build"],
                    ]
                    for item in _rows(object_value(comparison["measurements"])["rows"])
                ],
            )
        )
        descriptive = []
        for item in _rows(object_value(comparison["measurements"])["rows"]):
            for side, cohort_value in (
                ("baseline", item["baseline"]),
                ("candidate", item["candidate"]),
            ):
                cohort = _optional(cohort_value)
                if cohort is not None:
                    descriptive.append(
                        [
                            item["correspondence"],
                            side,
                            cohort["generation"],
                            cohort["population_id"],
                            cohort["sample_count"],
                            cohort["min_ns"],
                            cohort["mean_ns"],
                            cohort["max_ns"],
                            cohort["censored_populations"],
                            cohort["unavailable_associations"],
                        ]
                    )
        fragments.extend(
            (
                "<p>Original descriptive values below do not establish a measurement change verdict.</p>",
                _table(
                    [
                        "Correspondence",
                        "Side",
                        "Generation",
                        "Population",
                        "Samples",
                        "Min ns",
                        "Mean ns",
                        "Max ns",
                        "Censored populations",
                        "Unavailable associations",
                    ],
                    descriptive,
                ),
            )
        )
    if comparison["samples"] is not None:
        fragments.extend(
            (
                "<h2>Selected original samples</h2><p>Rows preserve source/sample IDs; this table does not infer temporal order or paired samples.</p>",
                _table(
                    [
                        "Side",
                        "Generation",
                        "Population",
                        "Sample",
                        "Member",
                        "Context",
                        "Value ns",
                    ],
                    [
                        [
                            item["side"],
                            item["generation"],
                            object_value(item["fact"]).get("population_id"),
                            object_value(item["fact"]).get("sample_id"),
                            object_value(item["fact"]).get("member"),
                            object_value(item["fact"]).get("context_id"),
                            object_value(item["fact"]).get("value_ns"),
                        ]
                        for item in _rows(object_value(comparison["samples"])["rows"])
                    ],
                ),
            )
        )
    for name in plots:
        if name.endswith(".svg"):
            fragments.append(
                "<figure><img src='"
                + _cell(name)
                + "' alt='Descriptive native evidence plot'><figcaption>Standalone SVG and PNG are available. Sample plots show at most "
                + str(MAX_PLOT_GROUPS)
                + " source/population groups and "
                + str(MAX_PLOT_SAMPLES)
                + " samples per group from this selected page; the original table and denominators remain authoritative.</figcaption></figure>"
            )
    if not plots:
        fragments.append(
            "<p>Plots were not selected. Existing typed owner data and this report remain usable without Matplotlib.</p>"
        )
    return (
        "<!doctype html><html lang='en'><meta charset='utf-8'><meta name='viewport' content='width=device-width, initial-scale=1'><title>Native evidence comparison</title><style>body{font:16px/1.45 system-ui,sans-serif;max-width:1500px;margin:2em auto;padding:0 1em;color:#20333e}table{border-collapse:collapse;width:100%;font-size:0.85em;margin:1em 0}th,td{border:1px solid #ccd6db;text-align:left;padding:0.45em;overflow-wrap:anywhere}th{background:#e7eff4}.notice{background:#fff1cf;padding:1em;border-left:5px solid #996a16}code{overflow-wrap:anywhere}img{width:100%;max-width:1100px}figure{margin:1.5em 0}</style><main>"
        + "".join(fragments)
        + "</main></html>"
    )


def write_report(
    source: Path, destination: Path, *, plots: bool = True
) -> dict[str, JsonValue]:
    from jsonschema import Draft202012Validator, ValidationError  # noqa: PLC0415
    from referencing import (  # noqa: PLC0415 -- selected owner loads only at this boundary
        Registry,
    )
    from referencing.exceptions import Unresolvable  # noqa: PLC0415

    metadata = source.lstat()
    if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > MAX_COMPARISON_BYTES:
        raise ValueError("select one bounded original comparison JSON file")
    raw = source.read_bytes()
    if len(raw) > MAX_COMPARISON_BYTES:
        raise ValueError("original comparison exceeds its selected bound")
    original_digest = hashlib.sha256(raw).hexdigest()
    comparison = JSON_OBJECT.validate_json(raw, strict=True)
    schemas = SchemaSet(ROOT, family="evidence")
    schema = schemas.document("evidence-comparison", "output")
    try:
        Draft202012Validator(schema, registry=Registry()).validate(comparison)
    except (ValidationError, Unresolvable):
        raise ValueError(
            "Selected original does not satisfy its available native output schema"
        ) from None
    # Validate page/cut correspondence before publishing any artifact.
    render(comparison, original_link="", original_sha256=original_digest, plots=[])
    destination.mkdir(mode=0o700, parents=True, exist_ok=False)
    names = _plots(comparison, destination) if plots else []
    relative = quote(
        os.path.relpath(source.absolute(), destination.absolute()), safe="/"
    )
    body = render(
        comparison, original_link=relative, original_sha256=original_digest, plots=names
    )
    (destination / "comparison.html").write_text(body, encoding="utf-8")
    if hashlib.sha256(source.read_bytes()).hexdigest() != original_digest:
        raise ValueError(
            "original comparison changed during report rendering; published view is not current"
        )
    return {
        "comparison_cut": comparison["cut"],
        "original_source": str(source.absolute()),
        "original_sha256": original_digest,
        "native_schema": schemas.identity("evidence-comparison", "output"),
        "html": str(destination / "comparison.html"),
        "plots": [str(destination / name) for name in names],
        "interpretation": "Read-only view of the explicit original native answer; no source references followed or conclusions upgraded.",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--no-plots", action="store_true")
    args = parser.parse_args()
    print(
        json.dumps(
            write_report(args.source, args.destination, plots=not args.no_plots),
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
