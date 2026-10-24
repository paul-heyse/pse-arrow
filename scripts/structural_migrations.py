# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Preview a selected import-aware JSON source migration, preserving concrete syntax.

ast-grep supplies contextual shape/fix candidates; LibCST resolves the imported
loader and lexical shadowing before editing. This is not receiver type inference.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

from scripts import semantic_edits

# The broad local-shape fix is deliberately filtered by qualified import identity.
# In particular tomllib.loads cannot consume bytes and must remain a near miss.
RULE = """id: json-source-bytes
language: Python
rule:
  all:
    - pattern: $LOADER($SOURCE.read_text())
    - has:
        pattern: $SOURCE.read_text()
        stopBy: end
fix: $LOADER($SOURCE.read_bytes())
"""


def candidates(source: str) -> list[dict]:
    binary = shutil.which("ast-grep")
    if binary is None:
        raise ValueError(
            "ast-grep is unavailable; install the repository bootstrap tool"
        )
    version = subprocess.run(
        [binary, "--version"], text=True, capture_output=True, check=True
    ).stdout.strip()
    if version != "ast-grep 0.50.0":
        raise ValueError(
            "structural migration requires qualified ast-grep 0.50.0; observed "
            + version
        )
    with tempfile.TemporaryDirectory(prefix="pse-structural-") as directory:
        rule = Path(directory) / "rule.yml"
        sample = Path(directory) / "source.py"
        rule.write_text(RULE)
        sample.write_bytes(source.encode("utf-8"))
        result = subprocess.run(
            [binary, "scan", "--rule", str(rule), "--json", str(sample)],
            text=True,
            capture_output=True,
            check=True,
        )
        if result.stderr.strip():
            raise ValueError(
                "ast-grep could not establish clean syntax coverage: " + result.stderr
            )
        return json.loads(result.stdout)


def json_source_bytes(source: str) -> tuple[str, dict]:
    """Change only resolved stdlib json.loads(Path-like.read_text()) candidates.

    Keep import aliases, comments and local shadows. A selected caller still owns
    the Path-like receiver contract; dynamic dispatch is outside this migration.
    """
    import libcst as cst
    from libcst.metadata import (
        MetadataWrapper,
        PositionProvider,
        QualifiedNameProvider,
        QualifiedNameSource,
    )

    shapes = candidates(source)
    starts = {
        (match["range"]["start"]["line"] + 1, match["range"]["start"]["column"])
        for match in shapes
    }
    applied = []

    class Transform(cst.CSTTransformer):
        METADATA_DEPENDENCIES = (QualifiedNameProvider, PositionProvider)

        def leave_Call(
            self, original_node: cst.Call, updated_node: cst.Call
        ) -> cst.Call:
            position = self.get_metadata(PositionProvider, original_node).start
            # ast-grep reports byte columns; LibCST reports Unicode code points.
            line = source.splitlines()[position.line - 1]
            byte_column = len(line[: position.column].encode("utf-8"))
            if (position.line, byte_column) not in starts:
                return updated_node
            names = self.get_metadata(QualifiedNameProvider, original_node.func, set())
            if len(names) != 1 or not any(
                name.name == "json.loads" and name.source == QualifiedNameSource.IMPORT
                for name in names
            ):
                return updated_node
            if (
                len(updated_node.args) != 1
                or updated_node.args[0].keyword
                or updated_node.args[0].star
            ):
                return updated_node
            argument = updated_node.args[0]
            value = argument.value
            if (
                not isinstance(value, cst.Call)
                or value.args
                or not isinstance(value.func, cst.Attribute)
                or value.func.attr.value != "read_text"
            ):
                return updated_node
            applied.append(
                {
                    "line": position.line,
                    "column": position.column,
                    "qualified_loader": "json.loads",
                }
            )
            return updated_node.with_changes(
                args=[
                    argument.with_changes(
                        value=value.with_changes(
                            func=value.func.with_changes(attr=cst.Name("read_bytes"))
                        )
                    )
                ]
            )

    transformed = MetadataWrapper(cst.parse_module(source)).visit(Transform())
    return transformed.code, {
        "ast_grep_candidates": len(shapes),
        "resolved_edits": applied,
        "limits": "import and lexical scope identity; caller owns Path-like receiver semantics; no type inference",
    }


def migration(root: Path, paths: list[str]) -> dict:
    cut = semantic_edits.capture(root, paths)
    outputs = {}
    observations = {}
    for name, before in cut.items():
        after, observed = json_source_bytes(before)
        outputs[name] = after
        observations[name] = observed
    selected = semantic_edits.replacements(
        root, cut, outputs, producer="ast-grep0.50.0+libcst1.9.0:json-source-bytes"
    )
    selected["observations"] = observations
    return selected


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("preview", "apply", "reconcile"))
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--paths", nargs="+")
    parser.add_argument("--preview", type=Path)
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    if args.command == "preview":
        if not args.paths:
            parser.error("preview requires explicit owned --paths")
        result = migration(args.root, args.paths)
    else:
        if not args.preview or not args.receipt:
            parser.error("apply/reconcile require --preview and --receipt")
        selected = json.loads(args.preview.read_bytes())
        result = (
            semantic_edits.apply(selected, args.receipt)
            if args.command == "apply"
            else semantic_edits.reconcile(selected)
        )
        if args.command == "reconcile":
            args.receipt.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0 if result.get("complete", True) else 1


if __name__ == "__main__":
    raise SystemExit(main())
