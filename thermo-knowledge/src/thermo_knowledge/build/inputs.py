# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What a build reads: the resolution result and every source's canonical Parquet.

```
<canonical>/_resolution/manifest.json   the resolution result (phase "resolution")
<canonical>/<id>/manifest.json          a source's records (phase "records")
<canonical>/<id>/_identity/manifest.json  the phase-1 output kept beside it, which carries the
                                        carrier: manifest id, resolved pin and tree hash
<canonical>/_qualification/<case>/manifest.json  a qualification run (phase "qualification")
```

`discover` reads and verifies those directories; `check_current` refuses the ones that were
written against another declaration fingerprint, naming the source that must be mapped again.
`discover_qualification` adds the qualification outputs that are still current and reports, with
its reason, each one it leaves out.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

import pyarrow.parquet as pq

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical import store
from thermo_knowledge.canonical.environment import RESOLUTION_DIR
from thermo_knowledge.canonical.provenance import CarrierInfo
from thermo_knowledge.canonical.schemas import canonical_schemas
from thermo_knowledge.canonical.store import CanonicalError, CanonicalManifest
from thermo_knowledge.declaration import model as m
from thermo_knowledge.mapping.claims import IDENTITY_DIR

RESOLUTION_ID = RESOLUTION_DIR
"""The stage id the resolution result goes by: its directory name, and its `source_id`."""
QUALIFICATION_DIR = "_qualification"
"""The directory of the qualification outputs, one directory per case, under the canonical store."""
QUALIFICATION_PHASE = "qualification"


@dataclass(frozen=True)
class SourceInput:
    """One directory of canonical Parquet a build loads, verified against its manifest."""

    source_id: str
    directory: Path
    manifest: CanonicalManifest
    files: Mapping[str, Path]
    carrier: CarrierInfo | None
    """The carrier of a source (its phase-1 output's); `None` for the resolution result."""

    @property
    def is_resolution(self) -> bool:
        return self.source_id == RESOLUTION_ID


def _load(directory: Path, source_id: str) -> SourceInput:
    manifest = store.read_manifest(directory)
    if manifest.source_id != source_id:
        raise CanonicalError(
            f"{directory}: the manifest belongs to {manifest.source_id!r}, not {source_id!r}"
        )
    resolution = source_id == RESOLUTION_ID
    expected = "resolution" if resolution else "records"
    if manifest.phase != expected:
        raise CanonicalError(
            f"{directory}: the manifest is of phase {manifest.phase!r}; a build reads phase "
            f"{expected!r}"
        )
    files = store.verify_directory(directory, manifest)
    carrier: CarrierInfo | None = None
    if not resolution:
        identity = directory / IDENTITY_DIR
        recorded = store.read_manifest(identity)
        if recorded.carrier is None:
            raise CanonicalError(f"{identity}: the phase-1 manifest records no carrier")
        if manifest.inputs.get("identity") != store.manifest_hash(identity):
            raise CanonicalError(
                f"{source_id}: the phase-1 output beside its records is not the one they were "
                f"mapped from; run `tk map {source_id}`"
            )
        carrier = recorded.carrier
    return SourceInput(source_id, directory, manifest, files, carrier)


def discover(canonical: Path, only: Sequence[str] = ()) -> list[SourceInput]:
    """The resolution result (when there is one) and the sources with records under
    `canonical`, in that order, each verified against its manifest.

    `only` names the sources to build; the resolution result is always included. A source
    that has only phase-1 output has no records and is left out, as is any directory whose name
    starts with `.` or `_` other than the resolution result.
    """
    found: list[SourceInput] = []
    if (canonical / RESOLUTION_ID / store.MANIFEST_NAME).is_file():
        found.append(_load(canonical / RESOLUTION_ID, RESOLUTION_ID))
    available = (
        sorted(
            entry.name
            for entry in canonical.iterdir()
            if entry.is_dir()
            and not entry.name.startswith((".", "_"))
            and (entry / store.MANIFEST_NAME).is_file()
        )
        if canonical.is_dir()
        else []
    )
    missing = sorted(set(only) - set(available))
    if missing:
        raise CanonicalError(
            f"no canonical records for {', '.join(missing)} under {canonical}; "
            "run `tk map <id>` for each"
        )
    wanted = [name for name in available if not only or name in only]
    found.extend(_load(canonical / name, name) for name in wanted)
    return found


def check_current(inputs: Sequence[SourceInput], fingerprint: str) -> None:
    """Refuse every input written against another declaration fingerprint than `fingerprint`,
    naming what must be run again."""
    problems: list[str] = []
    for item in inputs:
        recorded = item.manifest.inputs.get("declaration")
        if recorded == fingerprint:
            continue
        seen = "no declaration fingerprint" if recorded is None else f"fingerprint {recorded[:12]}"
        rerun = (
            "run `tk resolve`, then `tk map` for each source"
            if item.is_resolution
            else f"run `tk map {item.source_id}` (after `tk resolve` if that is stale too)"
        )
        what = "the resolution result" if item.is_resolution else f"{item.source_id}"
        problems.append(
            f"{what} was made against another declaration ({seen}, current "
            f"{fingerprint[:12]}): {rerun}"
        )
    if problems:
        raise CanonicalError("\n".join(problems))


def check_schemas(decl: m.Declaration, inputs: Sequence[SourceInput]) -> None:
    """Refuse a file of an unknown table or with a schema other than the canonical one."""
    schemas = canonical_schemas(decl)
    problems: list[str] = []
    for item in inputs:
        for name, path in sorted(item.files.items()):
            expected = schemas.get(name)
            if expected is None:
                problems.append(
                    f"{item.source_id}: {name} is not a canonical table of the declaration"
                )
                continue
            actual = pq.ParquetFile(path).schema_arrow
            if not actual.equals(expected, check_metadata=False):
                problems.append(
                    f"{item.source_id}: {name}: the file's schema is not the canonical schema\n"
                    f"    expected: {expected.to_string(show_field_metadata=False)!r}\n"
                    f"    found:    {actual.to_string(show_field_metadata=False)!r}"
                )
    if problems:
        raise CanonicalError("\n".join(problems))


@dataclass(frozen=True)
class SkippedOutput:
    """A qualification output a build leaves out, and why."""

    case: str
    reason: str


def _column(path: Path, column: str) -> list[object]:
    return pq.read_table(path, columns=[column]).column(column).to_pylist()


def discover_qualification(
    canonical: Path, inputs: Sequence[SourceInput], fingerprint: str
) -> tuple[list[SourceInput], list[SkippedOutput]]:
    """The qualification outputs under `canonical` that this build can load, and those it leaves
    out with the reason.

    An output is left out when it was made against another declaration fingerprint than
    `fingerprint`, or when a parameter set it evaluated is not among the `tk.parameter_set` rows
    of `inputs`. An output that does not match its own manifest is an error, like any other
    canonical directory.
    """
    root = canonical / QUALIFICATION_DIR
    if not root.is_dir():
        return [], []
    present: set[object] | None = None
    kept: list[SourceInput] = []
    skipped: list[SkippedOutput] = []
    for directory in sorted(
        entry for entry in root.iterdir() if entry.is_dir() and not entry.name.startswith(".")
    ):
        if not (directory / store.MANIFEST_NAME).is_file():
            continue
        manifest = store.read_manifest(directory)
        case = directory.name
        if manifest.source_id != case or manifest.phase != QUALIFICATION_PHASE:
            raise CanonicalError(
                f"{directory}: the manifest is of {manifest.source_id!r}, phase {manifest.phase!r}; "
                f"expected the qualification output of {case!r}"
            )
        files = store.verify_directory(directory, manifest)
        recorded = manifest.inputs.get("declaration")
        if recorded != fingerprint:
            seen = (
                "no declaration fingerprint" if recorded is None else f"fingerprint {recorded[:12]}"
            )
            skipped.append(
                SkippedOutput(
                    case,
                    f"made against another declaration ({seen}, current {fingerprint[:12]}): "
                    f"run `tk qualify {case}`",
                )
            )
            continue
        evaluated = files.get(pc.RUN_PARAMETER_SET.table)
        if evaluated is not None:
            if present is None:
                present = {
                    identifier
                    for item in inputs
                    if pc.PARAMETER_SET.table in item.files
                    for identifier in _column(item.files[pc.PARAMETER_SET.table], "id")
                }
            missing = sum(
                1 for identifier in _column(evaluated, pc.RUN_PARAMETER_SET.parameter_set) if identifier not in present
            )
            if missing:
                skipped.append(
                    SkippedOutput(
                        case, f"{missing} parameter set(s) it evaluated are not in this build"
                    )
                )
                continue
        kept.append(SourceInput(f"{QUALIFICATION_DIR}/{case}", directory, manifest, files, None))
    return kept, skipped
