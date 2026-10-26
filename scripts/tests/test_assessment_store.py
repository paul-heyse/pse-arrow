# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Storage-shaped assessment reads and interrupted publication controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import assessment_store as store
from scripts import resource_store, service_borrowers, test_resources, validation_result
from scripts.tests.assessment_fixture import publish


class AssessmentStoreTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.assessment_dir = Path(temporary.name)
        self.enterContext(
            patch.object(
                service_borrowers,
                "registry",
                return_value=self.assessment_dir / "registry",
            )
        )
        self.receipt = publish(
            self.assessment_dir,
            {
                "complete": True,
                "source_files": {"source/" + str(i): "a" * 64 for i in range(20000)},
                "scope": [
                    {"name": "chosen"},
                    {"name": "other"},
                    {"name": "unreported"},
                ],
                "checks": [
                    {
                        "gate": "chosen",
                        "status": "failed",
                        "results": [{"nodeid": "selected", "status": "failure"}],
                        "report_errors": ["original cause"],
                    },
                    {
                        "gate": "other",
                        "status": "passed",
                        "results": [
                            {"nodeid": str(i), "status": "passed"} for i in range(10000)
                        ],
                    },
                ],
            },
        )

    def test_summary_and_selected_gate_do_not_open_source_or_unrelated_detail(
        self,
    ) -> None:
        root = store.root(self.assessment_dir)
        self.assertLess((self.assessment_dir / "checks.json").stat().st_size, 5000)
        self.assertNotIn("source_files", root)
        chosen = root["checks"][0]["detail"]
        original = store.detail
        with patch.object(
            store,
            "detail",
            side_effect=lambda run, ref: (
                original(run, ref)
                if ref == chosen
                else self.fail("unrelated detail opened")
            ),
        ) as read:
            summary = validation_result.read_assessment(self.assessment_dir)
            self.assertEqual(read.call_count, 0)
            self.assertEqual(summary["assessment"]["complete"], True)
            self.assertEqual(
                summary["assessment"]["reader_custody"]["status"],
                "unregistered-inspection",
            )
            self.assertFalse(summary["assessment"]["reader_custody"]["automatic_reuse"])
            selected = validation_result.read_assessment(
                self.assessment_dir, gate="chosen"
            )
            self.assertEqual(read.call_count, 1)
        self.assertEqual(
            selected["checks"][0]["record"]["report_errors"], ["original cause"]
        )
        self.assertEqual([row["name"] for row in selected["unattempted_scope"]], [])
        self.assertEqual(summary["unattempted_scope"], [{"name": "unreported"}])
        self.assertEqual(store.export(self.assessment_dir), self.receipt)

    def test_dangling_and_corrupt_selected_detail_refuse_without_empty_success(
        self,
    ) -> None:
        root = store.root(self.assessment_dir)
        path = (
            self.assessment_dir
            / "details"
            / (root["checks"][0]["detail"]["sha256"] + ".json")
        )
        original = path.read_bytes()
        path.write_text("{}")
        with self.assertRaisesRegex(ValueError, "digest mismatch"):
            validation_result.read_assessment(self.assessment_dir, gate="chosen")
        path.write_bytes(original)
        path.unlink()
        with self.assertRaises(FileNotFoundError):
            validation_result.read_assessment(self.assessment_dir, failures=True)
        # An explicitly selected unrelated summary still works; unavailable detail
        # cannot erase the original recorded population or imply Pass.
        self.assertFalse(
            validation_result.read_assessment(self.assessment_dir)["checks"][0][
                "qualified"
            ]
        )

    def test_failed_root_replacement_keeps_previous_complete_cut(self) -> None:
        original = (self.assessment_dir / "checks.json").read_bytes()
        changed = {**self.receipt, "complete": False, "source_files": {"new": "b"}}
        replace = Path.replace

        def fail(path: Path, destination: Path) -> Path:
            if destination == self.assessment_dir / "checks.json":
                raise OSError("injected root publication failure")
            return replace(path, destination)

        with (
            patch.object(Path, "replace", fail),
            self.assertRaisesRegex(OSError, "injected"),
        ):
            store.publish(self.assessment_dir, changed)
        self.assertEqual((self.assessment_dir / "checks.json").read_bytes(), original)
        self.assertEqual(store.export(self.assessment_dir), self.receipt)

    def test_historical_bytes_have_explicit_inspection_and_no_automatic_reuse(
        self,
    ) -> None:
        path = self.assessment_dir / "checks.json"
        old = {**self.receipt, "version": 5}
        resource_store.publish(
            path, old
        ) if self.assessment_dir.stat().st_mode & 0o077 == 0 else path.write_text(
            json.dumps(old)
        )
        with self.assertRaisesRegex(ValueError, "automatic reuse refused"):
            store.root(self.assessment_dir)
        self.assertEqual(store.original_export(self.assessment_dir), old)
        historical = validation_result.read_assessment(
            self.assessment_dir, historical=True, full_export=True
        )
        self.assertFalse(historical["assessment"]["automatic_reuse"])

    def sealed(self, name: str = "sealed") -> tuple[Path, str]:
        directory = self.assessment_dir / "build" / name
        directory.mkdir(mode=0o700, parents=True)
        receipt = json.loads(json.dumps(self.receipt))
        publish(directory, receipt)
        (directory / "provenance.json").write_text("{}")
        resource = test_resources.register_report(directory, root=self.assessment_dir)
        test_resources.finish_report(
            resource,
            receipt,
            roles={"checks.json": "receipt", "provenance.json": "provenance"},
            required_provenance=["provenance.json"],
        )
        return directory, resource

    def test_sealed_root_status_and_completeness_tamper_cannot_report_false_pass(
        self,
    ) -> None:
        self.receipt["complete"] = False
        directory, _ = self.sealed()
        root = directory / "checks.json"
        original = root.read_bytes()
        self.assertEqual(store.root(directory)["reader_custody"]["status"], "sealed")
        for field in ("status", "complete", "required_checks_covered"):
            with self.subTest(field=field):
                image = json.loads(original)
                if field == "status":
                    image["checks"][0]["status"] = "passed"
                elif field == "complete":
                    image["complete"] = True
                else:
                    image["required_checks_covered"] = True
                # Mutate the same inode: a path/identity check alone is insufficient.
                root.write_text(json.dumps(image))
                with self.assertRaisesRegex(ValueError, "Sealed assessment artifact"):
                    validation_result.read_assessment(directory)
                root.write_bytes(original)
        self.assertEqual(store.root(directory)["checks"][0]["status"], "failed")
        self.assertFalse(store.root(directory)["complete"])

    def test_same_bytes_in_a_replacement_root_are_not_original_custody(self) -> None:
        directory, _ = self.sealed("replaced")
        root = directory / "checks.json"
        original = root.read_bytes()
        root.rename(directory / "retained-original")
        descriptor = os.open(root, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(original)
        with self.assertRaisesRegex(ValueError, "ownership or digest"):
            store.root(directory)

    def test_selected_origin_reference_does_not_open_unrelated_large_details(
        self,
    ) -> None:
        directory, resource = self.sealed("selected-reuse")
        image = store.root(directory)
        chosen = image["checks"][0]["detail"]
        original = store.detail
        consumer = self.assessment_dir / "build" / "consumer"
        with (
            patch.object(
                test_resources,
                "verify_manifest_files",
                side_effect=AssertionError("unselected full integrity"),
            ),
            patch.object(
                store,
                "detail",
                side_effect=lambda run, ref: (
                    original(run, ref)
                    if ref == chosen
                    else self.fail("unselected large detail opened")
                ),
            ),
            test_resources.reference_report(directory, consumer),
        ):
            record = test_resources.resource_status(resource)
            self.assertTrue(record["borrows"])
            self.assertIn(str(consumer.resolve()), record["references"])
            selected = store.selected(
                directory, store.root(directory), gates={"chosen"}
            )
            self.assertEqual(selected["checks"][0]["report_errors"], ["original cause"])
        self.assertTrue(test_resources.resource_status(resource)["references"])

    def test_full_integrity_is_explicit_and_detects_unselected_detail_tamper(
        self,
    ) -> None:
        directory, _ = self.sealed("whole-integrity")
        image = store.root(directory)
        unrelated = (
            directory / "details" / (image["checks"][1]["detail"]["sha256"] + ".json")
        )
        unrelated.write_text("{}")
        with test_resources.reference_report(directory, None):
            selected = store.selected(
                directory, store.root(directory), gates={"chosen"}
            )
            self.assertEqual(selected["checks"][0]["status"], "failed")
        with (
            self.assertRaisesRegex(ValueError, "ownership changed"),
            test_resources.reference_report(directory, None, full_integrity=True),
        ):
            self.fail("corrupt full archive was exposed")
        with self.assertRaisesRegex(ValueError, "ownership or digest"):
            validation_result.read_assessment(directory, gate="other")

    def test_reader_parses_the_same_authenticated_root_bytes_across_replacement(
        self,
    ) -> None:
        directory, _ = self.sealed("read-race")
        root = directory / "checks.json"
        original_read = test_resources.read_report_root

        def replace_after_observation(
            *_args: object, **_kwargs: object
        ) -> tuple[bytes, dict[str, object]]:
            observed = original_read(directory)
            changed = json.loads(root.read_bytes())
            changed["checks"][0]["status"] = "passed"
            root.write_text(json.dumps(changed))
            return observed

        with patch.object(
            test_resources, "read_report_root", side_effect=replace_after_observation
        ):
            image = store.root(directory)
        self.assertEqual(image["checks"][0]["status"], "failed")
        self.assertEqual(image["reader_custody"]["status"], "sealed")
        with self.assertRaisesRegex(ValueError, "ownership or digest"):
            store.root(directory)

    def test_same_byte_selected_detail_replacement_refuses_but_summary_stays_available(
        self,
    ) -> None:
        directory, _ = self.sealed("detail-replacement")
        image = store.root(directory)
        detail = (
            directory / "details" / (image["checks"][0]["detail"]["sha256"] + ".json")
        )
        original = detail.read_bytes()
        detail.rename(directory / "retained-original-detail")
        descriptor = os.open(detail, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(original)
        summary = validation_result.read_assessment(directory)
        self.assertEqual(summary["checks"][0]["record"]["status"], "failed")
        with self.assertRaisesRegex(ValueError, "ownership or digest"):
            validation_result.read_assessment(directory, gate="chosen")

    def test_registered_unknown_directory_custody_is_inspection_only(self) -> None:
        directory, resource = self.sealed("unknown-owner")
        with test_resources.resource_update(resource) as record:
            for key in (
                "artifact_manifest",
                "artifact_policy_version",
                "owner_root",
                "directory_identity",
            ):
                record.pop(key)
        image = store.root(directory)
        self.assertEqual(
            image["reader_custody"]["status"], "registered-unqualified-inspection"
        )
        self.assertFalse(image["reader_custody"]["automatic_reuse"])
        with (
            self.assertRaisesRegex(ValueError, "sealed receipt"),
            test_resources.reference_report(directory, None),
        ):
            self.fail("unknown custody supplied reuse")

    def test_known_historical_root_is_authenticated_without_relabeling_original(
        self,
    ) -> None:
        directory, resource = self.sealed("historical")
        original = {**self.receipt, "version": 5}
        resource_store.publish(directory / "checks.json", original)
        # Establish an actual historical sealed fixture; current production
        # publishers continue to accept only version6 assessments.
        with test_resources.resource_update(resource) as record:
            with test_resources.report_directory(record) as descriptor:
                artifact = test_resources.observe_artifact(descriptor, "checks.json")
            artifact["role"] = "receipt"
            record["artifact_manifest"]["entries"]["checks.json"] = artifact
            record["artifact_manifest"]["receipt_digest"] = artifact["digest"]
        self.assertEqual(store.original_export(directory), original)
        inspection = validation_result.read_assessment(
            directory, historical=True, full_export=True
        )
        self.assertTrue(inspection["assessment"]["original_inspection"])
        self.assertFalse(inspection["assessment"]["automatic_reuse"])
        self.assertNotIn("reader_custody", store.original_export(directory))
        changed = {**original, "complete": not original["complete"]}
        (directory / "checks.json").write_text(json.dumps(changed))
        with self.assertRaisesRegex(ValueError, "ownership or digest"):
            store.original_export(directory)
        with self.assertRaisesRegex(ValueError, "ownership or digest"):
            validation_result.read_assessment(
                directory, historical=True, full_export=True
            )

    def test_existing_detail_retry_settles_directory_before_publishing_root(
        self,
    ) -> None:
        changed = {**self.receipt, "source_files": {"new-source": "b" * 64}}
        source = {
            key: value
            for key, value in changed.items()
            if key not in store.SUMMARY | {"scope", "checks"}
        }
        selected = (
            self.assessment_dir / "details" / (resource_store.digest(source) + ".json")
        )
        before = (self.assessment_dir / "checks.json").read_bytes()
        sync = resource_store.sync_directory
        failed = False

        def interrupt(directory: Path) -> None:
            nonlocal failed
            if directory == selected.parent and selected.exists() and not failed:
                failed = True
                raise OSError("injected detail directory fsync failure after rename")
            sync(directory)

        with (
            patch.object(resource_store, "sync_directory", side_effect=interrupt),
            self.assertRaisesRegex(OSError, "after rename"),
        ):
            store.publish(self.assessment_dir, changed)
        self.assertTrue(selected.exists())
        self.assertEqual((self.assessment_dir / "checks.json").read_bytes(), before)
        order = []
        settle = resource_store.settle
        replace = Path.replace

        def settled(path: Path) -> None:
            if path == selected:
                order.append("detail-settled")
            settle(path)

        def replaced(path: Path, destination: Path) -> Path:
            if destination == self.assessment_dir / "checks.json":
                order.append("root-published")
            return replace(path, destination)

        with (
            patch.object(resource_store, "settle", side_effect=settled),
            patch.object(Path, "replace", replaced),
        ):
            store.publish(self.assessment_dir, changed)
        self.assertEqual(order, ["detail-settled", "root-published"])
        self.assertEqual(store.export(self.assessment_dir), changed)
