# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-owner setup observations before Python fixture assertion admission."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import importlib.util
import os
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType
from unittest.mock import Mock, patch

import pytest

from scripts import surreal_server, test_resources

ROOT = Path(__file__).resolve().parents[2]


class CanonicalSetupProducers(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.state = Path(directory.name)
        # Load the actual fixture producer with a bounded native boundary double;
        # these controls do not claim a live database or native extension journey.
        pse = ModuleType("pse")
        pse.__dict__["Runtime"] = Mock()
        build = ModuleType("pse._build")
        build.__dict__.update(
            CacheSettings=object,
            EngineSettings=object,
            build_info=Mock(),
            compiled_features=Mock(return_value=["force-validate"]),
        )
        extensions = ModuleType("pse.contracts.extension_types")
        extensions.__dict__["EXTENSION_NAMES"] = ()
        modules = {
            "pse": pse,
            "pse._build": build,
            "pse.contracts.extension_types": extensions,
        }
        imports = patch.dict(sys.modules, modules)
        imports.start()
        self.addCleanup(imports.stop)
        fixture = self.load_module(
            "pse.tests.canonical_fixture", "python/pse/tests/canonical_fixture.py"
        )
        sys.modules["pse.tests.canonical_fixture"] = fixture
        self.plugin = self.load_module(
            "canonical_setup_controls", "python/pse/tests/conftest.py"
        )
        self.runtime = pse.__dict__["Runtime"]
        self.request = Mock()
        self.request.node.nodeid = "test_selected.py::test_selected"
        self.request.node.get_closest_marker.return_value = None
        environment = patch.dict(
            os.environ,
            {
                "PSE_SURREAL_STATE": str(self.state),
                "PSE_TEST_INVOCATION": str(self.state / "invocation.json"),
                "PSE_WORKER_BINARY": str(self.state / "worker"),
            },
        )
        environment.start()
        self.addCleanup(environment.stop)
        self.mock(test_resources, "require_effect")
        self.observe = self.mock(test_resources, "observe_setup_failure", create=True)
        self.register = self.mock(test_resources, "register", return_value="resource")
        self.record_drain = self.mock(test_resources, "record_drain")
        self.context = self.mock(surreal_server, "register_context")
        self.config = self.mock(surreal_server, "config_for", return_value={})
        self.ready = self.mock(surreal_server, "ready", return_value=True)
        self.mock(surreal_server, "start")
        self.drain_context = self.mock(surreal_server, "drain_context", return_value=[])

    @staticmethod
    def load_module(name: str, relative: str) -> ModuleType:
        spec = importlib.util.spec_from_file_location(name, ROOT / relative)
        if spec is None or spec.loader is None:
            raise AssertionError("fixture module loader unavailable")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def mock(self, owner: object, name: str, **kwargs: object) -> Mock:
        target = patch.object(owner, name, **kwargs)
        result = target.start()
        self.addCleanup(target.stop)
        return result

    def generator(self):
        return self.plugin.canonical_substrate.__wrapped__(self.request)

    def assert_observed(self, stage: str) -> None:
        self.observe.assert_called_once_with(
            {
                "test": self.request.node.nodeid,
                "stage": stage,
                "code": "runtime::infrastructure",
                "kind": "infrastructure",
                "assertion_reach": "not-reached",
            }
        )

    def test_missing_state_observed_before_pytest_failure(self) -> None:
        os.environ.pop("PSE_SURREAL_STATE")
        with self.assertRaises(pytest.fail.Exception):
            next(self.generator())
        self.assert_observed("canonical-state")
        self.register.assert_not_called()

    def test_missing_state_path_observed_before_registration(self) -> None:
        os.environ["PSE_SURREAL_STATE"] = str(self.state / "missing")
        with self.assertRaises(FileNotFoundError):
            next(self.generator())
        self.assert_observed("canonical-state")
        self.register.assert_not_called()

    def test_source_setup_stages_preserve_original_failure_and_drain(self) -> None:
        for owner, attribute, stage, registered, context in (
            (self, "register", "canonical-register", False, False),
            (self, "context", "canonical-context", True, False),
            (self, "config", "canonical-context", True, True),
            (self.request, "getfixturevalue", "canonical-settings", True, True),
            (self.runtime, "initialize_database", "canonical-initialize", True, True),
        ):
            with self.subTest(stage=stage, operation=attribute):
                failure = RuntimeError("assertion-like prose does not classify setup")
                operation = getattr(owner, attribute)
                operation.side_effect = failure
                with self.assertRaises(RuntimeError) as caught:
                    next(self.generator())
                self.assertIs(caught.exception, failure)
                self.assert_observed(stage)
                self.assertEqual(self.record_drain.call_count, int(registered))
                self.assertEqual(self.drain_context.call_count, int(context))
                operation.side_effect = None
                self.observe.reset_mock()
                self.record_drain.reset_mock()
                self.drain_context.reset_mock()

    def test_settings_pytest_failure_is_still_pre_yield_setup(self) -> None:
        self.request.getfixturevalue.side_effect = pytest.fail.Exception("settings")
        with self.assertRaises(pytest.fail.Exception):
            next(self.generator())
        self.assert_observed("canonical-settings")
        self.record_drain.assert_called_once()

    def test_rejected_observation_is_visible_and_chains_original_failure(self) -> None:
        failure = RuntimeError("registration refused")
        self.register.side_effect = failure
        rejection = test_resources.ResourceError("invocation is frozen")
        self.observe.side_effect = rejection
        with self.assertRaises(test_resources.ResourceError) as caught:
            next(self.generator())
        self.assertIs(caught.exception, rejection)
        self.assertIs(caught.exception.__context__, failure)
        self.assert_observed("canonical-register")

    def test_missing_invocation_does_not_fabricate_authenticated_observation(self) -> None:
        os.environ.pop("PSE_TEST_INVOCATION")
        self.register.side_effect = RuntimeError("bare registration failure")
        with self.assertRaises(RuntimeError):
            next(self.generator())
        self.observe.assert_not_called()

    def test_assertion_through_yield_is_not_setup_failure(self) -> None:
        generator = self.generator()
        fixture = next(generator)
        borrower = Mock()
        fixture.borrowers.append(borrower)
        failure = AssertionError("runtime::infrastructure connect failed")
        with self.assertRaises(AssertionError) as caught:
            generator.throw(failure)
        self.assertIs(caught.exception, failure)
        self.observe.assert_not_called()
        borrower.clear_program_cache.assert_called_once()
        borrower.close.assert_called_once()
        self.assertEqual(fixture.borrowers, [])
        self.record_drain.assert_called_once_with("resource", units=[])

    def test_teardown_failure_is_not_setup_failure(self) -> None:
        generator = self.generator()
        fixture = next(generator)
        borrower = Mock()
        borrower.close.side_effect = RuntimeError("drain failed")
        fixture.borrowers.append(borrower)
        with self.assertRaisesRegex(RuntimeError, "drain failed"):
            next(generator)
        self.observe.assert_not_called()
        self.record_drain.assert_not_called()

    def test_successful_setup_does_not_report_failure(self) -> None:
        generator = self.generator()
        fixture = next(generator)
        self.assertEqual(fixture.resource, "resource")
        with self.assertRaises(StopIteration):
            next(generator)
        self.observe.assert_not_called()
        self.record_drain.assert_called_once()


if __name__ == "__main__":
    unittest.main()
