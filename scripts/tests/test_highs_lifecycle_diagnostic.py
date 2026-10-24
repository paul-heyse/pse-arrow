# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Sanitizer adoption requires both positive lifecycle and relevant sensitivity."""
import unittest
from scripts.highs_lifecycle_diagnostic import qualify_asan


class LifecycleQualificationTests(unittest.TestCase):
    def test_installation_crash_or_timeout_is_not_sensitive_native_qualification(self):
        normal={"exit_code":0,"timed_out":False}
        fault={"exit_code":1,"timed_out":False}
        diagnostic="ERROR: AddressSanitizer: heap-use-after-free"
        self.assertTrue(qualify_asan(normal,fault,diagnostic,True,True))
        self.assertFalse(qualify_asan(normal,fault,"segmentation fault",True,True))
        self.assertFalse(qualify_asan(normal,fault,diagnostic,False,True))
        self.assertFalse(qualify_asan(normal,fault,diagnostic,True,False))
        self.assertFalse(qualify_asan(normal,{"exit_code":-9,"timed_out":True},diagnostic,True,True))
        self.assertFalse(qualify_asan({"exit_code":1,"timed_out":False},fault,diagnostic,True,True))
