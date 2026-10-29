# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""HiGHS discovery and proof admission through the persistent candidate owner."""
# ruff: noqa: PT009

from __future__ import annotations

import json
import shutil
import tarfile
import tempfile
import unittest
from functools import partial
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

import pytest

from scripts import native_pipeline_cache as pipeline
from scripts import pse_env

if TYPE_CHECKING:
    from collections.abc import Callable


class HighsProviderCandidateTests(unittest.TestCase):
    def test_native_build_commands_follow_canonical_or_explicit_compiler_width(
        self,
    ) -> None:
        class BuildCommandCapturedError(Exception):
            pass

        for kind in ("isolation", "uno", "petsc"):
            for jobs, expected in ((None, "32"), ("7", "7")):
                with (
                    self.subTest(kind=kind, jobs=jobs),
                    tempfile.TemporaryDirectory() as temporary,
                ):
                    base = Path(temporary)
                    source = base / "fixture-source"
                    (source / "vendor").mkdir(parents=True)
                    (source / "vendor/input.c").write_text("fixture source")
                    archive = base / "source.tar.gz"
                    with tarfile.open(archive, "w:gz") as packed:
                        packed.add(source, arcname="fixture-source")
                    solver = base / "solver"
                    for name in (
                        "libmkl_intel_lp64.so.3",
                        "libmkl_gnu_thread.so.3",
                        "libmkl_core.so.3",
                    ):
                        path = solver / "lib" / name
                        path.parent.mkdir(parents=True, exist_ok=True)
                        path.touch()
                    environment = {"IPOPT_DIR": str(solver)}
                    if jobs is not None:
                        environment["CARGO_BUILD_JOBS"] = jobs
                    commands: list[list[str]] = []

                    def run(
                        command: list[str],
                        *,
                        archive: Path = archive,
                        commands: list[list[str]] = commands,
                        **_options: object,
                    ) -> None:
                        if command[0] == "curl":
                            shutil.copyfile(
                                archive, command[command.index("--output") + 1]
                            )
                        if command[:2] == ["cmake", "--build"] or (
                            command[0] == "make"
                            and any(value.startswith("-j") for value in command)
                        ):
                            commands.append(command)
                            raise BuildCommandCapturedError

                    def prepare(
                        _base: Path,
                        _kind: str,
                        _identity: object,
                        _required: object,
                        builder: Callable[[Path, Path], None],
                    ) -> Path:
                        stage = _base / "stage"
                        stage.mkdir()
                        builder(stage, _base / "work")
                        self.fail("the selected build command must be observed")

                    provider = (
                        partial(pipeline.cache.isolation, base)
                        if kind == "isolation"
                        else partial(pipeline.prepare, kind)
                    )
                    with (
                        patch.object(
                            pipeline.operation,
                            "observe",
                            side_effect=lambda _key, producer: producer(),
                        ),
                        patch.object(pipeline.cache, "prepare", side_effect=prepare),
                        patch.object(pipeline.cache, "cache_root", return_value=base),
                        patch.object(
                            pipeline.cache,
                            "tool_identity",
                            return_value={
                                "target": "fixture",
                                "version": "fixture",
                                "bytes": "fixture",
                            },
                        ),
                        patch.object(pipeline.cache, "admit_external"),
                        patch.object(
                            pipeline.cache,
                            "generation_destination",
                            return_value=base / "destination",
                        ),
                        patch.object(
                            pipeline.cache,
                            "digest",
                            return_value="b3cd09c3be137fd2ff0f2570c3bcbdfcd9ea62fcf04d45d058db3b0dbb1d8872",
                        ),
                        patch.object(pipeline.cache, "run", side_effect=run),
                        patch.object(pipeline.subprocess, "run", side_effect=run),
                        patch.object(
                            pipeline.subprocess,
                            "check_output",
                            return_value=json.dumps(
                                {
                                    "packages": [
                                        {
                                            "name": "suitesparse_sys",
                                            "manifest_path": str(source / "Cargo.toml"),
                                        }
                                    ]
                                }
                            ),
                        ),
                        patch.object(pipeline, "source_archive", return_value=archive),
                        patch.object(
                            pipeline, "highs_archive", return_value=base / "highs"
                        ),
                        patch.object(pse_env, "placement", return_value=[]),
                        pytest.raises(BuildCommandCapturedError),
                    ):
                        provider(environment)
                    self.assertEqual(len(commands), 1)
                    if kind == "petsc":
                        self.assertEqual(commands[0], ["make", f"-j{expected}", "all"])
                    else:
                        self.assertEqual(
                            commands[0][commands[0].index("--parallel") + 1], expected
                        )

    def test_discovery_candidate_accepts_later_receipt_without_cargo(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            directory = base / "native-out"
            directory.mkdir()
            package = (
                "registry+https://github.com/rust-lang/crates.io-index#highs-sys@1.15.0"
            )
            receipt = base / "provider.json"
            environment = {"RUSTFLAGS": "-C link-arg=-fuse-ld=mold"}

            def verify(
                path: Path, owner: str, out_dir: Path, env: dict[str, str]
            ) -> bool:
                self.assertEqual(owner, package)
                self.assertEqual(out_dir, directory)
                self.assertEqual(env["PSE_NATIVE_PROVIDER_RECEIPT"], str(path))
                return path.is_file() and path.read_text() == "qualified"

            with (
                patch.object(pipeline.cache, "cache_root", return_value=base),
                # Each call represents a separate actual native operation.
                patch.object(
                    pipeline.operation,
                    "observe",
                    side_effect=lambda _key, producer: producer(),
                ),
                patch.object(
                    pipeline, "_build_highs_archive", return_value=(directory, package)
                ) as build,
                patch.object(
                    pipeline.producer_receipts,
                    "verify_native_provider",
                    side_effect=verify,
                ) as admission,
            ):
                self.assertEqual(pipeline.highs_archive(environment), directory)
                self.assertEqual(build.call_count, 1)
                admission.assert_not_called()

                receipt.write_text("qualified")
                environment["PSE_NATIVE_PROVIDER_RECEIPT"] = str(receipt)
                self.assertEqual(pipeline.highs_archive(environment), directory)
                self.assertEqual(pipeline.highs_archive(environment), directory)
                self.assertEqual(build.call_count, 1)
                self.assertEqual(admission.call_count, 2)

                relocated = base / "relocated-provider.json"
                relocated.write_text("qualified")
                environment["PSE_NATIVE_PROVIDER_RECEIPT"] = str(relocated)
                self.assertEqual(pipeline.highs_archive(environment), directory)
                self.assertEqual(build.call_count, 1)
                self.assertEqual(
                    len(list((base / "highs-cargo").glob("*/.candidate.json"))), 1
                )

                relocated.write_text("changed-unqualified-proof")
                self.assertEqual(pipeline.highs_archive(environment), directory)
                self.assertEqual(build.call_count, 2)
                relocated.unlink()
                self.assertEqual(pipeline.highs_archive(environment), directory)
                self.assertEqual(build.call_count, 3)

    def test_material_flags_select_another_candidate(self) -> None:
        for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CXXFLAGS"):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                base = Path(temporary)
                environment = {
                    name: "initial",
                    "PSE_NATIVE_PROVIDER_RECEIPT": str(base / "proof.json"),
                }
                with (
                    patch.object(pipeline.cache, "cache_root", return_value=base),
                    patch.object(
                        pipeline.operation,
                        "observe",
                        side_effect=lambda _key, producer: producer(),
                    ),
                    patch.object(
                        pipeline,
                        "_build_highs_archive",
                        return_value=(base / "native-out", "highs-sys@1.15.0"),
                    ) as build,
                    patch.object(
                        pipeline.producer_receipts,
                        "verify_native_provider",
                        return_value=True,
                    ) as admission,
                ):
                    pipeline.highs_archive(environment)
                    environment[name] = "changed"
                    pipeline.highs_archive(environment)
                    self.assertEqual(build.call_count, 2)
                    admission.assert_not_called()
                    self.assertEqual(
                        len(list((base / "highs-cargo").glob("*/.candidate.json"))), 2
                    )


if __name__ == "__main__":
    unittest.main()
