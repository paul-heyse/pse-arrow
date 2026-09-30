# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kind `local`: verify an existing checkout of the repository; nothing is copied.

The checkout must be the top level of its own git repository, at the declared revision, with no
local changes. Only local git commands run; nothing touches the network.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from thermo_knowledge.acquire.errors import AcquireError
from thermo_knowledge.acquire.git import run_git
from thermo_knowledge.acquire.manifest import LocalSpec
from thermo_knowledge.acquire.store import PIN_LENGTH


@dataclass(frozen=True)
class Verified:
    commit: str
    path: str

    @property
    def pin(self) -> str:
        return self.commit[:PIN_LENGTH]


def declared_pin(spec: LocalSpec) -> str:
    """What `--list` shows: the declared commit prefix, else the declared tag."""
    return spec.commit[:PIN_LENGTH] if spec.commit else (spec.tag or "")


def verify(repo_root: Path, source_id: str, spec: LocalSpec) -> Verified:
    """The revision of the checkout when it matches the declaration and is clean."""
    path = (repo_root / spec.path).resolve()
    if not path.is_dir():
        raise AcquireError(f"{source_id}: {path}: the declared local checkout does not exist")
    top = Path(
        run_git(["rev-parse", "--show-toplevel"], cwd=path, source_id=source_id).strip()
    ).resolve()
    if top != path:
        raise AcquireError(
            f"{source_id}: {path} is not the top level of its own git checkout "
            f"(git reports {top}); a local source must be a checkout"
        )
    head = run_git(["rev-parse", "HEAD"], cwd=path, source_id=source_id).strip()
    if spec.commit is not None and head != spec.commit:
        raise AcquireError(
            f"{source_id}: {path} is at commit {head} but the declared commit is {spec.commit}"
        )
    if spec.tag is not None:
        tagged = run_git(
            ["rev-parse", "--verify", f"refs/tags/{spec.tag}^{{commit}}"],
            cwd=path,
            source_id=source_id,
        ).strip()
        if tagged != head:
            raise AcquireError(
                f"{source_id}: {path} is at commit {head} but tag {spec.tag!r} names {tagged}"
            )
    changes = run_git(
        ["status", "--porcelain", "--untracked-files=normal"], cwd=path, source_id=source_id
    ).splitlines()
    if changes:
        shown = "\n  ".join(changes[:10])
        more = f"\n  ... and {len(changes) - 10} more" if len(changes) > 10 else ""
        raise AcquireError(
            f"{source_id}: {path} has local changes; a local source must be clean:\n  {shown}{more}"
        )
    return Verified(commit=head, path=spec.path)
