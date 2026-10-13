# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded actual C2/C3 lifecycle controls on newly owned private profiles.

Run with `scripts/pse-env --resource-class exclusive --native= --` under the
coordinator's admitted allocation with the final current
canonical-tools binary. --directory must be a new directory below this checkout's
build/. Every profile and diagnostic is retained, including the intentionally
unsettled process-death profile. No TTL, acknowledgement, service configuration or
production settlement is fabricated. The forward predecessor is explicitly a
synthetic fixture obtained by removing two generated indexes from an empty store.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

from scripts import host_admission, native_operation
from scripts import surreal_rollout_producer as producer
from scripts import surreal_server as server
from scripts import surrealkit_maintenance as maintenance
from scripts import test_resources as resources

if TYPE_CHECKING:
    from collections.abc import Callable

ROOT = Path(__file__).resolve().parents[2]
BARRIER = "SLEEP 20s; RETURN 'actual-caller-death-barrier';"


def predecessors(schema: bytes) -> tuple[bytes, bytes, list[tuple[str, str]]]:
    """Choose two generated access indexes, leaving all uniqueness intact."""
    lines = schema.decode().splitlines(keepends=True)
    selected = []
    for line in lines:
        match = re.fullmatch(
            r"DEFINE INDEX ([A-Za-z0-9_]+) ON ([A-Za-z0-9_]+) FIELDS [^;]+;\n?", line
        )
        if match and " UNIQUE;" not in line:
            selected.append((line, match[1], match[2]))
    if len(selected) < 2:
        raise RuntimeError("Two generated nonunique indexes are required")
    selected = selected[-2:]
    baseline = "".join(
        line for line in lines if line not in {item[0] for item in selected}
    ).encode()
    middle = "".join(line for line in lines if line != selected[-1][0]).encode()
    return baseline, middle, [(item[1], item[2]) for item in selected]


class Fixture:
    def __init__(
        self, directory: Path, binary: Path, canonical: Path, port: int
    ) -> None:
        self.directory, self.binary, self.canonical, self.port = (
            directory,
            binary,
            canonical,
            port,
        )
        self.states: list[Path] = []
        self.report: dict[str, object] = {
            "scope": "bounded actual frozen-rollout and fresh-regeneration lifecycle controls",
            "complete": False,
            "checks": [],
            "profiles": [],
            "limits": [
                "Synthetic compatible predecessor; no historical product migration qualification.",
                "Process-death SQL outcome intentionally unknown and retained; no TTL edits or metadata repair.",
                "No scientific correctness, performance, power-loss or distributed deployment qualification.",
            ],
            "canonical_binary_sha256": server.file_digest(canonical),
            "fixture_sha256": server.file_digest(Path(__file__)),
        }

    def checkpoint(self) -> None:
        server.write_json(self.directory / "checks.json", self.report)

    def checked(self, condition: bool, name: str) -> None:
        if not condition:
            raise RuntimeError(name)
        cast("list[dict[str, object]]", self.report["checks"]).append(
            {"name": name, "outcome": "passed"}
        )
        self.checkpoint()
        print("PASS " + name, flush=True)

    def refuses(self, action: Callable[[], object], name: str) -> None:
        try:
            action()
        except (
            maintenance.MaintenanceError,
            server.SupervisorError,
            producer.ProducerError,
        ) as error:
            cast("list[dict[str, object]]", self.report["checks"]).append(
                {"name": name, "outcome": "passed", "refusal": str(error)}
            )
            self.checkpoint()
        else:
            raise RuntimeError(name)

    def setup(self, name: str, *, initialize: bool = False) -> Path:
        state = self.directory / name
        self.states.append(state)
        # Publish exact creation authority before setup can acquire any service.
        cast("list[dict[str, object]]", self.report["profiles"]).append(
            {"state": str(state), "creation": "requested"}
        )
        self.checkpoint()
        options = server.parser().parse_args(
            [
                "setup",
                "--state",
                str(state),
                "--port",
                str(self.port + len(self.states) - 1),
                "--version",
                "v3.3.0",
                "--memory-mib",
                "4096",
                "--server-memory-mib",
                "1024",
                "--native-workers",
                "1",
                "--native-worker-memory-mib",
                "2048",
                "--no-resident",
            ]
        )
        server.setup(options)
        server.start(state, server.config_for(state))
        if initialize:
            self.initialize(state)
        server.stop(state, server.config_for(state))
        config = server.config_for(state)
        cast("list[dict[str, object]]", self.report["profiles"])[-1].update(
            {
                "creation": "owned",
                "instance_id": config["instance_id"],
                "namespace": config["namespace"],
                "database": config["database"],
            }
        )
        self.checkpoint()
        return state

    def initialize(self, state: Path) -> None:
        environment = dict(os.environ)
        environment.pop("PSE_CANONICAL_INITIALIZER", None)
        result = subprocess.run(
            [str(self.canonical), "canonical-init", str(state)],
            env=environment,
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
        server.write_json(
            state / "fixture-initializer.json",
            {
                "exit_code": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
            },
        )
        if result.returncode:
            raise RuntimeError(
                "Actual canonical initializer failed; private diagnostic retained"
            )

    def query(
        self,
        state: Path,
        body: str,
        *,
        config: dict[str, object] | None = None,
        credentials: dict[str, object] | None = None,
    ) -> object:
        with server.lifecycle_reservation(state):
            current = server.config_for(state)
            server.private_offline_state(state, current)
            return server.maintenance_query(
                state,
                current if config is None else config,
                server.read_json(state / "credentials.json")
                if credentials is None
                else credentials,
                body,
            )["value"]

    def marker(self, state: Path) -> dict[str, object]:
        return server.object_mapping(
            self.query(
                state,
                "LET $value=(SELECT * FROM ONLY canonical_interpretations:current);",
            )
        )

    def wrong_authority(self, state: Path, nonce: str) -> None:
        with server.lifecycle_reservation(state):
            value = maintenance.pending(state, nonce)
            value["initializer"] = server.read_json(state / "lifecycle-owner.json")
            maintenance.update(state, value)
            server.maintenance_endpoint(state)
            environment = dict(os.environ)
            environment["PSE_CANONICAL_INITIALIZER"] = "wrong-" + str(
                value["initializer"]["nonce"]
            )
            result = subprocess.run(
                [
                    str(self.canonical),
                    "canonical-validate-definitions",
                    "--state",
                    str(state),
                ],
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            server.write_json(
                state / "wrong-initializer.json",
                {
                    "exit_code": result.returncode,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                },
            )
            self.checked(
                result.returncode != 0
                and "canonical store is quiesced" in result.stderr.lower(),
                "Actual canonical validator refuses wrong live initializer token",
            )

    def phased(self, folder: Path) -> None:
        state = self.setup("phased")
        identity = cast(
            "str",
            cast(
                "list[dict[str, object]]",
                producer.load(folder, current=True)["rollouts"],
            )[-1]["id"],
        )
        started = maintenance.execute(
            self.binary, state, folder, "start", rollout_id=identity
        )
        nonce = str(started["nonce"])
        self.checked(
            started["sql_outcome"] == "unknown", "CLI success alone leaves SQL unknown"
        )
        self.wrong_authority(state, nonce)
        self.refuses(
            lambda: maintenance.activate(state, nonce, self.canonical),
            "Unsettled start cannot activate",
        )
        self.refuses(
            lambda: maintenance.pending(state, "wrong-" + nonce),
            "Wrong pending nonce refused",
        )
        self.refuses(
            lambda: maintenance.execute(self.binary, state, folder, "up"),
            "Competing blind rollout refused",
        )
        settled = maintenance.settle_generated(state, nonce, folder, self.canonical)
        self.checked(
            settled["phase"] == "start_settled"
            and cast(
                "dict[str, object]",
                cast("dict[str, object]", settled["settlement"])["canonical"],
            )["scope"]
            == "rollout-phase",
            "Exact start schema and beta acknowledgements settle independently",
        )
        self.refuses(
            lambda: maintenance.activate(state, nonce, self.canonical),
            "Settled start cannot activate final target",
        )
        completed = maintenance.execute(
            self.binary,
            state,
            folder,
            "complete",
            rollout_id=identity,
            pending_nonce=nonce,
        )
        self.checked(
            completed["nonce"] == nonce
            and len(cast("list[object]", completed["phase_history"])) == 1,
            "Start and complete retain one nonce and immutable phase history",
        )
        settled = maintenance.settle_generated(state, nonce, folder, self.canonical)
        self.checked(
            settled["phase"] == "settled"
            and cast("dict[str, object]", settled["settlement"])["phase_complete"]
            is True,
            "Final exact target and beta step checksums settle",
        )
        actual_tool = maintenance.canonical_tool
        lost: dict[str, object] = {}

        def lost_ack(
            state: Path,
            endpoint: str,
            value: dict[str, object],
            binary: Path,
            *,
            activate: bool = False,
            expected_schemas: tuple[Path, ...] = (),
        ) -> dict[str, object]:
            witness = actual_tool(
                state,
                endpoint,
                value,
                binary,
                activate=activate,
                expected_schemas=expected_schemas,
            )
            if activate:
                lost.update(witness)
                raise maintenance.MaintenanceError(
                    "Injected caller ACK loss after actual activation and child drain"
                )
            return witness

        with patch.object(maintenance, "canonical_tool", side_effect=lost_ack):
            self.refuses(
                lambda: maintenance.activate(state, nonce, self.canonical),
                "Lost caller ACK keeps exact activation pending",
            )
        self.checked(
            bool(lost) and maintenance.pending(state, nonce)["phase"] == "settled",
            "Actual activation effect exists behind retained pending fence",
        )
        before = self.marker(state)
        activated = maintenance.activate(state, nonce, self.canonical)
        after = self.marker(state)
        self.checked(
            before["issuing_store"] == after["issuing_store"]
            and activated["nonce"] == nonce
            and maintenance.PENDING not in server.config_for(state),
            "Exact activation receipt replay preserves issuing UUID after lost ACK",
        )

    def forward_and_reset(
        self, schema: bytes, interpretation: str, digest: str
    ) -> None:
        state = self.setup("forward", initialize=True)
        before = self.marker(state)
        baseline, middle, indexes = predecessors(schema)
        old_digest = hashlib.sha256(baseline).hexdigest()
        folder = self.directory / "forward-chain"
        for declaration, identity in (
            (baseline, old_digest),
            (middle, hashlib.sha256(middle).hexdigest()),
            (schema, digest),
        ):
            producer.publish(self.binary, folder, declaration, interpretation, identity)
        removal = " ".join(
            "REMOVE INDEX " + index + " ON TABLE " + table + ";"
            for index, table in indexes
        )
        self.query(
            state,
            removal  # noqa: S608 -- generated fixture SQL and JSON literal
            + " UPDATE ONLY canonical_interpretations:current SET schema_digest="
            + json.dumps(old_digest)
            + " RETURN NONE; LET $value=true;",
        )
        self.report["synthetic_predecessor"] = {
            "indexes_removed": indexes,
            "digest": old_digest,
            "issuing_store": before["issuing_store"],
        }
        self.checkpoint()
        value = maintenance.execute(self.binary, state, folder, "up")
        nonce = str(value["nonce"])
        maintenance.settle_generated(state, nonce, folder, self.canonical)
        maintenance.activate(state, nonce, self.canonical)
        after = self.marker(state)
        self.checked(
            after["schema_digest"] == digest
            and after["issuing_store"] == before["issuing_store"],
            "Actual two-index forward chain preserves compatible issuing UUID",
        )
        guard = "fixture-old-creation-authority"
        self.query(
            state,
            "CREATE ONLY canonical_guards:`retention:fixture` SET key='retention:fixture',generation=0dec,incarnation="
            + json.dumps(guard)
            + " RETURN NONE; LET $value=true;",
        )
        original = server.config_for(state)
        credentials = server.read_json(state / "credentials.json")
        server.rebuild(state, [str(self.canonical), "canonical-init", str(state)])
        current = server.config_for(state)
        fresh = self.marker(state)
        self.checked(
            all(
                current[key] != original[key]
                for key in ("instance_id", "namespace", "database")
            )
            and fresh["issuing_store"] != after["issuing_store"],
            "Regeneration creates fresh instance, selection and issuing identity",
        )
        self.refuses(
            lambda: self.query(state, "LET $value=true;", credentials=credentials),
            "Old ROOT credentials refused by regenerated backend",
        )
        self.checked(
            self.query(
                state,
                "LET $value=(SELECT VALUE incarnation FROM ONLY canonical_guards:`retention:fixture`)="  # noqa: S608 -- fixed selector and JSON literal
                + json.dumps(guard)
                + ";",
            )
            is False,
            "Old creation capability cannot match regenerated guard",
        )
        with server.lifecycle_reservation(state):
            server.private_offline_state(state, current)
            absent = not server.database_present(
                state,
                {
                    **current,
                    "namespace": original["namespace"],
                    "database": original["database"],
                },
                server.read_json(state / "credentials.json"),
            )
        self.checked(absent, "Old selected database is absent without recreating it")

    def process_death(self, folder: Path) -> None:
        state = self.setup("process-death")
        frozen = self.directory / "death-chain"
        shutil.copytree(folder, frozen)
        manifest = next((frozen / "rollouts").glob("*.toml"))
        rollout = manifest.stem
        barrier = (
            '[[steps]]\nid = "caller_death_barrier"\nphase = "start"\nkind = "run_sql"\nsql = '
            + json.dumps(BARRIER)
            + "\n\n"
        )
        manifest.write_text(
            manifest.read_text().replace("[[steps]]", barrier + "[[steps]]", 1)
        )
        self.report["process_death"] = {
            "fixture_input_sha256": maintenance.inputs(frozen)[1],
            "barrier": BARRIER,
            "sql_outcome": "unknown",
            "retention": "required",
        }
        self.checkpoint()
        with (
            (state / "caller.stdout").open("xb") as stdout,
            (state / "caller.stderr").open("xb") as stderr,
        ):
            child = subprocess.Popen(
                [
                    sys.executable,
                    "-m",
                    "scripts.surrealkit_maintenance",
                    "start",
                    "--state",
                    str(state),
                    "--binary",
                    str(self.binary),
                    "--folder",
                    str(frozen),
                    "--rollout-id",
                    rollout,
                    "--timeout",
                    "90",
                ],
                cwd=ROOT,
                stdout=stdout,
                stderr=stderr,
            )
            descriptor = os.pidfd_open(child.pid)
            try:
                deadline = time.monotonic() + 30
                observed: dict[str, object] | None = None
                while time.monotonic() < deadline and child.poll() is None:
                    config = server.config_for(state)
                    value = config.get(maintenance.PENDING)
                    if (
                        isinstance(value, dict)
                        and value.get("phase") == "submitted"
                        and value.get("attempts")
                    ):
                        attempt = Path(value["attempts"][-1]["directory"])
                        if (attempt / "child.json").is_file():
                            try:
                                rows = resources.namespace_sql(
                                    config,
                                    "SELECT status,steps FROM __rollout;",
                                    namespace=str(config["namespace"]),
                                    database=str(config["database"]),
                                )
                            except resources.ResourceError:
                                rows = []  # Bootstrap not yet acknowledged; no outcome inferred.
                            if (
                                rows
                                and isinstance(rows[0], list)
                                and any(
                                    row.get("status") == "running_start"
                                    and any(
                                        step.get("step_id") == "caller_death_barrier"
                                        and step.get("status") == "running"
                                        for step in row.get("steps", [])
                                    )
                                    for row in rows[0]
                                )
                            ):
                                observed = {
                                    "pending_nonce": value["nonce"],
                                    "rows": rows,
                                    "owner": server.read_json(
                                        state / "lifecycle-owner.json"
                                    ),
                                    "child": server.read_json(attempt / "child.json"),
                                }
                                break
                    time.sleep(0.05)
                if observed is None:
                    raise RuntimeError(
                        "Actual running caller-death barrier was not observed"
                    )
                if cast("dict[str, object]", observed["owner"]).get(
                    "pid"
                ) != child.pid or not server.reservation_live(
                    cast("dict[str, object]", observed["owner"])
                ):
                    raise RuntimeError(
                        "Observed lifecycle owner is not this exact live child"
                    )
                cast("dict[str, object]", self.report["process_death"])[
                    "before_kill"
                ] = observed
                self.checkpoint()
                signal.pidfd_send_signal(descriptor, signal.SIGKILL)
                child.wait(timeout=10)
                self.checked(
                    child.returncode == -signal.SIGKILL
                    and not server.reservation_live(
                        cast("dict[str, object]", observed["owner"])
                    ),
                    "Actual lifecycle caller SIGKILL revokes exact process authority",
                )
            finally:
                if child.poll() is None:
                    signal.pidfd_send_signal(descriptor, signal.SIGTERM)
                    try:
                        child.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        signal.pidfd_send_signal(descriptor, signal.SIGKILL)
                        child.wait(timeout=10)
                os.close(descriptor)
        deadline = time.monotonic() + 45
        while True:
            value = server.object_mapping(server.config_for(state)[maintenance.PENDING])
            try:
                maintenance.require_child_drains(state, value)
                break
            except (maintenance.MaintenanceError, OSError):
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.1)
        with server.lifecycle_reservation(state):
            server.drain_maintenance_listener(state)
        nonce = str(value["nonce"])
        self.checked(
            cast("dict[str, object]", server.config_for(state)[maintenance.PENDING])[
                "sql_outcome"
            ]
            == "unknown"
            and not server.active(state),
            "Actual tool scope and listener drain leave SQL unknown and admission closed",
        )
        self.refuses(
            lambda: maintenance.execute(
                self.binary,
                state,
                frozen,
                "repair",
                rollout_id=rollout,
                pending_nonce=nonce,
            ),
            "Process death cannot authorize blind metadata repair",
        )
        self.refuses(
            lambda: maintenance.activate(state, nonce, self.canonical),
            "Process death cannot authorize target activation",
        )
        self.refuses(
            lambda: server.open_context_admission(state),
            "Unknown process-death SQL fences ordinary admission",
        )
        self.refuses(
            lambda: server.start(state, server.config_for(state)),
            "Unknown process-death SQL fences ordinary restart",
        )
        self.refuses(
            lambda: maintenance.settle_generated(state, nonce, frozen, self.canonical),
            "Arbitrary barrier cannot claim generated schema settlement",
        )

    def run(self) -> None:
        schema, interpretation, digest = producer.declaration()
        folder = self.directory / "phased-chain"
        producer.publish(self.binary, folder, schema, interpretation, digest)
        self.phased(folder)
        self.forward_and_reset(schema, interpretation, digest)
        self.process_death(folder)
        self.report["complete"] = True
        self.checkpoint()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument(
        "--binary", type=Path, required=True, help="Exact verified beta.6 surrealkit"
    )
    parser.add_argument(
        "--canonical-binary",
        type=Path,
        required=True,
        help="Final current xtask with canonical-tools",
    )
    parser.add_argument("--port", type=int, default=18341)
    args = parser.parse_args()
    directory = args.directory.absolute()
    if (
        directory.exists()
        or directory != directory.resolve(strict=False)
        or not directory.is_relative_to(ROOT / "build")
        or any(path.is_symlink() for path in (directory, *directory.parents))
    ):
        raise SystemExit("Requires a new nonsymlinked owned build directory")
    allocation = host_admission.inherit(os.environ)
    if (
        allocation is None
        or not allocation.profile.exclusive
        or native_operation.scope_owner() is None
    ):
        raise SystemExit(
            "Requires an authenticated exclusive native operation; use pse-env --resource-class exclusive --native= --"
        )
    binary, canonical = (
        args.binary.resolve(strict=True),
        args.canonical_binary.resolve(strict=True),
    )
    directory.mkdir(parents=True, mode=0o700)
    resource = resources.register_report(directory)
    fixture = Fixture(directory, binary, canonical, args.port)
    fixture.report["resource"] = resource
    fixture.checkpoint()
    with resources.borrow(resource):
        try:
            fixture.run()
        except BaseException as error:
            fixture.report["error"] = type(error).__name__ + ": " + str(error)
            fixture.checkpoint()
            raise
        finally:
            errors = []
            for state in fixture.states:
                if (state / "config.json").is_file():
                    try:
                        server.stop(state, server.config_for(state))
                        server.all_contexts_drained(state)
                    except (server.SupervisorError, OSError) as error:
                        errors.append({"state": str(state), "error": str(error)})
            fixture.report["cleanup_errors"] = errors
            fixture.report["retention"] = (
                "All owned profiles remain pinned; unknown process-death SQL is not disposable"
            )
            if errors:
                fixture.report["complete"] = False
            fixture.checkpoint()
    return 0 if fixture.report["complete"] is True else 1


if __name__ == "__main__":
    raise SystemExit(main())
