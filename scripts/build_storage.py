# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only metadata observation; disposal belongs to each store's producer."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import tomllib
from pathlib import Path
from typing import TypedDict

from scripts import build_environment, native_cache, pse_env


class Measurement(TypedDict):
    status: str
    apparent_bytes: int
    allocated_bytes: int
    unique_entries: int
    excluded_entries: int
    error_count: int
    errors: list[str]


def measure(path: Path) -> Measurement:
    """Count unique inodes without opening payloads or following nested symlinks."""
    seen: set[tuple[int, int]] = set()
    apparent = allocated = entries = excluded = errors = 0
    failures: list[str] = []

    def failed(item: Path, error: OSError) -> None:
        nonlocal errors
        errors += 1
        if len(failures) < 10:
            failures.append(f"{item}: {error}")

    def account(identity: os.stat_result) -> bool:
        nonlocal entries, apparent, allocated
        key = (identity.st_dev, identity.st_ino)
        if key in seen:
            return False
        seen.add(key)
        entries += 1
        apparent += identity.st_size
        allocated += identity.st_blocks * 512
        return True

    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC

    def visit(descriptor: int, item: Path) -> None:
        nonlocal excluded
        try:
            with os.scandir(descriptor) as children:
                for child in children:
                    child_path = item / child.name
                    if child.name == ".git" or child.name.startswith(".venv"):
                        excluded += 1
                        continue
                    try:
                        identity = os.stat(
                            child.name, dir_fd=descriptor, follow_symlinks=False
                        )
                        if not account(identity) or not stat.S_ISDIR(identity.st_mode):
                            continue
                        child_fd = os.open(child.name, flags, dir_fd=descriptor)
                        try:
                            observed = os.fstat(child_fd)
                            if (identity.st_dev, identity.st_ino) != (
                                observed.st_dev,
                                observed.st_ino,
                            ):
                                raise OSError(
                                    "directory identity changed during observation"
                                )
                            visit(child_fd, child_path)
                        finally:
                            os.close(child_fd)
                    except OSError as error:
                        failed(child_path, error)
        except OSError as error:
            failed(item, error)

    try:
        if path.name == ".git" or path.name.startswith(".venv"):
            excluded += 1
        else:
            descriptor = os.open(path, flags)
            try:
                account(os.fstat(descriptor))
                visit(descriptor, path)
            finally:
                os.close(descriptor)
    except OSError as error:
        failed(path, error)
    return {
        "status": "partial" if failures else "observed",
        "apparent_bytes": apparent,
        "allocated_bytes": allocated,
        "unique_entries": entries,
        "excluded_entries": excluded,
        "error_count": errors,
        "errors": failures,
    }


def stores(root: Path, env: dict[str, str]) -> list[tuple[str, Path, str, bool]]:
    """Configured stores and the bounded legacy cohorts selected by Plan 32."""
    selected = [
        ("target", root / "target", "checkout-compiler", True),
        ("build", root / "build", "producer-owned-reports-and-campaigns", True),
        ("native", native_cache.cache_root(env), "shared-native-owner", True),
        (
            "canonical-state",
            Path(env["PSE_SURREAL_STATE"]),
            "persistent-semantic-owner",
            False,
        ),
    ]
    if env.get("SCCACHE_DIR"):
        selected.append(
            ("compiler-cache", Path(env["SCCACHE_DIR"]), "shared-compiler-cache", True)
        )
    for name in ("plan10", "plan11", "assessment", "workspace-review-2026-10-09"):
        selected.append(
            (name, root / "build" / name, "legacy-producer-protected-unknown", True)
        )
    config = root / ".config/library-skills.toml"
    if config.is_file():
        skills = tomllib.loads(config.read_text())
        for name in skills["enabled"]:
            selected.append(
                (
                    f"skill:{name}",
                    root / skills["directory"] / name,
                    "shared-skill-owner",
                    False,
                )
            )
    return selected


def observe(
    selected: list[tuple[str, Path, str, bool]], *, metadata_only: bool = False
) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    aliases: dict[tuple[int, int], str] = {}
    for name, path, owner, recursive in selected:
        record: dict[str, object] = {
            "name": name,
            "path": str(path),
            "ownership": owner,
            "active_use": "unknown; consult the producer owner before disposal",
            "resource_identity": "unknown; this observer does not acquire owner locks",
            "reclaimable_bytes": None,
        }
        try:
            resolved = path.resolve(strict=True)
            identity = resolved.stat()
            key = (identity.st_dev, identity.st_ino)
            record.update(resolved_path=str(resolved), identity=[*key, identity.st_uid])
            if key in aliases:
                record.update(status="alias", alias_of=aliases[key])
            else:
                aliases[key] = name
                if recursive and not metadata_only:
                    record.update(measure(resolved))
                else:
                    record.update(
                        status="metadata-only",
                        apparent_bytes=None,
                        allocated_bytes=None,
                    )
        except OSError as error:
            record.update(
                status="unavailable",
                error=str(error),
                apparent_bytes=None,
                allocated_bytes=None,
            )
        records.append(record)
    return records


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--metadata-only",
        action="store_true",
        help="report store identities without traversing directories",
    )
    args = parser.parse_args()
    root = build_environment.ROOT
    settings = tomllib.loads((root / ".config/build.toml").read_text())
    env = build_environment.configure(
        root, pse_env.base_environment(root, dict(os.environ))
    )
    print(
        json.dumps(
            {
                "version": 1,
                "free_bytes": shutil.disk_usage(root).free,
                "free_space_floor_gib": settings["free_space_gib"],
                "observation": "non-atomic metadata snapshot; rows overlap and must not be summed; hardlinks deduplicated within each row; nested symlinks not followed",
                "retention": "no automatic deletion; persistent state and shared stores retain their existing owners",
                "stores": observe(stores(root, env), metadata_only=args.metadata_only),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
