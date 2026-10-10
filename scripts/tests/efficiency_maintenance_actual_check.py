# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Manual actual-backend maintenance control using fresh disposable profiles.

Run under the repository's existing admitted exclusive allocation, after its
other backend journeys have drained. Never select an existing state directory::

    .venv/bin/python scripts/tests/efficiency_maintenance_actual_check.py \
        --directory /absolute/fresh/maintenance-check --port 18243

Requires the pinned native SurrealDB CLI and a current target/debug/xtask built
with canonical-tools. This control installs the production canonical schema,
preserves explicit authored input hashes, and exercises native restore/rebuild,
account/catalog authority and interrupted-initializer recovery. It retains its
private acceptance.json and diagnostic files, then normally stops its profiles.
Scientific model reconstruction and delayed creation API replay are separate.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--port", type=int, default=18243)
    parser.add_argument("--initializer-fail", type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    sys.path.insert(0, str(root))
    # Resolve the repository root before importing its tooling.
    from scripts import surreal_server as server  # noqa: PLC0415

    xtask = root / "target/debug/xtask"
    report: dict[str, object] = {
        "interpretation": server.SUBSTRATE_INTERPRETATION,
        "checks": [],
    }
    checks: list[str] = []
    report["checks"] = checks
    drain_errors: list[str] = []

    def checked(condition: bool, name: str) -> None:
        if not condition:
            raise RuntimeError(name)
        checks.append(name)
        print("PASS " + name, flush=True)

    def initialize(
        state: Path, *, stale_nonce: str | None = None, success: bool = True
    ) -> None:
        env = dict(os.environ)
        env.pop("PSE_CANONICAL_INITIALIZER", None)
        if stale_nonce is not None:
            env["PSE_CANONICAL_INITIALIZER"] = stale_nonce
        result = subprocess.run(
            [str(xtask), "canonical-init", str(state)],
            env=env,
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        if (result.returncode == 0) != success or (
            not success and "canonical store is quiesced" not in result.stderr
        ):
            # Retain diagnostics privately; never emit credentials through this harness.
            directory = args.directory
            directory.mkdir(mode=0o700, parents=True, exist_ok=True)
            diagnostic = directory / ("initializer-" + uuid.uuid4().hex + ".log")
            descriptor = os.open(
                diagnostic, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600
            )
            with os.fdopen(descriptor, "w") as stream:
                stream.write(result.stdout + "\n" + result.stderr)
            raise RuntimeError(
                "production initializer unexpected exit; private diagnostic "
                + str(diagnostic)
            )

    if args.initializer_fail is not None:
        # Genuine production schema initialization under the live scoped authority,
        # followed by an explicit caller failure. No mocked CLI/backend completion.
        preserved = Path(os.environ["PSE_PRESERVED_INPUTS"])
        if not list(preserved.rglob("*.pse")):
            raise RuntimeError("explicit preserved authored input missing")
        initialize(
            args.initializer_fail, stale_nonce=os.environ["PSE_CANONICAL_INITIALIZER"]
        )
        initialized = server.config_for(args.initializer_fail)
        server.write_json(
            args.directory / "initializer-before-injected-failure.json",
            {
                "namespace": initialized["namespace"],
                "database": initialized["database"],
                "nonce": os.environ["PSE_CANONICAL_INITIALIZER"],
            },
        )
        return 23

    run = args.directory.absolute()
    if run.exists():
        raise SystemExit("Refusing existing acceptance directory")
    run.mkdir(mode=0o700, parents=True)
    report["directory"] = str(run)
    source = run / "source"
    restored = run / "restored"
    backup = run / "backup"
    authored = run / "authored"
    authored.mkdir(mode=0o700)
    shutil.copy2(
        root / "packages/reference/process/models/heat-exchanger.pse",
        authored / "heat-exchanger.pse",
    )
    input_hashes = server.input_inventory(authored)
    profiles = [source, restored]
    old_authority = "eff33-original-creation-authority"

    def query(
        state: Path,
        body: str,
        *,
        config: dict[str, object] | None = None,
        credentials: dict[str, object] | None = None,
    ) -> object:
        config = server.config_for(state) if config is None else config
        credentials = (
            server.read_json(state / "credentials.json")
            if credentials is None
            else credentials
        )
        with server.lifecycle_reservation(state):
            server.private_offline_state(state, server.config_for(state))
            return server.maintenance_query(state, config, credentials, body)["value"]

    def owned_database_present(
        state: Path, config: dict[str, object], credentials: dict[str, object]
    ) -> bool:
        with server.lifecycle_reservation(state):
            server.private_offline_state(state, server.config_for(state))
            return server.database_present(state, config, credentials)

    def owned_catalog(
        state: Path, config: dict[str, object], credentials: dict[str, object]
    ) -> None:
        with server.lifecycle_reservation(state):
            server.private_offline_state(state, server.config_for(state))
            server.require_private_catalog(state, config, credentials)

    def owned_root_users(
        state: Path,
        config: dict[str, object] | None = None,
        credentials: dict[str, object] | None = None,
    ) -> set[str]:
        config = server.config_for(state) if config is None else config
        credentials = (
            server.read_json(state / "credentials.json")
            if credentials is None
            else credentials
        )
        with server.lifecycle_reservation(state):
            server.private_offline_state(state, server.config_for(state))
            catalog = server.maintenance_catalog(state, config, credentials, "ROOT")
            return set(server.object_mapping(catalog["users"]))

    def refuses(action: Callable[[], object], name: str) -> None:
        try:
            action()
        except server.SupervisorError:
            checked(True, name)
        else:
            raise RuntimeError(name + ": unexpectedly accepted")

    try:
        setup = server.parser().parse_args(
            [
                "setup",
                "--state",
                str(source),
                "--port",
                str(args.port),
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
        server.setup(setup)
        server.start(source, server.config_for(source))
        initialize(source)
        server.stop(source, server.config_for(source))
        original = server.config_for(source)
        original_credentials = server.read_json(source / "credentials.json")
        # Mutations suppress their row results: offline maintenance requires
        # one complete nonce acknowledgment, never additional unchecked values.
        query(
            source,
            "CREATE ONLY canonical_guards:`retention:maintenance` SET key='retention:maintenance',generation=0dec,incarnation="
            + json.dumps(old_authority)
            + ",analysis_creation_closed_through=100dec RETURN NONE; "
            "CREATE ONLY canonical_guards:`analysis:maintenance` SET key='analysis:maintenance',generation=0dec RETURN NONE; "
            "CREATE ONLY canonical_analysis_nodes:maintenance SET key='maintenance',analysis='maintenance',semantic='maintenance',kind='control' RETURN NONE; "
            "CREATE ONLY canonical_roots:maintenance SET key='maintenance',problem='maintenance',revision='maintenance',sequence=0dec,owner_kind='analysis',owner='maintenance' RETURN NONE; "
            "DEFINE TABLE maintenance_external_inputs SCHEMALESS; CREATE ONLY maintenance_external_inputs:authored SET digest="
            + json.dumps(input_hashes["heat-exchanger.pse"])
            + " RETURN NONE; LET $value=true;",
        )
        server.backup(source, server.config_for(source), backup)
        backup_hashes = {
            str(path.relative_to(backup)): server.file_digest(path)
            for path in backup.rglob("*")
            if path.is_file()
        }
        checked(
            True,
            "production canonical initializer and real typed cleanup rows installed",
        )

        # Actual native sql + RocksDB export/import + authenticated ROOT rotation.
        server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        current = server.config_for(restored)
        checked(
            not current.get("derived_rebuild_pending")
            and not current["accepting_writes"]
            and current["admission"] == "validation_required",
            "restore remains closed for semantic validation",
        )
        checked(
            (current["namespace"], current["database"])
            != (original["namespace"], original["database"]),
            "restore publishes fresh physical namespace and database",
        )
        guard = query(
            restored,
            "LET $value=SELECT incarnation,analysis_creation_closed_through FROM ONLY canonical_guards:`retention:maintenance`;",
        )
        checked(
            isinstance(guard, dict)
            and guard["incarnation"] != old_authority
            and str(guard["analysis_creation_closed_through"]) in {"100", "100.0"},
            "source guard authority rotated while original closed-through floor retained",
        )
        checked(
            query(
                restored,
                "LET $nodes=SELECT key FROM canonical_analysis_nodes; LET $roots=SELECT key FROM canonical_roots WHERE owner_kind='analysis'; LET $value=array::len($nodes)+array::len($roots);",
            )
            == 0,
            "derived analysis rows and roots retired by actual bounded cleanup",
        )
        checked(
            query(
                restored,
                "LET $value=SELECT VALUE digest FROM ONLY maintenance_external_inputs:authored;",
            )
            == input_hashes["heat-exchanger.pse"],
            "unrelated external input bytes represented in backend preserved",
        )
        credentials = server.read_json(restored / "credentials.json")
        checked(
            not owned_database_present(
                restored,
                {
                    **current,
                    "namespace": original["namespace"],
                    "database": original["database"],
                },
                credentials,
            ),
            "old copied database no longer addressable",
        )
        refuses(
            lambda: query(
                restored, "LET $value=true;", credentials=original_credentials
            ),
            "old root credentials refused on restored backend",
        )
        checked(
            query(
                restored,
                "LET $value=(SELECT VALUE incarnation FROM ONLY canonical_guards:`retention:maintenance`)="  # noqa: S608 -- Old authority is encoded as a JSON string.
                + json.dumps(old_authority)
                + ";",
            )
            is False,
            "original creation incarnation cannot match restored source guard",
        )
        checked(
            backup_hashes
            == {
                str(path.relative_to(backup)): server.file_digest(path)
                for path in backup.rglob("*")
                if path.is_file()
            },
            "original backup inventory unchanged",
        )
        validator = [
            sys.executable,
            "-c",
            "import sys; from pathlib import Path; sys.path.insert(0,sys.argv[1]); from scripts import surreal_server as s; p=Path(sys.argv[2]); s.validate_interpretation(p,s.config_for(p),s.SUBSTRATE_INTERPRETATION)",
            str(root),
            str(restored),
        ]
        server.validate(restored, current, server.SUBSTRATE_INTERPRETATION, validator)
        validated = server.config_for(restored)
        checked(
            validated["admission"] == "quiesced" and not validated["accepting_writes"],
            "production semantic validation leaves explicit-start quiesced state",
        )
        server.start(restored, validated)
        initialize(restored)
        checked(
            server.config_for(restored)["accepting_writes"] is True,
            "explicit normal start readmits validated current interpretation",
        )
        server.stop(restored, server.config_for(restored))

        # Actual initializer succeeds against fresh backend, then caller deliberately
        # fails. Recovery must not reuse that unacknowledged target.
        fail_command = [
            sys.executable,
            str(Path(__file__).absolute()),
            "--directory",
            str(run),
            "--initializer-fail",
            str(restored),
        ]
        refuses(
            lambda: server.rebuild(restored, authored, run / "preserved", fail_command),
            "genuine initializer caller failure retains pending rebuild",
        )
        pending_config = server.config_for(restored)
        pending = server.object_mapping(pending_config["derived_rebuild_pending"])
        checked(
            pending["phase"] == "initializing"
            and not pending_config["accepting_writes"],
            "failed initializer leaves durable closed initializing phase",
        )
        checked(
            server.input_inventory(authored)
            == input_hashes
            == server.input_inventory(run / "preserved/inputs"),
            "explicit authored input hashes preserved through initializer failure",
        )
        failed_target = (pending_config["namespace"], pending_config["database"])
        initialized = server.read_json(run / "initializer-before-injected-failure.json")
        checked(
            (initialized["namespace"], initialized["database"]) == failed_target
            and initialized["nonce"]
            == str(server.object_mapping(pending["initializer"])["nonce"]),
            "production initializer acknowledged exact failed target before injected child exit",
        )
        retained_source = (pending["namespace"], pending["database"])
        checked(
            owned_database_present(
                restored,
                {
                    **pending_config,
                    "namespace": retained_source[0],
                    "database": retained_source[1],
                },
                server.read_json(restored / "credentials.json"),
            ),
            "old source retained until acknowledged validated current target",
        )
        initialize(restored, success=False)
        initialize(
            restored,
            stale_nonce=str(server.object_mapping(pending["initializer"])["nonce"]),
            success=False,
        )
        checked(
            True,
            "ordinary and dead-owner initializer authority refused by production API",
        )
        server.recover_maintenance(
            restored, [str(xtask), "canonical-init", str(restored)]
        )
        recovered = server.config_for(restored)
        checked(
            not recovered.get("derived_rebuild_pending")
            and recovered["admission"] == "quiesced"
            and not recovered["accepting_writes"],
            "real recovery completes current rebuild and remains closed",
        )
        checked(
            (recovered["namespace"], recovered["database"]) != failed_target,
            "recovery uses fresh target instead of unacknowledged initializer output",
        )
        for namespace, database in (failed_target, retained_source):
            checked(
                not owned_database_present(
                    restored,
                    {**recovered, "namespace": namespace, "database": database},
                    server.read_json(restored / "credentials.json"),
                ),
                "recovery disposes exact retired database " + str(database),
            )
        checked(
            server.input_inventory(run / "preserved/inputs") == input_hashes,
            "preserved authored input inventory unchanged after recovery",
        )

        # Unknown owner remains in a separate stopped disposable physical service.
        # Empty catalog entries may coexist and must not be removed.
        query(source, "DEFINE DATABASE maintenance_empty; LET $value=true;")
        owned_catalog(source, server.config_for(source), original_credentials)
        checked(True, "known empty database accepted by actual ROOT catalog preflight")
        query(source, "DEFINE DATABASE maintenance_unknown; LET $value=true;")
        unknown = {**server.config_for(source), "database": "maintenance_unknown"}
        query(
            source,
            "DEFINE TABLE unknown_owner SCHEMALESS; CREATE ONLY unknown_owner:keep SET value='preserve owner' RETURN NONE; LET $value=true;",
            config=unknown,
        )
        users = owned_root_users(source)
        refuses(
            lambda: server.rebuild(
                source,
                authored,
                run / "refused-preserved",
                [str(xtask), "canonical-init", str(source)],
            ),
            "nonempty unknown physical owner refuses global credential rotation",
        )
        checked(
            owned_root_users(source, original, original_credentials) == users,
            "refused cutover preserves original ROOT accounts",
        )
        checked(
            query(
                source,
                "LET $value=SELECT VALUE value FROM ONLY unknown_owner:keep;",
                config=unknown,
                credentials=original_credentials,
            )
            == "preserve owner",
            "unknown database content unchanged after refusal",
        )
        checked(
            owned_database_present(
                source,
                {**original, "database": "maintenance_empty"},
                original_credentials,
            ),
            "empty database retained after refusal",
        )
        report["result"] = "passed"
    except Exception as error:
        report["result"] = "failed"
        report["error"] = type(error).__name__ + ": " + str(error)
        raise
    finally:
        for state in profiles:
            if (state / "config.json").exists():
                try:
                    server.stop(state, server.config_for(state))
                except Exception as error:
                    drain_errors.append(type(error).__name__ + ": " + str(error))
        report["limits"] = [
            "Initializer installs production current schema; preserved .pse input is not compiled/reimported into a scientific model.",
            "Old authority checks cover actual namespace disposal, ROOT authentication and guard incarnation; end-to-end delayed analysis-creation API replay remains separate.",
            "Failure injection is a real child exit after successful production initialization; SIGKILL/fsync power-loss recovery remains covered by targeted phase controls only.",
        ]
        if drain_errors:
            report["drain_errors"] = drain_errors
            report["result"] = "failed"
        server.write_json(run / "acceptance.json", report)
        print(json.dumps(report, indent=2), flush=True)

    return 0 if report["result"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
