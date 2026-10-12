"""Disposable Plan 36 design probes; never changes product sources or registries.

Run from the checkout with scripts/pse-env -- .venv/bin/python <this-file> MODE.
The topology mode asks Cargo for unit graphs; it does not compile the proposed tree.
"""
from __future__ import annotations

import argparse
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))


def call(args, *, cwd=ROOT, env=None):
    result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{args}: {result.returncode}\n{result.stderr}")
    return result.stdout


def packages(graph):
    return sorted({u["pkg_id"].split("#")[-1].split("@")[0] for u in graph["units"]})


def unit_graph(root, package, *, test=False, features=None):
    args = ["cargo", "test" if test else "check", "--offline", "-p", package,
            "--lib", "--no-default-features", "-Z", "unstable-options", "--unit-graph"]
    if features:
        args += ["--features", features]
    return json.loads(call(args, cwd=root))


def topology():
    with tempfile.TemporaryDirectory(prefix="pse36-topology-") as temporary:
        stage = Path(temporary)
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
            shutil.copy2(ROOT / name, stage / name)
        shutil.copytree(ROOT / ".cargo", stage / ".cargo")
        (stage / "vendor").symlink_to(ROOT / "vendor", target_is_directory=True)
        manifests = list((ROOT / "crates").glob("*/Cargo.toml"))
        manifests += list((ROOT / "tests").glob("*/Cargo.toml"))
        manifests += [ROOT / "xtask/Cargo.toml", ROOT / "benches/Cargo.toml"]
        for source in manifests:
            destination = stage / source.relative_to(ROOT)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
            for child in source.parent.iterdir():
                if child.name != "Cargo.toml":
                    (destination.parent / child.name).symlink_to(child, target_is_directory=child.is_dir())
        baseline = unit_graph(stage, "pse-math")
        engine_baseline = unit_graph(stage, "pse-engine")
        # The proposed source move has exactly these direct library consumers.
        selected = ("egobox-ego", "egobox-moe", "egobox-gp", "argmin", "ndarray")
        math = stage / "crates/pse-math/Cargo.toml"
        text = math.read_text()
        for name in selected:
            text = text.replace(f"{name}.workspace = true\n", "")
        text = text.replace("egobox-doe.workspace = true\n", "")
        math.write_text(text)
        runtime = stage / "crates/pse-runtime/Cargo.toml"
        runtime.write_text(runtime.read_text().replace("[dependencies]\n", "[dependencies]\n" +
            "".join(f"{name}.workspace = true\n" for name in selected)).replace(
                "[dev-dependencies]\n", "[dev-dependencies]\negobox-doe.workspace = true\n"))
        moved = unit_graph(stage, "pse-math")
        for manifest in manifests:
            path = stage / manifest.relative_to(ROOT)
            path.write_text(path.read_text().replace("pse-workspace-hack.workspace = true\n", ""))
        # Keep an empty unreferenced member for lock-comparable probing; production removal also removes it.
        hack = stage / "crates/pse-workspace-hack/Cargo.toml"
        hack.write_text('[package]\nname="pse-workspace-hack"\nversion="0.0.1"\nedition="2024"\n')
        no_hack = unit_graph(stage, "pse-math")
        engine = unit_graph(stage, "pse-engine")
        engine_test = unit_graph(stage, "pse-engine", test=True, features="force-validate")
        ids_rayon = unit_graph(stage, "pse-ids")
        workspace = stage / "Cargo.toml"
        workspace.write_text(workspace.read_text().replace(
            'blake3 = { version = "=1.8.7", features = ["rayon"] }', 'blake3 = "=1.8.7"'))
        ids_no_rayon = unit_graph(stage, "pse-ids")
        old_lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        new_lock = tomllib.loads((stage / "Cargo.lock").read_text())
        def external(lock):
            return sorted((p["name"], p["version"], p.get("source"), p.get("checksum"))
                          for p in lock["package"] if "source" in p)
        assert external(old_lock) == external(new_lock), "external lock resolution moved"
        output = {"external_resolution_unchanged": True}
        for label, graph in (("math_baseline", baseline), ("math_surrogate_moved", moved),
                             ("engine_baseline", engine_baseline),
                             ("math_no_hack", no_hack), ("engine_no_hack", engine),
                             ("engine_test_force_validate_no_hack", engine_test)):
            names = packages(graph)
            output[label] = {"units": len(graph["units"]), "packages": len(names),
                "egobox": [n for n in names if "egobox" in n],
                "native": [n for n in names if n.startswith("pse-backend")],
                "arrow_leaf_features": {u["pkg_id"].split("#")[-1] + ":" + str(u["platform"]): u["features"]
                    for u in graph["units"] if any("#" + n + "@" in u["pkg_id"] for n in
                        ("arrow-array", "arrow-data", "arrow-cast"))}}
        output["engine_removed_packages"] = sorted(set(packages(engine_baseline)) - set(packages(engine)))
        output["ids_unused_blake3_rayon"] = {
            "before_units": len(ids_rayon["units"]), "after_units": len(ids_no_rayon["units"]),
            "removed_packages": sorted(set(packages(ids_rayon)) - set(packages(ids_no_rayon)))}
        output["math_removed_packages"] = sorted(set(packages(baseline)) - set(packages(no_hack)))
        print(json.dumps(output, indent=2))


def terminal():
    # Existing implementation negative control, completely patched to a temporary registry.
    from scripts.tests.test_test_resources import TestResourceTests
    from scripts import test_resources as resources
    test = TestResourceTests()
    test.setUp()
    try:
        resources.finalize(test.invocation, {}, complete=True, terminal_owner="runner")
        resource = test.register()
        record = resources.resource_status(resource)
        assert record["disposition"] == "incomplete" and record["pin"]
        print(json.dumps({"late_registration_after_finalize": "accepted-but-pinned-incomplete",
                          "false_pass": False}))
    finally:
        test.doCleanups()


def freeze_model():
    # Enumerate crash prefixes of the proposed invocation-owned registration protocol.
    # Membership intent MUST precede resource publication; recovery resolves every intent.
    cases = 0
    for register_first in (False, True):
        for crash_after in range(5):
            state = {"phase": "open", "intent": False, "resource": False,
                     "indexed": False, "accepted": False, "projected": False}
            if not register_first:
                state["phase"] = "frozen"
            actions = ("intent", "resource", "indexed", "accepted")
            for action in actions[:crash_after]:
                if state["phase"] == "frozen":
                    break
                state[action] = True
            recovered = copy.deepcopy(state)
            if recovered["intent"]:
                if recovered["resource"]:
                    recovered["indexed"] = True
                else:
                    # Proven no effect from this metadata-only publication; external creation starts later.
                    recovered["intent"] = False
            recovered["phase"] = "frozen"
            recovered["projected"] = recovered["resource"]
            assert not recovered["resource"] or (recovered["intent"] and recovered["projected"])
            assert not recovered["accepted"] or recovered["projected"]
            cases += 1
    print(json.dumps({"crash_order_cases": cases, "failures": 0,
                      "scope": "abstract finite protocol; not filesystem concurrency or durability"}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("topology", "terminal", "freeze-model"))
    mode = parser.parse_args().mode
    if mode == "topology":
        topology()
    elif mode == "terminal":
        terminal()
    else:
        freeze_model()
