# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run the existing setup suite with library-owned JUnit and exact selection."""

import argparse
import json
import os
import sys
import unittest
from pathlib import Path

import xmlrunner

from scripts import test_requirements, test_resources, test_run


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


def requirements(suite: unittest.TestSuite) -> dict[str, dict[str, object]]:
    """Class-local declarations accompany the loader's actual selected cases."""
    result = {}
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            result.update(requirements(item))
        else:
            owner, _, name = item.id().rpartition(".")
            demand = getattr(item, "test_requirements", test_requirements.LOCAL)
            if not isinstance(demand, test_requirements.Requirement):
                raise ValueError("Unknown setup test requirements")
            result[owner + "::" + name] = demand.document()
    return result


def execute(suite: unittest.TestSuite, output: Path) -> int:
    """Bind actual setup selection to the existing managed outcome composer."""
    selected = selection(suite)
    demands = requirements(suite)
    test_requirements.validate_roles(
        demands, test_requirements.required_roles(os.environ)
    )
    output = output.resolve()
    selected_path = output / "setup-test-selected.json"
    selected_path.write_text(json.dumps(selected, indent=2) + "\n")
    path = test_resources.invocation(
        "unittest",
        [{"name": item["class"] + "::" + item["name"]} for item in selected],
        requirements=demands,
    )
    return test_run.execute_selected(
        path,
        [
            sys.executable,
            "-m",
            "scripts.setup_report",
            str(output),
            "--child",
            "--selected",
            str(selected_path),
        ],
        {**os.environ, test_resources.MARKER: str(path)},
        output / "setup-test.xml",
        selected=selected,
    )


def main() -> int:
    """Use the same stdlib discovery as setup-test, adding a standard XML reporter."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--child", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--selected", type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    loader = unittest.TestLoader()
    if args.child != (args.selected is not None):
        parser.error("selected setup catalog belongs to its internal child")
    args.output.mkdir(parents=True, exist_ok=True)
    if not args.child:
        suite = unittest.TestSuite(
            loader.discover(
                str(root / "scripts/tests"), pattern=path.name, top_level_dir=str(root)
            )
            for path in sorted((root / "scripts/tests").glob("test_*.py"))
            if "unittest.TestCase" in path.read_text()
        )
        return execute(suite, args.output)
    selected = json.loads(args.selected.read_text())
    names = [item["class"] + "." + item["name"] for item in selected]
    if any(not name.startswith("scripts.tests.") for name in names):
        parser.error("setup selection must belong to repository test owners")
    suite = loader.loadTestsFromNames(names)
    with (args.output / "setup-test.xml").open("wb") as report:
        result = xmlrunner.XMLTestRunner(
            output=report, verbosity=2, failfast=False
        ).run(suite)
    return 0 if result.wasSuccessful() and result.testsRun and not result.skipped else 1


if __name__ == "__main__":
    raise SystemExit(main())
