# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""HiGHS discovery and proof admission through the persistent candidate owner."""
# ruff: noqa: PT009

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import native_pipeline_cache as pipeline


class HighsProviderCandidateTests(unittest.TestCase):
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
                    pipeline.producer_deployment,
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
                        pipeline.producer_deployment,
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
