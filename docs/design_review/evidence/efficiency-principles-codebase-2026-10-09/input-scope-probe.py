# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Probe current input projection with synthetic digest changes, without file edits.

This does not execute receipt reuse, product tests, a compiler or a native service.
Run from any working directory with the checkout's pinned interpreter.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> None:
    root = Path(__file__).resolve().parents[4]
    sys.path.insert(0, str(root))
    from scripts.validation_scope import INPUT_SCOPE_VERSION, input_identity

    paths = (
        "scripts/test_run.py",
        "scripts/test_resources.py",
        "scripts/host_admission.py",
        "scripts/surreal_server.py",
        "benches/benches/native_process.rs",
        "crates/pse-runtime/src/lib.rs",
        "scripts/native_tests.py",
    )
    environment_names = (
        "UNO_DIR",
        "PETSC_DIR",
        "SUITESPARSE_INCLUDE_DIR",
        "CC",
        "CXX",
        "CMAKE_TOOLCHAIN_FILE",
        "IPOPT_DIR",
    )
    scopes = ("rust-product", "python-product")
    result = {
        "schema": "review-input-projection-probe-v1",
        "input_scope_version": INPUT_SCOPE_VERSION,
        "python": sys.version.split()[0],
        "method": "synthetic before/after values passed to actual input_identity",
        "paths": {
            path: {
                scope: input_identity(scope, {path: "before"}, {})
                != input_identity(scope, {path: "after"}, {})
                for scope in scopes
            }
            for path in paths
        },
        "environment": {
            name: {
                scope: input_identity(scope, {}, {name: "before"})
                != input_identity(scope, {}, {name: "after"})
                for scope in scopes
            }
            for name in environment_names
        },
    }
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
