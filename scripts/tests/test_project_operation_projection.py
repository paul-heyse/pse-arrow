# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent public-interface/default/name and actual forwarding projection controls."""

# ruff: noqa: PT009, PT027 -- stdlib host controls
from __future__ import annotations

import ast
import importlib.util
import inspect
import sys
import tempfile
import types
import typing
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.project_operation_projection import render, validate_python_floor

if typing.TYPE_CHECKING:
    from pydantic import JsonValue


class ProjectProjectionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "pyproject.toml").write_text(
            '[project]\nrequires-python = ">=3.11"\n[tool.ruff]\nline-length = 88\n'
        )
        self.sources = {
            "pse._project_operations_glue": '''class ProjectRuntimeGlue:
    """Original project owner documentation."""
    def _host_project_create(self, operation: "str", *, branch_name: str) -> tuple[str, str]:
        """Original typed receiver doc."""
        return operation, branch_name
    def _host_unpublished(self):
        raise AssertionError("unpublished")
''',
            "pse._research": """class ResearchRuntimeGlue:
    def _host_research_read(self, request: str) -> str:
        return request
""",
            "pse._study_workbench": """class StudyWorkbenchGlue:
    def _host_observe_study(self, request: str, *, cursor: str | None) -> tuple[str, str | None]:
        return request, cursor
""",
        }
        for module, source in self.sources.items():
            path = self.root / "python" / (module.replace(".", "/") + ".py")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source)
        self.declarations: list[dict[str, JsonValue]] = [
            {
                "name": "project_create",
                "family": "project",
                "pyo3_signature": "signature=(operation,*,branch_name=pse_runtime::host_operations::project::DEFAULT_BRANCH)",
            },
            {
                "name": "research_read",
                "family": "research",
                "pyo3_signature": None,
                "receiver_description": "Original Rust research receiver documentation.",
            },
            {
                "name": "observe_study",
                "family": "study",
                "pyo3_signature": "signature=(request,*,cursor=None)",
                "receiver_description": "Original Rust study receiver documentation.",
            },
        ]
        self.declarations[0]["effect_facts"] = {
            "scope": "canonical_command",
            "read_only": False,
            "idempotent": True,
            "destructive": False,
        }
        self.declarations[1]["effect_facts"] = {
            "scope": "observation",
            "read_only": True,
            "idempotent": True,
            "destructive": False,
        }
        self.declarations[2]["effect_facts"] = {
            "scope": "capture",
            "read_only": False,
            "idempotent": False,
            "destructive": False,
        }

    def test_public_wrapper_preserves_metadata_owned_defaults_and_exact_native_arguments(
        self,
    ) -> None:
        generated = render(self.root, self.declarations)
        modules = {}
        for name in self.sources:
            path = self.root / "python" / (name.replace(".", "/") + ".py")
            modules[name] = self._load_module(name, path)
        defaults = types.ModuleType("pse.generated.project_operation_defaults")
        vars(defaults)["DEFAULT_BRANCH"] = "main"
        modules[defaults.__name__] = defaults
        candidate = self.root / "project_projection_candidate.py"
        candidate.write_text(generated)
        with patch.dict(sys.modules, modules):
            public = self._load_module("_project_projection_candidate", candidate)
        instance = public.ProjectRuntimeOperations()
        method = public.ProjectRuntimeOperations.project_create
        self.assertTrue(inspect.ismethod(instance.project_create))
        self.assertIs(instance.project_create.__self__, instance)
        self.assertEqual(instance.project_create("original"), ("original", "main"))
        self.assertEqual(
            instance.project_create("original", branch_name="selected"),
            ("original", "selected"),
        )
        self.assertEqual(method.__name__, "project_create")
        self.assertEqual(method.__qualname__, "Runtime.project_create")
        self.assertEqual(method.__module__, "pse._workflow")
        self.assertTrue(method.__doc__.startswith("Original typed receiver doc.\n\n"))
        self.assertIn(
            "Effects:\n    scope: canonical_command\n    read_only: false\n    idempotent: true\n    destructive: false",
            method.__doc__,
        )
        self.assertEqual(method.__annotations__["operation"], "str")
        self.assertIs(typing.get_type_hints(method)["operation"], str)
        self.assertEqual(
            public.ProjectRuntimeOperations.__doc__,
            "Original project owner documentation.",
        )
        if public.__doc__ is None:
            self.fail("generated public module must retain its documentation")
        self.assertIn("Public project operations", public.__doc__)
        self.assertEqual(
            inspect.signature(method).parameters["branch_name"].default, "main"
        )
        self.assertFalse(hasattr(instance, "unpublished"))
        self.assertEqual(
            public.StudyWorkbenchMixin().observe_study("study"), ("study", None)
        )
        self.assertTrue(
            public.StudyWorkbenchMixin.observe_study.__doc__.startswith(
                "Original Rust study receiver documentation.\n\n"
            )
        )
        self.assertIn(
            "Effects:\n    scope: capture\n    read_only: false\n    idempotent: false\n    destructive: false",
            public.StudyWorkbenchMixin.observe_study.__doc__,
        )

    def test_missing_helper_and_unknown_default_refuse_generation(self) -> None:
        for changed in [
            {"name": "unknown_operation"},
            {
                "pyo3_signature": "signature=(operation,*,branch_name=arbitrary_function())"
            },
        ]:
            declarations = [{**self.declarations[0], **changed}, *self.declarations[1:]]
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                render(self.root, declarations)

    def test_generated_helper_obeys_manifest_python_floor_and_refuses_newer_syntax(
        self,
    ) -> None:
        generated = render(self.root, self.declarations)
        validate_python_floor(self.root, generated)
        self.assertNotIn("def _public_metadata[", generated)
        newer = "def newer[T](value: T) -> T:\n    return value\n"
        with self.assertRaises(SyntaxError):
            validate_python_floor(self.root, newer)
        (self.root / "pyproject.toml").write_text(
            '[project]\nrequires-python = ">=3.12"\n'
        )
        validate_python_floor(self.root, newer)

    def test_effects_require_typed_owner_facts_and_missing_docs_refuse(self) -> None:
        changed = [{**row} for row in self.declarations]
        changed[0]["effect_facts"] = {
            "scope": "observation",
            "read_only": "false",
            "idempotent": True,
            "destructive": False,
        }
        with self.assertRaises(TypeError):
            render(self.root, changed)
        changed = [{**row} for row in self.declarations]
        changed[2]["receiver_description"] = ""
        with self.assertRaises(ValueError):
            render(self.root, changed)

    def test_long_original_documentation_wraps_without_losing_scientific_prose(
        self,
    ) -> None:
        path = self.root / "python/pse/_project_operations_glue.py"
        original = "Original typed receiver doc.\n\n    Keep the original intent until artifact and project acknowledgements settle. Scientific publication separately requires the original typed document codec."
        path.write_text(
            path.read_text().replace("Original typed receiver doc.", original)
        )
        tree = ast.parse(render(self.root, self.declarations))
        owner = next(
            node
            for node in tree.body
            if isinstance(node, ast.ClassDef)
            and node.name == "ProjectRuntimeOperations"
        )
        method = next(node for node in owner.body if isinstance(node, ast.FunctionDef))
        doc = ast.get_docstring(method)
        if doc is None:
            self.fail(
                "generated method must preserve original scientific documentation"
            )
        body = doc.split("\n\nEffects:")[0]
        self.assertEqual(" ".join(body.split()), " ".join(original.split()))
        self.assertTrue(all(len(line) <= 77 for line in body.splitlines()))

    def _load_module(self, name: str, path: Path) -> types.ModuleType:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            self.fail(
                "trusted generated module requires its ordinary file-backed loader"
            )
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module


if __name__ == "__main__":
    unittest.main()
