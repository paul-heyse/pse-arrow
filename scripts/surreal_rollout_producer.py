# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Append immutable beta.6 schema rollouts from the canonical registry output.

The beta executable owns planning, frozen SQL, catalogs and schema hashes. This
producer owns publication and the repository interpretation binding. Publication
is explicit: ordinary codegen never rewrites or prunes the published chain.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import cast

from scripts import surrealkit

ROOT = Path(__file__).resolve().parents[1]
PRODUCER = "scripts/surreal_rollout_producer.py"
REGISTER = "producer-register.json"
VERSION = "canonical-frozen-rollout-v1"
BETA = "1.0.0-beta.6"
FOLDER = ROOT / "database/canonical-rollout"
CUTOVER = {
    "kind": "design-phase-v6-regeneration",
    "source_interpretation": "pse.substrate.v5",
    "target_interpretation": "pse.substrate.v6",
    "requires_empty_uninitialized_target": True,
    "data_migration": False,
    "backfill": False,
    "preserved_data_upgrade": False,
}


class ProducerError(RuntimeError):
    """An immutable publication or its declared source was refused."""


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def inventory(folder: Path) -> dict[str, str]:
    files = {}
    for path in sorted(folder.rglob("*")):
        if path.is_symlink() or not (path.is_dir() or path.is_file()):
            raise ProducerError("Symlink or special producer input refused")
        if path.is_file():
            files[path.relative_to(folder).as_posix()] = sha(path.read_bytes())
    return files


def beta6_step_checksum(step: dict[str, object]) -> str:
    """Exact beta.6 rollout.rs::step_checksum serde_json field order.

    CLI manifests instead use load_rollout_spec's SHA256 of raw TOML. Only the
    schema actions emitted by run_plan are admitted here; arbitrary SQL is not.
    EntityKey's absent scope deserializes to None and serializes as JSON null.
    """
    normalized: dict[str, object] = {
        "id": step["id"],
        "phase": step["phase"],
        "kind": step["kind"],
    }
    kind = step["kind"]
    if kind == "apply_files":
        normalized["files"] = [
            {"path": file["path"], "hash": file["hash"]}
            for file in cast("list[dict[str, object]]", step["files"])
            if isinstance(file, dict) and set(file) == {"path", "hash"}
        ]
        if len(cast("list[object]", normalized["files"])) != len(
            cast("list[object]", step["files"])
        ):
            raise ProducerError("Only frozen schema files are admitted")
    elif kind in {"remove_entities", "restore_definitions"}:
        normalized["entities"] = [
            {
                "kind": entity["kind"],
                "scope": entity.get("scope"),
                "name": entity["name"],
            }
            for entity in cast("list[dict[str, object]]", step["entities"])
        ]
    else:
        raise ProducerError("Only beta-planned schema actions are admitted")
    return sha(
        json.dumps(normalized, separators=(",", ":"), ensure_ascii=False).encode()
    )


def declaration() -> tuple[bytes, str, str]:
    schema = (ROOT / surrealkit.GENERATED_SCHEMA).read_bytes()
    rust = (ROOT / "crates/pse-operations/src/generated/surreal.rs").read_text()
    values = []
    for name in ("INTERPRETATION", "SCHEMA_DIGEST"):
        match = re.search(r"pub const " + name + r': &str = "([^"]+)";', rust)
        if match is None:
            raise ProducerError("Canonical registry identity is absent")
        values.append(match[1])
    if not schema.strip() or re.fullmatch(r"[0-9a-f]{64}", values[1]) is None:
        raise ProducerError("Canonical registry declaration is invalid")
    return schema, values[0], values[1]


def beta(binary: Path, folder: Path, *args: str) -> str:
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SURREAL", "DATABASE_"))
    }
    result = subprocess.run(
        [str(binary), "rollout", *args, "--folder", str(folder)],
        cwd=folder.parent,
        env=environment,
        text=True,
        capture_output=True,
        timeout=60,
        check=False,
    )
    output = result.stdout + result.stderr
    if result.returncode:
        raise ProducerError("Beta offline planning/validation refused: " + output)
    return output


def load(folder: Path, *, current: bool = False) -> dict[str, object]:
    """Validate producer registration; return ordered immutable rollout bindings.

    Each binding includes its TOML spec, exact beta acknowledgements, source and
    target SQL paths, and phase postconditions. Start layers source then target
    definitions (last definition wins); complete requires exactly target SQL.
    The caller separately owns authentic SQL/listener drain and activation.
    """
    if folder.is_symlink():
        raise ProducerError("Symlinked publication refused")
    actual = inventory(folder)
    register = json.loads((folder / REGISTER).read_text())
    if (
        register.get("version"),
        register.get("producer"),
        register.get("tool_version"),
    ) != (
        VERSION,
        PRODUCER,
        BETA,
    ) or not register.get("releases"):
        raise ProducerError("Unknown or empty producer registration")
    expected = dict(register["head_files"])
    for entry in register["releases"]:
        if expected.keys() & entry["files"].keys():
            raise ProducerError("Published file ownership overlaps")
        expected.update(entry["files"])
    if actual != {**expected, REGISTER: actual[REGISTER]}:
        raise ProducerError("Published files differ from exact producer registration")
    rollouts = []
    predecessor = None
    previous_hash = None
    for entry in register["releases"]:
        identity = entry["id"]
        if re.fullmatch(r"[A-Za-z0-9_-]+", identity) is None:
            raise ProducerError("Invalid published rollout identity")
        metadata = json.loads(
            (folder / "rollouts" / identity / "producer.json").read_text()
        )
        raw = (folder / "rollouts" / (identity + ".toml")).read_bytes()
        spec = tomllib.loads(raw.decode())
        if (
            metadata["id"] != identity
            or metadata["predecessor"] != predecessor
            or spec["id"] != identity
        ):
            raise ProducerError("Published rollout order changed")
        if "cutover" in metadata and (
            metadata["cutover"] != CUTOVER
            or metadata["interpretation"] != CUTOVER["target_interpretation"]
            or not rollouts
            or rollouts[-1]["interpretation"] != CUTOVER["source_interpretation"]
        ):
            raise ProducerError("Explicit design-phase cutover binding changed")
        if metadata["manifest_checksum"] != sha(raw):
            raise ProducerError("Beta manifest checksum changed")
        if (
            metadata["source_schema_hash"] != spec["source_schema_hash"]
            or metadata["target_schema_hash"] != spec["target_schema_hash"]
        ):
            raise ProducerError("Beta schema hash binding changed")
        if previous_hash is not None and spec["source_schema_hash"] != previous_hash:
            raise ProducerError("Published rollout predecessor hash changed")
        if metadata["steps"] != step_bindings(
            spec, metadata["source_sql"], metadata["target_sql"]
        ):
            raise ProducerError("Published schema-step binding changed")
        if metadata["schema_sha256"] != actual[metadata["target_sql"]]:
            raise ProducerError("Published canonical schema bytes changed")
        rollouts.append({**metadata, "spec": spec})
        predecessor, previous_hash = identity, spec["target_schema_hash"]
    result = {**register, "rollouts": rollouts}
    if current:
        schema, interpretation, digest = declaration()
        head = rollouts[-1]
        if (sha(schema), interpretation, digest) != (
            head["schema_sha256"],
            head["interpretation"],
            head["schema_digest"],
        ):
            raise ProducerError(
                "Generated declaration changed; publish a forward rollout"
            )
    return result


def step_bindings(
    spec: dict[str, object], source: str, target: str
) -> list[dict[str, object]]:
    # The pinned beta planner owns these TOML schema-action records.
    bindings: list[dict[str, object]] = []
    for step in cast("list[dict[str, object]]", spec["steps"]):
        phase = step["phase"]
        bindings.append(
            {
                "id": step["id"],
                "phase": phase,
                "kind": step["kind"],
                "checksum": beta6_step_checksum(step),
                "postcondition": {
                    "mode": "layered_definitions_last_wins",
                    "sql": [source, target]
                    if phase == "start"
                    else [target]
                    if phase == "complete"
                    else [source],
                },
            }
        )
    return bindings


def empty_snapshots(folder: Path) -> None:
    (folder / "schema").mkdir(parents=True)
    (folder / "snapshots").mkdir()
    write_json(folder / "snapshots/schema_snapshot.json", {"version": 1, "files": []})
    write_json(
        folder / "snapshots/catalog_snapshot.json", {"version": 2, "entities": []}
    )


def object_mapping(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise ProducerError("Expected a string-keyed native metadata object")
    return value


def object_inventory(value: object) -> list[dict[str, object]]:
    if not isinstance(value, list):
        raise ProducerError("Expected a native metadata inventory")
    return [object_mapping(item) for item in value]


def cutover_steps(
    native: dict[str, object], source: dict[str, object], target: dict[str, object]
) -> list[dict[str, object]]:
    """Explicit schema actions from native catalogs; no data transformation."""

    def entities(
        catalog: dict[str, object],
    ) -> dict[tuple[str, str, str], dict[str, object]]:
        if catalog.get("version") != 2:
            raise ProducerError("Design-phase cutover requires beta.6 native catalogs")
        result: dict[tuple[str, str, str], dict[str, object]] = {}
        for entity in object_inventory(catalog.get("entities")):
            key = (
                str(entity["kind"]),
                str(entity.get("scope", "")),
                str(entity["name"]),
            )
            if key in result:
                raise ProducerError("Duplicate native catalog entity")
            result[key] = entity
        return result

    old, new = entities(source), entities(target)
    native_steps = object_inventory(native.get("steps"))
    apply = [step for step in native_steps if step["kind"] == "apply_files"]
    if len(apply) != 1 or apply[0]["phase"] != "start":
        raise ProducerError("Native bootstrap did not freeze exactly one target schema")
    steps = [apply[0]]
    groups = (
        (
            "rollback_expand_schema",
            "rollback",
            "remove_entities",
            sorted(new.keys() - old.keys()),
        ),
        (
            "rollback_modified_schema",
            "rollback",
            "restore_definitions",
            [
                key
                for key in sorted(old.keys() & new.keys())
                if old[key]["statement_hash"] != new[key]["statement_hash"]
                and key[0] not in {"sequence", "user", "access", "model"}
            ],
        ),
        (
            "remove_legacy_entities",
            "complete",
            "remove_entities",
            sorted(old.keys() - new.keys()),
        ),
    )
    for identity, phase, kind, keys in groups:
        if keys:
            steps.append(
                {
                    "id": identity,
                    "phase": phase,
                    "kind": kind,
                    "entities": [
                        {
                            name: (old.get(key) or new[key])[name]
                            for name in ("kind", "scope", "name")
                            if name in (old.get(key) or new[key])
                        }
                        for key in keys
                    ],
                }
            )
    return steps


def manual_manifest(spec: dict[str, object]) -> str:
    """Render only beta.6's supported frozen schema-action manifest format."""
    lines = [
        f"{key} = {json.dumps(spec[key], ensure_ascii=False)}"
        for key in (
            "id",
            "name",
            "source_schema_hash",
            "target_schema_hash",
            "compatibility",
        )
    ]
    if "module" in spec:
        lines.append("module = " + json.dumps(spec["module"]))
    lines.append("renames = []")
    for step in cast("list[dict[str, object]]", spec["steps"]):
        beta6_step_checksum(step)  # Refuse arbitrary data actions before publication.
        lines.extend(["", "[[steps]]"])
        lines.extend(
            f"{key} = {json.dumps(step[key])}" for key in ("id", "phase", "kind")
        )
        field = "files" if step["kind"] == "apply_files" else "entities"
        for item in cast("list[dict[str, object]]", step[field]):
            lines.extend(["", "[[steps." + field + "]]"])
            lines.extend(
                f"{key} = {json.dumps(value, ensure_ascii=False)}"
                for key, value in item.items()
                if value is not None
            )
    return "\n".join(lines) + "\n"


def plan_design_cutover(
    binary: Path, candidate: Path, schema_digest: str, head: dict[str, object]
) -> Path:
    """Native bootstrap owns target bytes/hashes; manual actions own explicit drops.

    beta.6 rollout.rs::validate_autoplan refuses same-scope add/drop as an
    ambiguous rename. Its manual manifests support these schema actions. This
    design-phase publication requires regeneration rather than preserving data.
    """
    bootstrap = candidate.parent / "native-bootstrap"
    empty_snapshots(bootstrap)
    shutil.copyfile(
        candidate / "schema/canonical.surql", bootstrap / "schema/canonical.surql"
    )
    beta(
        binary,
        bootstrap,
        "plan",
        "--name",
        "canonical_" + schema_digest[:16],
        "--allow-modified",
    )
    manifests = list((bootstrap / "rollouts").glob("*.toml"))
    if len(manifests) != 1:
        raise ProducerError("Native bootstrap did not produce exactly one rollout")
    spec = object_mapping(tomllib.loads(manifests[0].read_text()))
    identity = spec.get("id")
    if (
        not isinstance(identity, str)
        or not identity
        or Path(identity).name != identity
        or identity in {".", ".."}
    ):
        raise ProducerError("Invalid native rollout identity")
    owned = candidate / "rollouts" / identity
    if owned.exists() or (candidate / "rollouts" / manifests[0].name).exists():
        raise ProducerError("Design-phase rollout identity already exists")
    source = object_mapping(
        json.loads((candidate / "snapshots/catalog_snapshot.json").read_text())
    )
    target = object_mapping(
        json.loads((bootstrap / "snapshots/catalog_snapshot.json").read_text())
    )
    steps = cutover_steps(spec, source, target)
    spec["steps"] = steps
    if steps[0]["files"] != [
        {
            "path": "schema/canonical.surql",
            "hash": sha((candidate / "schema/canonical.surql").read_bytes()),
        }
    ]:
        raise ProducerError(
            "Native bootstrap target differs from exact generated schema"
        )
    spec["source_schema_hash"] = head["target_schema_hash"]
    frozen_copies: list[tuple[Path, Path]] = []
    for step in steps:
        if step["kind"] == "apply_files":
            for frozen in object_inventory(step.get("files")):
                path, digest = frozen.get("path"), frozen.get("hash")
                if not isinstance(path, str) or not isinstance(digest, str):
                    raise ProducerError("Invalid native frozen file descriptor")
                relative = Path(path)
                if relative.is_absolute() or ".." in relative.parts:
                    raise ProducerError("Native frozen path escapes rollout")
                original = bootstrap / "rollouts" / identity / relative
                if sha(original.read_bytes()) != digest:
                    raise ProducerError("Native frozen target bytes changed")
                frozen_copies.append((original, owned / relative))
    owned.mkdir(parents=True)
    for original, destination in frozen_copies:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(original, destination)
    for name in ("schema_snapshot.json", "catalog_snapshot.json"):
        (owned / "snapshots").mkdir(exist_ok=True)
        shutil.copyfile(candidate / "snapshots" / name, owned / "snapshots" / name)
        shutil.copyfile(bootstrap / "snapshots" / name, candidate / "snapshots" / name)
    manifest = candidate / "rollouts" / manifests[0].name
    manifest.write_text(manual_manifest(spec))
    return manifest


def publish(
    binary: Path,
    folder: Path,
    schema: bytes,
    interpretation: str,
    schema_digest: str,
    *,
    design_phase_v6_cutover: bool = False,
) -> dict[str, object]:
    """Plan offline and append one publication; never repair prior publications."""
    release = surrealkit.verify(binary)
    if (
        release["version"] != BETA
        or not schema.strip()
        or re.fullmatch(r"[0-9a-f]{64}", schema_digest) is None
    ):
        raise ProducerError(
            "Exact beta.6 and a nonempty registry declaration are required"
        )
    if design_phase_v6_cutover and (
        (schema, interpretation, schema_digest) != declaration()
        or interpretation != CUTOVER["target_interpretation"]
    ):
        raise ProducerError(
            "Design-phase cutover requires the exact generated v6 declaration"
        )
    folder.parent.mkdir(parents=True, exist_ok=True)
    # The lock is outside the publication. Incomplete publication stays refused,
    # never silently pruned or adopted after an interrupted publisher.
    with (folder.parent / ("." + folder.name + ".publish.lock")).open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if folder.exists():
            previous = load(folder)
            head = cast("list[dict[str, object]]", previous["rollouts"])[-1]
            if (sha(schema), interpretation, schema_digest) == (
                head["schema_sha256"],
                head["interpretation"],
                head["schema_digest"],
            ):
                if design_phase_v6_cutover and head.get("cutover") != CUTOVER:
                    raise ProducerError(
                        "Existing publication is not the explicit v6 cutover"
                    )
                return head
        else:
            previous, head = None, None
        if design_phase_v6_cutover and (
            head is None or head["interpretation"] != CUTOVER["source_interpretation"]
        ):
            raise ProducerError(
                "Design-phase cutover requires the immutable v5 predecessor"
            )
        with tempfile.TemporaryDirectory(
            prefix=".canonical-plan-", dir=folder.parent
        ) as temporary:
            candidate = Path(temporary) / "database"
            if previous is not None:
                shutil.copytree(folder, candidate)
            else:
                empty_snapshots(candidate)
            (candidate / "schema/canonical.surql").write_bytes(schema)
            if design_phase_v6_cutover:
                if head is None:
                    raise ProducerError("Design-phase cutover requires a predecessor")
                manifest = plan_design_cutover(binary, candidate, schema_digest, head)
            else:
                before = set((candidate / "rollouts").glob("*.toml"))
                beta(
                    binary,
                    candidate,
                    "plan",
                    "--name",
                    "canonical_" + schema_digest[:16],
                    "--allow-modified",
                )
                created = set((candidate / "rollouts").glob("*.toml")) - before
                if len(created) != 1:
                    raise ProducerError(
                        "Beta did not produce exactly one forward rollout"
                    )
                manifest = created.pop()
            spec = tomllib.loads(manifest.read_text())
            identity = spec["id"]
            checksum = sha(manifest.read_bytes())
            lint = beta(binary, candidate, "lint", identity)
            if "checksum " + checksum not in lint:
                raise ProducerError(
                    "Actual beta manifest checksum differs from CLI contract"
                )
            beta(binary, candidate, "lint")
            owned = candidate / "rollouts" / identity
            source_sql = "rollouts/" + identity + "/source-schema.surql"
            target_sql = "rollouts/" + identity + "/target-schema.surql"
            (candidate / source_sql).write_bytes(
                (folder / cast("str", head["target_sql"])).read_bytes() if head else b""
            )
            (candidate / target_sql).write_bytes(schema)
            for name in ("schema_snapshot.json", "catalog_snapshot.json"):
                shutil.copyfile(
                    candidate / "snapshots" / name,
                    owned / ("target-" + name.replace("_", "-")),
                )
            metadata = {
                "id": identity,
                "producer": PRODUCER,
                "tool_version": BETA,
                "predecessor": head["id"] if head else None,
                "interpretation": interpretation,
                "schema_digest": schema_digest,
                "schema_sha256": sha(schema),
                "manifest_checksum": checksum,
                "source_schema_hash": spec["source_schema_hash"],
                "target_schema_hash": spec["target_schema_hash"],
                "source_sql": source_sql,
                "target_sql": target_sql,
                "target_schema_snapshot": "rollouts/"
                + identity
                + "/target-schema-snapshot.json",
                "target_catalog_snapshot": "rollouts/"
                + identity
                + "/target-catalog-snapshot.json",
                "steps": step_bindings(spec, source_sql, target_sql),
                "release": release,
            }
            if design_phase_v6_cutover:
                metadata["cutover"] = dict(CUTOVER)
            write_json(owned / "producer.json", metadata)
            new_files = {"rollouts/" + identity + ".toml": checksum}
            new_files.update(
                {
                    "rollouts/" + identity + "/" + name: digest
                    for name, digest in inventory(owned).items()
                }
            )
            releases = (
                list(cast("list[dict[str, object]]", previous["releases"]))
                if previous
                else []
            )
            releases.append({"id": identity, "files": new_files})
            head_files = {
                name: digest
                for name, digest in inventory(candidate).items()
                if name.startswith(("schema/", "snapshots/"))
            }
            write_json(
                candidate / REGISTER,
                {
                    "version": VERSION,
                    "producer": PRODUCER,
                    "tool_version": BETA,
                    "releases": releases,
                    "head_files": head_files,
                },
            )
            load(candidate)
            if previous is None:
                candidate.rename(folder)
            else:
                # Existing immutable bytes are never opened for writing.
                shutil.copytree(owned, folder / "rollouts" / identity)
                shutil.copyfile(manifest, folder / "rollouts" / manifest.name)
                for name in head_files:
                    (candidate / name).replace(folder / name)
                (candidate / REGISTER).replace(folder / REGISTER)
            load(folder)
            return metadata


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("publish", "validate"))
    parser.add_argument("--folder", type=Path, default=FOLDER)
    parser.add_argument("--binary", type=Path)
    parser.add_argument(
        "--design-phase-v6-cutover",
        action="store_true",
        help="Explicit v5 to generated v6 schema-only publication; requires empty uninitialized target regeneration",
    )
    args = parser.parse_args()
    try:
        if args.command == "validate":
            result = load(args.folder, current=True)
        elif args.binary:
            result = publish(
                args.binary,
                args.folder,
                *declaration(),
                design_phase_v6_cutover=args.design_phase_v6_cutover,
            )
        else:
            raise ProducerError(  # noqa: TRY301 -- selection shares CLI error reporting
                "Select the exact verified beta executable with --binary"
            )
        print(json.dumps(result, indent=2))
        return 0  # noqa: TRY300 -- output errors share CLI error reporting
    except (
        ProducerError,
        surrealkit.ToolError,
        OSError,
        ValueError,
        KeyError,
        TypeError,
    ) as error:
        print("canonical-rollout: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
