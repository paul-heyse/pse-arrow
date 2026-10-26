# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Concrete generic header validation without scientific package startup."""

# ruff: noqa: PT009, PT027 -- source-only stdlib unittest controls

from __future__ import annotations

import unittest

from pydantic import JsonValue, ValidationError

from scripts.generated.native_operations import ENVELOPES, ProfileCaptureRequest
from scripts.host_contracts import Contract


class SuppliedSuspect(Contract):
    """An independent typed host payload, unrelated to scientific documents."""

    source: str


class SuppliedCapture(ProfileCaptureRequest[SuppliedSuspect]):
    """Concrete host association without a mutable field override."""


class HostOperationGenericTests(unittest.TestCase):
    """Actual generated defaults and explicit host specializations stay concrete."""

    def test_default_binder_retains_original_json_payload_and_unknown_refusal(
        self,
    ) -> None:
        model = ENVELOPES["ProfileCaptureRequest"]
        self.assertIs(model.model_fields["suspect"].annotation, JsonValue)
        value = {
            "expected": None,
            "request": None,
            "suspect": {"original": [1, True, None]},
        }
        self.assertEqual(model.model_validate(value).model_dump(mode="json"), value)
        with self.assertRaises(ValidationError):
            model.model_validate({**value, "unknown": True})

    def test_specialization_validates_original_typed_payload_and_schema(self) -> None:
        value = {"expected": None, "request": None, "suspect": {"source": "original"}}
        received = SuppliedCapture.model_validate(value)
        self.assertEqual(received.suspect.source, "original")
        self.assertEqual(received.model_dump(mode="json"), value)
        schema = SuppliedCapture.model_json_schema()
        self.assertEqual(
            schema["properties"]["suspect"]["$ref"], "#/$defs/SuppliedSuspect"
        )
        for suspect in ({"source": 1}, {"unknown": "original"}):
            with self.subTest(suspect=suspect), self.assertRaises(ValidationError):
                SuppliedCapture.model_validate({**value, "suspect": suspect})


if __name__ == "__main__":
    unittest.main()
