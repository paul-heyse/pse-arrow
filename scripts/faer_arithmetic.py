#!/usr/bin/env python3
"""Exercise the selected faer f64 closure and inspect its optimized arithmetic IR."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path


def source_controls(root: Path) -> None:
    sources = list((root / "vendor/faer/src").rglob("*.rs"))
    for path in sources:
        text = path.read_text()
        # These are the independent direct dispatch owners in the exact release.
        if re.search(
            r"private_gemm_x86|nano_gemm|gemm::gemm|is_x86_feature_detected!", text
        ):
            raise ValueError(f"uncontrolled native dispatch in {path}")
        if re.search(r"(?:core|std)::arch|\bfmaf?\s*\(", text):
            raise ValueError(f"direct architecture/fused arithmetic bypass in {path}")
        for call in re.finditer(r"\b([A-Za-z_]\w*)\s*\.\s*mul_add\s*\(", text):
            if call.group(1) != "simd":
                raise ValueError(f"direct multiply-add bypass in {path}")
    traits = (root / "vendor/faer-traits/src/lib.rs").read_text()
    for scalar in ["f32", "f64", "ComplexImpl<f32>", "ComplexImpl<f64>"]:
        block = traits.split(f"impl ComplexField for {scalar} {{", 1)[1]
        if block.split("type Index", 1)[0].count("type Arch = pulp::Scalar;") != 1:
            raise ValueError(f"uncontrolled {scalar} architecture")
    for scalar in ["f32", "f64", "ComplexImpl<f32>", "ComplexImpl<f64>"]:
        block = traits.split(f"impl ComplexField for {scalar} {{", 1)[1].split(
            "impl ComplexField for", 1
        )[0]
        if re.search(r"ctx\.(?:conj_)?mul_add_(?!e_)", block):
            raise ValueError(f"explicit fused scalar primitive for {scalar}")
    internal = (root / "vendor/faer/src/linalg/matmul/internal/mod.rs").read_text()
    if "return StackReq::EMPTY" in internal or "false\n}" not in internal:
        raise ValueError(
            "native scratch shortcut survived generic arithmetic migration"
        )
    matmul = (root / "vendor/faer/src/linalg/matmul/mod.rs").read_text()
    if "Par::Rayon" not in matmul or "matmul_vertical::matmul_simd" not in matmul:
        raise ValueError("generic parallel arithmetic route unavailable")
    print(
        f"source dispatch: {len(sources)} faer source paths; native f64 real closure selects Scalar and generic GEMM"
    )


def inspect_ir(path: Path) -> None:
    source = path.read_text()
    if "llvm.fma.f64" not in source:
        raise ValueError(
            "independent fused counterfactual is absent from optimized probe IR"
        )
    symbols = r'@(?:"([^"]+)"|([-A-Za-z$._0-9]+))'
    functions = {}
    for match in re.finditer(
        r"^define [^\n]*" + symbols + r"\([^\n]*\).*?^}",
        source,
        re.MULTILINE | re.DOTALL,
    ):
        name = match.group(1) or match.group(2)
        functions[name] = match.group(0)
    globals_ = {}
    for match in re.finditer(r"^" + symbols + r" = [^\n]*", source, re.MULTILINE):
        globals_[match.group(1) or match.group(2)] = match.group(0)
    nodes = {**globals_, **functions}
    roots = [
        "faer_nonfused_mul_add",
        "faer_nonfused_norm_add",
        "faer_nonfused_matmul",
        "faer_nonfused_decompositions",
        "faer_nonfused_sparse_modes",
    ]
    for root in roots:
        if root not in functions:
            raise ValueError(f"missing optimized entry point {root}")
        pending = [root]
        visited = set()
        external = set()
        while pending:
            name = pending.pop()
            if name in visited:
                continue
            visited.add(name)
            body = nodes[name]
            if re.search(
                r"llvm\.(?:experimental\.constrained\.)?(?:fma|fmuladd)|\b(?:fmul|fadd|fsub|fdiv)\b[^\n]*\b(?:fast|contract)\b",
                body,
            ):
                raise ValueError(
                    f"fused or contractible arithmetic reachable from {root}: {name}"
                )
            # Include function addresses passed into Rayon and solver callbacks,
            # as well as direct calls; inspecting only call instructions misses them.
            for call in re.finditer(symbols, body):
                target = call.group(1) or call.group(2)
                if target in nodes:
                    pending.append(target)
                else:
                    external.add(target)
        if root in roots[:2] and not all(
            op in functions[root] for op in ["fmul double", "fadd double"]
        ):
            raise ValueError(
                f"scalar primitive did not retain separate multiply/add in {root}"
            )

        # Concrete arithmetic generics should be instantiated in this probe.
        # Standard allocation/panic/rayon/runtime functions remain external.
        def reviewed_runtime(name: str) -> bool:
            # These exact upstream owners only read atomic parallelism settings
            # or schedule callbacks. The global-reference walk includes the
            # callback/vtable definitions, where the arithmetic actually lives.
            return (
                name.endswith("22get_global_parallelism")
                or ("6thread8join_raw" in name and name.endswith("14implementation"))
                or (
                    "11gemm_common4gemm" in name
                    and name.endswith("19THREADING_THRESHOLD")
                )
                # Sparse LU's MatU8::with_dims initializes integer mark bits.
                or (
                    "4faer6sparse6linalg2lu10supernodal" in name
                    and name.endswith("5MatU89with_dims")
                )
            )

        unresolved = [
            name
            for name in external
            if re.search(
                r"(?:4faer|11faer_traits|4pulp|4gemm|9nano_gemm|16private_gemm)", name
            )
            and not reviewed_runtime(name)
        ]
        if unresolved:
            raise ValueError(
                f"uninspected supplier arithmetic calls from {root}: {unresolved}"
            )
        bodies = len(visited & functions.keys())
        print(
            f"optimized IR {root}: {bodies} function bodies and {len(visited) - bodies} global initializers; no fused/contractible operations; {len(external)} runtime/intrinsic externals"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source_controls(root)
    result = subprocess.run(
        [
            "cargo",
            "build",
            "--package",
            "pse-backend-native",
            "--lib",
            "--locked",
            "--message-format=json",
        ],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    (output / "build.jsonl").write_text(result.stdout)
    (output / "build.stderr").write_text(result.stderr)
    artifacts = [
        json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")
    ]
    faer = next(
        item
        for item in artifacts
        if item.get("reason") == "compiler-artifact"
        and item.get("target", {}).get("name") == "faer"
    )
    if Path(faer["manifest_path"]).resolve() != root / "vendor/faer/Cargo.toml":
        raise ValueError("probe did not select the maintained faer source")
    library = next(Path(name) for name in faer["filenames"] if name.endswith(".rmeta"))
    dependency_directories = sorted(
        {
            str(Path(name).parent)
            for item in artifacts
            if item.get("reason") == "compiler-artifact"
            for name in item.get("filenames", [])
        }
    )
    binary = output / "faer-nonfused"
    ir = output / "faer-nonfused.ll"
    command = [
        "rustc",
        "--edition=2024",
        "--crate-name=faer_nonfused_probe",
        str(root / "scripts/probes/faer-nonfused.rs"),
        *(
            argument
            for directory in dependency_directories
            for argument in ["-L", f"dependency={directory}"]
        ),
        "--extern",
        f"faer={library}",
        "-C",
        "opt-level=3",
        "-C",
        "panic=unwind",
        "--extern",
        f"faer={library.with_suffix('.rlib')}",
        "-C",
        "target-feature=+fma",
        f"--emit=llvm-ir={ir},link={binary}",
    ]
    (output / "rustc-command.json").write_text(json.dumps(command, indent=2) + "\n")
    subprocess.run(command, cwd=root, check=True)
    inspect_ir(ir)
    subprocess.run([str(binary)], cwd=root, check=True)
    print(
        "adversarial and LU/SVD/EVD/sparse mode controls: passed (FMA enabled on probe target; strict separate operations retained)"
    )


if __name__ == "__main__":
    main()
