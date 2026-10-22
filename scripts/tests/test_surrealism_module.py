# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded immutable module controls; no server, network or compiler required."""
# ruff: noqa: PT009, PT027 -- stdlib operational boundary controls

from __future__ import annotations

import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import surrealism_module as module


def fixture_module(
    root: Path, builder: str = "1" * 64, *, qualification: bool = False,
    exports: set[str] | None = None, config: str | None = None,
) -> dict[str, object]:
    """Publish a tiny package-shaped fixture under the real receipt verifier."""
    kind = module.QUALIFICATION_KIND if qualification else module.KIND
    inputs: dict[str, object] = {
        "kind": kind, "sdk_version": "0.6.4", "target": module.TARGET,
        "profile": "release", "builder_sha256": builder,
        "features": ["module-qualification"] if qualification else [],
        "sources": {"control.rs": "2" * 64},
    }
    directory = root / module._input_digest(inputs)
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    artifact = directory / "study.surli"
    if exports is None:
        exports = module.QUALIFICATION_EXPORTS if qualification else module.PRODUCTION_EXPORTS
    manifest = "\n".join(
        f'[[functions]]\nname = "{name}"\nwriteable = false\n'
        for name in sorted(exports)
    )
    entries = {
        "surrealism/mod.wasm": module.COMPONENT_PREAMBLE,
        "surrealism/surrealism.toml": (config if config is not None else (module.ROOT / module.CRATE / "surrealism.toml").read_text()).encode(),
        "surrealism/exports.toml": manifest.encode(),
    }
    if artifact.exists():
        artifact.chmod(0o600)
    with module.tarfile.open(artifact, mode="w:zst") as archive:
        for name, payload in entries.items():
            header = module.tarfile.TarInfo(name)
            header.size = len(payload)
            archive.addfile(header, io.BytesIO(payload))
    artifact.chmod(0o400)
    receipt: dict[str, object] = {
        "kind": kind, "artifact": str(artifact), "sha256": module.surreal_source.digest(artifact),
        "bytes": artifact.stat().st_size, "sdk_version": "0.6.4", "abi": 2,
        "inputs": inputs, "limits": dict(module.LIMITS),
    }
    (directory / "module.json").write_text(json.dumps(receipt))
    (directory / "module.json").chmod(0o600)
    return receipt


class ModuleAdmissionTests(unittest.TestCase):
    def test_production_accepts_only_its_exact_readonly_export(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            receipt = fixture_module(root)
            self.assertEqual(module.verify(receipt), receipt)
            fixture_module(root, exports={"study::readiness", "qualification::trap"})
            with self.assertRaisesRegex(module.ModuleError, "exports differ"):
                module.inspect_archive(Path(str(receipt["artifact"])))

    def test_qualification_cannot_be_admitted_as_production(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            receipt = fixture_module(Path(scratch), qualification=True)
            self.assertEqual(module.verify(receipt, qualification=True), receipt)
            with self.assertRaisesRegex(module.ModuleError, "receipt or resource"):
                module.verify(receipt)
            with self.assertRaisesRegex(module.ModuleError, "exports differ"):
                module.inspect_archive(Path(str(receipt["artifact"])))

    def test_boolean_integer_aliases_cannot_change_capability_or_receipt_types(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            receipt = fixture_module(root)
            for field, value in (("max_kv_entries", False), ("strict_timeout", 1)):
                with self.subTest(field=field), self.assertRaises(module.ModuleError):
                    module.verify({**receipt, "limits": {**module.LIMITS, field: value}})
            config = (module.ROOT / module.CRATE / "surrealism.toml").read_text().replace(
                "max_kv_entries = 0", "max_kv_entries = false"
            )
            fixture_module(root, config=config)
            with self.assertRaisesRegex(module.ModuleError, "capabilities"):
                module.inspect_archive(Path(str(receipt["artifact"])))

    def test_artifact_tamper_and_mutable_permissions_refuse(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            receipt = fixture_module(Path(scratch))
            artifact = Path(str(receipt["artifact"]))
            artifact.chmod(0o600)
            with self.assertRaisesRegex(module.ModuleError, "immutability"):
                module.verify(receipt)
            original = artifact.read_bytes()
            artifact.write_bytes(bytes([original[0] ^ 1]) + original[1:])
            artifact.chmod(0o400)
            with self.assertRaisesRegex(module.ModuleError, "SHA256"):
                module.verify(receipt)

    def test_missing_provenance_capability_or_generation_refuses(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            receipt = fixture_module(Path(scratch))
            for field, value in (("inputs", {}), ("limits", {}), ("sdk_version", "0.6.3"), ("artifact", "relative/study.surli")):
                with self.subTest(field=field), self.assertRaises(module.ModuleError):
                    module.verify({**receipt, field: value})
            config = (module.ROOT / module.CRATE / "surrealism.toml").read_text().replace("allow_arbitrary_queries = false", "allow_arbitrary_queries = true")
            changed = fixture_module(Path(scratch), config=config)
            with self.assertRaisesRegex(module.ModuleError, "capabilities"):
                module.verify(changed)

    def test_symlink_and_duplicate_or_attached_members_refuse(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            receipt = fixture_module(Path(scratch))
            artifact = Path(str(receipt["artifact"]))
            original = artifact.read_bytes()
            artifact.chmod(0o600)
            with module.tarfile.open(artifact, mode="w:zst") as archive:
                header = module.tarfile.TarInfo("surrealism/fs/unwanted.txt")
                header.size = 1
                archive.addfile(header, io.BytesIO(b"x"))
            with self.assertRaisesRegex(module.ModuleError, "extra, duplicate or linked"):
                module.inspect_archive(artifact)
            artifact.write_bytes(original)
            artifact.chmod(0o400)
            link = artifact.parent / "linked.surli"
            artifact.rename(link)
            artifact.symlink_to(link)
            with self.assertRaisesRegex(module.ModuleError, "symlinks"):
                module.verify(receipt)

    def test_changed_sources_refuse_publication_after_build(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            receipt = fixture_module(root / "fixture")
            inputs = module._object(receipt["inputs"])
            with (
                patch.object(module, "source_inputs", side_effect=[inputs, {**inputs, "changed": True}]),
                patch.object(module, "_command"),
                self.assertRaisesRegex(module.ModuleError, "changed during"),
            ):
                module.install({"binary": "/controlled/builder"}, root / "tools")
            self.assertFalse(any((root / "tools/modules").glob("*/module.json")))


if __name__ == "__main__":
    unittest.main()
