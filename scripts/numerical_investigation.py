# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Inspect actual diagnostic samples against Decimal references and plot legal steps."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from decimal import Decimal, localcontext
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def reference(case: str, point: float) -> tuple[Decimal, Decimal]:
    """Closed-form derivative, independent of finite-difference machinery."""
    with localcontext() as context:
        context.prec = 100
        x = Decimal(point)
        if case.startswith("log-"):
            return (1 + x).ln(), 1 / (1 + x)
        return x**3, 3 * x**2


def investigate(document: dict) -> dict:
    if document.get("schema") != "numerical-investigation-v1":
        raise ValueError("unsupported numerical sample schema")
    samples = document["samples"]
    if not samples:
        raise ValueError("no actual diagnostic samples")
    for sample in samples:
        value, derivative = reference(sample["case"], sample["point"])
        sample["reference"] = {
            "value": str(value),
            "derivative": str(derivative),
            "precision": 100,
            "input": "exact binary64",
            "authority": "local analytic formula; not a physical-model certificate",
        }
        scale = Decimal(sample["coordinate_scale"]) / Decimal(sample["row_scale"])
        with localcontext() as context:
            context.prec = 100
            expected = derivative * scale
            for comparison in sample["comparisons"]:
                if comparison["label"].startswith("jac_g"):
                    comparison["reference_normalized"] = str(expected)
                    comparison["sampling_error"] = float(
                        abs(Decimal(comparison["numeric"]) - expected)
                    )
                    comparison["derivative_error"] = float(
                        abs(Decimal(comparison["analytic"]) - expected)
                    )
    implicit = document.get("implicit_products")
    if implicit is not None:
        if implicit.get("schema") != "implicit-product-step-sweep-v1":
            raise ValueError("unsupported implicit-product sweep")
        with localcontext() as context:
            context.prec = 100
            expected = Decimal(13) / Decimal(12)
            for sample in implicit["samples"]:
                sample["reference_decimal"] = str(expected)
                if sample["complete"]:
                    sample["sampling_error"] = float(
                        abs(Decimal(sample["numeric"]) - expected)
                    )
                    sample["derivative_error"] = float(
                        abs(Decimal(sample["analytic"]) - expected)
                    )
    # Preserve the Plan14 independent logarithm fixture rather than promoting the
    # diagnostic's own function evaluations into a value oracle.
    fixture = ROOT / "tests/fixtures/plan14/real-algebra-reference.json"
    frozen = json.loads(fixture.read_text())
    frozen_checks = []
    for case in frozen["cases"]:
        actual, _ = reference("log-central", case["x"])
        error = abs(actual - Decimal(case["log1px"]))
        frozen_checks.append({"x": case["x"], "decimal_difference": str(error)})
    document["plan14_reference"] = {
        "path": str(fixture.relative_to(ROOT)),
        "sha256": hashlib.sha256(fixture.read_bytes()).hexdigest(),
        "checks": frozen_checks,
    }
    document["limits"] = [
        "finite selected steps, fixed active mode",
        "differences challenge derivatives, not the physical formula",
        "small steps may lose representable movement or amplify roundoff",
        "refused or incomplete samples remain unavailable",
    ]
    return document


def plot(document: dict, output: Path) -> None:
    import matplotlib as mpl  # noqa: PLC0415 -- explicit optional investigation tool

    mpl.use("Agg")
    from matplotlib import pyplot as plt  # noqa: PLC0415

    figure, axes = plt.subplots(1, 2, figsize=(11, 4), layout="constrained")
    for name in dict.fromkeys(sample["case"] for sample in document["samples"]):
        rows = [
            (sample, comparison)
            for sample in document["samples"]
            if sample["case"] == name
            for comparison in sample["comparisons"]
            if "sampling_error" in comparison
        ]
        for axis, key in zip(axes, ("sampling_error", "derivative_error"), strict=True):
            if rows:
                axis.loglog(
                    [abs(c["step"]) for _, c in rows],
                    [max(c[key], 1e-30) for _, c in rows],
                    ".-",
                    label=name,
                )
    implicit = document.get("implicit_products", {})
    rows = [row for row in implicit.get("samples", []) if row["complete"]]
    if rows:
        for axis, key in zip(axes, ("sampling_error", "derivative_error"), strict=True):
            axis.loglog(
                [row["step"] for row in rows],
                [max(row[key], 1e-30) for row in rows],
                ".-",
                label="actual implicit products",
            )
    for axis, title in zip(
        axes,
        ("Actual library difference vs Decimal", "Supplied derivative vs Decimal"),
        strict=True,
    ):
        axis.set(
            xlabel="Actual normalized legal step",
            ylabel="Absolute normalized derivative error",
            title=title,
        )
        axis.grid(True, which="both", alpha=0.25)
        axis.legend(fontsize="small")
    figure.savefig(output / "error-versus-step.svg")
    figure.savefig(output / "error-versus-step.png", dpi=160)
    plt.close(figure)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--implicit-samples",
        type=Path,
        help="existing actual implicit-product sweep JSON",
    )
    parser.add_argument(
        "--samples", type=Path, help="existing actual example JSON; omit to capture"
    )
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    if args.samples:
        document = json.loads(args.samples.read_text())
    else:
        command = [
            str(ROOT / "scripts/pse-env"),
            "--",
            str(ROOT / ".venv/bin/python"),
            "-m",
            "scripts.arrow_validation",
            "run",
            "--locked",
            "-p",
            "pse-backend-native",
            "--example",
            "numerical-investigation",
            "--quiet",
        ]
        completed = subprocess.run(
            command, cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE
        )
        document = json.loads(completed.stdout)
        paths = (
            "Cargo.lock",
            "rust-toolchain.toml",
            "crates/pse-backend-native/examples/numerical-investigation.rs",
            "crates/pse-backend-native/src/derivative_diagnostics.rs",
            "crates/pse-backend-native/src/derivative_dynamics.rs",
            "crates/pse-math/src/implicit_product_tests.rs",
            "scripts/plan14_reference.py",
        )
        document["source_observations"] = {
            path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
            for path in paths
        }
        document["capture_command"] = command
        document["source_observation_limit"] = (
            "selected current source bytes; no independent executable-provenance attestation"
        )
    if args.implicit_samples:
        implicit = json.loads(args.implicit_samples.read_text())
    elif not args.samples:
        implicit_path = args.output.resolve() / "implicit-products.json"
        subprocess.run(
            [
                str(ROOT / "scripts/pse-env"),
                "--",
                "just",
                "unit-package",
                "pse-math",
                "implicit_product_legal_step_sweep",
            ],
            cwd=ROOT,
            env={**os.environ, "PSE_IMPLICIT_NUMERICAL_OUTPUT": str(implicit_path)},
            check=True,
        )
        implicit = json.loads(implicit_path.read_text())
    else:
        implicit = None
    if implicit is not None:
        if implicit.get("schema") != "implicit-product-step-sweep-v1":
            raise ValueError("unsupported implicit-product sweep")
        document["implicit_products"] = implicit
    result = investigate(document)
    (args.output / "samples.json").write_text(
        json.dumps(result, indent=2, allow_nan=False) + "\n"
    )
    plot(result, args.output)
    print(args.output / "samples.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
