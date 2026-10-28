# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Build the selected standalone toolkit with the released gRPC SDK."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]


class BuildError(RuntimeError):
    """Pinned toolkit input or an actual source build could not be established."""


def inputs(declared: Mapping[str, str]) -> str:
    """Bind the executable to the declared SDK pin and admitted build recipe."""
    state = hashlib.sha256()
    state.update(json.dumps(dict(declared), sort_keys=True).encode())
    state.update(Path(__file__).read_bytes())
    # These select the SDK pin, compiler and Cargo configuration used below.
    for relative in ("Cargo.toml", "rust-toolchain.toml", ".cargo/config.toml"):
        path = ROOT / relative
        state.update(relative.encode())
        state.update(path.read_bytes() if path.is_file() else b"absent")
    return state.hexdigest()


def manifest(upstream: str) -> str:
    """Keep toolkit behavior with an exact released remote SDK, without embedding."""
    old = "surrealdb = { version = '3.3.0', features = ['protocol-http', 'jwks'] }"
    pin = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["dependencies"][
        "surrealdb"
    ]["version"]
    if upstream.count(old) != 1 or not pin.startswith("="):
        raise BuildError("Pinned toolkit/SDK manifest integration changed")
    result = upstream.replace(
        old,
        f'surrealdb = {{ version = "{pin}", default-features = false, features = ["protocol-grpc", "rustls"] }}',
    )
    result = result.replace(
        "surrealdb-types = { version = '3.3.0' }",
        f'surrealdb-types = {{ version = "{pin}" }}',
    )
    return result.replace("panic = 'abort'", "panic = 'unwind'").replace(
        "homepage = 'https://github.com/surrealdb/surrealkit'\n", ""
    )


def sync_directory(directory: Path) -> None:
    """Persist directory entries before an artifact may back durable authority."""
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def publish(
    directory: Path,
    declared: Mapping[str, str],
    identity: str,
    built: Path,
    details: Mapping[str, object],
) -> Path:
    """Atomically publish a new immutable input identity; never replace a release."""
    if (
        directory != directory.resolve(strict=True)
        or len(identity) != 64
        or any(c not in "0123456789abcdef" for c in identity)
    ):
        raise BuildError(
            "Toolkit publication needs an exact directory and input identity"
        )
    artifacts = directory / "artifacts"
    if artifacts.is_symlink():
        raise BuildError("Toolkit artifact directory must not be a symlink")
    artifacts.mkdir(mode=0o700, exist_ok=True)
    sync_directory(directory)
    target = artifacts / identity
    if target.exists() or target.is_symlink():
        raise BuildError("Toolkit input identity is already published")
    binary = target / "surrealkit"
    with tempfile.TemporaryDirectory(prefix=".publish-", dir=artifacts) as temporary:
        stage = Path(temporary)
        staged_binary = stage / "surrealkit"
        shutil.copyfile(built, staged_binary)
        staged_binary.chmod(0o500)
        with staged_binary.open("rb") as executable:
            os.fsync(executable.fileno())
        receipt = {
            **declared,
            **details,
            "publication": "surrealkit-source-v1",
            "artifact": str(binary),
            "sdk_inputs_sha256": identity,
            "binary_sha256": hashlib.sha256(staged_binary.read_bytes()).hexdigest(),
        }
        receipt_path = stage / "release.json"
        receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
        receipt_path.chmod(0o400)
        with receipt_path.open("rb") as provenance:
            os.fsync(provenance.fileno())
        sync_directory(stage)
        # Same-filesystem directory rename exposes binary and receipt together.
        stage.rename(target)
        sync_directory(artifacts)
    return binary


def build(
    directory: Path, declared: Mapping[str, str], *, expected_inputs: str
) -> Path:
    """Build in the serialized source workspace and publish only admitted inputs."""
    before = inputs(declared)
    if before != expected_inputs:
        raise BuildError("SDK inputs changed before toolkit build admission")
    source = directory / "source"
    if source.is_symlink():
        raise BuildError("Toolkit source workspace must not be a symlink")
    environment = dict(os.environ)
    environment.pop("BASH_ENV", None)

    def run(args: list[str], *, capture: bool = False) -> str:
        result = subprocess.run(
            args,
            cwd=ROOT,
            env=environment,
            text=True,
            capture_output=capture,
            check=False,
        )
        if result.returncode:
            raise BuildError("Pinned toolkit source command failed: " + args[0])
        return result.stdout if capture else ""

    revision = declared["source_revision"]
    if not source.exists():
        run(
            [
                "git",
                "clone",
                "--no-checkout",
                "--filter=blob:none",
                "https://github.com/" + declared["repository"] + ".git",
                str(source),
            ]
        )
        run(["git", "-C", str(source), "checkout", "--detach", revision])
    if (
        run(["git", "-C", str(source), "rev-parse", "HEAD"], capture=True).strip()
        != revision
    ):
        raise BuildError("Toolkit source revision differs from selected input")
    owned = {
        "Cargo.toml",
        "Cargo.lock",
        "crates/surrealkit/Cargo.toml",
        "crates/surrealkit-macros/Cargo.toml",
    }
    changed = set(
        run(
            ["git", "-C", str(source), "diff", "HEAD", "--name-only"], capture=True
        ).splitlines()
    )
    untracked = run(
        ["git", "-C", str(source), "ls-files", "--others", "--exclude-standard"],
        capture=True,
    ).splitlines()
    if changed - owned or untracked:
        raise BuildError(
            "Toolkit acquired source contains changes outside its build integration"
        )
    admission = directory / "source-build.json"
    if (
        any((source / relative).is_symlink() for relative in owned)
        or admission.is_symlink()
    ):
        raise BuildError("Toolkit build inputs must not traverse symlinks")
    previous = admission if admission.exists() else directory / "release.json"
    if previous.exists() and hashlib.sha256(
        (source / "Cargo.lock").read_bytes()
    ).hexdigest() != json.loads(previous.read_text()).get("lock_sha256"):
        raise BuildError("Toolkit source lock differs from its admitted build")
    if not previous.exists() and "Cargo.lock" in changed:
        raise BuildError("Toolkit source lock has no admitted build identity")
    original = run(
        ["git", "-C", str(source), "show", revision + ":Cargo.toml"], capture=True
    )
    (source / "Cargo.toml").write_text(manifest(original))
    for relative in (
        "crates/surrealkit/Cargo.toml",
        "crates/surrealkit-macros/Cargo.toml",
    ):
        original_package = run(
            ["git", "-C", str(source), "show", revision + ":" + relative], capture=True
        )
        (source / relative).write_text(
            original_package.replace("homepage.workspace = true\n", "")
        )
    command = [str(ROOT / "scripts/pse-env"), "--", "env", "-u", "BASH_ENV", "cargo"]
    # Reconcile only the edited manifests against the admitted existing lock.
    # A path-to-registry move changes the SDK package ID, so selecting the old
    # package with `update -p` cannot resolve the newly declared dependency.
    # Normal resolution preserves all still-compatible locked dependencies.
    run(
        [
            *command,
            "metadata",
            "--manifest-path",
            str(source / "Cargo.toml"),
            "--format-version",
            "1",
        ],
        capture=True,
    )
    lock_digest = hashlib.sha256((source / "Cargo.lock").read_bytes()).hexdigest()
    # Admit the exact resolved workspace before compiling, including failed
    # compiles. Published release receipts are never workspace bookkeeping.
    temporary = admission.with_suffix(".json.pending")
    temporary.write_text(
        json.dumps({"sdk_inputs_sha256": before, "lock_sha256": lock_digest}) + "\n"
    )
    temporary.chmod(0o600)
    temporary.replace(admission)
    run(
        [
            *command,
            "build",
            "--manifest-path",
            str(source / "Cargo.toml"),
            "--locked",
            "--release",
            "-p",
            "surrealkit",
            "--no-default-features",
            "--features",
            "cli",
        ]
    )
    if inputs(declared) != before:
        raise BuildError(
            "SDK inputs changed during toolkit build; rebuild from stable inputs"
        )
    if hashlib.sha256((source / "Cargo.lock").read_bytes()).hexdigest() != lock_digest:
        raise BuildError("Toolkit source lock changed during its locked build")
    scaffold = (source / "crates/surrealkit/src/scaffold.rs").read_text()
    setup = scaffold.split('pub const DEFAULT_SETUP: &str = r#"', 1)[1].split('"#;', 1)[
        0
    ]
    resolved = tomllib.loads((source / "Cargo.lock").read_text())["package"]
    return publish(
        directory,
        declared,
        before,
        source / "target/release/surrealkit",
        {
            "source_commit": revision,
            "source_build": True,
            "setup_sha256": hashlib.sha256(setup.encode()).hexdigest(),
            "resolved_sdk": [
                p["version"] for p in resolved if p["name"] == "surrealdb"
            ],
            "release_build_locked": True,
            "lock_sha256": lock_digest,
        },
    )
