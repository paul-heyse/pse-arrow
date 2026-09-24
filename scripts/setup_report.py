# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run the existing setup suite with library-owned JUnit and exact selection."""

import argparse
import json
import unittest
from pathlib import Path

import xmlrunner


def selection(suite: unittest.TestSuite) -> list[dict[str, str]]:
    """Read the loader's actual selected cases before the suite consumes them."""
    result = []
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            result.extend(selection(item))
        else:
            owner, _, name = item.id().rpartition(".")
            result.append({"class": owner, "name": name})
    return result


def main() -> int:
    """Use the same stdlib discovery as setup-test, adding a standard XML reporter."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    suite = unittest.TestLoader().discover(
        str(root / "scripts/tests"), pattern="test_*.py", top_level_dir=str(root)
    )
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "setup-test-selected.json").write_text(
        json.dumps(selection(suite), indent=2) + "\n"
    )
    with (args.output / "setup-test.xml").open("wb") as report:
        result = xmlrunner.XMLTestRunner(
            output=report, verbosity=2, failfast=False
        ).run(suite)
    return 0 if result.wasSuccessful() and result.testsRun and not result.skipped else 1


if __name__ == "__main__":
    raise SystemExit(main())
