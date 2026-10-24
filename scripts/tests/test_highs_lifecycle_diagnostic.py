# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""A successful command alone does not qualify native instrumentation."""
import unittest
from scripts.highs_lifecycle_diagnostic import qualify_asan


class LifecycleQualificationTests(unittest.TestCase):
    def test_normal_lifecycle_needs_both_instrumented_suppliers_and_actual_completion(self):
        normal={"exit_code":0,"timed_out":False}
        self.assertTrue(qualify_asan(normal,True,True))
        self.assertFalse(qualify_asan(normal,False,True))
        self.assertFalse(qualify_asan(normal,True,False))
        self.assertFalse(qualify_asan({"exit_code":-9,"timed_out":True},True,True))
        self.assertFalse(qualify_asan({"exit_code":1,"timed_out":False},True,True))
