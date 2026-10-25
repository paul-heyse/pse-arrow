# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Copied population pages never acquire live snapshot or scientific authority."""

# ruff: noqa: PT009, PT027 -- stdlib host controls
from __future__ import annotations

import unittest
from unittest.mock import patch

from scripts import telemetry_selection as selection
from scripts.telemetry_investigation import QueryAnswer, QueryRequest
from scripts.capabilities.read_contracts import InvocationReference


class CopiedPages(unittest.TestCase):
    def setUp(self) -> None:
        self.reference = selection.SelectionReference(
            directory="/explicit/supplied", sha256="a" * 64
        )
        self.request = QueryRequest(
            signal="metrics", metric="work", start_time_us=10, end_time_us=20
        )
        self.document = selection.Investigation(
            created_at_ns=123,
            question="Which reported points remain available after reconnect?",
            schema_description={},
            selections=[
                QueryAnswer(
                    request=self.request,
                    status="partial",
                    records=[{"_timestamp": 15, "id": i} for i in range(5)],
                    returned_rows=5,
                    limits=["source cut is not atomic"],
                )
            ],
            missing_context=["related trace expired"],
        )

    def test_equal_keys_page_two_two_one_without_backend_or_current_time(self) -> None:
        with (
            patch.object(selection, "read_document", return_value=self.document),
            patch.object(
                selection.time,
                "time_ns",
                side_effect=AssertionError("offline clock read"),
            ),
        ):
            first = selection.page(self.reference, limit=2)
            second = selection.page(self.reference, limit=2, cursor=first.next_cursor)
            third = selection.page(self.reference, limit=2, cursor=second.next_cursor)
        self.assertEqual(
            [row["id"] for page in (first, second, third) for row in page.records],
            list(range(5)),
        )
        self.assertEqual(
            [page.returned_rows for page in (first, second, third)], [2, 2, 1]
        )
        self.assertTrue(third.copied_selection_complete)
        self.assertFalse(third.live_population_complete)
        self.assertIsNone(third.next_cursor)
        self.assertIn("related trace expired", third.missing_context)
        self.assertEqual(third.question, self.document.question)

    def test_question_handoff_keeps_original_readers_unexecuted(self) -> None:
        reference = InvocationReference(kind="invocation", invocation="authentic-id", registry="/original/registry")
        suggestion = selection.ReaderSuggestion(reference=reference, reason="Inspect original settlement")
        document = self.document.model_copy(update={"original_references": [reference], "next_readers": [suggestion]})
        with patch.object(selection, "read_document", return_value=document):
            page = selection.page(self.reference)
        self.assertEqual(page.original_references[0], reference)
        self.assertEqual(page.next_readers[0].disposition, "unexecuted")
        self.assertFalse(page.live_population_complete)
        raw = document.model_dump(mode="json")
        raw["version"] = 1
        with self.assertRaises(ValueError):
            selection.Investigation.model_validate(raw)

    def test_continuation_binds_original_bytes_selection_and_page_request(self) -> None:
        with patch.object(selection, "read_document", return_value=self.document):
            first = selection.page(self.reference, limit=2)
            for arguments in [{"limit": 3}, {"selection": 1, "limit": 2}]:
                with self.assertRaises(ValueError):
                    selection.page(
                        self.reference, cursor=first.next_cursor, **arguments
                    )
            changed = self.reference.model_copy(update={"sha256": "b" * 64})
            with self.assertRaises(ValueError):
                selection.page(changed, limit=2, cursor=first.next_cursor)
            with self.assertRaises(ValueError):
                selection.page(self.reference, cursor="invalid")

    def test_resource_reference_roundtrip_is_exact_and_bounded(self) -> None:
        self.assertEqual(
            selection.from_resource(selection.resource_reference(self.reference)),
            self.reference,
        )
        with self.assertRaises(ValueError):
            selection.from_resource("a" * 8193)


if __name__ == "__main__":
    unittest.main()
