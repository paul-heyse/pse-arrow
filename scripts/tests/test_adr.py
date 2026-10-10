# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""ADR native semantics and exact immutable-reference relocation."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import adr, document_metadata


class AdrMetadataTests(unittest.TestCase):
    def test_metadata_edit_preserves_body_bytes_and_newlines(self) -> None:
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(adr, "ROOT", Path(temporary)),
        ):
            path = Path(temporary) / "docs/adr/0001-test.md"
            path.parent.mkdir(parents=True)
            body = b"\r\n## Exact body\r\ntrailing spaces  \r\nstatus: body-value\r\n"
            path.write_bytes(b"---\r\nstatus: proposed\r\n---\r\n" + body)
            adr.set_field(path, "status", "accepted")
            self.assertEqual(
                path.read_bytes(), b"---\r\nstatus: accepted\r\n---\r\n" + body
            )

    def test_closed_legacy_encoding_requires_identity_and_exact_span(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "docs").mkdir()
            identity = "docs/adr/0002-old.md"
            literal = " not-required: old reason"
            digest = hashlib.sha256(literal.encode()).hexdigest()
            (root / "docs/lifecycle.toml").write_text(
                f'[[legacy_yaml]]\npath = "{identity}"\n[legacy_yaml.scalars]\nreview = "{digest}"\n'
            )
            raw = f"---\nreview:{literal}\n---\n\n# Exact body\n".encode()
            document = document_metadata.read(raw, identity=identity, root=root)
            self.assertEqual(document.metadata["review"], literal.strip())
            self.assertEqual(document.prefix + document.body, raw)
            for selected_identity, data in [
                (None, raw),
                ("docs/adr/0003-other.md", raw),
                (identity, raw.replace(b"old reason", b"changed reason")),
            ]:
                with (
                    self.subTest(identity=selected_identity),
                    self.assertRaises(ValueError),
                ):
                    document_metadata.read(data, root=root, identity=selected_identity)
            duplicate = raw.replace(b"---\n\n#", f"review:{literal}\n---\n\n#".encode())
            with self.assertRaises(ValueError):
                document_metadata.read(duplicate, identity=identity, root=root)

    def test_native_dates_lists_nulls_and_corpus(self) -> None:
        count = 0
        for path in adr.adr_paths():
            with self.subTest(path=path.name):
                front, body = adr.split_front_matter(
                    path.read_text(),
                    str(path),
                    identity=path.relative_to(adr.ROOT).as_posix(),
                )
                self.assertIsInstance(front["date"], str)
                self.assertIsInstance(front["principles"], list)
                self.assertEqual(
                    body.encode(),
                    document_metadata.read(
                        path.read_bytes(),
                        identity=path.relative_to(adr.ROOT).as_posix(),
                    ).body,
                )
                count += 1
        self.assertGreater(count, 0)

    def test_scenario_relocation_keeps_path_fragment_and_source(self) -> None:
        old = ["docs/plans/23-old.md#scenario"]
        url = "https://github.com/owner/repo/blob/" + "a" * 40 + "/" + old[0]
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(adr, "ROOT", Path(temporary)),
            patch.object(adr, "repository_slug", return_value="owner/repo"),
            patch.object(adr, "object_state", return_value="present"),
            patch.object(adr, "same_as_baseline", return_value=True),
        ):
            self.assertTrue(adr.relocated_scenarios(old, [url]))
            self.assertFalse(
                adr.relocated_scenarios(old, [url.replace("#scenario", "#other")])
            )
            self.assertFalse(
                adr.relocated_scenarios(old, [url.replace("owner/repo", "wrong/repo")])
            )
            self.assertFalse(adr.relocated_scenarios(old, []))
            with patch.object(adr, "same_as_baseline", return_value=False):
                self.assertFalse(adr.relocated_scenarios(old, [url]))
            path = Path(temporary) / old[0].split("#")[0]
            path.parent.mkdir(parents=True)
            path.write_text("# Not retired\n")
            self.assertFalse(adr.relocated_scenarios(old, [url]))


if __name__ == "__main__":
    unittest.main()
