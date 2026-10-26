# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual retained object identity and retirement, without product imports."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import unittest

from scripts.capabilities.native_read_context import NativeReadContext, ReadContextChanged


class ReadContextTests(unittest.TestCase):
    def test_successive_pages_retain_the_actual_runtime_physical_and_package(self) -> None:
        context = NativeReadContext("source-physical-provider-preparation-image")
        created: list[str] = []
        closed: list[object] = []

        def create(name: str) -> object:
            created.append(name)
            return object()

        runtime = context.runtime("deployment-and-resources", lambda: create("runtime"), closed.append)
        physical = context.physical("physical-original-bytes", lambda: create("physical"))
        package = context.modeling("physical-plus-model-original-bytes", lambda: create("modeling"))
        for _page in range(3):
            self.assertIs(context.runtime("deployment-and-resources", lambda: create("wrong-runtime"), closed.append), runtime)
            self.assertIs(context.physical("physical-original-bytes", lambda: create("wrong-physical")), physical)
            self.assertIs(context.modeling("physical-plus-model-original-bytes", lambda: create("wrong-modeling")), package)
        self.assertEqual(created, ["runtime", "physical", "modeling"])
        context.close()
        context.close()
        self.assertEqual(closed, [runtime])
        with self.assertRaises(ReadContextChanged):
            context.modeling("physical-plus-model-original-bytes", lambda: create("after-close"))

    def test_changed_admitted_or_runtime_context_refuses_before_factory(self) -> None:
        context = NativeReadContext("original-context")
        context.runtime("original-deployment", object, lambda _runtime: None)
        with self.assertRaises(ReadContextChanged):
            context.require("changed-context")
        with self.assertRaises(ReadContextChanged):
            context.runtime("changed-deployment", lambda: self.fail("changed Runtime was constructed"), lambda _runtime: None)
        context.close()

    def test_failed_physical_creation_is_not_retained_and_runtime_close_failure_survives(self) -> None:
        context = NativeReadContext("context")
        closes: list[str] = []

        def close(_runtime: object) -> None:
            closes.append("close")
            raise RuntimeError("original-native-drain-failure")

        context.runtime("runtime", object, close)
        with self.assertRaisesRegex(ValueError, "source-refused"):
            context.physical("physical", lambda: (_ for _ in ()).throw(ValueError("source-refused")))
        physical = object()
        self.assertIs(context.physical("physical", lambda: physical), physical)
        with self.assertRaisesRegex(RuntimeError, "original-native-drain-failure"):
            context.close()
        self.assertTrue(context.closed)
        context.close()
        self.assertEqual(closes, ["close"])


if __name__ == "__main__":
    unittest.main()
