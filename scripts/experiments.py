# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Compose selected experiments without replacing their execution/result owners."""

from __future__ import annotations

import argparse
import sys


def main(argv: list[str] | None = None) -> int:
    arguments = sys.argv[1:] if argv is None else argv
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "operation", choices=("archive", "inspect", "replay", "bisect", "compare", "tool-probe")
    )
    selected, remaining = parser.parse_known_args(arguments)
    if selected.operation in {"archive", "inspect", "replay"}:
        from scripts import test_archives  # noqa: PLC0415 -- optional selected capability

        return test_archives.main([selected.operation, *remaining])
    if selected.operation in {"bisect", "compare"}:
        from scripts import regression_experiments  # noqa: PLC0415 -- selected adapter

        return regression_experiments.main([selected.operation, *remaining])
    from scripts import tooling_experiments  # noqa: PLC0415 -- optional namespace probe

    return tooling_experiments.main(remaining)


if __name__ == "__main__":
    raise SystemExit(main())
