# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent invocation documents retain result, readability and protection meanings."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import (
    resource_store,
    run_inspection,
    test_resources,
    test_run,
    validation_result,
)


class RunInspectionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.registry = self.root / "registry"
        selected = patch.object(test_resources, "registry", return_value=self.registry)
        selected.start()
        self.addCleanup(selected.stop)
        self.path = test_resources.invocation(
            "python", [{"name": "suite::one"}, {"name": "suite::two"}]
        )

    def freeze(
        self,
        *,
        code: int = 0,
        errors: list[str] | None = None,
        rows: list | None = None,
        **extra: object,
    ) -> None:
        test_run.finish(
            self.path,
            {
                "exit_code": code,
                "results": rows
                if rows is not None
                else [
                    {"nodeid": "suite::one", "status": "passed"},
                    {"nodeid": "suite::two", "status": "passed"},
                ],
                "report_errors": errors or [],
                **extra,
            },
            "runner",
        )

    def read(
        self,
        *,
        registry: Path | None = None,
        subject: str | None = None,
        failures: bool = False,
        offset: int = 0,
        limit: int = 100,
        tail: int = 0,
    ) -> dict:
        return run_inspection.product_result(
            str(self.path),
            registry=registry,
            subject=subject,
            failures=failures,
            offset=offset,
            limit=limit,
            tail=tail,
        )

    def test_collected_empty_selection_is_known_before_terminal_publication(
        self,
    ) -> None:
        self.path = test_resources.invocation("rust", [])
        self.assertEqual(self.read()["selection_state"], "uncollected")
        test_resources.publish_selection(self.path, [])
        result = self.read()
        self.assertEqual(result["selection_state"], "collected")
        self.assertEqual(result["selected_count"], 0)
        self.assertEqual(result["terminal_status"], "missing")

    def test_pass_and_complete_failure_have_authentic_frozen_identity(self) -> None:
        self.freeze(
            code=1,
            rows=[
                {"nodeid": "suite::one", "status": "failure"},
                {"nodeid": "suite::two", "status": "passed"},
            ],
        )
        result = self.read()
        self.assertTrue(result["terminal"]["complete"])
        self.assertEqual(result["workload_exit"], 1)
        self.assertEqual(
            result["terminal_identity"],
            test_resources.storage().terminal(self.path.stem)[0],
        )
        self.assertEqual(result["selected_unreported"], [])
        self.assertEqual(self.read(failures=True)["subjects"][0]["outcome"], "failure")
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.assertEqual(
                validation_result.main(
                    [str(self.path), "--kind", "product-test", "--json"]
                ),
                0,
            )
        self.assertEqual(json.loads(output.getvalue())["workload_exit"], 1)

    def test_interruption_and_missing_terminal_never_invent_subject_outcomes(
        self,
    ) -> None:
        missing = self.read()
        self.assertEqual(missing["terminal_status"], "missing")
        self.assertIsNone(missing["terminal"])
        self.freeze(
            code=130,
            errors=["missing report"],
            rows=[{"nodeid": "suite::one", "status": "passed"}],
            process={"forwarded_signal": 2},
            owned_drain="unresolved",
        )
        result = self.read()
        self.assertFalse(result["terminal"]["complete"])
        self.assertEqual(result["selected_unreported"], ["suite::two"])
        self.assertEqual(result["subjects"][1]["outcome"], "unreported")
        self.assertEqual(result["process"]["forwarded_signal"], 2)
        self.assertEqual(result["owned_drain"], "unresolved")

    def test_frozen_report_mismatch_and_malformed_authority_are_explicit(self) -> None:
        self.freeze(rows=[{"nodeid": "other", "status": "passed"}])
        result = self.read()
        self.assertFalse(result["terminal"]["complete"])
        self.assertIn("outside selected catalog", result["report_errors"][0])
        state_path = self.registry / "invocation-state" / self.path.name
        image = json.loads(state_path.read_text())
        detail_path = (
            self.registry / "invocation-terminals" / (image["terminal_ref"] + ".json")
        )
        detail = resource_store.read(detail_path)
        detail["terminal"]["complete"] = True
        resource_store.publish(detail_path, detail)
        with self.assertRaisesRegex(ValueError, "Invalid frozen"):
            self.read()

    def test_bounded_exact_selection_and_registry_refusal(self) -> None:
        self.freeze()
        self.assertEqual(
            self.read(offset=1, limit=1)["subjects"][0]["identity"], "suite::two"
        )
        exact = run_inspection.product_result(self.path.stem, registry=self.registry)
        self.assertEqual(exact["invocation"], self.path.stem)
        with self.assertRaisesRegex(ValueError, "selected registry"):
            self.read(registry=self.root / "unrelated")
        with self.assertRaisesRegex(ValueError, "selected catalog"):
            self.read(subject="invented")

    def test_selection_detail_is_hydrated_only_after_subject_paging(self) -> None:
        self.freeze()
        original = run_inspection.resolve
        requests = []

        class Requirements(dict):
            def get(self, identity: str, default: object = None) -> object:
                requests.append(identity)
                return super().get(identity, default)

        def resolve(handle: str, registry: Path | None) -> tuple:
            root, declaration, state = original(handle, registry)
            state["terminal"]["evidence"]["requirements"] = Requirements(
                {"suite::one": ["first"], "suite::two": ["second"]}
            )
            return root, declaration, state

        with patch.object(run_inspection, "resolve", side_effect=resolve):
            selected = self.read(offset=1, limit=1)
        self.assertEqual(requests, ["suite::two"])
        self.assertEqual(selected["page"]["subject_count"], 2)
        self.assertEqual(selected["selected_count"], 2)
        self.assertEqual(selected["subjects"][0]["requirements"], ["second"])

    def test_detached_metadata_does_not_initialize_or_modify_registry(self) -> None:
        before = {p: p.read_bytes() for p in self.registry.rglob("*") if p.is_file()}
        with patch.object(
            resource_store.FileStore, "ensure", side_effect=AssertionError("ensure")
        ):
            self.read()
        self.assertEqual(
            before, {p: p.read_bytes() for p in self.registry.rglob("*") if p.is_file()}
        )

    def test_payload_tail_holds_report_borrow_until_read_and_releases_it(self) -> None:
        directory = self.root / "build" / "report"
        directory.mkdir(parents=True)
        log = directory / "failure.log"
        log.write_text("old\nnew\n")
        resource = test_resources.register_report(directory, root=self.root)
        self.freeze(
            code=1,
            artifacts={"failure.log": test_resources.file_digest(log)},
            artifact_base=str(directory),
        )
        original = run_inspection.artifact_observation

        def guarded(
            path: Path,
            digest: str,
            lines: int,
            record: test_resources.ResourceRecord | None,
        ) -> dict:
            self.assertTrue(test_resources.resource_status(resource)["borrows"])
            return original(path, digest, lines, record)

        with patch.object(run_inspection, "artifact_observation", side_effect=guarded):
            self.assertEqual(self.read(tail=1)["payloads"][0]["tail"], ["new"])
        self.assertEqual(test_resources.resource_status(resource)["borrows"], {})
        with (
            patch.object(
                run_inspection,
                "artifact_observation",
                side_effect=OSError("read error"),
            ),
            self.assertRaises(OSError),
        ):
            self.read(tail=1)
        self.assertEqual(test_resources.resource_status(resource)["borrows"], {})
        log.write_text("changed payload")
        self.assertEqual(self.read(tail=1)["payloads"][0]["status"], "changed")
        log.unlink()
        log.symlink_to(self.root / "outside")
        (self.root / "outside").write_text("not owned")
        self.assertEqual(self.read(tail=1)["payloads"][0]["status"], "unavailable")

    def test_artifact_page_is_selected_before_protected_tail_hydration(self) -> None:
        directory = self.root / "build" / "paged-report"
        directory.mkdir(parents=True)
        artifacts = {}
        for name in ("first.log", "second.log", "third.log"):
            path = directory / name
            path.write_text(name + "\n")
            artifacts[name] = test_resources.file_digest(path)
        test_resources.register_report(directory, root=self.root)
        self.freeze(artifacts=artifacts, artifact_base=str(directory))
        original = run_inspection.artifact_observation
        observed = []

        def inspect(path: Path, digest: str, tail: int, record: test_resources.ResourceRecord | None) -> dict:
            observed.append(path.name)
            return original(path, digest, tail, record)

        with patch.object(run_inspection, "artifact_observation", side_effect=inspect):
            result = self.read(offset=1, limit=1, tail=1)
        self.assertEqual(observed, ["second.log"])
        self.assertEqual(result["artifacts"], {"second.log": artifacts["second.log"]})
        self.assertEqual(result["payloads"][0]["tail"], ["second.log"])
        self.assertEqual(result["page"]["artifact_count"], 3)
        self.assertEqual(result["page"]["next_offset"], 2)
        self.assertEqual(result["page"]["subject_count"], 2)

    def test_product_payload_requires_current_owner_but_metadata_remains_readable(
        self,
    ) -> None:
        missing = self.root / "build" / "unregistered"
        retired = self.root / "build" / "retired"
        for directory in (missing, retired):
            directory.mkdir(parents=True)
            (directory / "failure.log").write_text("must not read\n")
        resource = test_resources.register_report(retired, root=self.root)
        with test_resources.storage().edit(resource) as entry:
            entry.phase = "disposed"
        artifacts = {
            str(directory / "failure.log"): test_resources.file_digest(
                directory / "failure.log"
            )
            for directory in (missing, retired)
        }
        self.freeze(artifacts=artifacts)
        with patch.object(
            run_inspection,
            "artifact_observation",
            side_effect=AssertionError("unprotected read"),
        ):
            self.assertEqual(self.read()["artifacts"], artifacts)
            payloads = self.read(tail=1)["payloads"]
        self.assertEqual(len(payloads), 2)
        for payload in payloads:
            self.assertEqual(payload["status"], "unavailable")
            self.assertEqual(payload["tail"], [])
            self.assertIn("protected payload read unavailable", payload["error"])
        self.assertEqual(
            test_resources.entry_facts(test_resources.storage().lookup(resource))[
                "borrows"
            ],
            {},
        )

    def test_standalone_action_artifact_reader_retains_independent_semantics(
        self,
    ) -> None:
        log = self.root / "action.log"
        log.write_text("standalone action\n")
        observed = run_inspection.artifact_observation(
            log, test_resources.file_digest(log), 1, None
        )
        self.assertEqual(observed["status"], "available")
        self.assertEqual(observed["tail"], ["standalone action"])

    def test_captured_owners_project_only_safe_metadata_without_live_observation(
        self,
    ) -> None:
        scope = {"unit": "captured.scope", "group": "/captured", "invocation": "a" * 32}
        producer = {
            "pid": 123,
            "start": "456",
            "boot": "recorded-boot",
            "scope": scope,
            "allocation": "/captured/allocation",
            "operation": "/captured/operation",
            "allocation_owner": {
                "pid": 789,
                "start": "012",
                "boot": "recorded-boot",
                "env": "secret",
            },
            "env": {"SECRET": "not exposed"},
        }
        with test_resources.storage().invocation_edit(self.path.stem) as state:
            state["producer"] = producer
            state["composer"] = {
                "pid": 234,
                "start": "567",
                "boot": "recorded-boot",
                "scope": scope,
            }
        self.freeze(owned_drain="unobserved", process={"exit_code": 0})
        with patch.object(
            test_resources,
            "execution_owner",
            side_effect=AssertionError("fresh owner lookup"),
        ):
            result = self.read()
        self.assertEqual(result["owned_drain"], "unobserved")
        self.assertEqual(result["producer"]["scope"], scope)
        self.assertEqual(result["producer"]["allocation"], producer["allocation"])
        self.assertEqual(
            result["producer"]["allocation_owner"],
            {"pid": 789, "start": "012", "boot": "recorded-boot"},
        )
        self.assertNotIn("env", result["producer"])
        self.assertEqual(result["producer"]["association"], "captured-metadata")
        self.assertEqual(result["producer"]["liveness"], "unobserved")
        self.assertEqual(result["composer"]["pid"], 234)
        with test_resources.storage().invocation_edit(self.path.stem) as state:
            state.pop("producer")
            state.pop("composer")
        missing = self.read()
        self.assertEqual(
            missing["producer"],
            {"association": "unavailable", "liveness": "unobserved"},
        )
        self.assertEqual(missing["composer"], missing["producer"])


if __name__ == "__main__":
    unittest.main()
