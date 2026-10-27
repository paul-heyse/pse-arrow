# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Construct immutable artifact generations and atomically publish their contract."""

from __future__ import annotations

import hashlib
import os
import shutil
import tempfile
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

Mode = Literal["dev", "native"]
Profile = Annotated[str, Field(pattern=r"^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")]


class PublicationContract(BaseModel):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )


class Artifact(PublicationContract):
    """Identity and compiled features recorded once by the constructor."""

    binary: str
    sha256: str
    features: list[str]


class Publication(PublicationContract):
    document_version: Literal[1] = 1
    kind: Literal["extension", "evidence-executable"]
    generation: str
    artifact: Artifact
    mode: Mode
    profile: Profile
    validated: bool
    interpreter: str


def read(path: Path) -> Publication | None:
    """Read the owner's issued contract without revalidating its controlled image."""
    try:
        return Publication.model_validate_json(path.read_bytes())
    except (OSError, ValueError):
        return None


def publish(
    receipt: Path,
    *,
    kind: Literal["extension", "evidence-executable"],
    source: Path,
    mode: Mode,
    profile: str,
    validated: bool,
    interpreter: str,
    features: Callable[[Path], Sequence[str]],
) -> Publication:
    """Copy once, describe the exact candidate, then switch the issued descriptor.

    Failures before replacement settle their fresh candidate. Once replacement
    is attempted, interruption can leave either descriptor selected: retain the
    candidate and all previously issued generations, without promising rollback.
    """
    receipt.parent.mkdir(parents=True, exist_ok=True)
    generations = receipt.parent / (receipt.stem + ".generations")
    generations.mkdir(mode=0o700, exist_ok=True)
    generation = uuid.uuid4().hex
    directory = generations / generation
    directory.mkdir(mode=0o700)
    binary = directory / source.name
    temporary: Path | None = None
    replace_attempted = False
    try:
        digest = hashlib.sha256()
        with source.open("rb") as original, binary.open("xb") as output:
            while chunk := original.read(1 << 20):
                output.write(chunk)
                digest.update(chunk)
            output.flush()
            os.fsync(output.fileno())
        binary.chmod(0o500)
        publication = Publication(
            kind=kind,
            generation=generation,
            artifact=Artifact(
                binary=str(binary.absolute()),
                sha256=digest.hexdigest(),
                features=list(features(binary)),
            ),
            mode=mode,
            profile=profile,
            validated=validated,
            interpreter=interpreter,
        )
        with tempfile.NamedTemporaryFile(
            mode="w", dir=receipt.parent, prefix=".artifact-publication-", delete=False
        ) as output:
            temporary = Path(output.name)
            output.write(publication.model_dump_json() + "\n")
            output.flush()
            os.fsync(output.fileno())
        replace_attempted = True
        temporary.replace(receipt)
    except BaseException as failure:
        failures: list[BaseException] = [failure]
        if temporary is not None:
            try:
                temporary.unlink(missing_ok=True)
            except BaseException as cleanup:
                failures.append(cleanup)
        if not replace_attempted:
            try:
                shutil.rmtree(directory)
            except BaseException as cleanup:
                failures.append(cleanup)
        if len(failures) > 1:
            raise BaseExceptionGroup(
                "artifact construction and cleanup failed", failures
            ) from None
        raise
    else:
        return publication
