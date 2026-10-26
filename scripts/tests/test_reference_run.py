# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed reference lifetime preserves exact owner, literal argv and independent outcomes."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import os
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts import reference_run, surreal_server


class ReferenceRunTests(unittest.TestCase):
    def test_initializer_is_frozen_before_the_shared_build_alias_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "pse-canonical-init"
            source.write_text(
                '#!/bin/sh\nprintf \'%s\\n\' \'{"version":1,"canonical_initialize":true,"phase":"description"}\'\n'
            )
            source.chmod(0o700)
            frozen = reference_run.freeze_initializer(source, root / "invocation.json")
            before = frozen.read_bytes()
            source.write_text("#!/bin/sh\nexit 7\n")
            self.assertEqual(frozen.read_bytes(), before)
            self.assertEqual(frozen.stat().st_mode & 0o777, 0o500)
            with self.assertRaisesRegex(ValueError, "does not expose"):
                reference_run.freeze_initializer(source, root / "other-invocation.json")

    def run_lifetime(
        self, *, code: int = 0, changed: bool = False, stop: int = 0
    ) -> tuple[dict, list]:
        calls: list[tuple] = []

        def phase(command: str, state: Path, *extra: str) -> int:
            calls.append((command, state, extra))
            return stop if command == "stop" else 0

        def work(command: list[str], _environment: dict, **options: object) -> int:
            self.assertEqual(
                command,
                ["tool with spaces", "a; touch never", "$(never)", "--value=a b"],
            )
            calls.append(("work", command))
            observation = options["observation"]
            assert isinstance(observation, dict)
            observation.update(
                owned_drain="complete", forwarded_signal=2 if code == 130 else None
            )
            return code

        owner = SimpleNamespace(
            nonce="a" * 32, profile=SimpleNamespace(name="reference")
        )
        with (
            patch.object(reference_run.host_admission, "inherit", return_value=owner),
            patch.object(
                reference_run.surreal_server,
                "config_for",
                side_effect=[
                    {"instance_id": "initial", "admission": "validation_required"},
                    {"instance_id": "initial"},
                    {
                        "instance_id": "initial",
                        "primary_receiver": {"worker_executable": "/admitted/worker"},
                    },
                    {"instance_id": "changed" if changed else "initial"},
                ],
            ),
            patch.object(
                reference_run.surreal_server,
                "lifecycle_reservation",
                side_effect=lambda *_args, **_kwargs: contextlib.nullcontext(),
            ),
            patch.object(
                reference_run.surreal_server,
                "service_readiness",
                return_value=SimpleNamespace(
                    phase=surreal_server.Readiness.ACTIONABLE_ABSENT
                ),
            ),
            patch.object(
                reference_run.surreal_server,
                "start",
                side_effect=lambda *_args: calls.append(("start",)),
            ),
            patch.object(
                reference_run.surreal_server,
                "storage_unit_observation",
                return_value={"InvocationID": "b" * 32},
            ),
            patch.object(reference_run, "phase", side_effect=phase),
            patch.object(reference_run, "checkpoint"),
            patch.object(
                reference_run.native_operation, "run_child_scope", side_effect=work
            ),
        ):
            result = reference_run.lifetime(
                Path("/selected"),
                Path("/invocation"),
                ["tool with spaces", "a; touch never", "$(never)", "--value=a b"],
                Path("/initializer"),
            )
        return result, calls

    def test_same_owner_phases_and_literal_workload(self) -> None:
        result, calls = self.run_lifetime()
        self.assertEqual(
            [item[0] for item in calls],
            ["initialize", "start", "work", "quiesce", "stop"],
        )
        self.assertEqual(result["allocation"], "a" * 32)
        self.assertEqual(result["workload"], 0)
        self.assertEqual(result["cleanup"], 0)
        self.assertEqual(result["service_drain"], "complete")

    def test_cancelled_workload_keeps_its_signal_and_still_drains(self) -> None:
        result, calls = self.run_lifetime(code=130)
        self.assertEqual(result["signal"], 2)
        self.assertEqual(result["workload"], 130)
        self.assertEqual(calls[-1][0], "stop")

    def test_changed_owner_is_preserved_and_failed_stop_is_not_success(self) -> None:
        result, calls = self.run_lifetime(changed=True)
        self.assertEqual(result["cleanup"], "unresolved")
        self.assertNotIn("stop", [item[0] for item in calls])
        result, _calls = self.run_lifetime(stop=1)
        self.assertEqual(result["workload"], 0)
        self.assertEqual(result["cleanup"], "unresolved")

    def test_setup_build_failure_never_enters_reference_lifetime(self) -> None:
        for results, expected in (
            ([9], "deployment build failed"),
            ([0, 9], "build failed"),
        ):
            with (
                self.subTest(results=results),
                tempfile.TemporaryDirectory() as temporary,
            ):
                worker = Path(temporary) / "worker"
                worker.touch()
                with (
                    patch.dict(os.environ, {}, clear=True),
                    patch.object(
                        reference_run.test_resources,
                        "invocation",
                        return_value=Path("/exact/invocation"),
                    ),
                    patch.object(reference_run, "checkpoint"),
                    patch.object(
                        reference_run.subprocess, "call", side_effect=results
                    ) as run,
                ):
                    result = reference_run.execute(Path("/selected"), worker, ["work"])
                self.assertEqual(result["setup"], expected)
                self.assertEqual(run.call_count, len(results))

    def test_unrelated_live_service_is_not_stopped(self) -> None:
        with (
            patch.object(
                reference_run.host_admission,
                "inherit",
                return_value=SimpleNamespace(
                    nonce="a", profile=SimpleNamespace(name="reference")
                ),
            ),
            patch.object(
                reference_run.surreal_server,
                "config_for",
                return_value={"instance_id": "initial"},
            ),
            patch.object(
                reference_run.surreal_server,
                "lifecycle_reservation",
                side_effect=lambda *_args, **_kwargs: contextlib.nullcontext(),
            ),
            patch.object(
                reference_run.surreal_server,
                "service_readiness",
                return_value=SimpleNamespace(phase=surreal_server.Readiness.READY),
            ),
            patch.object(reference_run, "checkpoint"),
            patch.object(reference_run, "phase") as phase,
        ):
            result = reference_run.lifetime(
                Path("/selected"), Path("/invocation"), ["work"], Path("/initializer")
            )
        phase.assert_not_called()
        self.assertEqual(result["cleanup"], "not_run")


if __name__ == "__main__":
    unittest.main()
