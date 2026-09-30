# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Manifests: valid declarations decode, and each manifest refusal names the file and the key."""

from __future__ import annotations

from pathlib import Path

import pytest
from test_acquire_support import FIXTURES, manifest_text

from thermo_knowledge.acquire import enumerators
from thermo_knowledge.acquire.errors import ManifestError
from thermo_knowledge.acquire.manifest import (
    ArchiveSpec,
    FileSpec,
    GitSpec,
    LocalSpec,
    NoneSpec,
    PagesSpec,
    load_sources,
    parse_manifest,
)

VALID = FIXTURES / "valid"
INVALID = FIXTURES / "invalid"


def test_a_valid_manifest_of_each_kind_decodes() -> None:
    manifests = load_sources(VALID)
    kinds = {source_id: manifest.acquire for source_id, manifest in manifests.items()}
    assert isinstance(kinds["git_source"], GitSpec)
    assert isinstance(kinds["archive_source"], ArchiveSpec)
    assert isinstance(kinds["archive_md5"], ArchiveSpec)
    assert isinstance(kinds["file_source"], FileSpec)
    assert isinstance(kinds["pages_source"], PagesSpec)
    assert isinstance(kinds["local_source"], LocalSpec)
    assert isinstance(kinds["local_tag"], LocalSpec)
    assert isinstance(kinds["none_source"], NoneSpec)
    assert {m.kind for m in manifests.values()} == {
        "git",
        "archive",
        "file",
        "pages",
        "local",
        "none",
    }
    assert list(manifests) == sorted(manifests)


def test_decoded_values() -> None:
    manifests = load_sources(VALID)
    git = manifests["git_source"]
    assert git.tier == "A"
    assert git.waves == [1, 2]
    assert git.payload.reader == "plain"
    assert git.payload.include == ["data/*.json"]
    assert git.rights[0].spdx == "MIT"
    assert git.rights[0].attribution is True
    spec = git.acquire
    assert isinstance(spec, GitSpec)
    assert spec.commit == "0123456789abcdef0123456789abcdef01234567"
    assert spec.tag == "v1.0.0"
    assert spec.submodules == ["vendor/sub"]
    assert spec.sparse == ["data"]
    files = manifests["file_source"].acquire
    assert isinstance(files, FileSpec)
    assert [(f.name, f.sha256 is not None, f.md5 is not None) for f in files.files] == [
        ("a.csv", True, False),
        ("b.csv", False, True),
        ("c.csv", False, False),
    ]
    minimal = manifests["minimal"]
    assert minimal.tier == "held"
    assert minimal.payload.environment == "core"
    assert minimal.waves == []


def test_checksums_are_normalised_to_lower_case() -> None:
    text = manifest_text(
        "upper", f'kind = "archive"\nurl = "https://example.org/a.tgz"\nsha256 = "{"AB" * 32}"'
    )
    spec = parse_manifest(text, file="upper.toml").acquire
    assert isinstance(spec, ArchiveSpec)
    assert spec.sha256 == "ab" * 32


@pytest.mark.parametrize(
    ("name", "expected"),
    [
        ("unknown_top_key", ["surprise", "unknown key"]),
        ("unknown_acquire_key", ["acquire.bogus", "unknown key"]),
        ("unknown_rights_key", ["rights", "mystery"]),
        ("unknown_kind", ["acquire.kind", "'ftp'", "expected one of"]),
        ("no_rights", ["rights", "[[rights]]"]),
        ("bad_commit", ["acquire.commit"]),
        ("archive_without_checksum", ["sha256", "md5"]),
        ("id_mismatch", ["id", "something_else", "id_mismatch"]),
        ("unquoted_date", ["observed", "quote"]),
    ],
)
def test_manifest_refusals_name_the_file_and_key(name: str, expected: list[str]) -> None:
    path = INVALID / f"{name}.toml"
    with pytest.raises(ManifestError) as caught:
        parse_manifest(path.read_text(), file=str(path), stem=path.stem)
    message = str(caught.value)
    assert str(path) in message
    for fragment in expected:
        assert fragment in message, message


def test_a_directory_reports_every_invalid_manifest_together() -> None:
    with pytest.raises(ManifestError) as caught:
        load_sources(INVALID)
    problems = caught.value.problems
    assert len(problems) == len(list(INVALID.glob("*.toml")))
    for path in INVALID.glob("*.toml"):
        assert any(str(path) in problem for problem in problems)


def test_missing_directory_is_refused(tmp_path: Path) -> None:
    with pytest.raises(ManifestError, match="does not exist"):
        load_sources(tmp_path / "absent")


def test_empty_directory_has_no_sources(tmp_path: Path) -> None:
    assert load_sources(tmp_path) == {}


@pytest.mark.parametrize(
    ("acquire", "fragment"),
    [
        (
            'kind = "archive"\nurl = "u"\nsha256 = "' + "a" * 64 + '"\nmd5 = "' + "b" * 32 + '"',
            "exactly one",
        ),
        (
            'kind = "file"\nfiles = [{ url = "u", name = "n", sha256 = "'
            + "a" * 64
            + '", md5 = "'
            + "b" * 32
            + '" }]',
            "at most one",
        ),
        ('kind = "file"\nfiles = [{ url = "u", name = "a/b" }]', "plain file name"),
        (
            'kind = "file"\nfiles = [{ url = "u", name = "n" }, { url = "v", name = "n" }]',
            "duplicate",
        ),
        ('kind = "file"\nfiles = []', "acquire.files"),
        ('kind = "git"\nurl = "u"\ncommit = "' + "a" * 40 + '"\nsparse = ["../x"]', "relative"),
        ('kind = "git"\nurl = "u"\ncommit = "' + "a" * 40 + '"\nsubmodules = ["/abs"]', "relative"),
        ('kind = "local"\npath = "external/x"', "commit"),
        ('kind = "local"\npath = "../outside"\ntag = "v1"', "relative"),
        (
            'kind = "pages"\nenumerator = "url_list"\nmin_interval_seconds = -1',
            "min_interval_seconds",
        ),
        ('kind = "pages"\nenumerator = "no_such"\nmin_interval_seconds = 1', "unknown enumerator"),
        ('kind = "none"\nreason = ""', "acquire.reason"),
    ],
)
def test_constraint_refusals(acquire: str, fragment: str) -> None:
    with pytest.raises(ManifestError) as caught:
        parse_manifest(manifest_text("bad", acquire), file="bad.toml", stem="bad")
    assert "bad.toml" in str(caught.value)
    assert fragment in str(caught.value), str(caught.value)


def test_id_must_be_snake_case() -> None:
    text = manifest_text("Bad-Id", 'kind = "none"\nreason = "r"')
    with pytest.raises(ManifestError, match="snake_case"):
        parse_manifest(text, file="Bad-Id.toml")


def test_invalid_toml_and_missing_acquire_are_reported() -> None:
    with pytest.raises(ManifestError, match="invalid TOML"):
        parse_manifest("id = ", file="x.toml")
    with pytest.raises(ManifestError, match=r"x.toml: acquire: the \[acquire\] table is required"):
        parse_manifest('id = "x"\n', file="x.toml")


def test_enumerator_registry_and_stable_names() -> None:
    assert "url_list" in enumerators.ENUMERATORS
    one = enumerators.stable_name("https://example.org/a/b.html?x=1")
    assert one == enumerators.stable_name("https://example.org/a/b.html?x=1")
    assert one != enumerators.stable_name("https://example.org/a/b.html?x=2")
    assert one.endswith(".html")
    assert "/" not in one
    assert enumerators.stable_name("https://example.org/").startswith("example.org-")
