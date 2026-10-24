# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real xdist collection publishes native demand before scheduling local effects."""

# ruff: noqa: PT009, PT027 -- stdlib process boundary controls
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import test_resources

ROOT = Path(__file__).resolve().parents[2]

FIXTURE = """import os
from pathlib import Path
import pytest
from scripts import test_resources

@pytest.fixture
def observed_owner(request):
    path = Path(os.environ["PSE_TEST_INVOCATION"])
    record = test_resources.read_invocation(path)
    state = test_resources.storage().invocation_snapshot(path.stem)
    selected = {item["name"] for item in record["selected"]}
    assert state["collected"] is True
    assert selected == {"test_owned.py::test_control[first]", "test_owned.py::test_control[second]"}
    assert set(state["requirements"]) == selected
    for requirement in state["requirements"].values():
        assert requirement["roles"] == ["collection-guard"]
        assert requirement["effects"] == []
    with pytest.raises(test_resources.ResourceError, match="Undeclared canonical"):
        test_resources.require_effect({"test": request.node.nodeid}, effect="canonical")
    effect = Path("effects") / request.node.name
    effect.write_text(os.environ["PYTEST_XDIST_WORKER"])
    return effect

@pytest.mark.unit
@pytest.mark.test_requirements(roles=["collection-guard"])
@pytest.mark.parametrize("label", ["first", "second"])
def test_control(observed_owner, label):
    assert label in observed_owner.name
    assert observed_owner.is_file()
"""


class PythonCollectionOrderTests(unittest.TestCase):
    def test_late_worker_uses_current_sealed_catalog_without_expanding_or_rewriting_it(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.dict(
                os.environ,
                {
                    "PSE_TEST_RESOURCE_STATE": str(Path(temporary) / "registry"),
                    "PSE_TEST_REQUIRED_ROLES": "",
                },
            ),
        ):
            invocation = test_resources.invocation("python", [])
            stale = test_resources.read_invocation(invocation)
            requirement = {
                "effects": [],
                "fixtures": [],
                "capacity": "local",
                "roles": ["actual"],
                "exclusion": None,
            }
            test_resources.publish_python_requirements(
                invocation, {"owned.py::case": requirement}
            )
            self.assertTrue(
                test_resources.publish_python_collection(invocation, ["owned.py::case"])
            )
            with patch.object(test_resources, "read_invocation", return_value=stale):
                test_resources.publish_python_requirements(
                    invocation, {"owned.py::case": requirement}
                )
                with self.assertRaisesRegex(
                    test_resources.ResourceError, "outside the collected"
                ):
                    test_resources.publish_python_requirements(
                        invocation, {"other.py::case": requirement}
                    )
                with self.assertRaisesRegex(test_resources.ResourceError, "disagree"):
                    test_resources.publish_python_requirements(
                        invocation,
                        {"owned.py::case": {**requirement, "roles": ["different"]}},
                    )
            current = test_resources.read_invocation(invocation)
            image = test_resources.storage().invocation_snapshot(invocation.stem)
            self.assertEqual(current["selected"], [{"name": "owned.py::case"}])
            self.assertEqual(image["requirements"], {"owned.py::case": requirement})
            test_resources.finalize(
                invocation,
                {"owned.py::case": "pass"},
                complete=True,
                terminal_owner="runner",
            )
            with self.assertRaisesRegex(
                test_resources.ResourceError, "open collected invocation"
            ):
                test_resources.publish_python_requirements(
                    invocation, {"owned.py::case": requirement}
                )

    def test_requirement_publication_refuses_wrong_identity_kind_and_store(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.dict(
                os.environ,
                {
                    "PSE_TEST_RESOURCE_STATE": str(Path(temporary) / "registry"),
                    "PSE_TEST_REQUIRED_ROLES": "",
                },
            ),
        ):
            path = test_resources.invocation("python", [])
            record = test_resources.read_invocation(path)
            for change in (
                {"nonce": "f" * 32},
                {"version": 99},
                {"terminal_owner": "foreign"},
            ):
                with (
                    self.subTest(change=change),
                    patch.object(
                        test_resources,
                        "read_invocation",
                        return_value={**record, **change},
                    ),
                    self.assertRaises(test_resources.ResourceError),
                ):
                    test_resources.publish_python_requirements(path, {})
            foreign = Path(temporary) / "different-registry"
            with (
                patch.object(
                    test_resources,
                    "storage",
                    return_value=test_resources.resource_store.FileStore(foreign),
                ),
                self.assertRaisesRegex(
                    test_resources.ResourceError, "selected registry"
                ),
            ):
                test_resources.publish_python_requirements(path, {})
            self.assertFalse(foreign.exists())
            rust = test_resources.invocation("rust", [])
            with self.assertRaisesRegex(
                test_resources.ResourceError, "open collected invocation"
            ):
                test_resources.publish_python_requirements(rust, {})
            self.assertEqual(
                test_resources.storage().invocation_snapshot(path.stem)["requirements"],
                {},
            )

    def test_actual_workers_publish_requirements_before_controller_seals_and_fixture_effects(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "conftest.py").write_bytes((ROOT / "conftest.py").read_bytes())
            (root / "pytest.ini").write_text(
                "[pytest]\nmarkers =\n    unit: local fixture control\n"
                "    test_requirements(roles): source-owned demand\n"
            )
            (root / "test_owned.py").write_text(FIXTURE)
            (root / "effects").mkdir()
            environment = {
                **os.environ,
                "PSE_TEST_RESOURCE_STATE": str(root / "registry"),
                "PSE_TEST_REQUIRED_ROLES": "collection-guard",
                "PYTHONPATH": str(ROOT),
                "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
                "PYTHONWARNINGS": "error",
            }
            with patch.dict(os.environ, environment, clear=True):
                invocation = test_resources.invocation("python", [])
                response = subprocess.run(
                    [
                        sys.executable,
                        "-m",
                        "pytest",
                        "test_owned.py",
                        "-q",
                        "-n",
                        "2",
                        "-p",
                        "no:cacheprovider",
                        "-p",
                        "xdist.plugin",
                        "--basetemp",
                        str(root / "pytest-temp"),
                    ],
                    cwd=root,
                    env={
                        **environment,
                        "PSE_TEST_INVOCATION": str(invocation),
                        "PSE_TEST_ENUMERATION": str(root / "selected.txt"),
                    },
                    capture_output=True,
                    text=True,
                    check=False,
                    timeout=30,
                )
                self.assertEqual(
                    response.returncode, 0, response.stdout + response.stderr
                )
                self.assertIn("2 passed", response.stdout)
                record = test_resources.read_invocation(invocation)
                state = test_resources.storage().invocation_snapshot(invocation.stem)
                expected = {
                    "test_owned.py::test_control[first]",
                    "test_owned.py::test_control[second]",
                }
                self.assertEqual(
                    {item["name"] for item in record["selected"]}, expected
                )
                self.assertEqual(set(state["requirements"]), expected)
                self.assertEqual(
                    set((root / "selected.txt").read_text().splitlines()), expected
                )
                self.assertTrue(state["collected"])
                self.assertNotEqual(record["collection_owner"]["pid"], os.getpid())
                self.assertEqual(
                    {path.read_text() for path in (root / "effects").iterdir()},
                    {"gw0", "gw1"},
                )
                before = json.dumps(state["requirements"], sort_keys=True)
                test_resources.publish_python_requirements(
                    invocation, state["requirements"]
                )
                first = next(iter(expected))
                changed = {**state["requirements"][first], "roles": ["inconsistent"]}
                with self.assertRaisesRegex(test_resources.ResourceError, "disagree"):
                    test_resources.publish_python_requirements(
                        invocation, {first: changed}
                    )
                after = test_resources.storage().invocation_snapshot(invocation.stem)
                self.assertEqual(
                    json.dumps(after["requirements"], sort_keys=True), before
                )


if __name__ == "__main__":
    unittest.main()
