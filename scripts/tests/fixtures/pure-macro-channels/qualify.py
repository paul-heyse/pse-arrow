# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit small real-compiler controls; deliberately outside default setup tests."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess


def main() -> None:
    root = Path(__file__).resolve().parent
    manifest = root / "Cargo.toml"
    repository = root.parents[3]
    literal = repository / "vendor/py_literal/src"
    source = (literal / "parse.rs").read_text()
    grammar = source.split('#[grammar_inline = r####"', 1)[1].split('"####]', 1)[0]
    # Independent golden value from the checksum-verified published crate archive.
    if hashlib.sha256(grammar.encode()).hexdigest() != "5ce3a6b819b2ced5448442248f9fdde5bdf7023709b9a7a1ceeb7b4443a3c0a3":
        raise RuntimeError("inline grammar differs from published py_literal 0.4.0 grammar")
    if (literal / "grammar.pest").exists():
        raise RuntimeError("the replaced grammar authority must be removed")
    common = ["cargo", "--locked", "--offline", "--manifest-path", str(manifest)]
    # Only this standalone fixture's ordinary target directory is used.
    # The enclosing pse-env scope owns command resources and cancellation.
    for profile in ([], ["--release"]):
        for selection in ([], ["-p", "py_literal"]):
            subprocess.run(
                [common[0], "test", *common[1:], "--lib", *profile, *selection],
                cwd=root,
                check=True,
            )
    environment = dict(os.environ, PSE_GUARDED_MACRO_PRESENT="selected")
    cases = {
        "paste-env-paste": "pse tokens-only Paste: environment selectors are unsupported",
        "paste-env-item": "pse tokens-only Paste: environment selectors are unsupported",
        "paste-env-expr": "pse tokens-only Paste: environment selectors are unsupported",
        "pest-file": "pse tokens-only Pest: file grammars are unsupported",
        "pest-generated-file": "pse tokens-only Pest: file grammars are unsupported",
        "pest-inline-expression": "pse tokens-only Pest: grammar_inline requires a string literal",
    }
    for profile in ([], ["--release"]):
        mode = "release" if profile else "debug"
        for name, diagnostic in cases.items():
            result = subprocess.run(
                [common[0], "check", *common[1:], *profile, "--features", "compile-refusal", "--bin", name],
                cwd=root,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
            if result.returncode == 0 or diagnostic not in result.stderr:
                raise RuntimeError(f"{mode} {name}: expected guard refusal, got {result.returncode}\n{result.stderr}")
            print(f"{mode} {name}: refused by actual guarded owner", flush=True)
    print("Pure macro controls passed in debug and release: normal parsing, upstream literal tests, six compiler refusals each.")



if __name__ == "__main__":
    main()
