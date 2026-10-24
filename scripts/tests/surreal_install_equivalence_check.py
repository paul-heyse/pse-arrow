# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual complete installer receipt reuse after Cargo TOML syntax-only changes.

Uses one explicitly supplied, already built installation. Never starts a service
or builds another server generation. The checkout config is restored exactly.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from unittest.mock import patch

from scripts import surreal_server as server
from scripts import surreal_source as source


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tool_root", type=Path)
    args = parser.parse_args()
    tool_root = args.tool_root.resolve()
    directory = tool_root / source.installation_name(tool_root)
    require(
        (directory / "release.json").is_file(),
        "complete current installation required; this probe never builds",
    )
    before = server.install(None, tool_root)
    binary = Path(str(before["binary"]))
    binary_digest = source.digest(binary)
    receipt_bytes = (directory / "release.json").read_bytes()
    config = source.ROOT / ".cargo/config.toml"
    original = config.read_bytes()
    (tool_root / "equivalence-original-cargo-config.toml").write_bytes(original)
    text = original.decode()
    ordered = 'linker = "clang"\nrustflags = ["-C", "link-arg=-fuse-ld=mold"]'
    require(
        ordered in text, "fixture requires current explicit clang/mold target table"
    )
    variants = {
        "comments": text + "\n# D03 actual syntax-equivalence probe\n",
        "formatting": text.replace("jobs = 16", "jobs=16").replace(
            "[build]", "[ build ]"
        ),
        "table-key-order": text.replace(
            ordered, 'rustflags = ["-C", "link-arg=-fuse-ld=mold"]\nlinker = "clang"'
        ),
    }
    observed = []
    owned = original
    actual_command = source.command
    commands = []

    def record(
        arguments: list[str],
        *,
        log_path: Path | None = None,
        environment: dict[str, str] | None = None,
        stream: bool = False,
    ) -> str:
        commands.append(arguments)
        return actual_command(
            arguments, log_path=log_path, environment=environment, stream=stream
        )

    try:
        for kind, value in variants.items():
            require(
                config.read_bytes() == owned,
                "concurrent config mutation; refusing overwrite",
            )
            owned = value.encode()
            config.write_bytes(owned)
            with patch.object(source, "command", side_effect=record):
                reused = server.install(None, tool_root)
            require(
                reused == before and source.digest(binary) == binary_digest,
                "syntax change did not reuse identical receipt/binary",
            )
            require(
                (directory / "release.json").read_bytes() == receipt_bytes,
                "reuse rewrote original provenance",
            )
            observed.append(kind)
        require(
            config.read_bytes() == owned,
            "concurrent config mutation; refusing overwrite",
        )
        owned = text.replace(
            '"link-arg=-fuse-ld=mold"]',
            '"link-arg=-fuse-ld=mold", "-C", "opt-level=1"]',
        ).encode()
        config.write_bytes(owned)
        require(
            source.installation_name(tool_root) != directory.name,
            "actual codegen input did not change effective namespace",
        )
        try:
            with patch.object(source, "command", side_effect=record):
                source.install(directory)
        except source.BuildError as error:
            refusal = str(error)
        else:
            raise ValueError("changed executable input reused the old installation")
        require(
            not any(
                command and Path(command[0]).name in {"cargo", "pse-env"}
                for command in commands
            ),
            "reuse unexpectedly invoked a build",
        )
    finally:
        if config.read_bytes() != owned:
            raise ValueError(
                "concurrent config mutation; original retained in equivalence-original-cargo-config.toml, no automatic overwrite"
            )
        config.write_bytes(original)
    result = {
        "tool_root": str(tool_root),
        "binary": str(binary),
        "binary_sha256": binary_digest,
        "effective_key": before["effective_key"],
        "syntax_reused": observed,
        "original_receipt_unchanged": (directory / "release.json").read_bytes()
        == receipt_bytes,
        "executable_input_refused": refusal,
        "build_invocations": 0,
        "services_started": 0,
    }
    (tool_root / "equivalence-check.json").write_text(
        json.dumps(result, indent=2) + "\n"
    )
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
